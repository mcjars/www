use super::{ApiError, State};
use crate::models::organization::Organization;
use axum::{body::Body, extract::Request, http::StatusCode, middleware::Next, response::Response};
use utoipa_axum::router::OpenApiRouter;

mod v1;

pub type GetOrganization = axum::extract::Extension<Option<Organization>>;

async fn auth(
    organization: GetOrganization,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if organization.is_none() {
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
        .nest("/v1", v1::router(state))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state.clone())
}
