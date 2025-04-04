use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _key_;

mod get {
    use crate::{
        models::organization::OrganizationKey,
        routes::{GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    struct Response {
        success: bool,
        api_keys: Vec<OrganizationKey>,
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
                api_keys: OrganizationKey::all_by_organization(&state.database, organization.id)
                    .await,
            })
            .unwrap(),
        )
    }
}

mod post {
    use crate::{
        models::organization::OrganizationKey,
        routes::{ApiError, GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct Payload {
        name: String,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,
        key: String,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = CREATED, body = inline(Response)),
        (status = CONFLICT, body = inline(ApiError)),
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        organization: GetOrganization,
        axum::Json(payload): axum::Json<Payload>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        if !(1..32).contains(&payload.name.len()) {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiError::new(&["name must be between 1 and 32 characters"]).to_value()),
            );
        }

        let count = OrganizationKey::count_by_organization(&state.database, organization.id).await;
        if count >= 15 {
            return (
                StatusCode::CONFLICT,
                axum::Json(ApiError::new(&["you cannot have more than 15 keys"]).to_value()),
            );
        }

        let (inserted, key) =
            OrganizationKey::new(&state.database, organization.id, &payload.name).await;
        if inserted {
            (
                StatusCode::CREATED,
                axum::Json(serde_json::to_value(&Response { success: true, key }).unwrap()),
            )
        } else {
            (
                StatusCode::CONFLICT,
                axum::Json(ApiError::new(&["key already exists"]).to_value()),
            )
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(post::route))
        .nest("/{key}", _key_::router(state))
        .with_state(state.clone())
}
