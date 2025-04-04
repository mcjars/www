use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::routes::{
        ApiError, GetState, api::user::organizations::_organization_::GetOrganization,
    };
    use axum::{body::Bytes, http::StatusCode};
    use image::{ImageReader, codecs::webp::WebPEncoder, imageops::FilterType};
    use rustis::commands::GenericCommands;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize, Deserialize)]
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
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let image = ImageReader::new(std::io::Cursor::new(image)).with_guessed_format();
        if image.is_err() {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiError::new(&["invalid image"]).to_value()),
            );
        }

        let image = image.unwrap().decode();
        if image.is_err() {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiError::new(&["invalid image"]).to_value()),
            );
        }

        let image = image.unwrap().resize_exact(512, 512, FilterType::Triangle);
        let mut data: Vec<u8> = Vec::new();
        let encoder = WebPEncoder::new_lossless(&mut data);
        let color = image.color();
        encoder
            .encode(image.into_bytes().as_slice(), 512, 512, color.into())
            .unwrap();

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
            .await;

        if organization.icon.starts_with(&state.env.s3_url)
            && !organization.icon.ends_with("default.webp")
        {
            state
                .s3
                .bucket
                .delete_object(&organization.icon[state.env.s3_url.len() + 1..])
                .await
                .map(|_| ())
                .unwrap_or_default();
        }

        organization.icon = url.clone();
        organization.save(&state.database).await;

        let keys: Vec<String> = state
            .cache
            .client
            .keys(format!("organization::{}*", organization.id))
            .await
            .unwrap();
        if !keys.is_empty() {
            state.cache.client.del(keys).await.unwrap();
        }

        (
            StatusCode::OK,
            axum::Json(serde_json::to_value(&Response { success: true, url }).unwrap()),
        )
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
