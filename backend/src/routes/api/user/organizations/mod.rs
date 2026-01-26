use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _organization_;

mod get {
    use crate::{
        models::organization::Organization,
        response::{ApiResponse, ApiResponseResult},
        routes::{GetState, api::user::GetUser},
    };
    use indexmap::IndexSet;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Organizations {
        owned: Vec<Organization>,
        member: Vec<Organization>,
        invites: Vec<Organization>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        organizations: Organizations,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState, user: GetUser) -> ApiResponseResult {
        let raw_organizations = Organization::all_by_owner(&state.database, user.id).await?;
        let mut used_organization_ids = IndexSet::new();
        let mut organizations = Organizations {
            owned: Vec::new(),
            member: Vec::new(),
            invites: Vec::new(),
        };

        for organization in raw_organizations {
            if used_organization_ids.contains(&organization.id) {
                continue;
            }

            used_organization_ids.insert(organization.id);

            if organization.owner.id == user.id {
                organizations.owned.push(organization);
            } else if organization.subuser_pending {
                organizations.invites.push(organization);
            } else {
                organizations.member.push(organization);
            }
        }

        ApiResponse::new_serialized(Response {
            success: true,
            organizations,
        })
        .ok()
    }
}

mod post {
    use crate::{
        models::organization::Organization,
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState, api::user::GetUser},
    };
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Payload {
        #[schema(min_length = 3, max_length = 16)]
        name: String,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
        (status = CONFLICT, body = inline(ApiError)),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        user: GetUser,
        crate::Payload(data): crate::Payload<Payload>,
    ) -> ApiResponseResult {
        if !(3..16).contains(&data.name.len()) {
            return ApiResponse::error("name must be between 3 and 16 characters")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let count = Organization::count_by_owner(&state.database, user.id).await;
        if count >= 1 {
            return ApiResponse::error("you already have an organization")
                .with_status(StatusCode::CONFLICT)
                .ok();
        }

        Organization::new(&state.database, user.id, &data.name).await?;

        ApiResponse::new_serialized(Response { success: true }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(post::route))
        .nest("/{organization}", _organization_::router(state))
        .with_state(state.clone())
}
