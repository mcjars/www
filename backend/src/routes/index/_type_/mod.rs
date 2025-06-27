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
                    let versions = Version::all(&state.database, &state.cache, r#type).await;

                    let files = versions
                        .into_iter()
                        .map(|(n, v)| IndexFile {
                            name: format!("{n}/"),
                            size: format!("{} builds", v.builds),
                            href: None,
                        })
                        .collect::<Vec<_>>();

                    super::render(state, &format!("/{}/", r#type.infos().name), files)
                },
            ),
        )
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
