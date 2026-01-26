use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{r#type::ServerType, version::Version},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use chrono::Datelike;
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    struct TotalStats {
        jar: i64,
        zip: i64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct AverageStats {
        jar: f64,
        zip: f64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Size {
        #[schema(inline)]
        total: TotalStats,

        #[schema(inline)]
        average: AverageStats,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Stats {
        day: i16,
        buids: i64,

        #[schema(inline)]
        size: Size,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        stats: Vec<Stats>,
    }

    #[utoipa::path(get, path = "/{year}/{month}", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
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
        Path((r#type, version, year, month)): Path<(ServerType, String, u16, u8)>,
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

        let location = Version::location(&state.database, &state.cache, r#type, &version).await?;

        if let Some(location) = location {
            let start = chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, 1).unwrap();
            let end = {
                let next_month = if month == 12 {
                    chrono::NaiveDate::from_ymd_opt(year as i32 + 1, 1, 1).unwrap()
                } else {
                    chrono::NaiveDate::from_ymd_opt(year as i32, month as u32 + 1, 1).unwrap()
                };

                next_month.pred_opt().unwrap()
            };

            let stats = state
                .cache
                .cached(
                    &format!("stats::types::{type}::{version}::history::{start}::{end}"),
                    10800,
                    || async {
                        let data = sqlx::query(&format!(
                            r#"
                            SELECT
                                COUNT(*) AS builds,
                                EXTRACT(DAY FROM builds.created)::smallint AS day,
                                SUM({} builds.jar_size) AS jar_total,
                                SUM(builds.zip_size) AS zip_total,
                                AVG(builds.jar_size)::FLOAT8 AS jar_average,
                                AVG(builds.zip_size)::FLOAT8 AS zip_average
                            FROM builds
                            WHERE
                                builds.type = $1
                                AND builds.{} = $2
                                AND builds.created >= $3
                                AND builds.created <= $4
                            GROUP BY day
                            ORDER BY day ASC
                            "#,
                            if r#type == ServerType::Fabric {
                                ""
                            } else {
                                "DISTINCT"
                            },
                            location
                        ))
                        .bind(r#type)
                        .bind(version)
                        .bind(start)
                        .bind(end)
                        .fetch_all(state.database.read())
                        .await?;

                        let mut stats = Vec::with_capacity(end.day() as usize);
                        for i in 0..end.day() {
                            stats.push(Stats {
                                day: i as i16 + 1,
                                buids: 0,
                                size: Size {
                                    total: TotalStats { jar: 0, zip: 0 },
                                    average: AverageStats { jar: 0.0, zip: 0.0 },
                                },
                            });
                        }

                        for row in data {
                            let entry = stats
                                .get_mut(row.try_get::<i16, _>("day")? as usize)
                                .unwrap();

                            entry.buids = row.try_get("builds")?;
                            entry.size.total.jar = row.try_get("jar_total").unwrap_or_default();
                            entry.size.total.zip = row.try_get("zip_total").unwrap_or_default();
                            entry.size.average.jar = row.try_get("jar_average").unwrap_or_default();
                            entry.size.average.zip = row.try_get("zip_average").unwrap_or_default();
                        }

                        Ok::<_, anyhow::Error>(stats)
                    },
                )
                .await?;

            ApiResponse::new_serialized(Response {
                success: true,
                stats,
            })
            .ok()
        } else {
            ApiResponse::error("version not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
