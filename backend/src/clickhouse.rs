use colored::Colorize;
use std::sync::Arc;

pub struct Clickhouse {
    client: clickhouse::Client,
}

impl Clickhouse {
    pub async fn new(env: Arc<crate::env::Env>) -> Self {
        let start = std::time::Instant::now();

        let instance = Self {
            client: clickhouse::Client::default()
                .with_url(&env.clickhouse_url)
                .with_database(&env.clickhouse_database)
                .with_user(&env.clickhouse_username)
                .with_password(&env.clickhouse_password),
        };

        let version: String = instance
            .client
            .query("SELECT version()")
            .fetch_one()
            .await
            .unwrap();

        crate::logger::log(
            crate::logger::LoggerLevel::Info,
            format!(
                "{} connected {}",
                "clickhouse".bright_red(),
                format!(
                    "(clickhouse@{}, {}ms)",
                    version.bright_black(),
                    start.elapsed().as_millis()
                )
                .bright_black()
            ),
        );

        instance
    }
}
