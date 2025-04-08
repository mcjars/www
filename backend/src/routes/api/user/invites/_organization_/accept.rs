use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::organization::{Organization, OrganizationSubuser},
        routes::{ApiError, GetState, api::user::GetUser},
    };
    use axum::extract::Path;
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
    ) -> axum::Json<serde_json::Value> {
        let organization = Organization::by_id(&state.database, &state.cache, organization).await;

        if let Some(organization) = organization {
            let subuser =
                OrganizationSubuser::by_ids(&state.database, organization.id, user.id).await;

            if let Some(mut subuser) = subuser {
                if !subuser.pending {
                    return axum::Json(ApiError::new(&["subuser already accepted"]).to_value());
                }

                subuser.pending = false;
                subuser.save(&state.database).await;

                axum::Json(serde_json::to_value(&Response { success: true }).unwrap())
            } else {
                axum::Json(ApiError::new(&["organization not found"]).to_value())
            }
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
