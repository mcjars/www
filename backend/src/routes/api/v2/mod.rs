use super::State;
use utoipa_axum::router::OpenApiRouter;

mod build;
mod builds;
mod config;
mod configs;
mod lookups;
mod requests;
mod stats;
mod types;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/types", types::router(state))
        .nest("/config", config::router(state))
        .nest("/configs", configs::router(state))
        .nest("/build", build::router(state))
        .nest("/builds", builds::router(state))
        .nest("/lookups", lookups::router(state))
        .nest("/requests", requests::router(state))
        .nest("/stats", stats::router(state))
        .with_state(state.clone())
}
