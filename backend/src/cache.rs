use crate::{env::RedisMode, response::ApiResponse};
use rustis::{
    client::Client,
    commands::{
        GenericCommands, InfoSection, ServerCommands, SetCondition, SetExpiration, StringCommands,
    },
    resp::BulkString,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    future::Future,
    sync::{Arc, atomic::AtomicUsize},
};

pub struct Cache {
    pub client: Client,

    cache_hits: AtomicUsize,
    cache_misses: AtomicUsize,
}

impl Cache {
    pub async fn new(env: Arc<crate::env::Env>) -> Self {
        let start = std::time::Instant::now();

        let instance = Self {
            client: match env.redis_mode {
                RedisMode::Redis => Client::connect(env.redis_url.as_ref().unwrap().clone())
                    .await
                    .unwrap(),
                RedisMode::Sentinel => Client::connect(
                    format!(
                        "redis-sentinel://{}/mymaster/0",
                        env.redis_sentinels.as_ref().unwrap().clone().join(",")
                    )
                    .as_str(),
                )
                .await
                .unwrap(),
            },
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
        };

        let version: String = instance.client.info([InfoSection::Server]).await.unwrap();
        let version = version
            .lines()
            .find(|line| line.starts_with("redis_version:"))
            .unwrap()
            .split(':')
            .collect::<Vec<&str>>()[1]
            .to_string();

        tracing::info!(
            "cache connected (redis@{}, {}ms)",
            version,
            start.elapsed().as_millis()
        );

        instance
    }

    #[inline]
    pub fn cache_hits(&self) -> usize {
        self.cache_hits.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[inline]
    pub fn cache_misses(&self) -> usize {
        self.cache_misses.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[tracing::instrument(skip(self, fn_compute))]
    pub async fn cached<T, F, Fut, FutErr>(
        &self,
        key: &str,
        ttl: u64,
        fn_compute: F,
    ) -> Result<T, anyhow::Error>
    where
        T: Serialize + DeserializeOwned + Send,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, FutErr>>,
        FutErr: Into<anyhow::Error> + Send + Sync + 'static,
    {
        let cached_value: Option<BulkString> = self.client.get(key).await?;

        match cached_value.and_then(|v| rmp_serde::from_slice::<T>(&v).ok()) {
            Some(value) => {
                self.cache_hits
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                Ok(value)
            }
            None => {
                let result = match fn_compute().await {
                    Ok(result) => result,
                    Err(err) => return Err(err.into()),
                };
                self.cache_misses
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let serialized = rmp_serde::to_vec(&result)?;
                self.client
                    .set_with_options(
                        key,
                        serialized,
                        SetCondition::None,
                        SetExpiration::Ex(ttl),
                        false,
                    )
                    .await?;

                Ok(result)
            }
        }
    }

    pub async fn ratelimit(
        &self,
        limit_identifier: impl AsRef<str>,
        limit: u64,
        limit_window: u64,
        client: impl AsRef<str>,
    ) -> Result<(), ApiResponse> {
        let key = compact_str::format_compact!(
            "ratelimit::{}::{}",
            limit_identifier.as_ref(),
            client.as_ref()
        );

        let now = chrono::Utc::now().timestamp();

        let expiry = self.client.expiretime(&*key).await.unwrap_or_default();
        let expire_unix: u64 = if expiry > now + 2 {
            expiry as u64
        } else {
            now as u64 + limit_window
        };

        let limit_used = self.client.get::<u64>(&*key).await.unwrap_or_default() + 1;
        self.client
            .set_with_options(
                &*key,
                limit_used,
                SetCondition::None,
                SetExpiration::Exat(expire_unix),
                false,
            )
            .await?;

        if limit_used >= limit {
            return Err(ApiResponse::error(&format!(
                "you are ratelimited, retry in {}s",
                expiry - now
            ))
            .with_status(axum::http::StatusCode::TOO_MANY_REQUESTS)
            .with_header("X-RateLimit-Limit", &limit.to_string())
            .with_header(
                "X-RateLimit-Remaining",
                &limit.saturating_sub(limit_used).to_string(),
            )
            .with_header("X-RateLimit-Reset", &expire_unix.to_string())
            .with_header("Retry-After", &(expiry - now).to_string()));
        }

        Ok(())
    }

    pub async fn clear_organization(&self, organization: i32) -> Result<(), anyhow::Error> {
        let keys: Vec<String> = self
            .client
            .keys(format!("organization::{organization}*"))
            .await?;

        if !keys.is_empty() {
            self.client.del(keys).await?;
        }

        Ok(())
    }
}
