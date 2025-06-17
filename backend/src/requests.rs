use crate::models::organization::Organization;
use axum::http::{Method, request::Parts};
use chrono::NaiveDateTime;
use colored::Colorize;
use rand::distr::SampleString;
use rustis::commands::StringCommands;
use serde::{Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Deserialize, Serialize)]
pub struct Request {
    id: String,
    organization_id: Option<i32>,
    end: bool,

    origin: String,
    method: String,
    path: String,
    time: i32,
    status: i16,
    body: Option<serde_json::Value>,

    ip: IpNetwork,
    continent: Option<String>,
    country: Option<String>,

    data: Option<serde_json::Value>,
    user_agent: String,
    created: NaiveDateTime,
}

const ACCEPTED_METHODS: &[Method] = &[
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::PATCH,
    Method::DELETE,
];

#[derive(Debug, Clone, Copy)]
pub struct RateLimitData {
    pub limit: i64,
    pub hits: i64,
}

pub struct RequestLogger {
    pending: Mutex<Vec<Request>>,
    processing: Mutex<Vec<Request>>,
    database: Arc<crate::database::Database>,
    cache: Arc<crate::cache::Cache>,

    client: reqwest::Client,
}

impl RequestLogger {
    pub fn new(database: Arc<crate::database::Database>, cache: Arc<crate::cache::Cache>) -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
            processing: Mutex::new(Vec::new()),
            database,
            cache,

            client: reqwest::Client::builder()
                .user_agent("MCJars API https://mcjars.app")
                .build()
                .unwrap(),
        }
    }

    pub async fn log(
        &self,
        request: &Parts,
        organization: Option<&Organization>,
    ) -> Result<(Option<String>, Option<RateLimitData>), Option<RateLimitData>> {
        let ip = match crate::utils::extract_ip(&request.headers) {
            Some(ip) => ip,
            None => return Err(None),
        };

        let mut ratelimit: Option<RateLimitData> = None;
        if organization.is_none_or(|o| !o.verified) {
            let ratelimit_key = format!("mcjars_api::ratelimit::{}", ip);

            let count: i64 = self
                .cache
                .client
                .get(&ratelimit_key)
                .await
                .unwrap_or_default();
            let count = if count == 0 {
                self.cache
                    .client
                    .setex(&ratelimit_key, 60, 1)
                    .await
                    .unwrap();

                1
            } else {
                self.cache.client.incr(&ratelimit_key).await.unwrap()
            };

            ratelimit = Some(RateLimitData {
                limit: if organization.is_some() { 240 } else { 120 },
                hits: count,
            });

            if count > ratelimit.unwrap().limit {
                return Err(ratelimit);
            }
        }

        if request
            .uri
            .query()
            .map(|q| q.contains("tracking=none"))
            .unwrap_or(false)
            || ACCEPTED_METHODS.iter().all(|m| *m != request.method)
            || !request.uri.path().starts_with("/api")
            || request.uri.path().starts_with("/api/github")
        {
            return Ok((None, ratelimit));
        };

        let data = Request {
            id: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 12),
            organization_id: organization.map(|o| o.id),
            end: false,

            origin: request
                .headers
                .get("origin")
                .map(|o| crate::utils::slice_up_to(o.to_str().unwrap_or("unknown"), 255))
                .unwrap_or("")
                .to_string(),
            method: request.method.to_string(),
            path: crate::utils::slice_up_to(
                &format!(
                    "{}{}",
                    request.uri.path(),
                    request
                        .uri
                        .query()
                        .map(|q| format!("?{}", q))
                        .unwrap_or_default()
                ),
                255,
            )
            .to_string(),
            time: 0,
            status: 0,
            body: None,

            ip: ip.into(),
            continent: None,
            country: None,

            data: None,
            user_agent: request
                .headers
                .get("User-Agent")
                .map(|ua| crate::utils::slice_up_to(ua.to_str().unwrap_or("unknown"), 255))
                .unwrap_or("unknown")
                .to_string(),
            created: chrono::Utc::now().naive_utc(),
        };

        let id = data.id.clone();
        self.pending.lock().await.push(data);

        Ok((Some(id), ratelimit))
    }

    pub async fn finish(
        &self,
        id: String,
        status: i16,
        time: i32,
        data: Option<serde_json::Value>,
        body: Option<serde_json::Value>,
    ) {
        let mut pending = self.pending.lock().await;

        if let Some(index) = pending.iter().position(|r| r.id == id) {
            let mut request = pending.remove(index);

            request.end = true;
            request.status = status;
            request.time = time;
            request.data = data;
            request.body = body;

            self.processing.lock().await.push(request);
        }
    }

    #[inline]
    async fn lookup_ips(
        &self,
        ips: Vec<String>,
    ) -> Result<HashMap<String, [String; 2]>, reqwest::Error> {
        let mut result = HashMap::new();

        let data = self
            .client
            .post("http://ip-api.com/batch")
            .header("Content-Type", "application/json")
            .json(
                &ips.into_iter()
                    .map(|ip| {
                        serde_json::json!({
                            "query": ip,
                            "fields": "continentCode,countryCode,query"
                        })
                    })
                    .collect::<HashSet<_>>(),
            )
            .send()
            .await?
            .json::<Vec<IpApiResponse>>()
            .await?;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct IpApiResponse {
            continent_code: String,
            country_code: String,
            query: String,
        }

        for entry in data {
            result.insert(entry.query, [entry.continent_code, entry.country_code]);
        }

        Ok(result)
    }

    pub async fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut processing = self.processing.lock().await;
        let now = chrono::Utc::now().naive_utc();
        let length = processing.len();

        self.pending
            .lock()
            .await
            .retain(|r| r.created > now - chrono::Duration::seconds(60));

        let mut requests = processing
            .splice(0..std::cmp::min(30, length), Vec::new())
            .collect::<Vec<_>>();
        processing.retain(|r| r.created > now - chrono::Duration::seconds(300));

        drop(processing);

        if requests.is_empty() {
            return Ok(());
        }

        let ips = self
            .lookup_ips(
                requests
                    .iter()
                    .map(|t| t.ip.to_string())
                    .collect::<Vec<_>>(),
            )
            .await
            .unwrap_or_default();

        for r in requests.iter_mut() {
            if let Some([continent, country]) = ips.get(&r.ip.to_string()) {
                r.continent = Some(continent.clone());
                r.country = Some(country.clone());
            }
        }

        for r in requests.iter() {
            match sqlx::query!(
                r#"
                INSERT INTO requests (id, organization_id, origin, method, path, time, status, body, ip, continent, country, data, user_agent, created)
                VALUES ($1, $2, $3, $4::text::Method, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                ON CONFLICT DO NOTHING
                "#,
                r.id,
                r.organization_id,
                r.origin,
                r.method,
                r.path,
                r.time,
                r.status,
                r.body,
                r.ip,
                r.continent,
                r.country,
                r.data,
                r.user_agent,
                r.created
            )
            .execute(self.database.write())
            .await {
                Ok(_) => {}
                Err(e) => {
                    crate::logger::log(
                        crate::logger::LoggerLevel::Error,
                        format!("{} {}", "failed to insert request".red(), e),
                    );

                    self.processing
                        .lock()
                        .await
                        .append(&mut requests);

                    return Err(Box::new(e));
                }
            }
        }

        crate::logger::log(
            crate::logger::LoggerLevel::Info,
            format!("processed {} requests", requests.len().to_string().cyan()),
        );

        Ok(())
    }
}
