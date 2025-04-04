use super::State;
use utoipa_axum::router::OpenApiRouter;

mod _build_;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/{build}", _build_::router(state))
        .with_state(state.clone())
}
