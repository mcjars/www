use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::{build::Build, config::Format, r#type::ServerType},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiErrorV3, GetData, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
    struct ResponseConfig {
        config_uuid: uuid::Uuid,
        value_uuid: uuid::Uuid,

        location: compact_str::CompactString,
        r#type: ServerType,
        format: Format,
        value: String,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        configs: Vec<ResponseConfig>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = inline(ApiErrorV3)),
    ), params(
        (
            "build",
            description = "The build id, uuid or hash",
            example = "2cd3b3b9-1250-47ff-9a18-81ab1a9bc348",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        Path(identifier): Path<String>,
    ) -> ApiResponseResult {
        let Some((build, _, version)) =
            Build::by_identifier(&state.database, &state.cache, &identifier).await?
        else {
            return ApiResponse::error("build not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        *request_data.lock().unwrap() = json!({
            "type": "lookup",
            "build": {
                "id": build.id,
                "type": build.r#type,
                "versionId": build.version_id,
                "projectVersionId": build.project_version_id,
                "buildNumber": build.build_number,
                "java": version.java,
            }
        });

        let configs = state
            .cache
            .cached(&format!("build_configs::{}", build.id), 3600, || async {
                let rows = sqlx::query(
                    r#"
                    SELECT
                        config_values.uuid AS value_uuid,
                        configs.uuid AS config_uuid,
                        configs.type AS type,
                        configs.format AS format,
                        configs.location AS location,
                        config_values.value AS value
                    FROM config_values
                    INNER JOIN build_configs ON build_configs.config_value_id = config_values.id
                    INNER JOIN configs ON configs.id = config_values.config_id
                    WHERE build_configs.build_id = $1
                    ORDER BY configs.id ASC
                    "#,
                )
                .bind(build.id)
                .fetch_all(state.database.read())
                .await?;

                let mut configs = Vec::new();

                for row in rows {
                    configs.push(ResponseConfig {
                        value_uuid: row.try_get("value_uuid")?,
                        config_uuid: row.try_get("config_uuid")?,
                        location: row.try_get("location")?,
                        r#type: row.try_get("type")?,
                        format: row.try_get("format")?,
                        value: row.try_get("value")?,
                    });
                }

                Ok::<_, sqlx::Error>(configs)
            })
            .await?;

        ApiResponse::new_serialized(Response { configs }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
