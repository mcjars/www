use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod history;

mod get {
    use crate::{
        models::r#type::ServerType,
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
    struct TypeStats {
        total: i64,
        unique_ips: i64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Requests {
        #[schema(inline)]
        root: TypeStats,

        #[schema(inline)]
        versions: IndexMap<compact_str::CompactString, TypeStats>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        requests: Requests,
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
        let requests = state
            .cache
            .cached(&format!("requests::types::{type}"), 10800, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        search_version AS version,
                        SUM(total_requests)::bigint AS total,
                        SUM(unique_ips)::bigint AS unique_ips
                    FROM ch_request_stats
                    WHERE
                        request_type = 'builds'
                        AND search_type = $1
                    GROUP BY search_version
                    ORDER BY total DESC
                    "#,
                )
                .bind(r#type.to_string())
                .fetch_all(state.database.read())
                .await?;

                let mut requests = Requests {
                    root: TypeStats {
                        total: 0,
                        unique_ips: 0,
                    },
                    versions: IndexMap::new(),
                };

                for row in data {
                    let version = row.try_get::<compact_str::CompactString, _>("version")?;

                    if version.is_empty() {
                        requests.root = TypeStats {
                            total: row.try_get::<i64, _>("total")?,
                            unique_ips: row.try_get::<i64, _>("unique_ips")?,
                        };
                    } else {
                        requests.versions.insert(
                            version,
                            TypeStats {
                                total: row.try_get::<i64, _>("total")?,
                                unique_ips: row.try_get::<i64, _>("unique_ips")?,
                            },
                        );
                    }
                }

                Ok::<_, anyhow::Error>(requests)
            })
            .await?;

        ApiResponse::new_serialized(Response {
            success: true,
            requests,
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
