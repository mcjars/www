use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::{BaseModel, build::Build, config::Config, r#type::ServerType},
        routes::{ApiError, GetState},
    };
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use sha1::Digest;
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
        build: Option<Build>,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Response {
        success: bool,
        formatted: String,

        #[schema(inline)]
        configs: Vec<Result>,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        axum::Json(data): axum::Json<Payload>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let config = Config::by_alias(&data.file);
        if config.is_none() {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(ApiError::new(&["version not found"]).to_value()),
            );
        }

        let config = config.unwrap();
        let (formatted, contains) = match Config::format(&data.file, &data.config) {
            Ok((formatted, contains)) => (formatted, contains),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(ApiError::new(&["unable to format config"]).to_value()),
                );
            }
        };

        let mut hash = sha1::Sha1::new();
        hash.update(&data.config);
        hash.update(&data.file);
        let hash = format!("{:x}", hash.finalize());

        let configs = state
            .cache
            .cached(
                &format!("config::{}", hash),
                10800,
                || async {
                    let data = if let Some(contains) = contains {
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
                                AND builds.type = $1::server_type
                            WHERE
                                configs.type = $1::server_type
                                AND configs.format = $2::format
                                AND configs.location = $3
                                AND config_values.value LIKE '%' || $4 || '%'
                            GROUP BY config_values.id, builds.id
                            LIMIT 3
                            "#,
                            Build::columns_sql(None, None)
                        ))
                        .bind(config.r#type.to_string())
                        .bind(config.format.to_string())
                        .bind(config.aliases[0].clone())
                        .bind(contains)
                        .fetch_all(state.database.read())
                        .await
                        .unwrap()
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
                                AND builds.type = $1::server_type
                            WHERE
                                configs.type = $1::server_type
                                AND configs.format = $2::format
                                AND configs.location = $3
                            ORDER BY similarity DESC
                            LIMIT 3
                            "#,
                            Build::columns_sql(None, None)
                        ))
                        .bind(config.r#type.to_string())
                        .bind(config.format.to_string())
                        .bind(config.aliases[0].clone())
                        .bind(&formatted)
                        .fetch_all(state.database.read())
                        .await
                        .unwrap()
                    };

                    let mut results = vec![];

                    for row in data {
                        let build = Build::map(None, &row);
                        let value: String = row.get("value");

                        results.push(Result {
                            from: build.r#type,
                            value,
                            build: Some(build),
                        });
                    }

                    results
                },
            )
            .await;

        (
            StatusCode::OK,
            axum::Json(
                serde_json::to_value(&Response {
                    success: true,
                    formatted,
                    configs,
                })
                .unwrap(),
            ),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
