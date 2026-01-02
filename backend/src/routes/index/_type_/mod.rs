use super::{GetState, IndexFile, State};
use crate::models::{r#type::ServerType, version::Version};
use axum::{extract::Path, routing::get};
use utoipa_axum::router::OpenApiRouter;

mod _version_;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .route(
            "/",
            get(
                |state: GetState, Path(r#type): Path<ServerType>| async move {
                    let versions = Version::all(&state.database, &state.cache, r#type).await?;

                    let files = versions
                        .into_iter()
                        .map(|(n, v)| IndexFile {
                            name: compact_str::format_compact!("{n}/"),
                            size: compact_str::format_compact!("{} builds", v.builds),
                            href: None,
                        })
                        .collect::<Vec<_>>();

                    super::render(
                        &state,
                        &compact_str::format_compact!("/{}/", r#type.infos(&state.env).name),
                        files,
                    )
                },
            ),
        )
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
