use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::models::config::{CONFIGS, Config};
    use indexmap::IndexMap;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        configs: &'static IndexMap<&'static str, Config>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route() -> axum::Json<serde_json::Value> {
        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                configs: &CONFIGS,
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
