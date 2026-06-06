use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use axum::http::StatusCode;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = ACCEPTED, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
    ), request_body = String)]
    pub async fn route(state: GetState, organization: GetOrganization) -> ApiResponseResult {
        if !organization.verified {
            return ApiResponse::error(
                "organization must be verified to request build data update",
            )
            .with_status(StatusCode::BAD_REQUEST)
            .ok();
        }

        let (Some(backend_url), Some(backend_refresh_token)) = (
            state.env.backend_url.as_ref(),
            state.env.backend_refresh_token.as_ref(),
        ) else {
            return ApiResponse::error(
                "backend URL and refresh token must be set to request build data update",
            )
            .with_status(StatusCode::BAD_REQUEST)
            .ok();
        };

        state
            .cache
            .ratelimit(
                "update_build_data",
                2,
                60 * 60 * 24,
                organization.id.to_string(),
            )
            .await?;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/update", backend_url))
            .bearer_auth(backend_refresh_token)
            .send()
            .await;

        let response = match response {
            Ok(response) => response,
            Err(err) => {
                tracing::error!("failed to send backend update request: {}", err);

                return ApiResponse::error("failed to send backend update request")
                    .with_status(StatusCode::BAD_GATEWAY)
                    .ok();
            }
        };

        if !response.status().is_success() {
            tracing::error!(
                "backend responded with error status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );

            return ApiResponse::error("backend responded with an error")
                .with_status(StatusCode::BAD_GATEWAY)
                .ok();
        }

        ApiResponse::new_serialized(Response { success: true })
            .with_status(StatusCode::ACCEPTED)
            .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
