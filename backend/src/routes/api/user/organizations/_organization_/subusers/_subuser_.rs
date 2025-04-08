use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{organization::OrganizationSubuser, user::User},
        routes::{ApiError, GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        user: OrganizationSubuser,
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
            "subuser" = String,
            description = "The subuser login name",
            example = 1,
        ),
    ))]
    pub async fn route(
        state: GetState,
        organization: GetOrganization,
        Path((_organization, login)): Path<(i32, String)>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let user = User::by_login(&state.database, &state.cache, &login).await;

        if let Some(user) = user {
            let subuser =
                OrganizationSubuser::by_ids(&state.database, organization.id, user.id).await;

            if let Some(subuser) = subuser {
                (
                    StatusCode::OK,
                    axum::Json(
                        serde_json::to_value(&Response {
                            success: true,
                            user: subuser,
                        })
                        .unwrap(),
                    ),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    axum::Json(ApiError::new(&["user not found"]).to_value()),
                )
            }
        } else {
            (
                StatusCode::NOT_FOUND,
                axum::Json(ApiError::new(&["user not found"]).to_value()),
            )
        }
    }
}

mod delete {
    use crate::{
        models::{organization::OrganizationSubuser, user::User},
        routes::{
            ApiError, GetState,
            api::user::{GetUser, organizations::_organization_::GetOrganization},
        },
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
        (status = FORBIDDEN, body = inline(ApiError)),
        (status = CONFLICT, body = inline(ApiError)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
        (
            "subuser" = String,
            description = "The subuser login name",
            example = 1,
        ),
    ))]
    pub async fn route(
        state: GetState,
        auth_user: GetUser,
        organization: GetOrganization,
        Path((_organization, login)): Path<(i32, String)>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let user = User::by_login(&state.database, &state.cache, &login).await;

        if auth_user.id != user.as_ref().map(|u| u.id).unwrap_or_default()
            && auth_user.id != organization.owner.id
        {
            return (
                StatusCode::FORBIDDEN,
                axum::Json(ApiError::new(&["only the owner can delete subusers"]).to_value()),
            );
        }

        if let Some(user) = user {
            let subuser =
                OrganizationSubuser::by_ids(&state.database, organization.id, user.id).await;

            if subuser.is_some() {
                OrganizationSubuser::delete_by_ids(&state.database, organization.id, user.id).await;

                (
                    StatusCode::OK,
                    axum::Json(serde_json::to_value(&Response { success: true }).unwrap()),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    axum::Json(ApiError::new(&["user not found"]).to_value()),
                )
            }
        } else {
            (
                StatusCode::NOT_FOUND,
                axum::Json(ApiError::new(&["user not found"]).to_value()),
            )
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(delete::route))
        .with_state(state.clone())
}
