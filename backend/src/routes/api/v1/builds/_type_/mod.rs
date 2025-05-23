use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _version_;

mod get {
    use crate::{
        models::{r#type::ServerType, version::Version},
        routes::{GetData, GetState},
    };
    use axum::extract::Path;
    use indexmap::IndexMap;
    use serde::Serialize;
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        versions: IndexMap<String, Version>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        Path(r#type): Path<ServerType>,
    ) -> axum::Json<serde_json::Value> {
        let data = Version::all(&state.database, &state.cache, r#type).await;

        *request_data.lock().unwrap() = json!({
            "type": "builds",
            "search": {
                "type": r#type,
            }
        });

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                versions: data,
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
