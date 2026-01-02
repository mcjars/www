use super::{GetState, State};
use crate::{
    models::user::{User, UserSession},
    response::ApiResponse,
};
use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::get,
};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::{Cookie, Cookies};
use utoipa_axum::router::OpenApiRouter;

#[derive(Deserialize)]
struct Params {
    code: String,
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .route(
            "/",
            get(|state: GetState| async move {
                let mut headers = HeaderMap::new();

                headers.insert(
                    "Location",
                    format!("https://github.com/login/oauth/authorize?allow_signup=true&client_id={}&redirect_uri={}/api/github/callback&scope=read:user,user:email", state.env.github_client_id, state.env.app_url).parse().unwrap(),
                );

                (StatusCode::FOUND, headers, "")
            }),
        )
        .route(
            "/callback",
            get(|state: GetState, headers: HeaderMap, cookies: Cookies, params: Query<Params>| async move {
                let client = reqwest::Client::builder()
                    .user_agent("MCJars API https://mcjars.app")
                    .build()?;
                let user = client
                    .post("https://github.com/login/oauth/access_token")
                    .header("Accept", "application/json")
                    .json(&json!({
                        "client_id": state.env.github_client_id,
                        "client_secret": state.env.github_client_secret,
                        "code": params.code,
                        "redirect_uri": format!("{}/api/github/callback", state.env.app_url),
                    }))
                    .send()
                    .await?
                    .json::<OAuthResponse>()
                    .await;

                let user = match user {
                    Ok(user) => user,
                    Err(_) => {
                        return ApiResponse::error("invalid access token returned")
                            .with_status(StatusCode::BAD_REQUEST)
                            .ok();
                    }
                };

                #[derive(Deserialize)]
                struct OAuthResponse {
                    access_token: String,
                }

                let (data, email) = tokio::join!(
                    client
                        .get("https://api.github.com/user")
                        .header("Accept", "application/vnd.github+json")
                        .header("Authorization", format!("Bearer {}", user.access_token))
                        .send(),
                    client
                        .get("https://api.github.com/user/emails")
                        .header("Accept", "application/vnd.github+json")
                        .header("Authorization", format!("Bearer {}", user.access_token))
                        .send()
                );

                let (data, email) = tokio::join!(
                    data.unwrap().json::<UserResponse>(),
                    email.unwrap().json::<Vec<EmailResponse>>(),
                );

                #[derive(Deserialize)]
                struct UserResponse {
                    id: i32,
                    name: Option<String>,
                    login: String,
                }

                #[derive(Deserialize)]
                struct EmailResponse {
                    email: String,
                    primary: bool,
                }

                let (data, email) = match (data, email) {
                    (Ok(data), Ok(email)) => (data, email),
                    _ => {
                        return ApiResponse::error("invalid user data returned")
                            .with_status(StatusCode::BAD_REQUEST)
                            .ok();
                    }
                };

                let email = email
                    .into_iter()
                    .find(|email| email.primary)
                    .unwrap();

                let user = User::new(&state.database, data.id, data.name, email.email, data.login).await?;

                let (_, key) = UserSession::new(
                    &state.database,
                    user.id,
                    crate::utils::extract_ip(&headers).unwrap().into(),
                    headers
                        .get("User-Agent")
                        .map(|ua| crate::utils::slice_up_to(ua.to_str().unwrap_or("unknown"), 255))
                        .unwrap_or("unknown"),
                ).await?;

                cookies.add(
                    Cookie::build(("session", key))
                        .http_only(true)
                        .same_site(tower_cookies::cookie::SameSite::Lax)
                        .secure(true)
                        .domain(state.env.app_cookie_domain.clone())
                        .path("/")
                        .expires(
                            tower_cookies::cookie::time::OffsetDateTime::now_utc()
                                + tower_cookies::cookie::time::Duration::days(7),
                        )
                        .build(),
                );

                ApiResponse::new(Body::empty()).with_status(StatusCode::FOUND).with_header(
                    "Location",
                    &state.env.app_frontend_url,
                ).ok()
            }),
        )
        .with_state(state.clone())
}
