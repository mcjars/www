use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, r#type::ServerType, version::Version},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetData, GetState},
    };
    use axum::{
        extract::{Path, Query},
        http::StatusCode,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Params {
        #[serde(default)]
        fields: String,
    }

    #[derive(ToSchema, Serialize)]
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
        params: Query<Params>,
        Path((r#type, version)): Path<(ServerType, String)>,
    ) -> ApiResponseResult {
        let location = Version::location(&state.database, &state.cache, r#type, &version).await?;

        if let Some(location) = location {
            let data = state
                .cache
                .cached(&format!("builds::{type}::{version}"), 1800, || async {
                    Build::all_for_version(&state.database, r#type, &location, &version).await
                })
                .await?;

            *request_data.lock().unwrap() = json!({
                "type": "builds",
                "search": {
                    "type": r#type,
                    "version": version,
                }
            });

            let fields = params
                .fields
                .split(',')
                .filter(|f| !f.is_empty())
                .collect::<Vec<_>>();

            ApiResponse::new_serialized(json!({
                "success": true,
                "builds": data
                    .into_iter()
                    .map(|build| crate::utils::extract_fields(build, &fields))
                    .collect::<Vec<_>>(),
            }))
            .ok()
        } else {
            ApiResponse::error("version not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
