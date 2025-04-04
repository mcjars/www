use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::routes::{GetState, api::user::organizations::_organization_::GetOrganization};
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    struct Stats {
        requests: i64,
        user_agents: i64,
        ips: i64,
        origins: i64,
        continents: i64,
        countries: i64,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
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
    pub async fn route(
        state: GetState,
        organization: GetOrganization,
    ) -> axum::Json<serde_json::Value> {
        let stats = state
            .cache
            .cached(
                &format!("organization::stats::{}", organization.id),
                300,
                || async {
                    let data = sqlx::query(
                        r#"
                        SELECT
                            COUNT(*),
                            COUNT(DISTINCT requests.user_agent),
                            COUNT(DISTINCT requests.ip),
                            COUNT(DISTINCT requests.origin),
                            COUNT(DISTINCT requests.continent),
                            COUNT(DISTINCT requests.country)
                        FROM requests
                        WHERE requests.organization_id = $1
                        "#,
                    )
                    .bind(organization.id)
                    .fetch_one(state.database.read())
                    .await
                    .unwrap();

                    Stats {
                        requests: data.get(0),
                        user_agents: data.get(1),
                        ips: data.get(2),
                        origins: data.get(3),
                        continents: data.get(4),
                        countries: data.get(5),
                    }
                },
            )
            .await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                stats,
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
