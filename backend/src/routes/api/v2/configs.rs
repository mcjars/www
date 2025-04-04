use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::models::config::{CONFIGS, Config};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,
        configs: IndexMap<String, Config>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route() -> axum::Json<serde_json::Value> {
        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                configs: CONFIGS.clone(),
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
