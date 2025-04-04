use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, r#type::ServerType},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
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
    pub async fn route(
        state: GetState,
        Path(version): Path<String>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let data = state
            .cache
            .cached(&format!("version::{}::builds", version), 1800, || async {
                let data = Build::all_for_minecraft_version(&state.database, &version).await;

                let mut builds = IndexMap::new();
                for r#type in ServerType::variants() {
                    builds.insert(r#type, vec![]);
                }

                for build in data {
                    builds[&build.r#type].push(build);
                }

                builds
            })
            .await;

        if data.is_empty() {
            (
                StatusCode::NOT_FOUND,
                axum::Json(ApiError::new(&["build not found"]).to_value()),
            )
        } else {
            (
                StatusCode::OK,
                axum::Json(
                    serde_json::to_value(&Response {
                        success: true,
                        builds: data,
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

#[allow(deprecated)]
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
