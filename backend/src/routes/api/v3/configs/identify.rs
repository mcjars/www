use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::{BaseModel, build::Build, config::Config, r#type::ServerType},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiErrorV3, GetState},
    };
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct Payload {
        file: String,
        config: String,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Result {
        from: ServerType,
        value: String,
        build: Option<crate::models::build::ApiBuildV3>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        formatted: String,

        #[schema(inline)]
        configs: Vec<Result>,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiErrorV3)),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        crate::Payload(data): crate::Payload<Payload>,
    ) -> ApiResponseResult {
        let config = match Config::by_alias(&data.file) {
            Some(config) => config,
            None => {
                return ApiResponse::error("invalid config file")
                    .with_status(StatusCode::BAD_GATEWAY)
                    .ok();
            }
        };

        let (formatted, key_value) = match Config::format(&data.file, &data.config) {
            Ok((formatted, key_value)) => (formatted, key_value),
            Err(_) => {
                return ApiResponse::error("unable to format config")
                    .with_status(StatusCode::BAD_REQUEST)
                    .ok();
            }
        };

        let configs = state
            .cache
            .cached(
                &format!("config::{}::v3", serde_json::to_string(&data).unwrap()),
                10800,
                || async {
                    let data = if let Some((key, value)) = key_value {
                        sqlx::query(&format!(
                            r#"
                            SELECT
                                {},
                                config_values.value
                            FROM build_configs
                            INNER JOIN config_values ON config_values.id = build_configs.config_value_id
                            INNER JOIN configs ON configs.id = config_values.config_id
                            INNER JOIN builds ON
                                builds.id = build_configs.build_id
                                AND builds.type = $1
                            WHERE
                                configs.type = $1
                                AND configs.format = $2
                                AND configs.location = $3
                                AND config_values.parsed->$4 = $5
                            GROUP BY config_values.id, builds.id
                            LIMIT 3
                            "#,
                            Build::columns_sql(None, None)
                        ))
                        .bind(config.r#type)
                        .bind(config.format)
                        .bind(config.aliases[0])
                        .bind(key)
                        .bind(serde_json::to_value(value)?)
                        .fetch_all(state.database.read())
                        .await
                    } else {
                        sqlx::query(&format!(
                            r#"
                            SELECT
                                {},
                                config_values.value,
                                SIMILARITY(config_values.value, $4) AS similarity
                            FROM build_configs
                            INNER JOIN config_values ON config_values.id = build_configs.config_value_id
                            INNER JOIN configs ON configs.id = config_values.config_id
                            INNER JOIN builds ON
                                builds.id = build_configs.build_id
                                AND builds.type = $1
                            WHERE
                                configs.type = $1
                                AND configs.format = $2
                                AND configs.location = $3
                            ORDER BY similarity DESC
                            LIMIT 3
                            "#,
                            Build::columns_sql(None, None)
                        ))
                        .bind(config.r#type)
                        .bind(config.format)
                        .bind(config.aliases[0])
                        .bind(&formatted)
                        .fetch_all(state.database.read())
                        .await
                    }?;

                    let mut results = Vec::with_capacity(data.len());
                    for row in data {
                        let build = Build::map(None, &row)?;
                        let value: String = row.try_get("value")?;

                        results.push(Result {
                            from: build.r#type,
                            value,
                            build: Some(build.into_api_v3()),
                        });
                    }

                    Ok::<_, anyhow::Error>(results)
                },
            )
            .await?;

        ApiResponse::new_serialized(Response { formatted, configs }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
