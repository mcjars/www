use super::{GetState, State};
use crate::models::r#type::ServerType;
use axum::{http::Response, routing::get};
use utoipa_axum::router::OpenApiRouter;

mod _type_;

const INDEX_HTML: &str = include_str!("../../../static/index.html");

pub struct IndexFile {
    pub name: String,
    pub size: String,
    pub href: Option<String>,
}

pub fn render(state: GetState, location: &str, files: &[IndexFile]) -> Response<String> {
    let html = INDEX_HTML
        .replace("{{VERSION}}", &state.version)
        .replace("{{LOCATION}}", location)
        .replace(
            "<!-- ENTRIES -->",
            &files
                .iter()
                .map(|f| {
                    let href = f.href.clone().unwrap_or(f.name.clone());

                    format!(
                        r#"
                        <tr>
                            <td><a href="{}">{}</a></td>
                            <td class="size">{}</td>
                        </tr>
                        "#,
                        if href.starts_with("https") {
                            href
                        } else {
                            format!("./{}", href)
                        },
                        f.name,
                        f.size
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );

    Response::builder()
        .header("Content-Type", "text/html")
        .header("Cache-Control", "no-cache")
        .body(html)
        .unwrap()
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .route(
            "/",
            get(|state: GetState| async move {
                let types = ServerType::all(&state.database, &state.cache).await;

                let files = types
                    .into_iter()
                    .map(|(k, t)| IndexFile {
                        name: format!("{}/", t.name),
                        size: format!("{} builds", t.builds),
                        href: Some(format!("{}/", k.to_string().to_lowercase())),
                    })
                    .collect::<Vec<_>>();

                render(state, "/", &files)
            }),
        )
        .nest("/{type}", _type_::router(state))
        .with_state(state.clone())
}
