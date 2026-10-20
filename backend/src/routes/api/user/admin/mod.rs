use crate::routes::{ApiError, State, api::user::GetUser};
use axum::{body::Body, extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde::Serialize;
use sqlx::Row;
use std::collections::HashMap;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

mod bandwidth;
mod files;
mod nodes;
mod organizations;
mod ratelimits;
mod requests;
mod stats;
mod webhooks;

const UNNAMED_NODE: &str = "local";

async fn auth(user: GetUser, req: Request, next: Next) -> Result<Response, StatusCode> {
    if !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&ApiError::new(&["unauthorized"])).unwrap(),
            ))
            .unwrap());
    }

    Ok(next.run(req).await)
}

#[derive(ToSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[schema(rename_all = "camelCase")]
pub struct NodeStatus {
    pub node: compact_str::CompactString,
    pub local: bool,

    pub latency: Option<i64>,
    pub error: Option<String>,
}

pub struct Fanned<T> {
    pub nodes: Vec<NodeStatus>,
    pub data: Vec<(compact_str::CompactString, T)>,
}

pub async fn fanout_with_local<T: serde::de::DeserializeOwned + Serialize>(
    state: &State,
    path: &str,
    local: T,
) -> Result<Fanned<T>, anyhow::Error> {
    let local_name =
        compact_str::CompactString::new(state.env.server_name.as_deref().unwrap_or(UNNAMED_NODE));

    let mut nodes = vec![NodeStatus {
        node: local_name.clone(),
        local: true,
        latency: Some(0),
        error: None,
    }];
    let mut data = vec![(local_name, local)];

    for result in state.nodes.fanout::<T>(path).await? {
        nodes.push(NodeStatus {
            node: result.node.clone(),
            local: false,
            latency: result.latency,
            error: result.error,
        });

        if let Some(value) = result.data {
            data.push((result.node, value));
        }
    }

    Ok(Fanned { nodes, data })
}

pub async fn organization_names(
    state: &State,
    ids: &[i32],
) -> Result<HashMap<i32, (compact_str::CompactString, bool)>, anyhow::Error> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    sqlx::query(
        r#"
        SELECT organizations.id, organizations.name, organizations.verified
        FROM organizations
        WHERE organizations.id = ANY($1)
        "#,
    )
    .bind(ids)
    .fetch_all(state.database.read())
    .await?
    .iter()
    .map(|row| Ok((row.try_get(0)?, (row.try_get(1)?, row.try_get(2)?))))
    .collect()
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/stats", stats::router(state))
        .nest("/nodes", nodes::router(state))
        .nest("/ratelimits", ratelimits::router(state))
        .nest("/bandwidth", bandwidth::router(state))
        .nest("/requests", requests::router(state))
        .nest("/files", files::router(state))
        .nest("/organizations", organizations::router(state))
        .nest("/webhooks", webhooks::router(state))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state.clone())
}
