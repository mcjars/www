use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _version_;
mod history;

mod get {
    use crate::{models::r#type::ServerType, routes::GetState};
    use axum::extract::Path;
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

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        stats: Stats,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
    ))]
    pub async fn route(
        state: GetState,
        Path(r#type): Path<ServerType>,
    ) -> axum::Json<serde_json::Value> {
        let stats = state
            .cache
            .cached(&format!("stats::types::{type}::all"), 10800, || async {
                let data = sqlx::query(&format!(
                    r#"
                    SELECT
                        COUNT(*) AS builds,
                        SUM({} builds.jar_size) AS jar_total,
                        SUM(builds.zip_size) AS zip_total,
                        AVG(builds.jar_size)::FLOAT8 AS jar_average,
                        AVG(builds.zip_size)::FLOAT8 AS zip_average
                    FROM builds
                    WHERE builds.type = $1
                    "#,
                    if r#type == ServerType::Fabric {
                        ""
                    } else {
                        "DISTINCT"
                    }
                ))
                .bind(r#type)
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
            })
            .await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                stats,
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/history", history::router(state))
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
