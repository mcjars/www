use crate::routes::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod types;

mod get {
    use crate::{
        models::r#type::ServerType,
        routes::{GetState, api::organization::GetOrganization},
    };
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Infos {
        icon: String,
        name: String,
        types: Vec<ServerType>,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    struct Stats {
        requests: i64,
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
    pub async fn route(
        state: GetState,
        mut organization: GetOrganization,
    ) -> axum::Json<serde_json::Value> {
        let organization = organization.take().unwrap();

        let stats = state
            .cache
            .cached(
                &format!("organization::{}::stats", organization.id),
                300,
                || async {
                    let (requests, user_agents, origins) = tokio::join!(
                        sqlx::query(
                            r#"
                            SELECT COUNT(*)
                            FROM requests
                            WHERE requests.organization_id = $1
                            "#,
                        )
                        .bind(organization.id)
                        .fetch_one(state.database.read()),
                        sqlx::query(
                            r#"
                            SELECT requests.user_agent
                            FROM requests
                            WHERE requests.organization_id = $1
                            GROUP BY requests.user_agent
                            "#,
                        )
                        .bind(organization.id)
                        .fetch_all(state.database.read()),
                        sqlx::query(
                            r#"
                            SELECT requests.origin
                            FROM requests
                            WHERE requests.organization_id = $1 AND requests.origin IS NOT NULL
                            GROUP BY requests.origin
                            "#,
                        )
                        .bind(organization.id)
                        .fetch_all(state.database.read())
                    );

                    let requests = requests.unwrap();
                    let user_agents = user_agents
                        .unwrap()
                        .into_iter()
                        .map(|row| row.get(0))
                        .collect();
                    let origins = origins.unwrap().into_iter().map(|row| row.get(0)).collect();

                    Stats {
                        requests: requests.get(0),
                        user_agents,
                        origins,
                    }
                },
            )
            .await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                infos: Infos {
                    icon: organization.icon,
                    name: organization.name,
                    types: organization.types,
                },
                stats,
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/types", types::router(state))
        .with_state(state.clone())
}
