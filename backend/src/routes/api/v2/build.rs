use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use crate::{
        models::{
            BaseModel,
            build::Build,
            config::Format,
            r#type::ServerType,
            version::{MinifiedVersion, VersionType},
        },
        routes::{ApiError, GetData, GetState},
    };
    use axum::{extract::Query, http::StatusCode};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Params {
        #[serde(default)]
        fields: String,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct Hash {
        primary: Option<bool>,
        sha1: Option<String>,
        sha224: Option<String>,
        sha256: Option<String>,
        sha384: Option<String>,
        sha512: Option<String>,
        md5: Option<String>,
    }

    impl Hash {
        #[inline]
        pub fn has_any(&self) -> bool {
            self.sha1.is_some()
                || self.sha224.is_some()
                || self.sha256.is_some()
                || self.sha384.is_some()
                || self.sha512.is_some()
                || self.md5.is_some()
        }
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[schema(rename_all = "camelCase")]
    pub struct BuildSearch {
        id: Option<u32>,
        r#type: Option<ServerType>,
        version_id: Option<String>,
        project_version_id: Option<String>,
        name: Option<String>,
        build_number: Option<u32>,
        experimental: Option<bool>,
        jar_url: Option<String>,
        jar_size: Option<u32>,
        zip_url: Option<String>,
        zip_size: Option<u32>,

        #[schema(inline)]
        hash: Option<Hash>,
    }

    impl BuildSearch {
        #[inline]
        pub fn has_any(&self) -> bool {
            self.id.is_some()
                || self.r#type.is_some()
                || self.version_id.is_some()
                || self.project_version_id.is_some()
                || self.build_number.is_some()
                || self.experimental.is_some()
                || self.jar_url.is_some()
                || self.jar_size.is_some()
                || self.zip_url.is_some()
                || self.zip_size.is_some()
                || self.hash.as_ref().map(|h| h.has_any()).unwrap_or(false)
        }
    }

    #[derive(ToSchema, Deserialize)]
    #[serde(untagged)]
    pub enum Payload {
        One(#[schema(inline)] Box<BuildSearch>),
        Many(#[schema(inline)] Vec<BuildSearch>),
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct ConfigValue {
        r#type: ServerType,
        format: Format,
        value: String,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct Result {
        build: Build,
        latest: Build,
        version: MinifiedVersion,

        #[schema(inline)]
        configs: IndexMap<String, ConfigValue>,
    }

    #[derive(ToSchema, Serialize)]
    struct ResponseOne {
        success: bool,
        build: Build,
        latest: Build,
        version: MinifiedVersion,

        #[schema(inline)]
        configs: IndexMap<String, ConfigValue>,
    }

    #[derive(ToSchema, Serialize)]
    struct ResponseMany {
        success: bool,

        #[schema(inline)]
        builds: Vec<Option<Result>>,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(ResponseOne)),
        (status = MULTI_STATUS, body = inline(ResponseMany)),
        (status = PAYLOAD_TOO_LARGE, body = inline(ApiError)),
        (status = NOT_FOUND, body = inline(ApiError)),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        request_data: GetData,
        params: Query<Params>,
        axum::Json(data): axum::Json<Payload>,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let fields = params
            .fields
            .split(',')
            .filter(|f| !f.is_empty())
            .collect::<Vec<_>>();

        match data {
            Payload::One(search) => {
                if let Some(result) = lookup_build(&state.database, &state.cache, *search).await {
                    *request_data.lock().unwrap() = json!({
                        "type": "lookup",
                        "build": {
                            "id": result.build.id,
                            "type": result.build.r#type,
                            "versionId": result.build.version_id,
                            "projectVersionId": result.build.project_version_id,
                            "buildNumber": result.build.build_number,
                            "java": result.version.java,
                        }
                    });

                    (
                        StatusCode::OK,
                        axum::Json(json!({
                            "success": true,
                            "build": crate::utils::extract_fields(result.build, &fields),
                            "latest": crate::utils::extract_fields(result.latest, &fields),
                            "version": result.version,
                            "configs": result.configs,
                        })),
                    )
                } else {
                    (
                        StatusCode::NOT_FOUND,
                        axum::Json(ApiError::new(&["build not found"]).to_value()),
                    )
                }
            }
            Payload::Many(searches) => {
                if searches.len() > 10 {
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        axum::Json(
                            ApiError::new(&["you can only search for up to 10 builds at a time"])
                                .to_value(),
                        ),
                    );
                }

                let mut results = Vec::with_capacity(searches.len());
                for search in searches {
                    results.push(lookup_build(&state.database, &state.cache, search));
                }

                let results = futures_util::future::join_all(results).await;

                if let Some(result) = results.iter().flatten().next() {
                    *request_data.lock().unwrap() = json!({
                        "type": "lookup",
                        "build": {
                            "id": result.build.id,
                            "type": result.build.r#type,
                            "versionId": result.build.version_id,
                            "projectVersionId": result.build.project_version_id,
                            "buildNumber": result.build.build_number,
                            "java": result.version.java,
                        }
                    });
                }

                (
                    StatusCode::MULTI_STATUS,
                    axum::Json(json!({
                        "success": true,
                        "builds": results.into_iter().map(|r| {
                            r.map(|result| {
                                json!({
                                    "build": crate::utils::extract_fields(result.build, &fields),
                                    "latest": crate::utils::extract_fields(result.latest, &fields),
                                    "version": result.version,
                                    "configs": result.configs,
                                })
                            })
                        }).collect::<Vec<_>>()
                    })),
                )
            }
        }
    }

    #[inline]
    async fn lookup_build(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        search: BuildSearch,
    ) -> Option<Result> {
        if !search.has_any() {
            return None;
        }

        cache.cached(&format!("build::{}", serde_json::to_string(&search).unwrap()), 3600, || async {
            let mut where_clause: Vec<String> = Vec::new();
            let mut data: Vec<serde_json::Value> = Vec::new();

            if let Some(id) = search.id {
                where_clause.push(format!("builds.id = ($1->>{})::int", data.len()));
                data.push(serde_json::to_value(id).unwrap());
            }
            if let Some(r#type) = search.r#type {
                where_clause.push(format!("builds.type = ($1->>{})::server_type", data.len()));
                data.push(serde_json::to_value(r#type).unwrap());
            }
            if let Some(version_id) = &search.version_id {
                where_clause.push(format!("builds.version_id = $1->>{}", data.len()));
                data.push(serde_json::to_value(version_id).unwrap());
            }
            if let Some(project_version_id) = &search.project_version_id {
                where_clause.push(format!("builds.project_version_id = $1->>{}", data.len()));
                data.push(serde_json::to_value(project_version_id).unwrap());
            }
            if let Some(name) = &search.name {
                where_clause.push(format!("builds.name = $1->>{}", data.len()));
                data.push(serde_json::to_value(name).unwrap());
            }
            if let Some(build_number) = search.build_number {
                where_clause.push(format!("builds.build_number = ($1->>{})::int", data.len()));
                data.push(serde_json::to_value(build_number).unwrap());
            }
            if let Some(experimental) = search.experimental {
                where_clause.push(format!("builds.experimental = ($1->>{})::bool", data.len()));
                data.push(serde_json::to_value(experimental).unwrap());
            }
            if let Some(jar_url) = &search.jar_url {
                where_clause.push(format!("builds.jar_url = $1->>{}", data.len()));
                data.push(serde_json::to_value(jar_url).unwrap());
            }
            if let Some(jar_size) = search.jar_size {
                where_clause.push(format!("builds.jar_size = ($1->>{})::int", data.len()));
                data.push(serde_json::to_value(jar_size).unwrap());
            }
            if let Some(zip_url) = &search.zip_url {
                where_clause.push(format!("builds.zip_url = ($1->>{})::int", data.len()));
                data.push(serde_json::to_value(zip_url).unwrap());
            }
            if let Some(zip_size) = search.zip_size {
                where_clause.push(format!("builds.zip_size = ($1->>{})::int", data.len()));
                data.push(serde_json::to_value(zip_size).unwrap());
            }
            if let Some(hash) = &search.hash && hash.has_any() {
                if let Some(primary) = hash.primary {
                    where_clause.push(format!("build_hashes.primary = ($1->>{})::bool", data.len()));
                    data.push(serde_json::to_value(primary).unwrap());
                }
                if let Some(sha1) = &hash.sha1 {
                    where_clause.push(format!("build_hashes.sha1 = decode($1->>{}, 'hex')", data.len()));
                    data.push(serde_json::to_value(sha1).unwrap());
                }
                if let Some(sha224) = &hash.sha224 {
                    where_clause.push(format!("build_hashes.sha224 = decode($1->>{}, 'hex')", data.len()));
                    data.push(serde_json::to_value(sha224).unwrap());
                }
                if let Some(sha256) = &hash.sha256 {
                    where_clause.push(format!("build_hashes.sha256 = decode($1->>{}, 'hex')", data.len()));
                    data.push(serde_json::to_value(sha256).unwrap());
                }
                if let Some(sha384) = &hash.sha384 {
                    where_clause.push(format!("build_hashes.sha384 = decode($1->>{}, 'hex')", data.len()));
                    data.push(serde_json::to_value(sha384).unwrap());
                }
                if let Some(sha512) = &hash.sha512 {
                    where_clause.push(format!("build_hashes.sha512 = decode($1->>{}, 'hex')", data.len()));
                    data.push(serde_json::to_value(sha512).unwrap());
                }
                if let Some(md5) = &hash.md5 {
                    where_clause.push(format!("build_hashes.md5 = decode($1->>{}, 'hex')", data.len()));
                    data.push(serde_json::to_value(md5).unwrap());
                }
            }

            let query = sqlx::query(&format!(
                r#"
                WITH spec_build AS (
                    SELECT {}
                    FROM {}
                    WHERE {}
                    ORDER BY builds.id DESC
                    LIMIT 1
                ),
    
                filtered_builds AS (
                    SELECT {}
                    FROM builds b
                    INNER JOIN spec_build sb ON
                        sb.id = b.id
                        OR (COALESCE(sb.version_id, sb.project_version_id) = COALESCE(b.version_id, b.project_version_id) AND sb.type = b.type)
                    WHERE b.type <> 'ARCLIGHT'
                        OR split_part(sb.project_version_id, '-', -1) = split_part(b.project_version_id, '-', -1)
                        OR split_part(sb.project_version_id, '-', -1) NOT IN ('forge', 'neoforge', 'fabric')
                ),

                build_count AS (
                    SELECT count(*) AS count
                    FROM builds
                    WHERE COALESCE(version_id, project_version_id) = COALESCE(
                        (SELECT version_id FROM spec_build),
                        (SELECT project_version_id FROM spec_build)
                    )
                )
    
                SELECT
                    *,
                    0 AS build_count,
                    now()::timestamp as version2_created,
                    'RELEASE'::version_type AS version_type,
                    false AS version_supported,
                    0 AS version_java,
                    now()::timestamp AS version_created
                FROM spec_build
    
                UNION ALL
    
                SELECT
                    x.*,
                    mv.type AS version_type,
                    mv.supported AS version_supported,
                    mv.java AS version_java,
                    mv.created AS version_created
                FROM (
                    SELECT *
                    FROM (
                        SELECT
                            {},
                            (SELECT count FROM build_count) AS build_count,
                            min(b.created) OVER () AS version2_created
                        FROM filtered_builds b
                        ORDER BY b.id DESC
                    ) LIMIT 1
                ) x
                LEFT JOIN minecraft_versions mv ON mv.id = x.version_id
                "#,
                Build::columns_sql(None, None),
                if search.hash.as_ref().map(|h| h.has_any()).unwrap_or(false) {
                    "build_hashes INNER JOIN builds ON builds.id = build_hashes.build_id"
                } else {
                    "builds"
                },
                where_clause.join(" AND "),
                Build::columns_sql(None, Some("b")),
                Build::columns_sql(None, Some("b"))
            ))
            .bind(serde_json::to_value(data).unwrap())
            .fetch_all(database.read())
            .await
            .unwrap();

            if query.len() != 2 {
                return None;
            }

            let mut configs = IndexMap::new();
            for row in sqlx::query(
                r#"
                SELECT
                    configs.type AS type,
                    configs.format AS format,
                    configs.location AS location,
                    config_values.value AS value
                FROM config_values
                INNER JOIN build_configs ON build_configs.config_value_id = config_values.id
                INNER JOIN configs ON configs.id = config_values.config_id
                WHERE build_configs.build_id = $1
                ORDER BY configs.id ASC
                "#,
            )
            .bind(query[0].get::<i32, _>("id"))
            .fetch_all(database.read())
            .await
            .unwrap()
            {
                let r#type = row.get("type");
                let format = row.get("format");
                let value = row.get("value");

                configs.insert(
                    row.get("location"),
                    ConfigValue { r#type, format, value },
                );
            }

            Some(Result {
                build: Build::map(None, &query[0]),
                latest: Build::map(None, &query[1]),
                version: MinifiedVersion {
                    id: query[1]
                        .try_get("version_id")
                        .unwrap_or_else(|_| query[1].get("project_version_id")),
                    r#type: query[1].try_get("version_type").unwrap_or(VersionType::Release),
                    supported: query[1].try_get("version_supported").unwrap_or(true),
                    java: query[1].try_get("version_java").unwrap_or(21),
                    builds: query[1].try_get("build_count").unwrap_or(0),
                    created: query[1]
                        .try_get("version_created")
                        .unwrap_or(query[1].try_get("version2_created").unwrap_or_default()),
                },
                configs
            })
        })
        .await
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
