use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::ServerType,
        response::{ApiResponse, ApiResponseResult},
        routes::GetState,
    };
    use indexmap::IndexMap;
    use serde::Serialize;
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        java: IndexMap<ServerType, IndexMap<i16, i64>>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState) -> ApiResponseResult {
        let java = state
            .cache
            .cached("stats::builds::java", 10800, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        builds.type,
                        COALESCE(builds.java, minecraft_versions.java, 21)::smallint AS effective_java,
                        COUNT(*) AS builds
                    FROM builds
                    LEFT JOIN minecraft_versions ON minecraft_versions.id = builds.version_id
                    GROUP BY builds.type, effective_java
                    ORDER BY builds.type, effective_java
                    "#,
                )
                .fetch_all(state.database.read())
                .await?;

                let mut java: IndexMap<ServerType, IndexMap<i16, i64>> = IndexMap::new();
                for row in data {
                    java.entry(row.try_get("type")?)
                        .or_default()
                        .insert(row.try_get("effective_java")?, row.try_get("builds")?);
                }

                Ok::<_, anyhow::Error>(java)
            })
            .await?;

        ApiResponse::new_serialized(Response {
            success: true,
            java,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
