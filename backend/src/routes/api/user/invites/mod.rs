use super::State;
use utoipa_axum::router::OpenApiRouter;

mod _organization_;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/{organization}", _organization_::router(state))
        .with_state(state.clone())
}
