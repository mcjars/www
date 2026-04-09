use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _config_;
mod format;
mod identify;

mod get {
    use crate::{
        models::config::ConfigStats,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiErrorV3, GetState},
    };
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        configs: Vec<crate::models::config::ApiConfigStatsV3>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiErrorV3)),
    ))]
    pub async fn route(state: GetState) -> ApiResponseResult {
        let configs = ConfigStats::all(&state.database, &state.cache).await?;

        ApiResponse::new_serialized(Response {
            configs: configs.into_iter().map(|c| c.into_api_stats_v3()).collect(),
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/{config}", _config_::router(state))
        .nest("/identify", identify::router(state))
        .nest("/format", format::router(state))
        .with_state(state.clone())
}
