use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod builds;

mod get {
    use crate::{
        models::version::MinifiedVersionStats,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        version: MinifiedVersionStats,
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
    pub async fn route(state: GetState, Path(version): Path<String>) -> ApiResponseResult {
        let data = MinifiedVersionStats::by_id(&state.database, &state.cache, &state.env, &version)
            .await?;

        if let Some(data) = data {
            ApiResponse::json(Response {
                success: true,
                version: data,
            })
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
        .nest("/builds", builds::router(state))
        .with_state(state.clone())
}
