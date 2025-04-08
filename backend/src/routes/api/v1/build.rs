use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, version::MinifiedVersion},
        routes::{ApiError, GetData, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        build: Build,
        latest: Build,
        version: MinifiedVersion,
    }

    #[utoipa::path(get, path = "/{build}", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "build",
            description = "The build number or hash to lookup",
            example = "b1f3eeac53355d9ba5cf19e36abe8b2a30278c0e60942f3d07ac9ac9e4564951",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        Path(identifier): Path<String>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let data = Build::by_v1_identifier(&state.database, &state.cache, &identifier).await;

        if let Some((build, latest, version)) = data {
            *request_data.lock().unwrap() = json!({
                "type": "lookup",
                "build": {
                    "id": build.id,
                    "type": build.r#type,
                    "versionId": build.version_id,
                    "projectVersionId": build.project_version_id,
                    "buildNumber": build.build_number,
                    "java": version.java,
                }
            });

            (
                StatusCode::OK,
                axum::Json(
                    serde_json::to_value(&Response {
                        success: true,
                        build,
                        latest,
                        version,
                    })
                    .unwrap(),
                ),
            )
        } else {
            (
                StatusCode::NOT_FOUND,
                axum::Json(ApiError::new(&["build not found"]).to_value()),
            )
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
