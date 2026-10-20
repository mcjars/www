use super::State;
use crate::models::r#type::ServerType;
use garde::Validate;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _webhook_;

#[derive(ToSchema, Validate, Deserialize)]
#[serde(rename_all = "camelCase")]
#[schema(rename_all = "camelCase")]
pub struct Payload {
    #[garde(range(min = 1))]
    pub organization: Option<i32>,

    #[garde(length(chars, min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255)]
    pub label: Option<compact_str::CompactString>,
    #[garde(
        length(max = 255),
        pattern(
            r"^https://(?:ptb\.|canary\.)?discord\.com/api(?:/v[0-9]{1,2})?/webhooks/[0-9]{17,19}/[A-Za-z0-9_-]{68}$"
        )
    )]
    #[schema(max_length = 255)]
    pub url: compact_str::CompactString,
    #[garde(length(min = 1))]
    #[schema(min_items = 1)]
    pub types: Vec<ServerType>,
    #[garde(skip)]
    pub enabled: bool,
}

mod get {
    use crate::{
        models::{Pagination, PaginationParamsWithSearch, webhook::Webhook},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::http::StatusCode;
    use axum_extra::extract::Query;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        webhooks: Pagination<Webhook>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "page" = i64, Query,
            description = "The page number (starting from 1)",
            minimum = 1,
            example = 1,
        ),
        (
            "per_page" = i64, Query,
            description = "The number of items per page (maximum 200)",
            minimum = 1,
            maximum = 200,
            example = 50,
        ),
        (
            "search" = String, Query,
            description = "Filter webhooks by label, url or organization name",
        ),
    ))]
    pub async fn route(
        state: GetState,
        params: Query<PaginationParamsWithSearch>,
    ) -> ApiResponseResult {
        if let Err(errors) = crate::utils::validate_data(&params.0) {
            return ApiResponse::new_serialized(serde_json::json!({
                "success": false,
                "errors": errors,
            }))
            .with_status(StatusCode::BAD_REQUEST)
            .ok();
        }

        ApiResponse::new_serialized(Response {
            success: true,
            webhooks: Webhook::all_with_pagination(
                &state.database,
                params.page,
                params.per_page,
                params.search.as_deref(),
            )
            .await?,
        })
        .ok()
    }
}

mod post {
    use super::Payload;
    use crate::{
        models::{organization::Organization, webhook::Webhook},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState},
    };
    use axum::http::StatusCode;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        id: i32,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = CREATED, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
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

        if let Some(organization) = data.organization
            && Organization::by_id(&state.database, &state.cache, organization)
                .await?
                .is_none()
        {
            return ApiResponse::error("organization not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        }

        let id = Webhook::new(
            &state.database,
            data.organization,
            data.label.as_deref(),
            &data.url,
            &data.types,
            data.enabled,
        )
        .await?;

        state.cache.invalidate("webhooks").await?;

        ApiResponse::new_serialized(Response { success: true, id })
            .with_status(StatusCode::CREATED)
            .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(post::route))
        .nest("/{webhook}", _webhook_::router(state))
        .with_state(state.clone())
}
