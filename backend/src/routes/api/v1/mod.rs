use super::State;
use utoipa_axum::router::OpenApiRouter;

mod build;
mod builds;
mod script;
mod stats;
mod types;
mod version;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/types", types::router(state))
        .nest("/stats", stats::router(state))
        .nest("/build", build::router(state))
        .nest("/builds", builds::router(state))
        .nest("/script", script::router(state))
        .nest("/version", version::router(state))
        .with_state(state.clone())
}
