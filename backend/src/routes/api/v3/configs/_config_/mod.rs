use super::State;
use utoipa_axum::router::OpenApiRouter;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new().with_state(state.clone())
}
