use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _subuser_;

mod get {
    use crate::{
        models::organization::OrganizationSubuser,
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
    pub async fn route(
        state: GetState,
        organization: GetOrganization,
    ) -> axum::Json<serde_json::Value> {
        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                users: OrganizationSubuser::all_by_organization(&state.database, organization.id)
                    .await,
            })
            .unwrap(),
        )
    }
}

mod post {
    use crate::{
        models::{organization::OrganizationSubuser, user::User},
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
        axum::Json(payload): axum::Json<Payload>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        if user.id != organization.owner.id {
            return (
                StatusCode::FORBIDDEN,
                axum::Json(ApiError::new(&["only the owner can add subusers"]).to_value()),
            );
        }

        let user = User::by_login(&state.database, &state.cache, &payload.login).await;

        let count =
            OrganizationSubuser::count_by_organization(&state.database, organization.id).await;
        if count >= 15 {
            return (
                StatusCode::CONFLICT,
                axum::Json(ApiError::new(&["you cannot have more than 15 subusers"]).to_value()),
            );
        }

        if let Some(user) = user {
            if user.id == organization.owner.id {
                return (
                    StatusCode::CONFLICT,
                    axum::Json(ApiError::new(&["user is the owner"]).to_value()),
                );
            }

            let inserted =
                OrganizationSubuser::new(&state.database, organization.id, user.id).await;

            if inserted {
                (
                    StatusCode::CREATED,
                    axum::Json(serde_json::to_value(&Response { success: true }).unwrap()),
                )
            } else {
                (
                    StatusCode::CONFLICT,
                    axum::Json(ApiError::new(&["user already a subuser"]).to_value()),
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
        .routes(routes!(post::route))
        .nest("/{subuser}", _subuser_::router(state))
        .with_state(state.clone())
}
