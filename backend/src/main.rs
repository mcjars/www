mod cache;
mod clickhouse;
mod database;
mod env;
mod files;
mod logger;
mod models;
mod requests;
mod routes;
mod s3;
mod utils;

use axum::{
    ServiceExt,
    body::Body,
    extract::{Path, Request},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::Response,
    routing::get,
};
use colored::Colorize;
use include_dir::{Dir, include_dir};
use models::r#type::ServerType;
use routes::{ApiError, GetState};
use sentry_tower::SentryHttpLayer;
use sha2::Digest;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tikv_jemallocator::Jemalloc;
use tower::Layer;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    catch_panic::CatchPanicLayer, cors::CorsLayer, normalize_path::NormalizePathLayer,
};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa_axum::router::OpenApiRouter;

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_COMMIT: &str = env!("CARGO_GIT_COMMIT");
const FRONTEND_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/lib");

#[inline]
fn render_index(meta: HashMap<&str, String>, state: GetState) -> (StatusCode, HeaderMap, String) {
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

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/html".parse().unwrap());

    (
        StatusCode::OK,
        headers,
        index
            .replace("<!-- META -->", &metadata)
            .replace("{{VERSION}}", &state.version),
    )
}

fn handle_panic(_err: Box<dyn std::any::Any + Send + 'static>) -> Response<Body> {
    logger::log(
        logger::LoggerLevel::Error,
        "a request panic has occurred".bright_red().to_string(),
    );

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

    logger::log(
        logger::LoggerLevel::Info,
        format!(
            "{} {}{} {}",
            format!("HTTP {}", req.method()).green().bold(),
            req.uri().path().cyan(),
            if let Some(query) = req.uri().query() {
                format!("?{query}")
            } else {
                "".to_string()
            }
            .bright_cyan(),
            format!("({ip})").bright_black(),
        ),
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
    let env = env::Env::parse();

    let _guard = sentry::init((
        env.sentry_url.clone(),
        sentry::ClientOptions {
            server_name: env.server_name.clone().map(|s| s.into()),
            release: Some(format!("{VERSION}:{GIT_COMMIT}").into()),
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ));

    let env = Arc::new(env);
    let s3 = Arc::new(s3::S3::new(env.clone()).await);
    let database = Arc::new(database::Database::new(env.clone()).await);
    let clickhouse = Arc::new(clickhouse::Clickhouse::new(env.clone()).await);
    let cache = Arc::new(cache::Cache::new(env.clone()).await);

    let state = Arc::new(routes::AppState {
        start_time: Instant::now(),
        version: format!("{VERSION}:{GIT_COMMIT}"),

        database: database.clone(),
        clickhouse,
        cache: cache.clone(),
        requests: requests::RequestLogger::new(database.clone(), cache.clone()),
        files: files::FileCache::new(database.clone(), env.clone()).await,
        env,
        s3,
    });

    {
        let state = state.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                state.requests.process().await.unwrap_or_default();
            }
        });
    }

    {
        let state = state.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                state.files.process().await.unwrap_or_default();
            }
        });
    }

    let app =
        OpenApiRouter::new()
            .merge(routes::router(&state))
            .route(
                "/icons/{type}",
                get(|state: GetState, Path::<ServerType>(r#type)| async move {
                    let mut headers = HeaderMap::new();

                    headers.insert(
                        "Location",
                        format!(
                            "{}/icons/{}.png",
                            state.env.s3_url,
                            r#type.to_string().to_lowercase()
                        )
                        .parse()
                        .unwrap(),
                    );

                    (StatusCode::FOUND, headers, "")
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
                        let mut headers = HeaderMap::new();

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
                            _ => return (
                                StatusCode::NOT_FOUND,
                                headers,
                                Body::from("project not supported"),
                            ),
                        };

                        let response = match response {
                            Ok(response) => response,
                            Err(_) => {
                                return (
                                    StatusCode::NOT_FOUND,
                                    headers,
                                    Body::from("error fetching build"),
                                );
                            }
                        };

                        if !response.status().is_success() {
                            return (
                                StatusCode::NOT_FOUND,
                                headers,
                                Body::from("build not found"),
                            );
                        }

                        headers.insert(
                            "Content-Type",
                            response.headers().get("Content-Type").unwrap().clone(),
                        );
                        headers.insert(
                            "Content-Disposition",
                            response
                                .headers()
                                .get("Content-Disposition")
                                .unwrap()
                                .clone(),
                        );

                        (
                            StatusCode::OK,
                            headers,
                            Body::from(response.bytes().await.unwrap()),
                        )
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
                let types = ServerType::all(&state.database, &state.cache).await;
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
                let data = r#type.infos();

                let meta = HashMap::from([
                    ("description", format!("View the latest statistics for {}. Not affiliated with Mojang AB.", data.name).to_string()),
                    ("og:description", format!("View the latest statistics for {}. Not affiliated with Mojang AB.", data.name).to_string()),
                    ("og:title", format!("MCJars | {} Statistics", data.name).to_string()),
                    ("og:image", format!("{}/icons/{}.png", state.env.s3_url, r#type.to_string().to_lowercase())),
                    ("og:url", format!("https://mcjars.app/{type}/versions")),
                ]);

                render_index(meta, state)
            }))
            .route("/sitemap.xml", get(|| async move {
                let mut headers = HeaderMap::new();
                let mut sitemap = "
                <?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">
                <url><loc>https://mcjars.app</loc></url>
                <url><loc>https://mcjars.app/lookup</loc></url>
                ".trim().to_string();

                let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

                for r#type in ServerType::variants() {
                    sitemap.push_str(&format!(
                        "<url><loc>https://mcjars.app/{type}/versions</loc><lastmod>{now}</lastmod></url>"
                    ));
                    sitemap.push_str(&format!(
                        "<url><loc>https://mcjars.app/{type}/statistics</loc><lastmod>{now}</lastmod></url>"
                    ));
                }

                sitemap.push_str("</urlset>");

                headers.insert(
                    "Content-Type",
                    "application/xml".parse().unwrap(),
                );

                (StatusCode::OK, headers, sitemap)
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

                    return Response::builder()
                        .header("Content-Type", match file.path().extension() {
                            Some(ext) if ext == "js" => "application/javascript",
                            Some(ext) if ext == "css" => "text/css",
                            Some(ext) if ext == "html" => "text/html",
                            _ => "text/plain",
                        })
                        .body(content)
                        .unwrap();
                }

                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(
                        ApiError::new(&["route not found"]).to_value().to_string(),
                    )
                    .unwrap()
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

    logger::log(
        logger::LoggerLevel::Info,
        format!(
            "{} listening on {} {}",
            "http server".bright_red(),
            listener.local_addr().unwrap().to_string().cyan(),
            format!(
                "(app@{}, {}ms)",
                VERSION,
                state.start_time.elapsed().as_millis()
            )
            .bright_black()
        ),
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
