use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod history;

mod get {
    use crate::routes::GetState;
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TypeStats {
        total: i64,
        unique_ips: i64,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,

        #[schema(inline)]
        types: IndexMap<String, TypeStats>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState) -> axum::Json<serde_json::Value> {
        let types = state
            .cache
            .cached("lookups::types::all", 10800, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        x.type AS type,
                        COUNT(*) AS total,
                        COUNT(DISTINCT ip) AS unique_ips
                    FROM (
                        SELECT
                            requests.data->'build'->>'type' AS type,
                            requests.ip AS ip
                        FROM requests
                        WHERE
                            requests.status = 200
                            AND requests.data IS NOT NULL
                            AND requests.path NOT LIKE '%tracking=nostats%'
                            AND requests.data->>'type' = 'lookup'
                    ) AS x
                    WHERE x.type IS NOT NULL
                    GROUP BY x.type
                    ORDER BY total DESC
                    "#,
                )
                .fetch_all(state.database.read())
                .await
                .unwrap();

                let mut types = IndexMap::new();

                for row in data {
                    types.insert(
                        row.get("type"),
                        TypeStats {
                            total: row.get("total"),
                            unique_ips: row.get("unique_ips"),
                        },
                    );
                }

                types
            })
            .await;

        axum::Json(
            serde_json::to_value(&Response {
                success: true,
                types,
            })
            .unwrap(),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/history", history::router(state))
        .with_state(state.clone())
}
