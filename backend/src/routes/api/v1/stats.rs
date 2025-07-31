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

    #[derive(ToSchema, Serialize)]
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
                let data = sqlx::query(
                    r#"
                    SELECT 
                        COUNT(*), SUM(DISTINCT jar_size), SUM(DISTINCT zip_size)
                    FROM builds
                    UNION ALL
                    SELECT
                        value, pg_database_size(current_database()), 0
                    FROM counts
                    WHERE key = 'build_hashes'
                    UNION ALL
                    SELECT 
                        value, 0, 0
                    FROM counts
                    WHERE key = 'requests'
                    "#,
                )
                .fetch_all(state.database.read())
                .await
                .unwrap();

                Stats {
                    builds: data[0].get(0),
                    hashes: data[1].get(0),
                    requests: data[2].get(0),
                    size: StatsSize {
                        database: data[1].get(1),
                    },
                    total: StatsTotal {
                        jar_size: data[0].get(1),
                        zip_size: data[0].get(2),
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
