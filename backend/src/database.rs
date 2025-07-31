use colored::Colorize;
use sqlx::{Row, postgres::PgPoolOptions};
use std::sync::Arc;

#[inline]
async fn update_count(pool: &sqlx::PgPool, key: &str, value: i64) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO counts (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = counts.value + $2")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;

    Ok(())
}

#[inline]
async fn reset_count(pool: &sqlx::PgPool, key: &str, value: i64) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO counts (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = $2")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;

    Ok(())
}

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
                    .min_connections(5)
                    .max_connections(10)
                    .test_before_acquire(false)
                    .connect(url)
                    .await
                    .unwrap(),

                None => PgPoolOptions::new()
                    .min_connections(10)
                    .max_connections(30)
                    .test_before_acquire(false)
                    .connect(&env.database_url)
                    .await
                    .unwrap(),
            },
            read: if env.database_url_primary.is_some() {
                Some(
                    PgPoolOptions::new()
                        .min_connections(10)
                        .max_connections(30)
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

                    let (builds, build_hashes) = match tokio::try_join!(
                        sqlx::query("SELECT COUNT(*) FROM builds").fetch_one(&writer),
                        sqlx::query("SELECT COUNT(*) FROM build_hashes").fetch_one(&writer)
                    ) {
                        Ok((b, h)) => (b, h),
                        Err(e) => {
                            crate::logger::log(
                                crate::logger::LoggerLevel::Error,
                                format!(
                                    "{} failed to refresh counts: {}",
                                    "database".bright_cyan(),
                                    e.to_string().bright_red()
                                ),
                            );
                            continue;
                        }
                    };

                    let builds_count: i64 = builds.get(0);
                    let build_hashes_count: i64 = build_hashes.get(0);

                    match tokio::try_join!(
                        reset_count(&writer, "builds", builds_count),
                        reset_count(&writer, "build_hashes", build_hashes_count)
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            crate::logger::log(
                                crate::logger::LoggerLevel::Error,
                                format!(
                                    "{} failed to update counts: {}",
                                    "database".bright_cyan(),
                                    e.to_string().bright_red()
                                ),
                            );
                            continue;
                        }
                    }

                    crate::logger::log(
                        crate::logger::LoggerLevel::Info,
                        format!(
                            "{} counts updated {}",
                            "database".bright_cyan(),
                            format!(
                                "(builds: {}, hashes: {})",
                                builds_count.to_string().bright_black(),
                                build_hashes_count.to_string().bright_black()
                            )
                            .bright_black()
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

    pub async fn update_count(&self, key: &str, value: i64) -> Result<(), sqlx::Error> {
        update_count(&self.write, key, value).await
    }
}
