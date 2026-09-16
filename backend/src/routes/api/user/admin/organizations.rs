use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{Pagination, PaginationParamsWithSearch, organization::Organization},
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
        organizations: Pagination<Organization>,
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
            description = "Filter organizations by name or owner login/email",
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
            organizations: Organization::all_with_pagination(
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

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
