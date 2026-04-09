use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::config::Config,
        response::{ApiResponse, ApiResponseResult},
        routes::ApiErrorV3,
    };
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct Payload {
        file: String,
        config: String,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        formatted: String,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiErrorV3)),
    ), request_body = inline(Payload))]
    pub async fn route(crate::Payload(data): crate::Payload<Payload>) -> ApiResponseResult {
        if Config::by_alias(&data.file).is_none() {
            return ApiResponse::error("invalid config file")
                .with_status(StatusCode::BAD_GATEWAY)
                .ok();
        };

        let formatted = match Config::format(&data.file, &data.config) {
            Ok((formatted, _)) => formatted,
            Err(_) => {
                return ApiResponse::error("unable to format config")
                    .with_status(StatusCode::BAD_REQUEST)
                    .ok();
            }
        };

        ApiResponse::new_serialized(Response { formatted }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
