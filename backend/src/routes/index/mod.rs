use super::{GetState, State};
use crate::{
    models::r#type::ServerType,
    response::{ApiResponse, ApiResponseResult},
};
use axum::{body::Body, routing::get};
use utoipa_axum::router::OpenApiRouter;

mod _type_;

const INDEX_HTML: &str = include_str!("../../../static/index.html");

pub struct IndexFile {
    pub name: compact_str::CompactString,
    pub size: compact_str::CompactString,
    pub href: Option<compact_str::CompactString>,
}

pub fn render(state: &GetState, location: &str, files: Vec<IndexFile>) -> ApiResponseResult {
    let html = INDEX_HTML
        .replace("{{VERSION}}", &state.version)
        .replace("{{LOCATION}}", location)
        .replace(
            "<!-- ENTRIES -->",
            &files
                .into_iter()
                .map(|f| {
                    let href = f.href.unwrap_or_else(|| f.name.clone());
                    let element = if href == "#" { "span" } else { "a" };

                    format!(
                        r#"
                        <tr>
                            <td>
                                <span class="icon {}-icon"></span>
                                <{element} href="{}">{}</{element}>
                            </td>
                            <td class="size">{}</td>
                        </tr>
                        "#,
                        if href.ends_with("/") {
                            "folder"
                        } else {
                            "file"
                        },
                        if href.starts_with("https") {
                            href
                        } else {
                            compact_str::format_compact!("./{href}")
                        },
                        f.name,
                        f.size
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );

    ApiResponse::new(Body::from(html))
        .with_header("Content-Type", "text/html")
        .with_header("Cache-Control", "no-cache")
        .ok()
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .route(
            "/",
            get(|state: GetState| async move {
                let types = ServerType::all(&state.database, &state.cache, &state.env).await?;

                let files = types
                    .into_iter()
                    .map(|(k, t)| IndexFile {
                        name: compact_str::format_compact!("{}/", t.name),
                        size: compact_str::format_compact!("{} builds", t.builds),
                        href: Some(compact_str::format_compact!(
                            "{}/",
                            k.to_string().to_lowercase()
                        )),
                    })
                    .collect::<Vec<_>>();

                render(&state, "/", files)
            }),
        )
        .nest("/{type}", _type_::router(state))
        .with_state(state.clone())
}
