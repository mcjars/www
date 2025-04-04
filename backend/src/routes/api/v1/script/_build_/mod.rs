use super::State;
use utoipa_axum::router::OpenApiRouter;

mod bash;
mod powershell;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/bash", bash::router(state))
        .nest("/powershell", powershell::router(state))
        .with_state(state.clone())
}
