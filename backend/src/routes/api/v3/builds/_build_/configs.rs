use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::build::Build,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiErrorV3, GetData, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        configs: Vec<crate::models::build::BuildConfig>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiErrorV3)),
    ), params(
        (
            "build",
            description = "The build id, uuid or hash",
            example = "2cd3b3b9-1250-47ff-9a18-81ab1a9bc348",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        Path(identifier): Path<String>,
    ) -> ApiResponseResult {
        let Some((build, _, version)) =
            Build::by_identifier(&state.database, &state.cache, &identifier).await?
        else {
            return ApiResponse::error("build not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

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

        let configs = build.configs(&state.database, &state.cache).await?;

        ApiResponse::new_serialized(Response { configs }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
