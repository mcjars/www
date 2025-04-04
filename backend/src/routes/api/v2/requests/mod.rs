use super::State;
use utoipa_axum::router::OpenApiRouter;

mod _type_;
mod version;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/version", version::router(state))
        .nest("/{type}", _type_::router(state))
        .with_state(state.clone())
}
