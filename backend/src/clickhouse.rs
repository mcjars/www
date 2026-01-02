use clickhouse::query::RowCursor;
use colored::Colorize;
use serde::Deserialize;
use std::sync::Arc;

pub struct Clickhouse {
    client: clickhouse::Client,
}

impl Clickhouse {
    pub async fn new(env: Arc<crate::env::Env>, database: Arc<crate::database::Database>) -> Self {
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

        tracing::info!(
            "{} connected {}",
            "clickhouse".bright_red(),
            format!(
                "(clickhouse@{}, {}ms)",
                version.bright_black(),
                start.elapsed().as_millis()
            )
            .bright_black()
        );

        if env.clickhouse_refresh {
            let client = instance.client.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_mins(30)).await;

                    let run_global = async || -> Result<(), anyhow::Error> {
                        let start = std::time::Instant::now();

                        #[derive(Deserialize, clickhouse::Row)]
                        struct GlobalStatsRow {
                            request_type: Option<String>,
                            search_type: Option<String>,
                            search_version: Option<String>,
                            build_type: Option<String>,
                            build_version_id: Option<String>,
                            build_project_version_id: Option<String>,
                            total_requests: u64,
                            unique_ips: u64,
                        }

                        let mut global_stats: RowCursor<GlobalStatsRow> = client
                            .query(
                                r#"
                                SELECT
                                    JSONExtractString(data, 'type') AS request_type,
                                    JSONExtractString(data, 'search', 'type') AS search_type,
                                    JSONExtractString(data, 'search', 'version') AS search_version,
                                    JSONExtractString(data, 'build', 'type') AS build_type,
                                    JSONExtractString(data, 'build', 'versionId') AS build_version_id,
                                    JSONExtractString(data, 'build', 'projectVersionId') AS build_project_version_id,
                                    count(*) AS total_requests,
                                    uniqExact(ip) AS unique_ips
                                FROM requests
                                WHERE
                                    status = 200
                                    AND data IS NOT NULL
                                    AND path NOT LIKE '%tracking=nostats%'
                                GROUP BY
                                    request_type,
                                    search_type,
                                    search_version,
                                    build_type,
                                    build_version_id,
                                    build_project_version_id
                                "#
                            )
                            .fetch()?;

                        let flush_buffer =
                            async |buffer: &[GlobalStatsRow]| -> Result<(), anyhow::Error> {
                                if buffer.is_empty() {
                                    return Ok(());
                                }

                                let request_types: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.request_type.as_deref()).collect();
                                let search_types: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.search_type.as_deref()).collect();
                                let search_versions: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.search_version.as_deref()).collect();
                                let build_types: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.build_type.as_deref()).collect();
                                let build_version_ids: Vec<Option<&str>> = buffer
                                    .iter()
                                    .map(|r| r.build_version_id.as_deref())
                                    .collect();
                                let build_project_version_ids: Vec<Option<&str>> = buffer
                                    .iter()
                                    .map(|r| r.build_project_version_id.as_deref())
                                    .collect();
                                let total_requests: Vec<i64> =
                                    buffer.iter().map(|r| r.total_requests as i64).collect();
                                let unique_ips: Vec<i64> =
                                    buffer.iter().map(|r| r.unique_ips as i64).collect();

                                sqlx::query(
                                    r#"
                                    INSERT INTO ch_request_stats (
                                        request_type, search_type, search_version, build_type, build_version_id, build_project_version_id, 
                                        total_requests, unique_ips
                                    )
                                    SELECT * FROM UNNEST(
                                        $1::text[],
                                        $2::text[],
                                        $3::text[],
                                        $4::text[],
                                        $5::text[],
                                        $6::text[],
                                        $7::bigint[],
                                        $8::bigint[]
                                    )
                                    ON CONFLICT (request_type, search_type, search_version, build_type, build_version_id, build_project_version_id)
                                    DO UPDATE SET
                                        total_requests = EXCLUDED.total_requests,
                                        unique_ips = EXCLUDED.unique_ips
                                    "#,
                                )
                                .bind(&request_types)
                                .bind(&search_types)
                                .bind(&search_versions)
                                .bind(&build_types)
                                .bind(&build_version_ids)
                                .bind(&build_project_version_ids)
                                .bind(&total_requests)
                                .bind(&unique_ips)
                                .execute(database.write())
                                .await?;

                                Ok(())
                            };

                        let mut row_buffer = Vec::new();
                        row_buffer.reserve_exact(2048);
                        while let Some(row) = global_stats.next().await? {
                            if row_buffer.len() < 2048 {
                                row_buffer.push(row);
                            } else {
                                flush_buffer(&row_buffer).await?;
                                row_buffer.clear();
                                row_buffer.push(row);
                            }
                        }
                        flush_buffer(&row_buffer).await?;

                        tracing::info!(
                            "{} global stats refreshed in {}ms",
                            "clickhouse".bright_red(),
                            start.elapsed().as_millis()
                        );

                        Ok(())
                    };

                    if let Err(err) = run_global().await {
                        tracing::error!("failed to refresh global clickhouse stats: {:?}", err);
                        sentry_anyhow::capture_anyhow(&err);
                    }

                    let run_daily = async || -> Result<(), anyhow::Error> {
                        let start = std::time::Instant::now();

                        #[derive(Deserialize, clickhouse::Row)]
                        struct DailyStatsRow {
                            request_type: Option<String>,
                            search_type: Option<String>,
                            search_version: Option<String>,
                            build_type: Option<String>,
                            build_version_id: Option<String>,
                            build_project_version_id: Option<String>,
                            #[serde(with = "clickhouse::serde::chrono::date")]
                            date_only: chrono::NaiveDate,
                            day: u8,
                            total_requests: u64,
                            unique_ips: u64,
                        }

                        let mut global_stats: RowCursor<DailyStatsRow> = client
                            .query(
                                r#"
                                SELECT
                                    JSONExtractString(data, 'type') AS request_type,
                                    JSONExtractString(data, 'search', 'type') AS search_type,
                                    JSONExtractString(data, 'search', 'version') AS search_version,
                                    JSONExtractString(data, 'build', 'type') AS build_type,
                                    JSONExtractString(data, 'build', 'versionId') AS build_version_id,
                                    JSONExtractString(data, 'build', 'projectVersionId') AS build_project_version_id,
                                    toDate(created) AS date_only,
                                    toDayOfMonth(created) AS day,
                                    count(*) AS total_requests,
                                    uniqExact(ip) AS unique_ips
                                FROM requests
                                WHERE 
                                    status = 200 
                                    AND data IS NOT NULL 
                                    AND path NOT LIKE '%tracking=nostats%'
                                GROUP BY 
                                    request_type, 
                                    search_type, 
                                    search_version, 
                                    build_type, 
                                    build_version_id, 
                                    build_project_version_id, 
                                    date_only, 
                                    day
                                ORDER BY 
                                    date_only DESC
                                "#
                            )
                            .fetch()?;

                        let flush_buffer =
                            async |buffer: &[DailyStatsRow]| -> Result<(), anyhow::Error> {
                                if buffer.is_empty() {
                                    return Ok(());
                                }

                                let request_types: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.request_type.as_deref()).collect();
                                let search_types: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.search_type.as_deref()).collect();
                                let search_versions: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.search_version.as_deref()).collect();
                                let build_types: Vec<Option<&str>> =
                                    buffer.iter().map(|r| r.build_type.as_deref()).collect();
                                let build_version_ids: Vec<Option<&str>> = buffer
                                    .iter()
                                    .map(|r| r.build_version_id.as_deref())
                                    .collect();
                                let build_project_version_ids: Vec<Option<&str>> = buffer
                                    .iter()
                                    .map(|r| r.build_project_version_id.as_deref())
                                    .collect();
                                let date_only: Vec<chrono::NaiveDate> =
                                    buffer.iter().map(|r| r.date_only).collect();
                                let day: Vec<i16> = buffer.iter().map(|r| r.day as i16).collect();
                                let total_requests: Vec<i64> =
                                    buffer.iter().map(|r| r.total_requests as i64).collect();
                                let unique_ips: Vec<i64> =
                                    buffer.iter().map(|r| r.unique_ips as i64).collect();

                                sqlx::query(
                                    r#"
                                    INSERT INTO ch_request_stats_daily (
                                        request_type, search_type, search_version, build_type, build_version_id, build_project_version_id, 
                                        date_only, day, 
                                        total_requests, unique_ips
                                    )
                                    SELECT * FROM UNNEST(
                                        $1::text[], 
                                        $2::text[], 
                                        $3::text[], 
                                        $4::text[], 
                                        $5::text[], 
                                        $6::text[], 
                                        $7::date[], 
                                        $8::smallint[], 
                                        $9::bigint[], 
                                        $10::bigint[]
                                    )
                                    ON CONFLICT (request_type, search_type, search_version, build_type, build_version_id, build_project_version_id, date_only)
                                    DO UPDATE SET
                                        total_requests = EXCLUDED.total_requests,
                                        unique_ips = EXCLUDED.unique_ips
                                    "#,
                                )
                                .bind(&request_types)
                                .bind(&search_types)
                                .bind(&search_versions)
                                .bind(&build_types)
                                .bind(&build_version_ids)
                                .bind(&build_project_version_ids)
                                .bind(&date_only)
                                .bind(&day)
                                .bind(&total_requests)
                                .bind(&unique_ips)
                                .execute(database.write())
                                .await?;

                                Ok(())
                            };

                        let mut row_buffer = Vec::new();
                        row_buffer.reserve_exact(2048);
                        while let Some(row) = global_stats.next().await? {
                            if row_buffer.len() < 2048 {
                                row_buffer.push(row);
                            } else {
                                flush_buffer(&row_buffer).await?;
                                row_buffer.clear();
                                row_buffer.push(row);
                            }
                        }
                        flush_buffer(&row_buffer).await?;

                        tracing::info!(
                            "{} daily stats refreshed in {}ms",
                            "clickhouse".bright_red(),
                            start.elapsed().as_millis()
                        );

                        Ok(())
                    };

                    if let Err(err) = run_daily().await {
                        tracing::error!("failed to refresh daily clickhouse stats: {:?}", err);
                        sentry_anyhow::capture_anyhow(&err);
                    }
                }
            });
        }

        instance
    }

    #[inline]
    pub fn client(&self) -> &clickhouse::Client {
        &self.client
    }
}
