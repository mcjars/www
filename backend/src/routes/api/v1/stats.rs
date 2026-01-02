use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        response::{ApiResponse, ApiResponseResult},
        routes::GetState,
    };
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
    pub async fn route(state: GetState) -> ApiResponseResult {
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
                .await?;

                Ok::<_, anyhow::Error>(Stats {
                    builds: data[0].try_get(0)?,
                    hashes: data[1].try_get(0)?,
                    requests: data[2].try_get(0)?,
                    size: StatsSize {
                        database: data[1].try_get(1)?,
                    },
                    total: StatsTotal {
                        jar_size: data[0].try_get(1)?,
                        zip_size: data[0].try_get(2)?,
                    },
                })
            })
            .await?;

        ApiResponse::json(Response {
            success: true,
            stats,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
