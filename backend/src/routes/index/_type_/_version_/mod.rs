use super::{GetState, IndexFile, State};
use crate::models::{build::Build, r#type::ServerType, version::Version};
use axum::{extract::Path, routing::get};
use utoipa_axum::router::OpenApiRouter;

mod _build_;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .route(
            "/",
            get(
                |state: GetState, Path((r#type, version)): Path<(ServerType, String)>| async move {
                    let location =
                        Version::location(&state.database, &state.cache, r#type, &version).await;

                    let mut files = Vec::new();
                    if let Some(location) = location {
                        let data = state
                            .cache
                            .cached(&format!("builds::{}::{}", r#type, version), 1800, || {
                                Build::all_for_version(&state.database, r#type, &location, &version)
                            })
                            .await;

                        files = data
                            .into_iter()
                            .rev()
                            .map(|b| IndexFile {
                                name: format!("{}/", b.name),
                                size: format!("{} bytes", b.installation_size()),
                                href: Some(format!("{}/", b.id)),
                            })
                            .collect::<Vec<_>>();
                    }

                    crate::routes::index::render(
                        state,
                        &format!("/{}/{}/", r#type.infos().name, version),
                        &files,
                    )
                },
            ),
        )
        .nest("/{build}", _build_::router(state))
        .with_state(state.clone())
}
