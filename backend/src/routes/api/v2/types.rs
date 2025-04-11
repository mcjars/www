use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::{ServerType, ServerTypeInfo},
        routes::{ApiError, GetState},
    };
    use indexmap::IndexMap;
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Types<'a> {
        recommended: IndexMap<ServerType, &'a ServerTypeInfo>,
        established: IndexMap<ServerType, &'a ServerTypeInfo>,
        experimental: IndexMap<ServerType, &'a ServerTypeInfo>,
        miscellaneous: IndexMap<ServerType, &'a ServerTypeInfo>,
        limbos: IndexMap<ServerType, &'a ServerTypeInfo>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response<'a> {
        success: bool,

        #[schema(inline)]
        types: Types<'a>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ))]
    pub async fn route(state: GetState) -> axum::Json<serde_json::Value> {
        let data = ServerType::all(&state.database, &state.cache).await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                types: Types {
                    recommended: ServerType::extract(
                        &data,
                        &[
                            ServerType::Vanilla,
                            ServerType::Paper,
                            ServerType::Fabric,
                            ServerType::Forge,
                            ServerType::Neoforge,
                            ServerType::Velocity,
                        ],
                    ),
                    established: ServerType::extract(
                        &data,
                        &[
                            ServerType::Purpur,
                            ServerType::Pufferfish,
                            ServerType::Sponge,
                            ServerType::Spigot,
                            ServerType::Bungeecord,
                            ServerType::Waterfall,
                        ],
                    ),
                    experimental: ServerType::extract(
                        &data,
                        &[
                            ServerType::Folia,
                            ServerType::Quilt,
                            ServerType::Canvas,
                            ServerType::Divinemc,
                        ],
                    ),
                    miscellaneous: ServerType::extract(
                        &data,
                        &[
                            ServerType::Arclight,
                            ServerType::Mohist,
                            ServerType::Magma,
                            ServerType::Leaf,
                            ServerType::Leaves,
                            ServerType::Aspaper,
                            ServerType::LegacyFabric,
                        ],
                    ),
                    limbos: ServerType::extract(
                        &data,
                        &[ServerType::LoohpLimbo, ServerType::Nanolimbo],
                    ),
                },
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
