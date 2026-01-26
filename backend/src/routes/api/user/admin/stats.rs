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
    struct StatsRequests {
        total: u64,

        minute: u64,
        hour: u64,
        day: u64,
        week: u64,
        month: u64,
        year: u64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    struct StatsInternal {
        idle_read_connections: usize,
        idle_write_connections: usize,

        cache_hits: usize,
        cache_misses: usize,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Stats {
        organizations: i64,
        users: i64,
        sessions: i64,
        webhooks: i64,

        #[schema(inline)]
        requests: StatsRequests,
        #[schema(inline)]
        internal: StatsInternal,
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
            .cached("stats::admin::all", 60, || async {
                let (data, requests_data) = tokio::try_join!(
                    async {
                        let data = sqlx::query(
                            r#"
                            SELECT COUNT(*)
                            FROM organizations
                            UNION ALL
                            SELECT COUNT(*)
                            FROM users
                            UNION ALL
                            SELECT COUNT(*)
                            FROM user_sessions
                            UNION ALL
                            SELECT COUNT(*)
                            FROM webhooks
                            "#,
                        )
                        .fetch_all(state.database.read())
                        .await?;

                        Ok::<_, anyhow::Error>(data)
                    },
                    async {
                        let requests_data = state.clickhouse
                            .client()
                            .query(
                                r#"
                                SELECT
                                    COUNT(*), 
                                    SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 minute' THEN 1 ELSE 0 END),
                                    SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 hour' THEN 1 ELSE 0 END),
                                    SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 day' THEN 1 ELSE 0 END),
                                    SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 week' THEN 1 ELSE 0 END),
                                    SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 month' THEN 1 ELSE 0 END),
                                    SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 year' THEN 1 ELSE 0 END)
                                FROM requests
                                "#
                            )
                            .fetch_one::<(u64, u64, u64, u64, u64, u64, u64)>()
                            .await?;

                        Ok(requests_data)
                    },
                )?;

                Ok::<_, anyhow::Error>(Stats {
                    organizations: data[0].try_get(0)?,
                    users: data[1].try_get(0)?,
                    sessions: data[2].try_get(0)?,
                    webhooks: data[3].try_get(0)?,
                    requests: StatsRequests {
                        total: requests_data.0,

                        minute: requests_data.1,
                        hour: requests_data.2,
                        day: requests_data.3,
                        week: requests_data.4,
                        month: requests_data.5,
                        year: requests_data.6,
                    },
                    internal: StatsInternal {
                        idle_read_connections: 0,
                        idle_write_connections: 0,

                        cache_hits: 0,
                        cache_misses: 0,
                    },
                })
            })
            .await?;

        ApiResponse::new_serialized(Response {
            success: true,
            stats: Stats {
                internal: StatsInternal {
                    idle_read_connections: state.database.read().num_idle(),
                    idle_write_connections: state.database.write().num_idle(),

                    cache_hits: state.cache.cache_hits(),
                    cache_misses: state.cache.cache_misses(),
                },
                ..stats
            },
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
