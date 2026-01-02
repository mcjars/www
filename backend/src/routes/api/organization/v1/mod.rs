use crate::routes::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod types;

mod get {
    use crate::{
        models::r#type::ServerType,
        response::{ApiResponse, ApiResponseResult},
        routes::{GetState, api::organization::GetOrganization},
    };
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Infos {
        icon: compact_str::CompactString,
        name: compact_str::CompactString,
        types: Vec<ServerType>,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    struct Stats {
        requests: u64,
        user_agents: Vec<String>,
        origins: Vec<String>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        infos: Infos,
        #[schema(inline)]
        stats: Stats,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState, mut organization: GetOrganization) -> ApiResponseResult {
        let organization = organization.take().unwrap();

        let stats = state
            .cache
            .cached(
                &format!("organization::{}::stats", organization.id),
                300,
                || async {
                    let (requests, user_agents, origins) = tokio::try_join!(
                        state
                            .clickhouse
                            .client()
                            .query(
                                r#"
                                SELECT COUNT(*)
                                FROM requests
                                WHERE requests.organization_id = ?
                                "#,
                            )
                            .bind(organization.id)
                            .fetch_one::<u64>(),
                        state
                            .clickhouse
                            .client()
                            .query(
                                r#"
                                SELECT requests.user_agent
                                FROM requests
                                WHERE requests.organization_id = ?
                                GROUP BY requests.user_agent
                                "#,
                            )
                            .bind(organization.id)
                            .fetch_all::<String>(),
                        state
                            .clickhouse
                            .client()
                            .query(
                                r#"
                                SELECT COALESCE(requests.origin, '')
                                FROM requests
                                WHERE requests.organization_id = ? AND requests.origin IS NOT NULL
                                GROUP BY requests.origin
                                "#,
                            )
                            .bind(organization.id)
                            .fetch_all::<String>(),
                    )?;

                    Ok::<_, anyhow::Error>(Stats {
                        requests,
                        user_agents,
                        origins,
                    })
                },
            )
            .await?;

        ApiResponse::json(Response {
            success: true,
            infos: Infos {
                icon: organization.icon,
                name: organization.name,
                types: organization.types,
            },
            stats,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/types", types::router(state))
        .with_state(state.clone())
}
