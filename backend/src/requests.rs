use crate::models::organization::Organization;
use axum::http::{Method, request::Parts};
use chrono::NaiveDateTime;
use compact_str::ToCompactString;
use rand::distr::SampleString;
use rustis::commands::{GenericCommands, SetCondition, SetExpiration, StringCommands};
use serde::{Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;
use std::{
    collections::{HashMap, HashSet},
    net::Ipv6Addr,
    sync::Arc,
};
use tokio::sync::Mutex;

pub struct Request {
    id: String,
    organization_id: Option<i32>,
    end: bool,

    origin: String,
    method: Method,
    path: String,
    time: i32,
    status: i16,
    body: Option<serde_json::Value>,

    ip: IpNetwork,
    continent: Option<compact_str::CompactString>,
    country: Option<compact_str::CompactString>,

    data: Option<serde_json::Value>,
    user_agent: String,
    created: NaiveDateTime,
}

#[derive(Debug, Serialize, clickhouse::Row)]
pub struct ClickhouseRequest {
    id: [u8; 12],
    organization_id: Option<i32>,

    origin: Option<String>,
    method: i8,
    path: String,
    time: i32,
    status: i16,

    body: Option<String>,
    data: Option<String>,

    ip: Ipv6Addr,

    continent: Option<[u8; 2]>,
    country: Option<[u8; 2]>,

    user_agent: String,

    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    created: chrono::DateTime<chrono::Utc>,
}

impl From<Request> for ClickhouseRequest {
    fn from(req: Request) -> Self {
        Self {
            id: req.id.as_bytes().try_into().unwrap_or([0u8; 12]),
            organization_id: req.organization_id,
            origin: if req.origin.is_empty() {
                None
            } else {
                Some(req.origin)
            },
            method: match req.method {
                Method::GET => 1,
                Method::POST => 2,
                Method::PUT => 3,
                Method::DELETE => 4,
                Method::PATCH => 5,
                Method::OPTIONS => 6,
                Method::HEAD => 7,
                _ => 1,
            },
            path: req.path,
            time: req.time,
            status: req.status,
            body: req.body.map(|b| b.to_string()),
            data: req.data.map(|d| d.to_string()),
            ip: match req.ip {
                IpNetwork::V4(ipv4) => ipv4.ip().to_ipv6_mapped(),
                IpNetwork::V6(ipv6) => ipv6.ip(),
            },
            continent: req.continent.and_then(|c| c.as_bytes().try_into().ok()),
            country: req.country.and_then(|c| c.as_bytes().try_into().ok()),
            user_agent: req.user_agent,
            created: req.created.and_utc(),
        }
    }
}

const ACCEPTED_METHODS: &[Method] = &[
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::DELETE,
    Method::PATCH,
];

#[derive(Debug, Clone, Copy)]
pub struct RateLimitData {
    pub limit: i64,
    pub hits: i64,
}

pub struct RequestLogger {
    pending: Mutex<Vec<Request>>,
    processing: Mutex<Vec<Request>>,
    uncounted_requests: Mutex<i64>,
    database: Arc<crate::database::Database>,
    clickhouse: Arc<crate::clickhouse::Clickhouse>,
    cache: Arc<crate::cache::Cache>,

    client: reqwest::Client,
}

impl RequestLogger {
    pub fn new(
        database: Arc<crate::database::Database>,
        clickhouse: Arc<crate::clickhouse::Clickhouse>,
        cache: Arc<crate::cache::Cache>,
    ) -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
            processing: Mutex::new(Vec::new()),
            uncounted_requests: Mutex::new(0),
            database,
            clickhouse,
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
            None => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        };

        let mut ratelimit: Option<RateLimitData> = None;
        if organization.is_none_or(|o| !o.verified) {
            let ratelimit_key = format!(
                "mcjars_api::ratelimit::{ip}::{}",
                if request.uri.path().contains("files") {
                    "files"
                } else {
                    "regular"
                }
            );

            let now = chrono::Utc::now().timestamp();
            let expiry = self
                .cache
                .client
                .expiretime(&ratelimit_key)
                .await
                .unwrap_or_default();
            let expire_unix: u64 = if expiry > now + 2 {
                expiry as u64
            } else {
                now as u64 + 60
            };

            let mut count: i64 = self
                .cache
                .client
                .get(&ratelimit_key)
                .await
                .unwrap_or_default();
            self.cache
                .client
                .set_with_options(
                    ratelimit_key,
                    count + 1,
                    SetCondition::None,
                    SetExpiration::Exat(expire_unix),
                    false,
                )
                .await
                .unwrap();
            count += 1;

            ratelimit = Some(RateLimitData {
                limit: if request.uri.path().contains("files") {
                    30
                } else if organization.is_some() {
                    240
                } else {
                    120
                },
                hits: count,
            });

            if count > ratelimit.unwrap().limit {
                return Err(ratelimit);
            }
        }

        *self.uncounted_requests.lock().await += 1;

        if ACCEPTED_METHODS.iter().all(|m| *m != request.method)
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
            method: request.method.clone(),
            path: crate::utils::slice_up_to(
                &format!(
                    "{}{}",
                    request.uri.path(),
                    request
                        .uri
                        .query()
                        .map(|q| format!("?{}", q.replacen("tracking=none", "tracking=nostats", 1)))
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
        ips: Vec<compact_str::CompactString>,
    ) -> Result<HashMap<compact_str::CompactString, [compact_str::CompactString; 2]>, reqwest::Error>
    {
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
            continent_code: compact_str::CompactString,
            country_code: compact_str::CompactString,
            query: compact_str::CompactString,
        }

        for entry in data {
            result.insert(entry.query, [entry.continent_code, entry.country_code]);
        }

        Ok(result)
    }

    pub async fn process(&self) -> Result<(), anyhow::Error> {
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
                    .map(|t| t.ip.to_compact_string())
                    .collect::<Vec<_>>(),
            )
            .await
            .unwrap_or_default();

        for r in requests.iter_mut() {
            if let Some([continent, country]) = ips.get(&r.ip.to_compact_string()) {
                r.continent = Some(continent.clone());
                r.country = Some(country.clone());
            }
        }

        let requests_len = requests.len();
        let mut insert = self
            .clickhouse
            .client()
            .insert::<ClickhouseRequest>("requests")
            .await?;
        for r in requests {
            let ch_request: ClickhouseRequest = r.into();

            insert.write(&ch_request).await?;
        }
        insert.end().await?;

        let mut uncounted_requests = self.uncounted_requests.lock().await;
        if *uncounted_requests > 0 {
            let count = *uncounted_requests;
            *uncounted_requests = 0;
            drop(uncounted_requests);

            if let Err(err) = self.database.update_count("requests", count).await {
                tracing::error!("failed to update request count: {:?}", err);
            }
        }

        tracing::info!("processed {} requests", requests_len);

        Ok(())
    }
}
