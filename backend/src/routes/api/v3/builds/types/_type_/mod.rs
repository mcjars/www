use super::State;
use utoipa_axum::router::OpenApiRouter;

mod versions;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/versions", versions::router(state))
        .with_state(state.clone())
}
