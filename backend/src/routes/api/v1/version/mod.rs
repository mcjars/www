use super::State;
use utoipa_axum::router::OpenApiRouter;

mod _version_;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
