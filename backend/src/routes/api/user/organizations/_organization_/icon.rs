use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        response::{ApiResponse, ApiResponseResult},
        routes::{ApiError, GetState, api::user::organizations::_organization_::GetOrganization},
    };
    use axum::{body::Bytes, http::StatusCode};
    use compact_str::ToCompactString;
    use image::{ImageReader, codecs::webp::WebPEncoder, imageops::FilterType};
    use serde::Serialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        success: bool,
        url: String,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = inline(ApiError)),
    ), params(
        (
            "organization" = i32,
            description = "The organization ID",
            example = 1,
        ),
    ), request_body = String)]
    pub async fn route(
        state: GetState,
        mut organization: GetOrganization,
        image: Bytes,
    ) -> ApiResponseResult {
        let image = match ImageReader::new(std::io::Cursor::new(image)).with_guessed_format() {
            Ok(reader) => reader,
            Err(_) => {
                return ApiResponse::error("invalid image")
                    .with_status(StatusCode::BAD_REQUEST)
                    .ok();
            }
        };

        let image = match tokio::task::spawn_blocking(move || image.decode()).await? {
            Ok(image) => image,
            Err(_) => {
                return ApiResponse::error("image: unable to decode")
                    .with_status(StatusCode::BAD_REQUEST)
                    .ok();
            }
        };

        let data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, image::ImageError> {
            let image = image.resize_exact(512, 512, FilterType::Triangle);
            let mut data: Vec<u8> = Vec::new();
            let encoder = WebPEncoder::new_lossless(&mut data);
            let color = image.color();
            encoder.encode(image.as_bytes(), 512, 512, color.into())?;

            Ok(data)
        })
        .await??;

        let url = state
            .s3
            .url(
                &format!(
                    "organization-icons/{}-{}.webp",
                    organization.id,
                    rand::random::<u32>()
                ),
                &data,
                Some("image/webp"),
            )
            .await?;

        if organization.icon.starts_with(&state.env.s3_url)
            && !organization.icon.ends_with("default.webp")
        {
            state
                .s3
                .bucket
                .delete_object(&organization.icon[state.env.s3_url.len() + 1..])
                .await
                .ok();
        }

        organization.icon = url.to_compact_string();
        organization.save(&state.database).await?;

        state.cache.clear_organization(organization.id).await?;

        ApiResponse::new_serialized(Response { success: true, url }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
