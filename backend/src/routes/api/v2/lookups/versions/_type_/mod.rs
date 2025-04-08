use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod history;

mod get {
    use crate::{
        models::r#type::{SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER, ServerType},
        routes::GetState,
    };
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
        versions: IndexMap<String, VersionStats>,
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
        let versions = state
            .cache
            .cached(&format!("lookups::versions::{}", r#type), 10800, || async {
                let column = if SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER.contains(&r#type) {
                    "project_version"
                } else {
                    "version"
                };

                let data = sqlx::query(&format!(
                    r#"
                    SELECT
                        build_{}_id AS version,
                        SUM(total_requests)::bigint AS total,
                        SUM(unique_ips)::bigint AS unique_ips
                    FROM mv_requests_stats
                    WHERE
                        request_type = 'lookup'
                        AND build_type = $1
                        AND build_{}_id IS NOT NULL
                    GROUP BY build_{}_id
                    ORDER BY total DESC
                    "#,
                    column, column, column
                ))
                .bind(r#type.to_string())
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
        .with_state(state.clone())
}
