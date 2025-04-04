use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, r#type::ServerType, version::Version},
        routes::{ApiError, GetData, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,
        builds: Vec<Build>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
        (
            "version" = String,
            description = "The server version",
            example = "1.17.1",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        Path((r#type, version)): Path<(ServerType, String)>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let location = Version::location(&state.database, &state.cache, r#type, &version).await;

        if let Some(location) = location {
            let data = state
                .cache
                .cached(
                    &format!("builds::{}::{}", r#type, version),
                    1800,
                    || async {
                        Build::all_for_version(&state.database, r#type, &location, &version).await
                    },
                )
                .await;

            *request_data.lock().unwrap() = json!({
                "type": "builds",
                "search": {
                    "type": r#type,
                    "version": version,
                }
            });

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
        } else {
            (
                StatusCode::NOT_FOUND,
                axum::Json(ApiError::new(&["version not found"]).to_value()),
            )
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
