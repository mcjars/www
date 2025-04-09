use crate::routes::{ApiError, State, api::user::GetUser};
use axum::{body::Body, extract::Request, http::StatusCode, middleware::Next, response::Response};
use utoipa_axum::router::OpenApiRouter;

mod stats;

async fn auth(user: GetUser, req: Request, next: Next) -> Result<Response, StatusCode> {
    if !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&ApiError::new(&["unauthorized"])).unwrap(),
            ))
            .unwrap());
    }

    Ok(next.run(req).await)
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/stats", stats::router(state))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state.clone())
}
