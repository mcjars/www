use super::State;
use crate::models::{build::Build, r#type::ServerType, version::Version};
use garde::Validate;
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
    },
};
use serde::Deserialize;
use serde_json::json;
use std::{str::FromStr, sync::Arc};

const INSTRUCTIONS: &str = "MCJars indexes Minecraft server software (Vanilla, Paper, Purpur, Fabric, Forge, \
NeoForge, Velocity, and many more) and every build of it, with download links, changelogs, file hashes and \
default config files.

Start with list_server_types to see which server types exist, then list_versions for a type and list_builds \
or get_latest_build for a version. Server types are identifiers such as \"PAPER\" or \"VELOCITY\". Versions \
are Minecraft versions such as \"1.21.4\", except for Velocity, Velocity-CTD and NanoLimbo, which use their own \
project versions such as \"3.4.0-SNAPSHOT\".

Version aliases: \"latest\" is the newest Minecraft release that has builds of the type, \"latest-snapshot\" is \
the newest version including snapshots. For project-versioned types both mean the project version with the \
newest build, which is often a -SNAPSHOT line, since those types do not distinguish releases from snapshots; \
for them `supported` only marks the newest project version. Aliases can lag a new version by up to a day. \
Builds also carry their own `experimental` flag: check it before recommending a build for production.

Every build carries an `installation` array of stages. Run the stages in order; the steps inside one stage are \
independent and may run in parallel. Step types: `download` (create the parent directory of `file`, then fetch \
`url` into `file`; `size` is in bytes), `unzip` (extract `file` into `location`, overwriting) and `remove` \
(recursively delete `location`, which can be a whole directory such as `libraries`, so warn before running it \
on an existing server).

To identify an unknown server jar, hash it and pass the hash to lookup_build.";

#[derive(Deserialize, JsonSchema, Validate)]
#[schemars(crate = "rmcp::schemars")]
struct ListVersionsParams {
    /// Server type identifier, e.g. "PAPER". See list_server_types.
    #[garde(skip)]
    r#type: String,
    /// Case-sensitive substring the version id must contain, e.g. "1.21".
    #[garde(length(chars, min = 1, max = 128))]
    #[schemars(length(min = 1, max = 128))]
    search: Option<String>,
    /// Page number, starting from 1.
    #[garde(range(min = 1))]
    #[schemars(range(min = 1))]
    #[serde(default = "default_page")]
    page: i64,
    /// Versions per page.
    #[garde(range(min = 1, max = 200))]
    #[schemars(range(min = 1, max = 200))]
    #[serde(default = "default_versions_per_page")]
    per_page: i64,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[schemars(crate = "rmcp::schemars")]
struct ListBuildsParams {
    /// Server type identifier, e.g. "PAPER". See list_server_types.
    #[garde(skip)]
    r#type: String,
    /// Version id as returned by list_versions, or the alias "latest" / "latest-snapshot" (see the server instructions).
    #[garde(length(chars, min = 1, max = 128))]
    #[schemars(length(min = 1, max = 128))]
    version: String,
    /// Case-insensitive substring the build name must contain.
    #[garde(length(chars, min = 1, max = 128))]
    #[schemars(length(min = 1, max = 128))]
    search: Option<String>,
    /// Page number, starting from 1.
    #[garde(range(min = 1))]
    #[schemars(range(min = 1))]
    #[serde(default = "default_page")]
    page: i64,
    /// Builds per page, newest first.
    #[garde(range(min = 1, max = 200))]
    #[schemars(range(min = 1, max = 200))]
    #[serde(default = "default_builds_per_page")]
    per_page: i64,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[schemars(crate = "rmcp::schemars")]
struct LatestBuildParams {
    /// Server type identifier, e.g. "PAPER". See list_server_types.
    #[garde(skip)]
    r#type: String,
    /// Version id as returned by list_versions, or the alias "latest" / "latest-snapshot" (see the server instructions).
    #[garde(length(chars, min = 1, max = 128))]
    #[schemars(length(min = 1, max = 128))]
    version: String,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[schemars(crate = "rmcp::schemars")]
struct BuildParams {
    /// Build uuid, or the hex md5/sha1/sha224/sha256/sha384/sha512 hash of one of its files.
    #[garde(length(chars, min = 1, max = 128))]
    #[schemars(length(min = 1, max = 128))]
    build: String,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[schemars(crate = "rmcp::schemars")]
struct BuildConfigsParams {
    /// Build uuid, or the hex md5/sha1/sha224/sha256/sha384/sha512 hash of one of its files.
    #[garde(length(chars, min = 1, max = 128))]
    #[schemars(length(min = 1, max = 128))]
    build: String,
    /// Only return the config file at this path, e.g. "server.properties" or "config/paper-global.yml".
    #[garde(length(chars, min = 1, max = 256))]
    #[schemars(length(min = 1, max = 256))]
    location: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_versions_per_page() -> i64 {
    25
}

fn default_builds_per_page() -> i64 {
    10
}

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(value.to_string())])
}

fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message)])
}

fn internal_error(err: anyhow::Error) -> ErrorData {
    tracing::error!("an mcp tool error occurred: {:?}", err);
    sentry_anyhow::capture_anyhow(&err);

    ErrorData::internal_error("internal server error", None)
}

fn invalid<T: Validate>(data: &T) -> Option<CallToolResult>
where
    T::Context: Default,
{
    crate::utils::validate_data(data)
        .err()
        .map(|errors| error_result(format!("invalid arguments: {}", errors.join(", "))))
}

fn no_builds(r#type: ServerType, version: &str) -> CallToolResult {
    error_result(format!(
        "{type} has no builds for version `{version}`, use list_versions to see the versions it has"
    ))
}

fn build_not_found(identifier: &str) -> CallToolResult {
    let is_hash = matches!(identifier.len(), 32 | 40 | 56 | 64 | 96 | 128)
        && identifier.chars().all(|c| c.is_ascii_hexdigit());

    if is_hash || identifier.parse::<uuid::Uuid>().is_ok() {
        error_result(format!("build `{identifier}` not found"))
    } else {
        error_result(format!(
            "`{identifier}` is not a build uuid or a hex md5/sha1/sha224/sha256/sha384/sha512 hash"
        ))
    }
}

fn parse_type(r#type: &str) -> Option<ServerType> {
    ServerType::from_str(r#type.trim()).ok()
}

fn version_alias(version: &str) -> &str {
    let version = version.trim();

    ["latest", "latest-snapshot"]
        .into_iter()
        .find(|alias| version.eq_ignore_ascii_case(alias))
        .unwrap_or(version)
}

fn unknown_type(r#type: &str) -> CallToolResult {
    error_result(format!(
        "unknown server type `{type}`, use list_server_types to see the valid identifiers"
    ))
}

#[derive(Clone)]
pub struct McjarsMcp {
    state: State,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl McjarsMcp {
    fn new(state: State) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }

    /// List every server type MCJars has builds for, keyed by its identifier (e.g. "PAPER"), with its name,
    /// description, homepage, categories, compatibility, whether it is deprecated or experimental, and how many
    /// builds and versions it has. Pass the identifier as `type` to the other tools.
    #[tool(annotations(
        title = "List server types",
        read_only_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn list_server_types(&self) -> Result<CallToolResult, ErrorData> {
        let types = ServerType::all(&self.state.database, &self.state.cache, &self.state.env)
            .await
            .map_err(internal_error)?;

        Ok(json_result(json!({ "types": types })))
    }

    /// List the versions of a server type, newest first, with whether each is still supported, the Java major
    /// version it needs, its build count and when it was released. For most types these are Minecraft versions;
    /// Velocity, Velocity-CTD and NanoLimbo use their own project versions, are all typed RELEASE, and only
    /// their newest project version is marked `supported`. Page through `total` with `page`.
    #[tool(annotations(
        title = "List versions",
        read_only_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn list_versions(
        &self,
        Parameters(params): Parameters<ListVersionsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if let Some(result) = invalid(&params) {
            return Ok(result);
        }
        let Some(r#type) = parse_type(&params.r#type) else {
            return Ok(unknown_type(&params.r#type));
        };

        let versions = Version::all(&self.state.database, &self.state.cache, r#type)
            .await
            .map_err(internal_error)?;

        let matched = versions
            .into_iter()
            .rev()
            .filter(|(id, _)| {
                params
                    .search
                    .as_deref()
                    .is_none_or(|search| id.contains(search))
            })
            .collect::<Vec<_>>();
        let total = matched.len();

        let data = matched
            .into_iter()
            .skip(((params.page - 1) * params.per_page) as usize)
            .take(params.per_page as usize)
            .map(|(id, version)| {
                let version = version.into_api_version_v3(id);

                json!({
                    "id": version.id,
                    "type": version.r#type,
                    "supported": version.supported,
                    "java": version.java,
                    "builds": version.builds,
                    "created": version.created,
                })
            })
            .collect::<Vec<_>>();

        Ok(json_result(json!({
            "total": total,
            "page": params.page,
            "per_page": params.per_page,
            "versions": data,
        })))
    }

    /// List the builds of one version of a server type, newest first, each with its name, changelog (`changes`),
    /// creation date and `installation`. Use get_latest_build instead when only the newest build matters.
    /// "latest" is the newest Minecraft release with builds of the type, "latest-snapshot" includes snapshots;
    /// the resolved version is returned as `version`.
    /// `installation` is an array of stages holding the download links: run the stages in order, the steps
    /// inside a stage may run in parallel. Steps are `download` (fetch `url` into `file`), `unzip` (extract
    /// `file` into `location`) or `remove` (recursively delete `location`, which may be a directory such as
    /// `libraries`, so warn before running it on an existing server). Check `experimental` before recommending
    /// a build for production.
    #[tool(annotations(
        title = "List builds",
        read_only_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn list_builds(
        &self,
        Parameters(params): Parameters<ListBuildsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if let Some(result) = invalid(&params) {
            return Ok(result);
        }
        let Some(r#type) = parse_type(&params.r#type) else {
            return Ok(unknown_type(&params.r#type));
        };

        let Some((location, version)) = Version::resolve(
            &self.state.database,
            &self.state.cache,
            r#type,
            version_alias(&params.version),
        )
        .await
        .map_err(internal_error)?
        else {
            return Ok(error_result(format!(
                "version `{}` not found for {type}, use list_versions to see the valid ids",
                params.version
            )));
        };

        let builds = self
            .state
            .cache
            .cached(
                &format!(
                    "builds::{type}::{version}::pagination::{}::{}::{}",
                    params.page,
                    params.per_page,
                    params.search.as_deref().unwrap_or("")
                ),
                900,
                || async {
                    Build::all_by_version_with_pagination(
                        &self.state.database,
                        r#type,
                        &location,
                        &version,
                        params.page,
                        params.per_page,
                        params.search.as_deref(),
                    )
                    .await
                },
            )
            .await
            .map_err(internal_error)?;

        if builds.total == 0 && params.search.is_none() {
            return Ok(no_builds(r#type, &version));
        }

        Ok(json_result(json!({
            "version": version,
            "builds": builds.map(|build| build.into_api_v3()),
        })))
    }

    /// Get the newest build of one version of a server type, with its changelog (`changes`) and `installation`.
    /// "latest" is the newest Minecraft release with builds of the type, "latest-snapshot" includes snapshots;
    /// for Velocity, Velocity-CTD and NanoLimbo both mean the project version with the newest build, often a
    /// -SNAPSHOT line. The newest build can itself be experimental.
    /// `installation` is an array of stages holding the download links: run the stages in order, the steps
    /// inside a stage may run in parallel. Steps are `download` (fetch `url` into `file`), `unzip` (extract
    /// `file` into `location`) or `remove` (recursively delete `location`, which may be a directory such as
    /// `libraries`, so warn before running it on an existing server). Check `experimental` before recommending
    /// a build for production.
    #[tool(annotations(
        title = "Get latest build",
        read_only_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn get_latest_build(
        &self,
        Parameters(params): Parameters<LatestBuildParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if let Some(result) = invalid(&params) {
            return Ok(result);
        }
        let Some(r#type) = parse_type(&params.r#type) else {
            return Ok(unknown_type(&params.r#type));
        };

        let Some((location, version)) = Version::resolve(
            &self.state.database,
            &self.state.cache,
            r#type,
            version_alias(&params.version),
        )
        .await
        .map_err(internal_error)?
        else {
            return Ok(error_result(format!(
                "version `{}` not found for {type}, use list_versions to see the valid ids",
                params.version
            )));
        };

        let build = self
            .state
            .cache
            .cached(&format!("build::{type}::{version}::latest"), 3600, || {
                Build::by_build_number(&self.state.database, r#type, &location, &version, None)
            })
            .await
            .map_err(internal_error)?;

        Ok(match build {
            Some(build) => json_result(json!({ "build": build.into_api_v3() })),
            None => no_builds(r#type, &version),
        })
    }

    /// Look up a single build by its uuid or the hex hash (md5, sha1, sha224, sha256, sha384 or sha512) of one
    /// of its files. Hashing an unknown server jar and passing it here identifies which server type, version
    /// and build it is. Returns the build, `is_latest` telling whether it is the newest build of its version,
    /// `latest` with that newest build only when it is a different one, and the version with its required Java
    /// major version.
    #[tool(annotations(
        title = "Look up build",
        read_only_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn lookup_build(
        &self,
        Parameters(params): Parameters<BuildParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if let Some(result) = invalid(&params) {
            return Ok(result);
        }

        let identifier = params.build.trim();
        let Some((build, latest, version)) =
            Build::by_identifier(&self.state.database, &self.state.cache, identifier)
                .await
                .map_err(internal_error)?
        else {
            return Ok(build_not_found(identifier));
        };

        let is_latest = build.uuid == latest.uuid;
        let mut result = json!({
            "build": build.into_api_v3(),
            "is_latest": is_latest,
            "version": version.into_api_v3(),
        });
        if !is_latest {
            result["latest"] = json!(latest.into_api_v3());
        }

        Ok(json_result(result))
    }

    /// Get the default config files a build ships with (e.g. server.properties, bukkit.yml,
    /// config/paper-global.yml), each with its path (`location`), format and full contents (`value`). The
    /// contents can be large, so pass `location` when only one file is needed; an unknown `location` answers
    /// with the list of locations the build has.
    #[tool(annotations(
        title = "Get build configs",
        read_only_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn get_build_configs(
        &self,
        Parameters(params): Parameters<BuildConfigsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if let Some(result) = invalid(&params) {
            return Ok(result);
        }

        let identifier = params.build.trim();
        let Some((build, _, _)) =
            Build::by_identifier(&self.state.database, &self.state.cache, identifier)
                .await
                .map_err(internal_error)?
        else {
            return Ok(build_not_found(identifier));
        };

        let mut configs = build
            .configs(&self.state.database, &self.state.cache)
            .await
            .map_err(internal_error)?;

        if configs.is_empty() {
            return Ok(json_result(json!({
                "configs": configs,
                "note": "this build has no indexed config files",
            })));
        }

        if let Some(location) = &params.location {
            let location = location.trim().trim_start_matches("./");
            let available = configs
                .iter()
                .map(|config| config.location.clone())
                .collect::<Vec<_>>();

            configs.retain(|config| config.location == location);

            if configs.is_empty() {
                return Ok(error_result(format!(
                    "this build has no config file at `{location}`, available locations: {}",
                    available.join(", ")
                )));
            }
        }

        Ok(json_result(json!({ "configs": configs })))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for McjarsMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("mcjars", self.state.version.clone()))
            .with_instructions(INSTRUCTIONS)
    }
}

pub fn service(state: &State) -> StreamableHttpService<McjarsMcp, NeverSessionManager> {
    let state = state.clone();

    StreamableHttpService::new(
        move || Ok(McjarsMcp::new(state.clone())),
        Arc::new(NeverSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true)
            .with_sse_keep_alive(None)
            .disable_allowed_hosts(),
    )
}
