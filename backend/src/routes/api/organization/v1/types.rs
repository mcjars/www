use crate::routes::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::{ServerType, ServerTypeInfo},
        routes::{GetState, api::organization::GetOrganization},
    };
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,
        types: IndexMap<ServerType, ServerTypeInfo>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(
        state: GetState,
        organization: GetOrganization,
    ) -> axum::Json<serde_json::Value> {
        let organization = organization.as_ref().unwrap();
        let data = ServerType::all(&state.database, &state.cache).await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                types: if !organization.types.is_empty() {
                    ServerType::extract(&data, &organization.types)
                } else {
                    data
                },
            })
            .unwrap(),
        )
    }
}

mod patch {
    use crate::{
        models::r#type::ServerType,
        routes::{GetState, api::organization::GetOrganization},
    };
    use rustis::commands::GenericCommands;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct Payload {
        types: Vec<ServerType>,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,
    }

    #[utoipa::path(patch, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "organization" = u32,
            description = "The organization ID",
            example = 1,
        ),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        mut organization: GetOrganization,
        axum::Json(data): axum::Json<Payload>,
    ) -> axum::Json<serde_json::Value> {
        let organization = organization.as_mut().unwrap();

        organization.types = data.types;
        organization.save(&state.database).await;

        let keys: Vec<String> = state
            .cache
            .client
            .keys(format!("organization::{}*", organization.id))
            .await
            .unwrap();
        if !keys.is_empty() {
            state.cache.client.del(keys).await.unwrap();
        }

        axum::Json(serde_json::to_value(&Response { success: true }).unwrap())
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(patch::route))
        .with_state(state.clone())
}
