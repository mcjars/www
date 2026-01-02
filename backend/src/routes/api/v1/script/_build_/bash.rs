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
        http::StatusCode,
    };
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
#!/bin/bash
export JAVA_VERSION={}

echo "Installing Server"
                "#,
                version.java
            )
            .trim()
            .to_string();

            for combined in build.installation {
                let mut steps = vec![];

                for step in combined {
                    match step {
                        InstallationStep::Remove(step) => {
                            steps.push(format!(
                                r#"
echo "Removing {}"
rm -rf {}
                                "#,
                                step.location, step.location
                            ));
                        }
                        InstallationStep::Download(step) => {
                            steps.push(format!(
                                r#"
echo "Downloading {}"
mkdir -p ./{}
rm -f {}
curl -s -o {} '{}'&
                                "#,
                                step.file,
                                std::path::Path::new(&step.file)
                                    .parent()
                                    .unwrap()
                                    .to_str()
                                    .unwrap(),
                                step.file,
                                step.file,
                                step.url
                            ));
                        }
                        InstallationStep::Unzip(step) => {
                            steps.push(format!(
                                r#"
echo "Unzipping {}"
mkdir -p {}
unzip -o {} -d {}&
                                "#,
                                step.file, step.location, step.file, step.location
                            ));
                        }
                    }
                }

                steps.push("wait".to_string());

                script.push_str(
                    &steps
                        .iter()
                        .map(|s| s.trim_end())
                        .collect::<Vec<&str>>()
                        .join("\n"),
                );
            }

            script.push_str(
                r#"

echo "Installation complete"
echo "Use Java version: $JAVA_VERSION"
exit 0
                "#
                .trim_end(),
            );

            ApiResponse::new(Body::from(
                script
                    .lines()
                    .filter(|l| !l.starts_with("echo") || query.echo)
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ))
            .with_header("Content-Type", "text/plain")
            .ok()
        } else {
            ApiResponse::new(Body::from(
                r#"
#!/bin/bash

echo "Build not found"
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
