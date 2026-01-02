use crate::response::{ApiResponse, ApiResponseResult};
use axum::{
    ServiceExt,
    body::Body,
    extract::{Path, Request},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
    routing::get,
};
use colored::Colorize;
use compact_str::ToCompactString;
use include_dir::{Dir, include_dir};
use models::r#type::ServerType;
use routes::{ApiError, GetState};
use sentry_tower::SentryHttpLayer;
use sha2::Digest;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tower::Layer;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    catch_panic::CatchPanicLayer, cors::CorsLayer, normalize_path::NormalizePathLayer,
};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa_axum::router::OpenApiRouter;

mod cache;
mod clickhouse;
mod database;
mod env;
mod files;
mod models;
mod prelude;
mod requests;
mod response;
mod routes;
mod s3;
mod utils;

#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_COMMIT: &str = env!("CARGO_GIT_COMMIT");
const FRONTEND_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/lib");

fn render_index(meta: HashMap<&str, String>, state: GetState) -> ApiResponseResult {
    let index = FRONTEND_ASSETS
        .get_file("index.html")
        .unwrap()
        .contents_utf8()
        .unwrap()
        .to_string();
    let mut metadata = String::new();

    for (key, value) in meta {
        metadata.push_str(&format!("<meta name=\"{key}\" content=\"{value}\">"));
    }

    ApiResponse::new(Body::from(
        index
            .replace("<!-- META -->", &metadata)
            .replace("{{VERSION}}", &state.version),
    ))
    .with_header("Content-Type", "text/html")
    .ok()
}

fn handle_panic(_err: Box<dyn std::any::Any + Send + 'static>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "application/json")
        .body(Body::from(
            ApiError::new(&["internal server error"])
                .to_value()
                .to_string(),
        ))
        .unwrap()
}

async fn handle_request(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let ip = utils::extract_ip(req.headers())
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!(
        "http {} {}{} (from {})",
        req.method().to_string().to_lowercase(),
        req.uri().path().cyan(),
        if let Some(query) = req.uri().query() {
            format!("?{query}")
        } else {
            "".to_string()
        }
        .bright_cyan(),
        ip
    );

    Ok(next.run(req).await)
}

async fn handle_postprocessing(req: Request, next: Next) -> Result<Response, StatusCode> {
    let if_none_match = req.headers().get("If-None-Match").cloned();
    let mut response = next.run(req).await;

    if let Some(content_type) = response.headers().get("Content-Type")
        && content_type
            .to_str()
            .map(|c| c.starts_with("text/plain"))
            .unwrap_or(false)
        && response.status().is_client_error()
        && response.status() != StatusCode::NOT_FOUND
    {
        let (mut parts, body) = response.into_parts();

        let text_body = String::from_utf8(
            axum::body::to_bytes(body, usize::MAX)
                .await
                .unwrap()
                .into_iter()
                .by_ref()
                .collect::<Vec<u8>>(),
        )
        .unwrap();

        parts
            .headers
            .insert("Content-Type", "application/json".parse().unwrap());

        response = Response::from_parts(
            parts,
            Body::from(ApiError::new(&[&text_body]).to_value().to_string()),
        );
    }

    if !response.headers().contains_key("ETag") {
        let (mut parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

        let mut hash = sha2::Sha256::new();
        hash.update(body_bytes.as_ref());
        let hash = format!("{:x}", hash.finalize());

        parts.headers.insert("ETag", hash.parse().unwrap());

        if if_none_match == Some(hash.parse().unwrap()) {
            let mut cached_response = Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::empty())
                .unwrap();

            for (key, value) in parts.headers.iter() {
                cached_response.headers_mut().insert(key, value.clone());
            }

            return Ok(cached_response);
        }

        Ok(Response::from_parts(parts, Body::from(body_bytes)))
    } else {
        Ok(response)
    }
}

#[tokio::main]
async fn main() {
    let (env, _tracing_guard) = match env::Env::parse() {
        Ok((env, tracing_guard)) => (env, tracing_guard),
        Err(err) => {
            eprintln!("{}: {err:#?}", "failed to parse environment".red());
            std::process::exit(1);
        }
    };

    let _guard = sentry::init((
        env.sentry_url.clone(),
        sentry::ClientOptions {
            server_name: env.server_name.clone().map(|s| s.into()),
            release: Some(format!("{VERSION}:{GIT_COMMIT}").into()),
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ));

    let s3 = Arc::new(s3::S3::new(env.clone()).await);
    let database = Arc::new(database::Database::new(env.clone()).await);
    let clickhouse = Arc::new(clickhouse::Clickhouse::new(env.clone(), database.clone()).await);
    let cache = Arc::new(cache::Cache::new(env.clone()).await);

    let state = Arc::new(routes::AppState {
        start_time: Instant::now(),
        version: format!("{VERSION}:{GIT_COMMIT}"),

        database: database.clone(),
        clickhouse: clickhouse.clone(),
        cache: cache.clone(),
        requests: requests::RequestLogger::new(database.clone(), clickhouse.clone(), cache.clone()),
        files: files::FileCache::new(database.clone(), env.clone()).await,
        env,
        s3,
    });

    {
        let state = state.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                if let Err(err) = state.requests.process().await {
                    tracing::error!("failed to process requests: {:?}", err);
                    sentry_anyhow::capture_anyhow(&err);
                }
            }
        });
    }

    {
        let state = state.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                if let Err(err) = state.files.process().await {
                    tracing::error!("failed to process files: {:?}", err);
                    sentry_anyhow::capture_anyhow(&err);
                }
            }
        });
    }

    let app =
        OpenApiRouter::new()
            .merge(routes::router(&state))
            .route(
                "/icons/{type}",
                get(|state: GetState, Path::<ServerType>(r#type)| async move {
                    ApiResponse::new(Body::empty())
                        .with_status(StatusCode::FOUND)
                        .with_header(
                            "Location",
                            &format!(
                                "{}/icons/{}.png",
                                state.env.s3_url,
                                r#type.to_string().to_lowercase()
                            ),
                        )
                        .ok()
                }),
            )
            .route(
                "/download/{project}/{version}/{project_version}/{installer_version}",
                get(
                    |Path::<(String, String, String, String)>((
                        project,
                        version,
                        project_version,
                        installer_version,
                    ))| async move {
                        let response = match project.as_str() {
                            "fabric" => reqwest::get(
                                format!(
                                    "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
                                    version,
                                    project_version,
                                    installer_version.replace(".jar", "")
                                )
                                .as_str(),
                            )
                            .await,
                            "legacy-fabric" => reqwest::get(
                                format!(
                                    "https://meta.legacyfabric.net/v2/versions/loader/{}/{}/{}/server/jar",
                                    version,
                                    project_version,
                                    installer_version.replace(".jar", "")
                                )
                                .as_str(),
                            )
                            .await,
                            _ => return ApiResponse::error("project not supported")
                                .with_status(StatusCode::NOT_FOUND)
                                .ok(),
                        };

                        let response = match response {
                            Ok(response) => response,
                            Err(_) => {
                                return ApiResponse::error("error fetching build")
                                    .with_status(StatusCode::NOT_FOUND)
                                    .ok();
                            }
                        };

                        if !response.status().is_success() {
                            return ApiResponse::error("build not found")
                                .with_status(StatusCode::NOT_FOUND)
                                .ok();
                        }

                        let content_type = response
                            .headers()
                            .get("Content-Type")
                            .unwrap()
                            .to_str()?
                            .to_compact_string();
                        let content_disposition = response
                            .headers()
                            .get("Content-Disposition")
                            .unwrap()
                            .to_str()?
                            .to_compact_string();

                        ApiResponse::new(Body::from(response.bytes().await?))
                            .with_header("Content-Type", &content_type)
                            .with_header("Content-Disposition", &content_disposition)
                            .ok()
                    },
                ),
            )
            .route("/", get(|state: GetState, req: Request<Body>| async move {
                let meta = HashMap::from([
                    ("description", "MCJars is a service that provides Minecraft server owners with the ability to download server jars and other files with ease. Not affiliated with Mojang AB.".to_string()),
                    ("og:description", "MCJars is a service that provides Minecraft server owners with the ability to download server jars and other files with ease. Not affiliated with Mojang AB.".to_string()),
                    ("og:title", "MCJars".to_string()),
                    ("og:image", format!("{}/icons/vanilla.png", state.env.s3_url)),
                    ("og:url", req.uri().to_string()),
                ]);

                render_index(meta, state)
            }))
            .route("/lookup", get(|state: GetState, req: Request<Body>| async move {
                let meta = HashMap::from([
                    ("description", "Lookup Minecraft server jars and configs by their hash. Not affiliated with Mojang AB.".to_string()),
                    ("og:description", "Lookup Minecraft server jars and configs by their hash. Not affiliated with Mojang AB.".to_string()),
                    ("og:title", "MCJars | Reverse Lookup".to_string()),
                    ("og:image", format!("{}/icons/vanilla.png", state.env.s3_url)),
                    ("og:url", req.uri().to_string()),
                ]);

                render_index(meta, state)
            }))
            .route("/job-status", get(|state: GetState, req: Request<Body>| async move {
                let meta = HashMap::from([
                    ("description", "View the job status for MCJars. Not affiliated with Mojang AB.".to_string()),
                    ("og:description", "View the job status for MCJars. Not affiliated with Mojang AB.".to_string()),
                    ("og:title", "MCJars | Job Status".to_string()),
                    ("og:image", format!("{}/icons/vanilla.png", state.env.s3_url)),
                    ("og:url", req.uri().to_string()),
                ]);

                render_index(meta, state)
            }))
            .route("/organizations", get(|state: GetState, req: Request<Body>| async move {
                let meta = HashMap::from([
                    ("description", "MCJars is a service that provides Minecraft server owners with the ability to download server jars and other files with ease. Not affiliated with Mojang AB.".to_string()),
                    ("og:description", "MCJars is a service that provides Minecraft server owners with the ability to download server jars and other files with ease. Not affiliated with Mojang AB.".to_string()),
                    ("og:title", "MCJars".to_string()),
                    ("og:image", format!("{}/icons/vanilla.png", state.env.s3_url)),
                    ("og:url", req.uri().to_string()),
                ]);

                render_index(meta, state)
            }))
            .route("/{type}/versions", get(|state: GetState, Path::<ServerType>(r#type)| async move {
                let types = ServerType::all(&state.database, &state.cache, &state.env).await?;
                let data = types
                    .iter()
                    .find(|(k, _)| **k == r#type)
                    .unwrap();

                let builds = data.1.builds;
                let versions = if data.1.versions.minecraft == 0 {
                    data.1.versions.project
                } else {
                    data.1.versions.minecraft
                };

                let meta = HashMap::from([
                    ("description", format!("Download the latest {} server builds with ease. Browse {} builds for {} different versions on our website. Not affiliated with Mojang AB.", data.1.name, builds, versions).to_string()),
                    ("og:description", format!("Download the latest {} server builds with ease. Browse {} builds for {} different versions on our website. Not affiliated with Mojang AB.", data.1.name, builds, versions).to_string()),
                    ("og:title", format!("MCJars | {} Versions", data.1.name).to_string()),
                    ("og:image", format!("{}/icons/{}.png", state.env.s3_url, r#type.to_string().to_lowercase())),
                    ("og:url", format!("https://mcjars.app/{type}/versions")),
                ]);

                render_index(meta, state)
            }))
            .route("/{type}/statistics", get(|state: GetState, Path::<ServerType>(r#type)| async move {
                let data = r#type.infos(&state.env);

                let meta = HashMap::from([
                    ("description", format!("View the latest statistics for {}. Not affiliated with Mojang AB.", data.name).to_string()),
                    ("og:description", format!("View the latest statistics for {}. Not affiliated with Mojang AB.", data.name).to_string()),
                    ("og:title", format!("MCJars | {} Statistics", data.name).to_string()),
                    ("og:image", format!("{}/icons/{}.png", state.env.s3_url, r#type.to_string().to_lowercase())),
                    ("og:url", format!("https://mcjars.app/{type}/versions")),
                ]);

                render_index(meta, state)
            }))
            .route("/sitemap.xml", get(|state: GetState| async move {
                let mut sitemap = "
                <?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">
                <url><loc>https://mcjars.app</loc></url>
                <url><loc>https://mcjars.app/lookup</loc></url>
                ".trim().to_string();

                let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

                for r#type in ServerType::variants(&state.env) {
                    sitemap.push_str(&format!(
                        "<url><loc>https://mcjars.app/{type}/versions</loc><lastmod>{now}</lastmod></url>"
                    ));
                    sitemap.push_str(&format!(
                        "<url><loc>https://mcjars.app/{type}/statistics</loc><lastmod>{now}</lastmod></url>"
                    ));
                }

                sitemap.push_str("</urlset>");

                ApiResponse::new(Body::from(sitemap)).with_header("Content-Type", "application/xml").ok()
            }))
            .fallback(|state: GetState, req: Request<Body>| async move {
                if !req.uri().path().starts_with("/api") {
                    let path = &req.uri().path()[1..];

                    let file = if path.starts_with("assets") {
                        FRONTEND_ASSETS.get_dir("assets").unwrap().get_file(path).unwrap_or_else(|| {
                            FRONTEND_ASSETS
                                .get_file("index.html").unwrap()
                        })
                    } else {
                        FRONTEND_ASSETS.get_file(path).unwrap_or_else(|| {
                            FRONTEND_ASSETS
                                .get_file("index.html").unwrap()
                        })
                    };

                    let mut content = file.contents_utf8().unwrap().to_string();
                    if file.path().extension() == Some("html".as_ref()) {
                        content = content.replace("{{VERSION}}", &state.version);
                    }

                    return ApiResponse::new(Body::from(content))
                        .with_header(
                            "Content-Type",
                            match file.path().extension() {
                                Some(ext) if ext == "js" => "application/javascript",
                                Some(ext) if ext == "css" => "text/css",
                                Some(ext) if ext == "html" => "text/html",
                                _ => "text/plain",
                            },
                        )
                        .ok();
                }

                ApiResponse::error("route not found")
                    .with_status(StatusCode::NOT_FOUND)
                    .ok()
            })
            .layer(CatchPanicLayer::custom(handle_panic))
            .layer(CorsLayer::permissive().allow_methods([Method::GET, Method::POST]))
            .layer(axum::middleware::from_fn(handle_request))
            .layer(CookieManagerLayer::new())
            .route_layer(axum::middleware::from_fn(handle_postprocessing))
            .route_layer(SentryHttpLayer::new().enable_transaction())
            .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", &state.env.bind, state.env.port))
        .await
        .unwrap();

    tracing::info!(
        "{} listening on {} {}",
        "http server".bright_red(),
        state.env.bind.cyan(),
        format!(
            "(app@{VERSION}, {}ms)",
            state.start_time.elapsed().as_millis()
        )
        .bright_black()
    );

    let (router, mut openapi) = app.split_for_parts();
    openapi.info.version = state.version.clone();
    openapi.info.description = None;
    openapi.info.title = "MCJars API".to_string();
    openapi.info.contact = None;
    openapi.info.license = None;
    openapi.servers = Some(vec![utoipa::openapi::Server::new(
        state.env.app_url.clone(),
    )]);
    openapi.components.as_mut().unwrap().add_security_scheme(
        "api_key",
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
    );

    for (path, item) in openapi.paths.paths.iter_mut() {
        let operations = [
            ("get", &mut item.get),
            ("post", &mut item.post),
            ("put", &mut item.put),
            ("patch", &mut item.patch),
            ("delete", &mut item.delete),
        ];

        let path = path
            .replace('/', "_")
            .replace(|c| ['{', '}'].contains(&c), "");

        for (method, operation) in operations {
            if let Some(operation) = operation {
                operation.operation_id = Some(format!("{method}{path}"))
            }
        }
    }

    let openapi = Arc::new(openapi);
    let router = router.route("/openapi.json", get(|| async move { axum::Json(openapi) }));

    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service(
            NormalizePathLayer::trim_trailing_slash().layer(router),
        ),
    )
    .await
    .unwrap();
}
