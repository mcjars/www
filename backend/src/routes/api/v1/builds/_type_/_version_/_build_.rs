use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, r#type::ServerType, version::Version},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        build: Build,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
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
        (
            "build" = String,
            description = "The build number or latest",
            example = "latest",
        )
    ))]
    pub async fn route(
        state: GetState,
        Path((r#type, version, build)): Path<(ServerType, String, String)>,
    ) -> ApiResponseResult {
        let build: Option<i32> = if build == "latest" {
            None
        } else {
            match build.parse() {
                Ok(build) => {
                    if build < 0 {
                        return ApiResponse::error("invalid build")
                            .with_status(StatusCode::BAD_REQUEST)
                            .ok();
                    }

                    Some(build)
                }
                Err(_) => {
                    return ApiResponse::error("invalid build")
                        .with_status(StatusCode::BAD_REQUEST)
                        .ok();
                }
            }
        };

        let location = Version::location(&state.database, &state.cache, r#type, &version).await?;

        if let Some(location) = location {
            let data = state
                .cache
                .cached(
                    &format!(
                        "build::{}::{}::{}",
                        r#type,
                        version,
                        build.map(|b| b.to_string()).unwrap_or("latest".to_string())
                    ),
                    3600,
                    || Build::by_build_number(&state.database, r#type, &location, &version, build),
                )
                .await?;

            if let Some(data) = data {
                ApiResponse::json(Response {
                    success: true,
                    build: data,
                })
                .ok()
            } else {
                ApiResponse::error("build not found")
                    .with_status(StatusCode::NOT_FOUND)
                    .ok()
            }
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
