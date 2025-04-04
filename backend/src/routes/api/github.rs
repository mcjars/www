use crate::models::user::{User, UserSession};

use super::{GetState, State};
use axum::{
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
                    .build()
                    .unwrap();
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
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await
                    .unwrap();

                if !user["access_token"].is_string() {
                    return (StatusCode::BAD_REQUEST, HeaderMap::new(), "Invalid access token returned");
                }

                let (data, email) = tokio::join!(
                    client
                        .get("https://api.github.com/user")
                        .header("Accept", "application/vnd.github+json")
                        .header("Authorization", format!("Bearer {}", user["access_token"].as_str().unwrap()))
                        .send(),
                    client
                        .get("https://api.github.com/user/emails")
                        .header("Accept", "application/vnd.github+json")
                        .header("Authorization", format!("Bearer {}", user["access_token"].as_str().unwrap()))
                        .send()
                );

                let (data, email) = tokio::join!(
                    data.unwrap().json::<serde_json::Value>(),
                    email.unwrap().json::<serde_json::Value>(),
                );

                let (data, email) = (data.unwrap(), email.unwrap());

                let email = email
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|email| email["primary"].as_bool().unwrap())
                    .unwrap();

                let user = User::new(
                    &state.database,
                    data["id"].as_i64().unwrap() as i32,
                    data["name"].as_str().map(|s| s.to_string()),
                    email["email"].as_str().unwrap().to_string(),
                    data["login"].as_str().unwrap().to_string(),
                ).await;

                let (_, key) = UserSession::new(
                    &state.database,
                    user.id,
                    headers
                        .get("x-real-ip")
                        .or_else(|| headers.get("x-forwarded-for"))
                        .map(|ip| ip.to_str().unwrap_or_default())
                        .unwrap_or_default()
                        .parse()
                        .unwrap(),
                    headers.get("User-Agent").map(|ua| ua.to_str().unwrap()).unwrap_or("")
                ).await;

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

                let mut headers = HeaderMap::new();

                headers.insert(
                    "Location",
                    state.env.app_frontend_url.parse().unwrap(),
                );

                (StatusCode::FOUND, headers, "")
            }),
        )
        .with_state(state.clone())
}
