use super::{
    BaseModel,
    build::Build,
    r#type::{SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER, ServerType},
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::{Row, prelude::Type, types::chrono::NaiveDateTime};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Type)]
#[serde(rename_all = "UPPERCASE")]
#[schema(rename_all = "UPPERCASE")]
#[sqlx(type_name = "version_type", rename_all = "UPPERCASE")]
pub enum VersionType {
    Release,
    Snapshot,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct MinifiedVersion {
    pub id: compact_str::CompactString,

    pub r#type: VersionType,
    pub supported: bool,
    pub java: i32,

    pub builds: i64,

    pub created: NaiveDateTime,
}

impl MinifiedVersion {
    pub fn into_api_v3(self) -> ApiMinifiedVersionV3 {
        ApiMinifiedVersionV3 {
            id: self.id,
            r#type: self.r#type,
            supported: self.supported,
            java: self.java as u16,
            builds: self.builds as u64,
            created: self.created.and_utc(),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct MinifiedVersionStats {
    pub r#type: VersionType,
    pub supported: bool,
    pub java: i16,

    pub created: NaiveDateTime,
    pub builds: IndexMap<ServerType, i64>,
}

impl MinifiedVersionStats {
    #[inline]
    pub async fn by_id(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        env: &crate::env::Env,
        id: &str,
    ) -> Result<Option<Self>, anyhow::Error> {
        cache
            .cached(&format!("version::{id}::stats"), 3600, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        builds.type AS type,
                        COUNT(builds.id) AS builds,
                        minecraft_versions.type AS minecraft_version_type,
                        minecraft_versions.created AS minecraft_version_created,
                        minecraft_versions.supported AS minecraft_version_supported,
                        minecraft_versions.java AS minecraft_version_java
                    FROM builds
                    LEFT JOIN minecraft_versions ON minecraft_versions.id = builds.version_id
                    WHERE builds.version_id = $1
                    GROUP BY
                        builds.type,
                        minecraft_versions.type,
                        minecraft_versions.created,
                        minecraft_versions.supported,
                        minecraft_versions.java
                    "#,
                )
                .bind(id)
                .fetch_all(database.read())
                .await?;

                if data.is_empty() {
                    return Ok(None);
                }

                let mut builds = IndexMap::new();
                for r#type in ServerType::variants(env) {
                    builds.insert(
                        r#type,
                        data.iter()
                            .find(|x| x.get::<ServerType, _>("type") == r#type)
                            .map(|x| x.get("builds"))
                            .unwrap_or_default(),
                    );
                }

                Ok::<_, anyhow::Error>(Some(MinifiedVersionStats {
                    r#type: data[0].try_get("minecraft_version_type")?,
                    supported: data[0].try_get("minecraft_version_supported")?,
                    java: data[0].try_get("minecraft_version_java")?,
                    builds,
                    created: data[0].try_get("minecraft_version_created")?,
                }))
            })
            .await
    }

    pub fn into_api_v3(self) -> ApiMinifiedVersionStatsV3 {
        ApiMinifiedVersionStatsV3 {
            r#type: self.r#type,
            supported: self.supported,
            java: self.java as u16,
            created: self.created.and_utc(),
            builds: self
                .builds
                .into_iter()
                .map(|(k, v)| (k, v as u64))
                .collect(),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Version {
    pub r#type: VersionType,
    pub supported: bool,
    pub java: i16,

    pub builds: i64,

    pub created: NaiveDateTime,
    pub latest: super::build::Build,
}

impl Version {
    /// Resolves a type + version to the field name and value that can be used to query builds for that version.
    ///
    /// e.g. ``resolve("VANILLA", "1.17.1") -> Some(("version_id", "1.17.1"))``
    ///
    /// e.g. ``resolve("PAPER", "latest") -> Some(("version_id", "26.1.1"))``
    pub async fn resolve(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        r#type: ServerType,
        id: &str,
    ) -> Result<Option<(compact_str::CompactString, compact_str::CompactString)>, anyhow::Error>
    {
        cache
            .cached(
                &format!("version_location::{type}::{id}"),
                86400,
                || async {
                    if id == "latest" || id == "latest-snapshot" {
                        if SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER.contains(&r#type) {
                            let version_id = sqlx::query_scalar(
                                r#"
                                SELECT project_versions.id
                                FROM project_versions
                                INNER JOIN builds ON builds.project_version_id = project_versions.id
                                WHERE builds.type = $1 AND project_versions.type = $1
                                ORDER BY builds.created DESC
                                LIMIT 1
                                "#,
                            )
                            .bind(r#type)
                            .fetch_optional(database.read())
                            .await?;

                            let Some(version_id) = version_id else {
                                return Ok(None);
                            };

                            return Ok(Some(("project_version_id".into(), version_id)));
                        } else {
                            let version_id = sqlx::query_scalar(
                                r#"
                                SELECT minecraft_versions.id
                                FROM minecraft_versions
                                INNER JOIN builds ON builds.version_id = minecraft_versions.id
                                WHERE builds.type = $1 AND minecraft_versions.type = ANY($2)
                                ORDER BY minecraft_versions.created DESC
                                LIMIT 1
                                "#,
                            )
                            .bind(r#type)
                            .bind(if id == "latest-snapshot" {
                                &[VersionType::Snapshot, VersionType::Release][..]
                            } else {
                                &[VersionType::Release][..]
                            })
                            .fetch_optional(database.read())
                            .await?;

                            let Some(version_id) = version_id else {
                                return Ok(None);
                            };

                            return Ok(Some(("version_id".into(), version_id)));
                        }
                    }

                    let (minecraft, project) = tokio::join!(
                        sqlx::query(
                            r#"
                            SELECT 1
                            FROM minecraft_versions
                            WHERE id = $1
                            LIMIT 1
                            "#
                        )
                        .bind(id)
                        .fetch_optional(database.read()),
                        sqlx::query(
                            r#"
                            SELECT 1
                            FROM project_versions
                            WHERE
                                id = $1
                                AND type = $2
                            LIMIT 1
                            "#
                        )
                        .bind(id)
                        .bind(r#type)
                        .fetch_optional(database.read()),
                    );

                    Ok::<_, anyhow::Error>(if project.map(|x| x.is_some()).unwrap_or_default() {
                        Some(("project_version_id".into(), id.into()))
                    } else if minecraft.map(|x| x.is_some()).unwrap_or_default() {
                        Some(("version_id".into(), id.into()))
                    } else {
                        None
                    })
                },
            )
            .await
    }

    #[inline]
    pub async fn all(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        r#type: ServerType,
    ) -> Result<IndexMap<compact_str::CompactString, Self>, anyhow::Error> {
        cache
            .cached(&format!("versions::{type}"), 1800, || async {
                let mut versions = IndexMap::new();

                if SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER.contains(&r#type) {
                    let data = sqlx::query(&format!(
                        r#"
                        SELECT
                            {},
                            builds,
                            latest,
                            created_oldest
                        FROM (
                            SELECT
                                COUNT(builds.id) AS builds,
                                MAX(builds.id) AS latest,
                                MIN(builds.created) AS created_oldest,
                                project_versions.id AS project_version_id
                            FROM project_versions
                            INNER JOIN builds ON builds.project_version_id = project_versions.id
                            WHERE builds.type = $1 AND project_versions.type = $1
                            GROUP BY project_versions.id
                        ) AS x
                        INNER JOIN builds ON builds.id = x.latest
                        ORDER BY x.created_oldest ASC
                        "#,
                        Build::columns_sql(None, None)
                    ))
                    .bind(r#type)
                    .fetch_all(database.read())
                    .await?;

                    for (i, row) in data.iter().enumerate() {
                        let latest = super::build::Build::map(None, row)?;
                        let id = latest.project_version_id.clone().unwrap();

                        let version = Version {
                            r#type: VersionType::Release,
                            supported: i == data.len() - 1,
                            java: 21,
                            builds: row.get("builds"),
                            created: row.get("created_oldest"),
                            latest,
                        };

                        versions.insert(id, version);
                    }
                } else {
                    let data = sqlx::query(&format!(
                        r#"
                        SELECT
                            {},
                            builds,
                            latest,
                            minecraft_version_type,
                            minecraft_version_created,
                            minecraft_version_supported,
                            minecraft_version_java,
                            minecraft_version_id
                        FROM (
                            SELECT
                                COUNT(builds.id) AS builds,
                                MAX(builds.id) AS latest,
                                minecraft_versions.type AS minecraft_version_type,
                                minecraft_versions.created AS minecraft_version_created,
                                minecraft_versions.supported AS minecraft_version_supported,
                                minecraft_versions.java AS minecraft_version_java,
                                minecraft_versions.id AS minecraft_version_id
                            FROM minecraft_versions
                            INNER JOIN builds ON builds.version_id = minecraft_versions.id
                            WHERE builds.type = $1
                            GROUP BY minecraft_versions.id
                        ) AS x
                        INNER JOIN builds ON builds.id = x.latest
                        ORDER BY x.minecraft_version_created ASC
                        "#,
                        Build::columns_sql(None, None),
                    ))
                    .bind(r#type)
                    .fetch_all(database.read())
                    .await?;

                    for row in data {
                        let version = Version {
                            r#type: row.try_get("minecraft_version_type")?,
                            supported: row.try_get("minecraft_version_supported")?,
                            java: row.try_get("minecraft_version_java")?,
                            builds: row.try_get("builds")?,
                            created: row.try_get("minecraft_version_created")?,
                            latest: super::build::Build::map(None, &row)?,
                        };

                        versions.insert(row.try_get("minecraft_version_id")?, version);
                    }
                }

                Ok::<_, anyhow::Error>(versions)
            })
            .await
    }

    pub fn into_api_version_v3(self, id: compact_str::CompactString) -> ApiVersionV3 {
        ApiVersionV3 {
            id,
            r#type: self.r#type,
            supported: self.supported,
            java: self.java as u16,
            builds: self.builds as u64,
            created: self.created.and_utc(),
            latest: self.latest.into_api_v3(),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(title = "MinifiedVersionV3")]
pub struct ApiMinifiedVersionV3 {
    pub id: compact_str::CompactString,

    pub r#type: VersionType,
    pub supported: bool,
    pub java: u16,

    pub builds: u64,

    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(title = "MinifiedVersionStatsV3")]
pub struct ApiMinifiedVersionStatsV3 {
    pub r#type: VersionType,
    pub supported: bool,
    pub java: u16,

    pub created: chrono::DateTime<chrono::Utc>,
    pub builds: IndexMap<ServerType, u64>,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(title = "VersionV3")]
pub struct ApiVersionV3 {
    pub id: compact_str::CompactString,

    pub r#type: VersionType,
    pub supported: bool,
    pub java: u16,

    pub builds: u64,

    pub created: chrono::DateTime<chrono::Utc>,
    pub latest: super::build::ApiBuildV3,
}
