use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod history;

mod get {
    use crate::routes::GetState;
    use axum::extract::Path;
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct VersionStats {
        total: i64,
        unique_ips: i64,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        requests: IndexMap<String, VersionStats>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "version" = String,
            description = "The server version",
            example = "1.17.1",
        ),
    ))]
    pub async fn route(
        state: GetState,
        Path(version): Path<String>,
    ) -> axum::Json<serde_json::Value> {
        let requests = state
            .cache
            .cached(&format!("requests::versions::{version}"), 10800, || async {
                let data = sqlx::query(
                    r#"
                        SELECT
                            search_type AS type,
                            SUM(total_requests)::bigint AS total,
                            SUM(unique_ips)::bigint AS unique_ips
                        FROM mv_requests_stats
                        WHERE
                            request_type = 'builds'
                            AND search_version = $1
                        GROUP BY search_type
                        ORDER BY total DESC
                        "#,
                )
                .bind(version)
                .fetch_all(state.database.read())
                .await
                .unwrap();

                let mut requests = IndexMap::new();

                for row in data {
                    requests.insert(
                        row.get("type"),
                        VersionStats {
                            total: row.get("total"),
                            unique_ips: row.get("unique_ips"),
                        },
                    );
                }

                requests
            })
            .await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                requests,
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/history", history::router(state))
        .with_state(state.clone())
}
