use colored::Colorize;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

pub struct Database {
    write: sqlx::PgPool,
    read: Option<sqlx::PgPool>,
}

impl Database {
    pub async fn new(env: Arc<crate::env::Env>) -> Self {
        let start = std::time::Instant::now();

        let instance = Self {
            write: match &env.database_url_primary {
                Some(url) => PgPoolOptions::new()
                    .min_connections(10)
                    .max_connections(60)
                    .test_before_acquire(false)
                    .connect(url)
                    .await
                    .unwrap(),

                None => PgPoolOptions::new()
                    .min_connections(10)
                    .max_connections(60)
                    .test_before_acquire(false)
                    .connect(&env.database_url)
                    .await
                    .unwrap(),
            },
            read: if env.database_url_primary.is_some() {
                Some(
                    PgPoolOptions::new()
                        .min_connections(10)
                        .max_connections(60)
                        .test_before_acquire(false)
                        .connect(&env.database_url)
                        .await
                        .unwrap(),
                )
            } else {
                None
            },
        };

        let version: (String,) = sqlx::query_as("SELECT split_part(version(), ' ', 4)")
            .fetch_one(instance.read())
            .await
            .unwrap();

        crate::logger::log(
            crate::logger::LoggerLevel::Info,
            format!(
                "{} connected {}",
                "database".bright_cyan(),
                format!(
                    "(postgres@{}, {}ms)",
                    version.0[..version.0.len() - 1].bright_black(),
                    start.elapsed().as_millis()
                )
                .bright_black()
            ),
        );

        if env.database_migrate {
            let writer = instance.write.clone();
            tokio::spawn(async move {
                let start = std::time::Instant::now();

                sqlx::migrate!("../database/migrations")
                    .run(&writer)
                    .await
                    .unwrap();

                crate::logger::log(
                    crate::logger::LoggerLevel::Info,
                    format!(
                        "{} migrated {}",
                        "database".bright_cyan(),
                        format!("({}ms)", start.elapsed().as_millis()).bright_black()
                    ),
                );
            });
        }

        if env.database_refresh {
            let writer = instance.write.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60 * 30)).await;

                    let start = std::time::Instant::now();

                    let (_, _) = tokio::join!(
                        sqlx::query("REFRESH MATERIALIZED VIEW mv_requests_stats").execute(&writer),
                        sqlx::query("REFRESH MATERIALIZED VIEW mv_requests_stats_daily")
                            .execute(&writer)
                    );

                    crate::logger::log(
                        crate::logger::LoggerLevel::Info,
                        format!(
                            "{} views refreshed {}",
                            "database".bright_cyan(),
                            format!("({}ms)", start.elapsed().as_millis()).bright_black()
                        ),
                    );
                }
            });
        }

        instance
    }

    #[inline]
    pub fn write(&self) -> &sqlx::PgPool {
        &self.write
    }

    #[inline]
    pub fn read(&self) -> &sqlx::PgPool {
        self.read.as_ref().unwrap_or(&self.write)
    }
}
