use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, r#type::ServerType, version::Version},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiErrorV3, GetState},
    };
    use axum::{
        extract::{Path, Query},
        http::StatusCode,
    };
    use garde::Validate;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    pub struct Params {
        #[garde(skip)]
        #[serde(default)]
        pub fields: Vec<compact_str::CompactString>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        build: crate::models::build::ApiBuildV3,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiErrorV3)),
        (status = NOT_FOUND, body = inline(ApiErrorV3)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
        (
            "version" = String,
            description = "The server version (can be 'latest'/'latest-snapshot' for the latest version)",
            example = "1.17.1",
        ),
        (
            "fields" = String, Query,
            description = "HTML form data array of build fields to include in the response (e.g. fields=created&fields=java)",
        ),
    ))]
    pub async fn route(
        state: GetState,
        params: Query<Params>,
        Path((r#type, version)): Path<(ServerType, String)>,
    ) -> ApiResponseResult {
        if let Err(errors) = crate::utils::validate_data(&params.0) {
            return ApiResponse::new_serialized(ApiErrorV3::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let Some((location, version)) =
            Version::resolve(&state.database, &state.cache, r#type, &version).await?
        else {
            return ApiResponse::error("version not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        let data = state
            .cache
            .cached(
                &format!("build::{}::{}::latest", r#type, version,),
                3600,
                || Build::by_build_number(&state.database, r#type, &location, &version, None),
            )
            .await?;

        if let Some(data) = data {
            ApiResponse::new_serialized(json!({
                "build": crate::utils::extract_fields(data.into_api_v3(), &params.fields),
            }))
            .ok()
        } else {
            ApiResponse::error("build not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
