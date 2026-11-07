use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::ServerType,
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
    struct DayStats {
        day: i16,
        builds: i64,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        builds: IndexMap<ServerType, Vec<DayStats>>,
    }

    #[utoipa::path(get, path = "/{year}/{month}", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "year" = u16,
            description = "The year to get the build history for",
            minimum = 2009,
        ),
        (
            "month" = u8,
            description = "The month to get the build history for",
            minimum = 1,
            maximum = 12,
        ),
    ))]
    pub async fn route(state: GetState, Path((year, month)): Path<(u16, u8)>) -> ApiResponseResult {
        if year < 2009 || year > chrono::Utc::now().year() as u16 {
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
        let next_month = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year as i32 + 1, 1, 1).unwrap()
        } else {
            chrono::NaiveDate::from_ymd_opt(year as i32, month as u32 + 1, 1).unwrap()
        };
        let days = next_month.pred_opt().unwrap().day();

        let builds = state
            .cache
            .cached(
                &format!("stats::builds::history::{start}"),
                10800,
                || async {
                    let data = sqlx::query(
                        r#"
                        SELECT
                            builds.type,
                            EXTRACT(DAY FROM builds.created)::smallint AS day,
                            COUNT(*) AS builds
                        FROM builds
                        WHERE
                            builds.created >= $1
                            AND builds.created < $2
                        GROUP BY builds.type, day
                        ORDER BY builds DESC
                        "#,
                    )
                    .bind(start)
                    .bind(next_month)
                    .fetch_all(state.database.read())
                    .await?;

                    let mut builds: IndexMap<ServerType, Vec<DayStats>> = IndexMap::new();
                    for row in data {
                        let r#type = row.try_get::<ServerType, _>("type")?;
                        let day = row.try_get::<i16, _>("day")? as usize - 1;

                        let stats = builds.entry(r#type).or_insert_with(|| {
                            (1..=days as i16)
                                .map(|day| DayStats { day, builds: 0 })
                                .collect()
                        });
                        stats[day].builds = row.try_get("builds")?;
                    }

                    Ok::<_, anyhow::Error>(builds)
                },
            )
            .await?;

        ApiResponse::new_serialized(Response {
            success: true,
            builds,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
