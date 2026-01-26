use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _version_;

mod get {
    use crate::{
        models::{r#type::ServerType, version::Version},
        response::{ApiResponse, ApiResponseResult},
        routes::{GetData, GetState},
    };
    use axum::extract::{Path, Query};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Params {
        #[serde(default)]
        fields: String,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        builds: IndexMap<String, Version>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        params: Query<Params>,
        Path(r#type): Path<ServerType>,
    ) -> ApiResponseResult {
        let data = Version::all(&state.database, &state.cache, r#type).await?;

        *request_data.lock().unwrap() = json!({
            "type": "builds",
            "search": {
                "type": r#type,
            }
        });

        let fields = params
            .fields
            .split(',')
            .filter(|f| !f.is_empty())
            .collect::<Vec<_>>();

        ApiResponse::new_serialized(json!({
            "success": true,
            "builds": data
                .into_iter()
                .map(|(name, version)| (name, json!({
                    "type": version.r#type,
                    "supported": version.supported,
                    "java": version.java,
                    "builds": version.builds,
                    "created": version.created,
                    "latest": crate::utils::extract_fields(version.latest, &fields),
                })))
                .collect::<IndexMap<_, _>>(),
        }))
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
