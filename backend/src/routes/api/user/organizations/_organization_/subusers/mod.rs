use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _subuser_;

mod get {
    use crate::{
        models::organization::OrganizationSubuser,
        response::{ApiResponse, ApiResponseResult},
        routes::{GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        users: Vec<OrganizationSubuser>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
    ))]
    pub async fn route(state: GetState, organization: GetOrganization) -> ApiResponseResult {
        ApiResponse::new_serialized(Response {
            success: true,
            users: OrganizationSubuser::all_by_organization(&state.database, organization.id)
                .await?,
        })
        .ok()
    }
}

mod post {
    use crate::{
        models::{organization::OrganizationSubuser, user::User},
        response::{ApiResponse, ApiResponseResult},
        routes::{
            ApiError, GetState,
            api::user::{GetUser, organizations::_organization_::GetOrganization},
        },
    };
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Payload {
        login: String,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = CREATED, body = inline(Response)),
        (status = FORBIDDEN, body = inline(ApiError)),
        (status = CONFLICT, body = inline(ApiError)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        user: GetUser,
        organization: GetOrganization,
        crate::Payload(data): crate::Payload<Payload>,
    ) -> ApiResponseResult {
        if user.id != organization.owner.id {
            return ApiResponse::error("only the owner can add subusers")
                .with_status(StatusCode::FORBIDDEN)
                .ok();
        }

        let user = User::by_login(&state.database, &state.cache, &data.login).await?;

        let count =
            OrganizationSubuser::count_by_organization(&state.database, organization.id).await;
        if count >= 15 {
            return ApiResponse::error("you cannot have more than 15 subusers")
                .with_status(StatusCode::CONFLICT)
                .ok();
        }

        if let Some(user) = user {
            if user.id == organization.owner.id {
                return ApiResponse::error("user is the owner")
                    .with_status(StatusCode::CONFLICT)
                    .ok();
            }

            let inserted =
                OrganizationSubuser::new(&state.database, organization.id, user.id).await?;

            if inserted {
                ApiResponse::new_serialized(Response { success: true })
                    .with_status(StatusCode::CREATED)
                    .ok()
            } else {
                ApiResponse::error("user already a subuser")
                    .with_status(StatusCode::CONFLICT)
                    .ok()
            }
        } else {
            ApiResponse::error("user not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok()
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(post::route))
        .nest("/{subuser}", _subuser_::router(state))
        .with_state(state.clone())
}
