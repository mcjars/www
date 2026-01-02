use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::{
        models::build::{Build, InstallationStep},
        response::{ApiResponse, ApiResponseResult},
        routes::GetState,
    };
    use axum::{
        body::Body,
        extract::{Path, Query},
    };
    use reqwest::StatusCode;
    use serde::Deserialize;
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Params {
        #[serde(default)]
        echo: bool,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = String),
        (status = NOT_FOUND, body = String),
    ), params(
        (
            "build",
            description = "The build number or hash to lookup",
            example = "b1f3eeac53355d9ba5cf19e36abe8b2a30278c0e60942f3d07ac9ac9e4564951",
        ),
        (
            "echo",
            Query,
            description = "Whether to echo inside the script",
            example = "true",
        ),
    ))]
    pub async fn route(
        state: GetState,
        Path(identifier): Path<String>,
        Query(query): Query<Params>,
    ) -> ApiResponseResult {
        let data = Build::by_v1_identifier(&state.database, &state.cache, &identifier).await?;

        if let Some((build, _, version)) = data {
            let mut script = format!(
                r#"
Write-Host "Installing Server"
$env:JAVA_VERSION = {}
                "#,
                version.java
            )
            .trim()
            .to_string();

            for combined in build.installation {
                let mut steps = vec![
                    "".to_string(),
                    "".to_string(),
                    "Invoke-Command {".to_string(),
                ];

                for step in combined {
                    match step {
                        InstallationStep::Remove(step) => {
                            steps.push(format!(
                                r#"
Write-Host "Removing {}"
Remove-Item -Recurse -Force {}
                                "#,
                                step.location, step.location
                            ));
                        }
                        InstallationStep::Download(step) => {
                            steps.push(format!(
                                r#"
Write-Host "Downloading {}"
New-Item -ItemType Directory -Force ./{}
Invoke-WebRequest -Uri '{}' -OutFile {}
                                "#,
                                step.file,
                                std::path::Path::new(&step.file)
                                    .parent()
                                    .unwrap()
                                    .to_str()
                                    .unwrap(),
                                step.url,
                                step.file
                            ));
                        }
                        InstallationStep::Unzip(step) => {
                            steps.push(format!(
                                r#"
Write-Host "Unzipping {}"
New-Item -ItemType Directory -Force {}
Expand-Archive -Path {} -DestinationPath {}
                                "#,
                                step.file, step.location, step.file, step.location
                            ));
                        }
                    }
                }

                steps.push("}".to_string());

                script.push_str(
                    &steps
                        .iter()
                        .map(|s| s.trim())
                        .collect::<Vec<&str>>()
                        .join("\n"),
                );
            }

            script.push_str(
                r#"

Write-Host "Installation complete"
Write-Host "Use Java version: $env:JAVA_VERSION"
exit 0
                "#
                .trim_end(),
            );

            ApiResponse::new(Body::from(
                script
                    .lines()
                    .filter(|l| !l.starts_with("Write-Host") || query.echo)
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ))
            .ok()
        } else {
            ApiResponse::new(Body::from(
                r#"
Write-Host "Build not found"
exit 1
                "#
                .trim(),
            ))
            .with_status(StatusCode::NOT_FOUND)
            .with_header("Content-Type", "text/plain")
            .ok()
        }
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
