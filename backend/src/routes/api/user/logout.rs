use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::user::UserSession,
        response::{ApiResponse, ApiResponseResult},
        routes::GetState,
    };
    use serde::Serialize;
    use tower_cookies::{Cookie, Cookies};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState, cookies: Cookies) -> ApiResponseResult {
        let session = cookies.get("session").unwrap();
        UserSession::delete_by_session(&state.database, session.value()).await?;

        cookies.add(
            Cookie::build(("session", ""))
                .http_only(true)
                .same_site(tower_cookies::cookie::SameSite::Lax)
                .secure(true)
                .domain(state.env.app_cookie_domain.clone())
                .path("/")
                .expires(
                    tower_cookies::cookie::time::OffsetDateTime::now_utc()
                        + tower_cookies::cookie::time::Duration::seconds(2),
                )
                .build(),
        );

        ApiResponse::new_serialized(Response { success: true }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
