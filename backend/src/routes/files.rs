use super::{GetState, State};
use crate::{
    models::file::File,
    response::ApiResponse,
    routes::index::{IndexFile, render},
};
use axum::{body::Body, extract::Request, http::Method, routing::any};
use std::path::{Component, Path, PathBuf};
use utoipa_axum::router::OpenApiRouter;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .fallback(any(|state: GetState, request: Request| async move {
            let path = Path::new(&request.uri().path()[1..]);

            if path.components().any(|c| matches!(c, Component::ParentDir)) {
                return render(&state, &format!("/{}", path.to_string_lossy()), vec![]);
            }

            if path.components().next_back().is_some_and(|c| {
                let string = c.as_os_str().to_string_lossy();

                string.ends_with(".txt") || string.ends_with(".jar") || string.ends_with(".zip")
            }) {
                let last = path
                    .components()
                    .next_back()
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy();

                if let Some(last) = last.strip_suffix(".CHECKSUMS.txt") {
                    let file = match File::by_path(
                        &state.database,
                        &state.cache,
                        &path
                            .components()
                            .take(path.components().count() - 1)
                            .collect::<PathBuf>()
                            .join(last),
                    )
                    .await?
                    {
                        Some(file) => file,
                        None => {
                            return render(&state, &format!("/{}", path.to_string_lossy()), vec![]);
                        }
                    };

                    if request.method() == Method::HEAD {
                        return ApiResponse::new(Body::empty())
                            .with_header("Content-Type", "text/plain")
                            .with_header("Content-Length", "459")
                            .ok();
                    } else {
                        let mut string = String::new();
                        string.reserve_exact(459);

                        string.push_str(&format!(
                            "md5    {}\n",
                            file.md5
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        ));
                        string.push_str(&format!(
                            "sha1   {}\n",
                            file.sha1
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        ));
                        string.push_str(&format!(
                            "sha224 {}\n",
                            file.sha224
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        ));
                        string.push_str(&format!(
                            "sha256 {}\n",
                            file.sha256
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        ));
                        string.push_str(&format!(
                            "sha384 {}\n",
                            file.sha384
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        ));
                        string.push_str(&format!(
                            "sha512 {}\n",
                            file.sha512
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        ));

                        return ApiResponse::new(Body::from(string))
                            .with_header("Content-Type", "text/plain")
                            .ok();
                    }
                }

                let file = match File::by_path(&state.database, &state.cache, path).await? {
                    Some(file) => file,
                    None => return render(&state, &format!("/{}", path.to_string_lossy()), vec![]),
                };

                if request.method() == Method::HEAD {
                    return ApiResponse::new(Body::empty())
                        .with_header(
                            "Content-Type",
                            if last.ends_with(".jar") {
                                "application/java-archive"
                            } else {
                                "application/zip"
                            },
                        )
                        .with_header("Content-Length", &file.size.to_string())
                        .with_header(
                            "ETag",
                            &file
                                .sha256
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>(),
                        )
                        .with_header("Cache-Control", "public, max-age=604800")
                        .ok();
                } else {
                    let file_reader = state.files.get(path, &file).await.unwrap();

                    return ApiResponse::new(Body::from_stream(tokio_util::io::ReaderStream::new(
                        file_reader,
                    )))
                    .with_header(
                        "Content-Type",
                        if last.ends_with(".jar") {
                            "application/java-archive"
                        } else {
                            "application/zip"
                        },
                    )
                    .with_header("Content-Length", &file.size.to_string())
                    .with_header(
                        "ETag",
                        &file
                            .sha256
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>(),
                    )
                    .with_header("Cache-Control", "public, max-age=604800")
                    .ok();
                }
            }

            let files = File::all_for_root(&state.database, &state.cache, path).await?;

            let mut index_files = Vec::new();
            index_files.reserve_exact(
                files
                    .iter()
                    .map(|f| if f.is_directory { 1 } else { 2 })
                    .sum(),
            );

            for f in files {
                index_files.push(IndexFile {
                    name: compact_str::format_compact!(
                        "{}{}",
                        f.name,
                        if f.is_directory { "/" } else { "" }
                    ),
                    size: human_bytes::human_bytes(f.size as f64).into(),
                    href: Some(compact_str::format_compact!(
                        "{}{}",
                        f.name,
                        if f.is_directory { "/" } else { "" }
                    )),
                });

                if !f.is_directory {
                    index_files.push(IndexFile {
                        name: compact_str::format_compact!("{}.CHECKSUMS.txt", f.name),
                        size: human_bytes::human_bytes(459).into(),
                        href: Some(compact_str::format_compact!("{}.CHECKSUMS.txt", f.name)),
                    });
                }
            }

            render(
                &state,
                &compact_str::format_compact!("/{}", path.to_string_lossy()),
                index_files,
            )
        }))
        .with_state(state.clone())
}
