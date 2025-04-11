use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::ServerType,
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
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
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
        Path((r#type, year, month)): Path<(ServerType, u16, u8)>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        if year < 2024 || year > chrono::Utc::now().year() as u16 {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiError::new(&["Invalid year"]).to_value()),
            );
        }

        if !(1..=12).contains(&month) {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiError::new(&["Invalid month"]).to_value()),
            );
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

        let stats = state
            .cache
            .cached(
                &format!("stats::types::{}::all::history::{}::{}", r#type, start, end),
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
                            AND builds.created >= $2
                            AND builds.created <= $3
                        GROUP BY day
                        ORDER BY day ASC
                        "#,
                        if r#type == ServerType::Fabric {
                            ""
                        } else {
                            "DISTINCT"
                        }
                    ))
                    .bind(r#type)
                    .bind(start)
                    .bind(end)
                    .fetch_all(state.database.read())
                    .await
                    .unwrap();

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
                        let entry = stats.get_mut(row.get::<i16, _>("day") as usize).unwrap();

                        entry.buids = row.get("builds");
                        entry.size.total.jar = row.try_get("jar_total").unwrap_or_default();
                        entry.size.total.zip = row.try_get("zip_total").unwrap_or_default();
                        entry.size.average.jar = row.try_get("jar_average").unwrap_or_default();
                        entry.size.average.zip = row.try_get("zip_average").unwrap_or_default();
                    }

                    stats
                },
            )
            .await;

        (
            StatusCode::OK,
            axum::Json(
                serde_json::to_value(&Response {
                    success: true,
                    stats,
                })
                .unwrap(),
            ),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
