use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, r#type::ServerType, version::Version},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
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
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let build: Option<i32> = if build == "latest" {
            None
        } else {
            match build.parse() {
                Ok(build) => {
                    if build < 0 {
                        return (
                            StatusCode::BAD_REQUEST,
                            axum::Json(ApiError::new(&["invalid build"]).to_value()),
                        );
                    }

                    Some(build)
                }
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        axum::Json(ApiError::new(&["invalid build"]).to_value()),
                    );
                }
            }
        };

        let location = Version::location(&state.database, &state.cache, r#type, &version).await;

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
                .await;

            if let Some(data) = data {
                (
                    StatusCode::OK,
                    axum::Json(
                        serde_json::to_value(&Response {
                            success: true,
                            build: data,
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
