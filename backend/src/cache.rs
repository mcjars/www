use crate::env::RedisMode;
use colored::Colorize;
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
            "{} connected {}",
            "cache".bright_yellow(),
            format!("(redis@{}, {}ms)", version, start.elapsed().as_millis()).bright_black()
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

        match cached_value {
            Some(value) => {
                let result: T = serde_json::from_slice(value.as_bytes())?;
                self.cache_hits
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                Ok(result)
            }
            None => {
                let result = match fn_compute().await {
                    Ok(result) => result,
                    Err(err) => return Err(err.into()),
                };
                self.cache_misses
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let serialized = serde_json::to_vec(&result)?;
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
