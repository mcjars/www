use super::{GetState, IndexFile, State};
use crate::models::{
    build::{Build, InstallationStep},
    r#type::ServerType,
};
use axum::{extract::Path, routing::get};
use utoipa_axum::router::OpenApiRouter;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .route(
            "/",
            get(
                |state: GetState, Path((r#type, version, identifier)): Path<(ServerType, String, String)>| async move {
                    let build = Build::by_v1_identifier(&state.database, &state.cache, &identifier).await;

                    if let Some((build, _, _)) = build {
                        let mut files = Vec::new();

                        for combined in build.installation {
                            for step in combined {
                                match step {
                                    InstallationStep::Download(step) => {
                                        files.push(IndexFile {
                                            name: step.file,
                                            size: format!("{} bytes", step.size),
                                            href: Some(step.url),
                                        });
                                    }
                                    InstallationStep::Unzip(step) => {
                                        files.push(IndexFile {
                                            name: format!("unzip {} in {}/", step.file, step.location),
                                            size: "-".to_string(),
                                            href: Some("#".to_string()),
                                        });
                                    }
                                    InstallationStep::Remove(step) => {
                                        files.push(IndexFile {
                                            name: format!("remove {}/", step.location),
                                            size: "-".to_string(),
                                            href: Some("#".to_string()),
                                        });
                                    }
                                }
                            }
                        }

                        crate::routes::index::render(
                            &state,
                            &format!("/{}/{}/{}/", r#type.infos().name, version, build.name),
                            &files,
                        )
                    } else {
                        crate::routes::index::render(
                            &state,
                            &format!("/{}/{}/{}/", r#type.infos().name, version, identifier),
                            &[],
                        )
                    }
                },
            ),
        )
        .with_state(state.clone())
}
