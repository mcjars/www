use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::r#type::{ServerType, ServerTypeInfo},
        response::{ApiResponse, ApiResponseResult},
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
    pub async fn route(state: GetState) -> ApiResponseResult {
        let data = ServerType::all(&state.database, &state.cache, &state.env).await?;

        ApiResponse::json(Response {
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
                        ServerType::Folia,
                        ServerType::Sponge,
                        ServerType::Spigot,
                        ServerType::Bungeecord,
                        ServerType::Waterfall,
                    ],
                ),
                experimental: ServerType::extract(&data, &[ServerType::Quilt, ServerType::Canvas]),
                miscellaneous: ServerType::extract(
                    &data,
                    &[
                        ServerType::VelocityCtd,
                        ServerType::Arclight,
                        ServerType::Mohist,
                        ServerType::Youer,
                        ServerType::Magma,
                        ServerType::Divinemc,
                        ServerType::Leaf,
                        ServerType::Leaves,
                        ServerType::Aspaper,
                        ServerType::LegacyFabric,
                        ServerType::Pluto,
                    ],
                ),
                limbos: ServerType::extract(
                    &data,
                    &[ServerType::LoohpLimbo, ServerType::Nanolimbo],
                ),
            },
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
