use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::webhook::Webhook,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        webhook: Webhook,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "webhook" = i32,
            description = "The webhook ID",
            example = 1,
        ),
    ))]
    pub async fn route(state: GetState, Path(webhook): Path<i32>) -> ApiResponseResult {
        match Webhook::by_id(&state.database, webhook).await? {
            Some(webhook) => ApiResponse::new_serialized(Response {
                success: true,
                webhook,
            })
            .ok(),
            None => ApiResponse::error("webhook not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok(),
        }
    }
}

mod patch {
    use crate::{
        models::{
            organization::Organization,
            webhook::{Webhook, WebhookOrganization},
        },
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use garde::Validate;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    pub struct Payload {
        #[garde(dive)]
        #[serde(flatten)]
        #[schema(inline)]
        webhook: super::super::Payload,
        #[garde(skip)]
        #[serde(default)]
        reset_counters: bool,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        webhook: Webhook,
    }

    #[utoipa::path(patch, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "webhook" = i32,
            description = "The webhook ID",
            example = 1,
        ),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        Path(webhook): Path<i32>,
        crate::Payload(data): crate::Payload<Payload>,
    ) -> ApiResponseResult {
        if let Err(errors) = crate::utils::validate_data(&data) {
            return ApiResponse::new_serialized(serde_json::json!({
                "success": false,
                "errors": errors,
            }))
            .with_status(StatusCode::BAD_REQUEST)
            .ok();
        }

        let Some(mut webhook) = Webhook::by_id(&state.database, webhook).await? else {
            return ApiResponse::error("webhook not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        webhook.organization = match data.webhook.organization {
            Some(organization) => {
                match Organization::by_id(&state.database, &state.cache, organization).await? {
                    Some(organization) => Some(WebhookOrganization {
                        id: organization.id,
                        name: organization.name,
                        verified: organization.verified,
                    }),
                    None => {
                        return ApiResponse::error("organization not found")
                            .with_status(StatusCode::NOT_FOUND)
                            .ok();
                    }
                }
            }
            None => None,
        };

        webhook.label = data.webhook.label;
        webhook.url = data.webhook.url;
        webhook.types = data.webhook.types;
        webhook.enabled = data.webhook.enabled;
        if data.reset_counters {
            webhook.successful = 0;
            webhook.failed = 0;
        }

        webhook.save(&state.database, data.reset_counters).await?;

        state.cache.invalidate("webhooks").await?;

        ApiResponse::new_serialized(Response {
            success: true,
            webhook,
        })
        .ok()
    }
}

mod delete {
    use crate::{
        models::webhook::Webhook,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(delete, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "webhook" = i32,
            description = "The webhook ID",
            example = 1,
        ),
    ))]
    pub async fn route(state: GetState, Path(webhook): Path<i32>) -> ApiResponseResult {
        let Some(webhook) = Webhook::by_id(&state.database, webhook).await? else {
            return ApiResponse::error("webhook not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        webhook.delete(&state.database).await?;

        state.cache.invalidate("webhooks").await?;

        ApiResponse::new_serialized(Response { success: true }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(patch::route))
        .routes(routes!(delete::route))
        .with_state(state.clone())
}
