use crate::models::organization::Organization;
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, Response, StatusCode},
    middleware::Next,
};
use serde::Serialize;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

mod api;
mod index;

#[derive(ToSchema, Serialize)]
pub struct ApiError<'a> {
    #[schema(default = false)]
    pub success: bool,
    pub errors: &'a [&'a str],
}

impl<'a> ApiError<'a> {
    pub fn new(errors: &'a [&'a str]) -> Self {
        Self {
            success: false,
            errors,
        }
    }

    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

pub struct AppState {
    pub start_time: Instant,
    pub version: String,

    pub database: Arc<crate::database::Database>,
    pub cache: Arc<crate::cache::Cache>,
    pub requests: crate::requests::RequestLogger,
    pub env: Arc<crate::env::Env>,
    pub s3: Arc<crate::s3::S3>,
}

pub type State = Arc<AppState>;
pub type GetState = axum::extract::State<State>;
pub type GetData = axum::extract::Extension<Arc<Mutex<serde_json::Value>>>;

async fn handle_api_request(state: GetState, req: Request, next: Next) -> Response<Body> {
    let mut organization: Option<Organization> = None;
    if let Some(authorization) = req.headers().get("Authorization") {
        if let Ok(authorization) = authorization.to_str() {
            if authorization.len() == 64 {
                organization =
                    Organization::by_key(&state.database, &state.cache, authorization).await;
            }
        }
    }

    let (parts, body) = req.into_parts();
    let request_id = state
        .requests
        .log(parts.clone(), organization.as_ref())
        .await;

    if let Err(Some(ratelimit)) = request_id {
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .header("X-RateLimit-Limit", ratelimit.limit.to_string())
            .header(
                "X-RateLimit-Remaining",
                (ratelimit.limit - ratelimit.hits).to_string(),
            )
            .header("X-RateLimit-Reset", "60")
            .body(Body::from(
                ApiError::new(&["too many requests"]).to_value().to_string(),
            ))
            .unwrap();
    } else if let Err(None) = request_id {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::from(
                ApiError::new(&["broken request, likely invalid IP"])
                    .to_value()
                    .to_string(),
            ))
            .unwrap();
    }

    let mut headers = HeaderMap::new();
    let request_id = request_id.unwrap();
    let (request_id, ratelimit) = request_id;
    if let Some(ref request_id) = request_id {
        headers.insert("X-Request-ID", request_id.parse().unwrap());
    }

    if let Some(ratelimit) = ratelimit {
        headers.insert(
            "X-RateLimit-Limit",
            ratelimit.limit.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            (ratelimit.limit - ratelimit.hits)
                .to_string()
                .parse()
                .unwrap(),
        );
        headers.insert("X-RateLimit-Reset", "60".parse().unwrap());
    }

    let mut req = Request::from_parts(parts, body);
    let data = Arc::new(Mutex::new(serde_json::Value::Null));
    req.extensions_mut().insert(data.clone());
    req.extensions_mut().insert(organization);

    let start = Instant::now();
    let mut response = next.run(req).await;

    if let Some(request_id) = request_id {
        let data = {
            let data = data.lock().unwrap();

            if let serde_json::Value::Object(data) = &*data {
                Some(serde_json::Value::Object(data.clone()))
            } else {
                None
            }
        };

        state
            .requests
            .finish(
                request_id,
                response.status().as_u16() as i16,
                start.elapsed().as_millis() as i32,
                data,
                None,
            )
            .await;
    }

    response.headers_mut().extend(headers);
    if let Some(server_name) = state.env.server_name.as_ref() {
        response
            .headers_mut()
            .insert("X-Server-Name", server_name.parse().unwrap());
    }

    response
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/api", api::router(state))
        .nest("/index", index::router(state))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            handle_api_request,
        ))
        .with_state(state.clone())
}
