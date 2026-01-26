use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::organization::OrganizationKey,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    struct Response {
        success: bool,
        api_key: OrganizationKey,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
        (
            "key" = i32,
            description = "The api key ID",
            example = 1,
        ),
    ))]
    pub async fn route(
        state: GetState,
        organization: GetOrganization,
        Path((_organization, key)): Path<(i32, i32)>,
    ) -> ApiResponseResult {
        let key = OrganizationKey::by_id(&state.database, key).await?;

        if let Some(key) = key {
            if key.organization_id != organization.id {
                return ApiResponse::error("key not found")
                    .with_status(StatusCode::NOT_FOUND)
                    .ok();
            }

            ApiResponse::new_serialized(Response {
                success: true,
                api_key: key,
            })
            .ok()
        } else {
            ApiResponse::error("key not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        }
    }
}

mod delete {
    use crate::{
        models::organization::OrganizationKey,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState, api::user::organizations::_organization_::GetOrganization},
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
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
        (
            "key" = i32,
            description = "The api key ID",
            example = 1,
        ),
    ))]
    pub async fn route(
        state: GetState,
        organization: GetOrganization,
        Path((_organization, key)): Path<(i32, i32)>,
    ) -> ApiResponseResult {
        let key = OrganizationKey::by_id(&state.database, key).await?;

        if let Some(key) = key {
            if key.organization_id != organization.id {
                return ApiResponse::error("key not found")
                    .with_status(StatusCode::NOT_FOUND)
                    .ok();
            }

            OrganizationKey::delete_by_id(&state.database, key.id).await?;

            ApiResponse::new_serialized(Response { success: true }).ok()
        } else {
            ApiResponse::error("key not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(delete::route))
        .with_state(state.clone())
}
