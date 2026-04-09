use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _version_;

mod get {
    use crate::{
        models::{PaginationParamsWithSearchAndFields, r#type::ServerType, version::Version},
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiErrorV3, GetData, GetState},
    };
    use axum::{extract::Path, http::StatusCode};
    use axum_extra::extract::Query;
    use serde::Serialize;
    use serde_json::json;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        versions: crate::models::Pagination<crate::models::version::ApiVersionV3>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiErrorV3)),
    ), params(
        (
            "type" = ServerType,
            description = "The server type",
            example = "VANILLA",
        ),
        (
            "fields" = Vec<String>, Query,
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
            description = "The number of items per page (maximum 200)",
            minimum = 1,
            maximum = 200,
            example = 50,
        ),
        (
            "search" = String, Query,
            description = "Search term to filter versions by name",
        ),
    ))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        params: Query<PaginationParamsWithSearchAndFields>,
        Path(r#type): Path<ServerType>,
    ) -> ApiResponseResult {
        if let Err(errors) = crate::utils::validate_data(&params.0) {
            return ApiResponse::new_serialized(ApiErrorV3::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let data = Version::all(&state.database, &state.cache, r#type).await?;

        *request_data.lock().unwrap() = json!({
            "type": "builds",
            "search": {
                "type": r#type,
            }
        });

        let mut paginated = crate::models::Pagination {
            total: 0,
            per_page: params.per_page,
            page: params.page,
            data: Vec::new(),
        };

        let mut skipped = 0;
        for (id, version) in data.into_iter().rev() {
            if let Some(search) = &params.search
                && !id.contains(search.as_str())
            {
                continue;
            }

            paginated.total += 1;

            if skipped < (paginated.page - 1) * paginated.per_page {
                skipped += 1;
                continue;
            }

            if paginated.data.len() as i64 >= paginated.per_page {
                continue;
            }

            let version = version.into_api_version_v3(id);

            paginated.data.push(json!({
                "id": version.id,
                "type": version.r#type,
                "supported": version.supported,
                "java": version.java,
                "builds": version.builds,
                "created": version.created,
                "latest": crate::utils::extract_fields(version.latest, &params.fields),
            }));
        }

        ApiResponse::new_serialized(json!({
            "versions": paginated,
        }))
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
