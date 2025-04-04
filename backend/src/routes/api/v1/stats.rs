use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::routes::GetState;
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    struct StatsSize {
        database: i64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct StatsTotal {
        jar_size: i64,
        zip_size: i64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Stats {
        builds: i64,
        hashes: i64,
        requests: i64,

        #[schema(inline)]
        size: StatsSize,
        #[schema(inline)]
        total: StatsTotal,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        stats: Stats,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState) -> axum::Json<serde_json::Value> {
        let stats = state
            .cache
            .cached("stats::all", 3600, || async {
                let (hashes, requests, builds) = tokio::join!(
                    sqlx::query(
                        r#"
                        SELECT
                            COUNT(*)
                        FROM build_hashes
                        "#
                    )
                    .fetch_one(state.database.read()),
                    sqlx::query(
                        r#"
                        SELECT
                            COUNT(*),
                            pg_database_size(current_database())
                        FROM requests
                        "#
                    )
                    .fetch_one(state.database.read()),
                    sqlx::query(
                        r#"
                        SELECT
                            COUNT(*),
                            SUM(DISTINCT jar_size),
                            SUM(DISTINCT zip_size)
                        FROM builds
                        "#
                    )
                    .fetch_one(state.database.read())
                );

                let (hashes, requests, builds) =
                    (hashes.unwrap(), requests.unwrap(), builds.unwrap());

                Stats {
                    builds: builds.get(0),
                    hashes: hashes.get(0),
                    requests: requests.get(0),
                    size: StatsSize {
                        database: requests.get(1),
                    },
                    total: StatsTotal {
                        jar_size: builds.get(1),
                        zip_size: builds.get(2),
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
        .with_state(state.clone())
}
