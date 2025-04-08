use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod builds;

mod get {
    use crate::{
        models::version::MinifiedVersionStats,
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
    pub async fn route(
        state: GetState,
        Path(version): Path<String>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let data = MinifiedVersionStats::by_id(&state.database, &state.cache, &version).await;

        if let Some(data) = data {
            (
                StatusCode::OK,
                axum::Json(
                    serde_json::to_value(&Response {
                        success: true,
                        version: data,
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
        .nest("/builds", builds::router(state))
        .with_state(state.clone())
}
