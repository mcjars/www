use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::ServerType,
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
    struct TypeStats {
        total: i64,
        unique_ips: i64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Requests {
        day: i16,

        #[schema(inline)]
        root: TypeStats,

        #[schema(inline)]
        versions: IndexMap<String, TypeStats>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        requests: Vec<Requests>,
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

        let requests = state
            .cache
            .cached(
                &format!("requests::types::{type}::history::{start}::{end}"),
                10800,
                || async {
                    let data = sqlx::query(
                        r#"
                        SELECT
                            search_version AS version,
                            day::smallint AS day,
                            SUM(total_requests)::bigint AS total,
                            SUM(unique_ips)::bigint AS unique_ips
                        FROM mv_requests_stats_daily
                        WHERE
                            request_type = 'builds'
                            AND search_type = $1
                            AND date_only >= $2::date
                            AND date_only <= $3::date
                        GROUP BY day, search_version
                        ORDER BY total DESC
                        "#,
                    )
                    .bind(r#type.to_string())
                    .bind(start)
                    .bind(end)
                    .fetch_all(state.database.read())
                    .await
                    .unwrap();

                    let mut requests: Vec<Requests> = Vec::with_capacity(end.day() as usize);
                    for i in 0..end.day() {
                        requests.push(Requests {
                            day: i as i16 + 1,
                            root: TypeStats {
                                total: 0,
                                unique_ips: 0,
                            },
                            versions: IndexMap::new(),
                        });
                    }

                    for row in data {
                        let version = row.get::<Option<String>, _>("version");

                        if let Some(version) = version {
                            let day = row.get::<i16, _>("day") as usize - 1;

                            if !requests[day].versions.contains_key(&version) {
                                requests[day].versions.insert(
                                    version.clone(),
                                    TypeStats {
                                        total: 0,
                                        unique_ips: 0,
                                    },
                                );
                            }

                            let entry = requests[day].versions.get_mut(&version).unwrap();
                            entry.total = row.get::<i64, _>("total");
                            entry.unique_ips = row.get::<i64, _>("unique_ips");
                        } else {
                            let day = row.get::<i16, _>("day") as usize - 1;
                            requests[day].root = TypeStats {
                                total: row.get::<i64, _>("total"),
                                unique_ips: row.get::<i64, _>("unique_ips"),
                            };
                        }
                    }

                    requests
                },
            )
            .await;

        (
            StatusCode::OK,
            axum::Json(
                serde_json::to_value(&Response {
                    success: true,
                    requests,
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
