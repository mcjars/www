use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        response::{ApiResponse, ApiResponseResult},
        routes::{GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    struct Stats {
        requests: u64,
        user_agents: u64,
        ips: u64,
        origins: u64,
        continents: u64,
        countries: u64,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        stats: Stats,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
    ))]
    pub async fn route(state: GetState, organization: GetOrganization) -> ApiResponseResult {
        let stats = state
            .cache
            .cached(
                &format!("organization::stats::{}", organization.id),
                300,
                || async {
                    let data = state
                        .clickhouse
                        .client()
                        .query(
                            r#"
                            SELECT
                                COUNT(*),
                                uniqExact(requests.user_agent),
                                uniqExact(requests.ip),
                                uniqExact(requests.origin),
                                uniqExact(requests.continent),
                                uniqExact(requests.country)
                            FROM requests
                            WHERE requests.organization_id = ?
                            "#,
                        )
                        .bind(organization.id)
                        .fetch_one::<(u64, u64, u64, u64, u64, u64)>()
                        .await?;

                    Ok::<_, anyhow::Error>(Stats {
                        requests: data.0,
                        user_agents: data.1,
                        ips: data.2,
                        origins: data.3,
                        continents: data.4,
                        countries: data.5,
                    })
                },
            )
            .await?;

        ApiResponse::json(Response {
            success: true,
            stats,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
