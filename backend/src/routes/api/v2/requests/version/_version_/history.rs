use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use chrono::Datelike;
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct VersionStats {
        day: i16,
        total: i64,
        unique_ips: i64,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        requests: IndexMap<compact_str::CompactString, Vec<VersionStats>>,
    }

    #[utoipa::path(get, path = "/{year}/{month}", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "version" = String,
            description = "The server version",
            example = "1.17.1",
        ),
        (
            "year" = u16,
            description = "The year to get the version history for",
            minimum = 2024,
        ),
        (
            "month" = u8,
            description = "The month to get the version history for",
            minimum = 1,
            maximum = 12,
        ),
    ))]
    pub async fn route(
        state: GetState,
        Path((version, year, month)): Path<(String, u16, u8)>,
    ) -> ApiResponseResult {
        if year < 2024 || year > chrono::Utc::now().year() as u16 {
            return ApiResponse::error("invalid year")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        if !(1..=12).contains(&month) {
            return ApiResponse::error("invalid month")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let start = chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, 1).unwrap();
        let end = {
            let next_month = if month == 12 {
                chrono::NaiveDate::from_ymd_opt(year as i32 + 1, 1, 1).unwrap()
            } else {
                chrono::NaiveDate::from_ymd_opt(year as i32, month as u32 + 1, 1).unwrap()
            };

            next_month.pred_opt().unwrap()
        };

        let requests = state
            .cache
            .cached(
                &format!("requests::versions::{version}::history::{start}::{end}"),
                10800,
                || async {
                    let data = sqlx::query(
                        r#"
                        SELECT
                            search_type AS type,
                            day::smallint AS day,
                            SUM(total_requests)::bigint AS total,
                            SUM(unique_ips)::bigint AS unique_ips
                        FROM ch_request_stats_daily
                        WHERE
                            request_type = 'builds'
                            AND search_version = $1
                            AND date_only >= $2::date
                            AND date_only <= $3::date
                        GROUP BY day, search_type
                        ORDER BY total DESC
                        "#,
                    )
                    .bind(version)
                    .bind(start)
                    .bind(end)
                    .fetch_all(state.database.read())
                    .await?;

                    let mut requests = IndexMap::new();
                    for row in &data {
                        let r#type = row.try_get::<compact_str::CompactString, _>("type")?;
                        if !requests.contains_key(&r#type) {
                            let mut stats = Vec::with_capacity(end.day() as usize);

                            for i in 0..end.day() {
                                stats.push(VersionStats {
                                    day: i as i16 + 1,
                                    total: 0,
                                    unique_ips: 0,
                                });
                            }

                            requests.insert(r#type, stats);
                        }
                    }

                    for row in data {
                        let r#type = row.try_get::<compact_str::CompactString, _>("type")?;
                        let day = row.try_get::<i16, _>("day")? as usize - 1;

                        let entry = requests.get_mut(&r#type).unwrap().get_mut(day).unwrap();
                        entry.total = row.try_get("total")?;
                        entry.unique_ips = row.try_get("unique_ips")?;
                    }

                    Ok::<_, anyhow::Error>(requests)
                },
            )
            .await?;

        ApiResponse::json(Response {
            success: true,
            requests,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
