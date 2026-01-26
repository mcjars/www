use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::organization::{Organization, OrganizationSubuser},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState, api::user::GetUser},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            minimum = 1,
        ),
    ))]
    pub async fn route(
        state: GetState,
        user: GetUser,
        Path(organization): Path<i32>,
    ) -> ApiResponseResult {
        let organization = Organization::by_id(&state.database, &state.cache, organization).await?;

        if let Some(organization) = organization {
            let deleted =
                OrganizationSubuser::delete_by_ids(&state.database, organization.id, user.id)
                    .await?;

            if !deleted {
                return ApiResponse::error("subuser not found")
                    .with_status(StatusCode::NOT_FOUND)
                    .ok();
            }

            ApiResponse::new_serialized(Response { success: true }).ok()
        } else {
            ApiResponse::error("organization not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
