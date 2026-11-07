use super::State;
use utoipa_axum::router::OpenApiRouter;

mod _type_;
mod history;
mod java;
mod version;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/history", history::router(state))
        .nest("/java", java::router(state))
        .nest("/{type}", _type_::router(state))
        .nest("/version", version::router(state))
        .with_state(state.clone())
}
