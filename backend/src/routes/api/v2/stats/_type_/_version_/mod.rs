use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod history;

mod get {
    use crate::{
        models::{r#type::ServerType, version::Version},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
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
        buids: i64,

        #[schema(inline)]
        size: Size,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        stats: Stats,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
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
    ))]
    pub async fn route(
        state: GetState,
        Path((r#type, version)): Path<(ServerType, String)>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let location = Version::location(&state.database, &state.cache, r#type, &version).await;

        if let Some(location) = location {
            let stats = state
                .cache
                .cached(
                    &format!("stats::types::{}::{}", r#type, version),
                    10800,
                    || async {
                        let data = sqlx::query(&format!(
                            r#"
                            SELECT
                                COUNT(*) AS builds,
                                SUM({} builds.jar_size) AS jar_total,
                                SUM(builds.zip_size) AS zip_total,
                                AVG(builds.jar_size)::FLOAT8 AS jar_average,
                                AVG(builds.zip_size)::FLOAT8 AS zip_average
                            FROM builds
                            WHERE
                                builds.type = $1::server_type
                                AND builds.{} = $2
                            "#,
                            if r#type == ServerType::Fabric {
                                ""
                            } else {
                                "DISTINCT"
                            },
                            location
                        ))
                        .bind(r#type.to_string())
                        .bind(version)
                        .fetch_one(state.database.read())
                        .await
                        .unwrap();

                        Stats {
                            buids: data.get("builds"),
                            size: Size {
                                total: TotalStats {
                                    jar: data.try_get("jar_total").unwrap_or_default(),
                                    zip: data.try_get("zip_total").unwrap_or_default(),
                                },
                                average: AverageStats {
                                    jar: data.try_get("jar_average").unwrap_or_default(),
                                    zip: data.try_get("zip_average").unwrap_or_default(),
                                },
                            },
                        }
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
        } else {
            (
                StatusCode::NOT_FOUND,
                axum::Json(ApiError::new(&["version not found"]).to_value()),
            )
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/history", history::router(state))
        .with_state(state.clone())
}
