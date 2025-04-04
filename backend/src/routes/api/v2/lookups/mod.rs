use super::State;
use utoipa_axum::router::OpenApiRouter;

mod types;
mod versions;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/versions", versions::router(state))
        .nest("/types", types::router(state))
        .with_state(state.clone())
}
