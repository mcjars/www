use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, r#type::ServerType},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use indexmap::IndexMap;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        builds: IndexMap<ServerType, Vec<Build>>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "version" = String,
            description = "The server version",
            example = "1.17.1",
        ),
    ))]
    #[deprecated]
    pub async fn route(state: GetState, Path(version): Path<String>) -> ApiResponseResult {
        let data = state
            .cache
            .cached(&format!("version::{version}::builds"), 1800, || async {
                let data = Build::all_for_minecraft_version(&state.database, &version).await?;

                let mut builds = IndexMap::new();
                for r#type in ServerType::variants(&state.env) {
                    builds.insert(r#type, vec![]);
                }

                for build in data {
                    builds[&build.r#type].push(build);
                }

                Ok::<_, anyhow::Error>(builds)
            })
            .await?;

        if data.is_empty() {
            ApiResponse::error("build not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        } else {
            ApiResponse::new_serialized(Response {
                success: true,
                builds: data,
            })
            .ok()
        }
    }
}

#[allow(deprecated)]
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
