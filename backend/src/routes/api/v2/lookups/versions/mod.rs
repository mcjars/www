use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _type_;
mod history;

mod get {
    use crate::routes::GetState;
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

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        versions: IndexMap<String, VersionStats>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState) -> axum::Json<serde_json::Value> {
        let versions = state
            .cache
            .cached("lookups::versions::all", 10800, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        build_version_id AS version,
                        SUM(total_requests)::bigint AS total,
                        SUM(unique_ips)::bigint AS unique_ips
                    FROM mv_requests_stats
                    WHERE
                        request_type = 'lookup'
                        AND build_version_id IS NOT NULL
                    GROUP BY build_version_id
                    ORDER BY total DESC
                    "#,
                )
                .fetch_all(state.database.read())
                .await
                .unwrap();

                let mut versions = IndexMap::new();

                for row in data {
                    versions.insert(
                        row.get("version"),
                        VersionStats {
                            total: row.get("total"),
                            unique_ips: row.get("unique_ips"),
                        },
                    );
                }

                versions
            })
            .await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                versions,
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/history", history::router(state))
        .nest("/{type}", _type_::router(state))
        .with_state(state.clone())
}
