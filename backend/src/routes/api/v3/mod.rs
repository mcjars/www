use super::State;
use utoipa_axum::router::OpenApiRouter;

mod builds;
mod configs;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/builds", builds::router(state))
        .nest("/configs", configs::router(state))
        .with_state(state.clone())
}
