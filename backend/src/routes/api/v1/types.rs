use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::{ServerType, ServerTypeInfo, V1_TYPES},
        response::{ApiResponse, ApiResponseResult},
        routes::GetState,
    };
    use indexmap::IndexMap;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response<'a> {
        success: bool,
        types: IndexMap<ServerType, &'a ServerTypeInfo>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    #[deprecated]
    pub async fn route(state: GetState) -> ApiResponseResult {
        let data = ServerType::all(&state.database, &state.cache, &state.env).await?;

        ApiResponse::new_serialized(Response {
            success: true,
            types: ServerType::extract(&data, &V1_TYPES),
        })
        .ok()
    }
}

#[allow(deprecated)]
pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
