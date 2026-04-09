use super::State;
use utoipa_axum::router::OpenApiRouter;

mod _build_;
mod search;
mod types;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/{build}", _build_::router(state))
        .nest("/types", types::router(state))
        .nest("/search", search::router(state))
        .with_state(state.clone())
}
