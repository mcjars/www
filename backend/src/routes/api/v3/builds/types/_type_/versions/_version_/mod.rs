use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod latest;

mod get {
    use crate::{
        models::{
            PaginationParamsWithSearchAndFields, build::Build, r#type::ServerType, version::Version,
        },
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiErrorV3, GetData, GetState},
    };
    use axum::{
        extract::{Path, Query},
        http::StatusCode,
    };
    use serde::Serialize;
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        builds: crate::models::Pagination<crate::models::build::ApiBuildV3>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiErrorV3)),
        (status = NOT_FOUND, body = inline(ApiErrorV3)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
        (
            "version" = String,
            description = "The server version (can be 'latest'/'latest-snapshot' for the latest version)",
            example = "1.17.1",
        ),
        (
            "fields" = String, Query,
            description = "HTML form data array of build fields to include in the response (e.g. fields=created&fields=java)",
        ),
        (
            "page" = i64, Query,
            description = "The page number (starting from 1)",
            minimum = 1,
            example = 1,
        ),
        (
            "per_page" = i64, Query,
            description = "The number of items per page",
            minimum = 1,
            maximum = 200,
            example = 50,
        ),
        (
            "search" = String, Query,
            description = "A search term to filter builds by name",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        params: Query<PaginationParamsWithSearchAndFields>,
        Path((r#type, version)): Path<(ServerType, String)>,
    ) -> ApiResponseResult {
        if let Err(errors) = crate::utils::validate_data(&params.0) {
            return ApiResponse::new_serialized(ApiErrorV3::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let Some((location, version)) =
            Version::resolve(&state.database, &state.cache, r#type, &version).await?
        else {
            return ApiResponse::error("version not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        let data = state
            .cache
            .cached(
                &format!(
                    "builds::{type}::{version}::pagination::{}::{}::{}",
                    params.page,
                    params.per_page,
                    params.search.as_deref().unwrap_or("")
                ),
                900,
                || async {
                    Build::all_by_version_with_pagination(
                        &state.database,
                        r#type,
                        &location,
                        &version,
                        params.page,
                        params.per_page,
                        params.search.as_deref(),
                    )
                    .await
                },
            )
            .await?;

        *request_data.lock().unwrap() = json!({
            "type": "builds",
            "search": {
                "type": r#type,
                "version": version,
            }
        });

        ApiResponse::new_serialized(json!({
                "builds": data.map(|build| crate::utils::extract_fields(build.into_api_v3(), &params.fields))
            }))
            .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/latest", latest::router(state))
        .with_state(state.clone())
}
