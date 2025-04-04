use super::State;
use utoipa_axum::router::OpenApiRouter;

mod accept;
mod decline;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/decline", decline::router(state))
        .nest("/accept", accept::router(state))
        .with_state(state.clone())
}
