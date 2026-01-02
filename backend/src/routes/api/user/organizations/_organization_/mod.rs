use crate::{
    models::organization::Organization,
    routes::{ApiError, GetState, State, api::user::GetUser},
};
use axum::{
    body::Body,
    extract::{Path, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use utoipa_axum::{router::OpenApiRouter, routes};

mod api_keys;
mod icon;
mod stats;
mod subusers;

pub type GetOrganization = axum::extract::Extension<Organization>;

async fn auth(
    state: GetState,
    user: GetUser,
    Path(organization): Path<Vec<String>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let organization = match organization[0].parse::<i32>() {
        Ok(organization) => {
            if organization < 1 {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&ApiError::new(&["invalid organization"])).unwrap(),
                    ))
                    .unwrap());
            }

            organization
        }
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&ApiError::new(&["invalid organization"])).unwrap(),
                ))
                .unwrap());
        }
    };

    let organization = match Organization::by_id_and_user(
        &state.database,
        &state.cache,
        user.id,
        user.admin,
        organization,
    )
    .await
    {
        Ok(Some(organization)) => organization,
        Ok(None) | Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&ApiError::new(&["organization not found"])).unwrap(),
                ))
                .unwrap());
        }
    };

    req.extensions_mut().insert(organization);

    Ok(next.run(req).await)
}

mod get {
    use super::GetOrganization;
    use crate::{
        models::organization::Organization,
        response::{ApiResponse, ApiResponseResult},
    };
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        organization: Organization,
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
    pub async fn route(organization: GetOrganization) -> ApiResponseResult {
        ApiResponse::json(Response {
            success: true,
            organization: organization.0,
        })
        .ok()
    }
}

mod patch {
    use super::GetOrganization;
    use crate::{
        models::{
            organization::{Organization, OrganizationSubuser},
            r#type::ServerType,
            user::User,
        },
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState, api::user::GetUser},
    };
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Payload {
        pub name: Option<String>,
        pub owner: Option<String>,
        pub public: Option<bool>,
        pub types: Option<Vec<ServerType>>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(patch, path = "/", responses(
        (status = OK, body = inline(Response)),
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
        mut organization: GetOrganization,
        axum::Json(data): axum::Json<Payload>,
    ) -> ApiResponseResult {
        let mut owner_id = organization.owner.id;
        if let Some(owner) = data.owner {
            if user.id != organization.owner.id {
                return ApiResponse::error("unauthorized")
                    .with_status(StatusCode::UNAUTHORIZED)
                    .ok();
            }

            owner_id = match User::by_login(&state.database, &state.cache, &owner).await? {
                Some(user) => user.id,
                None => {
                    return ApiResponse::error("owner not found")
                        .with_status(StatusCode::NOT_FOUND)
                        .ok();
                }
            };

            let count = Organization::count_by_owner(&state.database, owner_id).await;
            if count >= 1 {
                return ApiResponse::error("new owner already has an organization")
                    .with_status(StatusCode::CONFLICT)
                    .ok();
            }

            OrganizationSubuser::delete_by_ids(&state.database, organization.id, owner_id).await?;
        }

        if let Some(name) = data.name {
            organization.name = name.into();
        }

        if let Some(public) = data.public {
            organization.public = public;
        }

        if let Some(types) = data.types {
            organization.types = types;
        }

        organization.owner.id = owner_id;
        organization.save(&state.database).await?;

        state.cache.clear_organization(organization.id).await?;

        ApiResponse::json(Response { success: true }).ok()
    }
}

mod delete {
    use super::GetOrganization;
    use crate::{
        models::organization::Organization,
        response::{ApiResponse, ApiResponseResult},
        routes::{GetState, api::user::GetUser},
    };
    use axum::http::StatusCode;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(delete, path = "/", responses(
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
        user: GetUser,
        organization: GetOrganization,
    ) -> ApiResponseResult {
        if user.id != organization.owner.id {
            return ApiResponse::error("unauthorized")
                .with_status(StatusCode::UNAUTHORIZED)
                .ok();
        }

        if organization.icon.starts_with(&state.env.s3_url)
            && !organization.icon.ends_with("default.webp")
        {
            state
                .s3
                .bucket
                .delete_object(&organization.icon[state.env.s3_url.len() + 1..])
                .await
                .map(|_| ())
                .unwrap_or_default();
        }

        Organization::delete_by_id(&state.database, organization.id).await?;

        ApiResponse::json(Response { success: true }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(patch::route))
        .routes(routes!(delete::route))
        .nest("/stats", stats::router(state))
        .nest("/icon", icon::router(state))
        .nest("/api-keys", api_keys::router(state))
        .nest("/subusers", subusers::router(state))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state.clone())
}
