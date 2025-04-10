use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::routes::GetState;
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    struct StatsRequests {
        total: i64,

        minute: i64,
        hour: i64,
        day: i64,
        week: i64,
        month: i64,
        year: i64,
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
    pub async fn route(state: GetState) -> axum::Json<serde_json::Value> {
        let stats = state
            .cache
            .cached("stats::admin::all", 60, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        COUNT(*), 0, 0, 0, 0, 0, 0
                    FROM organizations
                    UNION ALL
                    SELECT
                        COUNT(*), 0, 0, 0, 0, 0, 0
                    FROM users
                    UNION ALL
                    SELECT
                        COUNT(*), 0, 0, 0, 0, 0, 0
                    FROM user_sessions
                    UNION ALL
                    SELECT
                        COUNT(*), 0, 0, 0, 0, 0, 0
                    FROM webhooks
                    UNION ALL
                    SELECT
                        COUNT(*), 
                        SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 minute' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 hour' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 day' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 week' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 month' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN requests.created > NOW() - INTERVAL '1 year' THEN 1 ELSE 0 END)
                    FROM requests
                    "#,
                )
                .fetch_all(state.database.read())
                .await
                .unwrap();

                Stats {
                    organizations: data[0].get(0),
                    users: data[1].get(0),
                    sessions: data[2].get(0),
                    webhooks: data[3].get(0),
                    requests: StatsRequests {
                        total: data[4].get(0),

                        minute: data[4].get(1),
                        hour: data[4].get(2),
                        day: data[4].get(3),
                        week: data[4].get(4),
                        month: data[4].get(5),
                        year: data[4].get(6),
                    },
                    internal: StatsInternal {
                        idle_read_connections: 0,
                        idle_write_connections: 0,

                        cache_hits: 0,
                        cache_misses: 0,
                    },
                }
            })
            .await;

        axum::Json(
            serde_json::to_value(&Response {
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
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
