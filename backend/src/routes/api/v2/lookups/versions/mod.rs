use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _type_;
mod history;

mod get {
    use crate::{
        response::{ApiResponse, ApiResponseResult},
        routes::GetState,
    };
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
    ))]
    pub async fn route(state: GetState) -> ApiResponseResult {
        let versions = state
            .cache
            .cached("lookups::versions::all", 10800, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        build_version_id AS version,
                        SUM(total_requests)::bigint AS total,
                        SUM(unique_ips)::bigint AS unique_ips
                    FROM ch_request_stats
                    WHERE
                        request_type = 'lookup'
                        AND build_version_id != ''
                    GROUP BY build_version_id
                    ORDER BY total DESC
                    "#,
                )
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

        ApiResponse::json(Response {
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
        .nest("/{type}", _type_::router(state))
        .with_state(state.clone())
}
