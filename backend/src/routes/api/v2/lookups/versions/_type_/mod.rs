use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod history;

mod get {
    use crate::{
        models::r#type::{SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER, ServerType},
        response::{ApiResponse, ApiResponseResult},
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
        versions: IndexMap<compact_str::CompactString, VersionStats>,
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
    pub async fn route(state: GetState, Path(r#type): Path<ServerType>) -> ApiResponseResult {
        let versions = state
            .cache
            .cached(&format!("lookups::versions::{type}"), 10800, || async {
                let column = if SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER.contains(&r#type) {
                    "project_version"
                } else {
                    "version"
                };

                let data = sqlx::query(&format!(
                    r#"
                    SELECT
                        build_{column}_id AS version,
                        SUM(total_requests)::bigint AS total,
                        SUM(unique_ips)::bigint AS unique_ips
                    FROM ch_request_stats
                    WHERE
                        request_type = 'lookup'
                        AND build_type = $1
                        AND build_{column}_id != ''
                    GROUP BY build_{column}_id
                    ORDER BY total DESC
                    "#
                ))
                .bind(r#type.to_string())
                .fetch_all(state.database.read())
                .await?;

                let mut versions = IndexMap::new();

                for row in data {
                    versions.insert(
                        row.try_get("version")?,
                        VersionStats {
                            total: row.try_get("total")?,
                            unique_ips: row.try_get("unique_ips")?,
                        },
                    );
                }

                Ok::<_, anyhow::Error>(versions)
            })
            .await?;

        ApiResponse::new_serialized(Response {
            success: true,
            versions,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/history", history::router(state))
        .with_state(state.clone())
}
