use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::organization::{Organization, OrganizationSubuser},
        routes::{ApiError, GetState, api::user::GetUser},
    };
    use axum::extract::Path;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
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
    ) -> axum::Json<serde_json::Value> {
        let organization = Organization::by_id(&state.database, &state.cache, organization).await;

        if let Some(organization) = organization {
            let deleted =
                OrganizationSubuser::delete_by_ids(&state.database, organization.id, user.id).await;

            if !deleted {
                return axum::Json(ApiError::new(&["subuser not found"]).to_value());
            }

            axum::Json(serde_json::to_value(&Response { success: true }).unwrap())
        } else {
            axum::Json(ApiError::new(&["organization not found"]).to_value())
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
