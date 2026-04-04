use super::{GetState, State};
use crate::{
    models::{build::Build, r#type::ServerType},
    response::ApiResponse,
};
use axum::{extract::Path, http::StatusCode, routing::get};
use compact_str::ToCompactString;
use utoipa_axum::router::OpenApiRouter;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .route(
            "/",
            get(
                |state: GetState, Path((_, _, identifier)): Path<(ServerType, String, String)>| async move {
                    let build = Build::by_v1_identifier(&state.database, &state.cache, &identifier).await?;

                    if let Some((build, _, _)) = build {
                        if build.changes.is_empty() {
                            return ApiResponse::new("no changelog available".into()).ok();
                        }

                        let mut changelog = String::new();
                        changelog.reserve_exact(build.changes.iter().map(|line| line.len() + 3).sum());

                        for line in build.changes {
                            changelog.push_str("- ");
                            changelog.push_str(&line);
                            changelog.push('\n');
                        }

                        let changelog_len = changelog.len();

                        ApiResponse::new(changelog.into()).with_header("Content-Length", &changelog_len.to_compact_string()).ok()
                    } else {
                        ApiResponse::new("build not found".into()).with_status(StatusCode::NOT_FOUND).ok()
                    }
                },
            ),
        )
        .with_state(state.clone())
}
