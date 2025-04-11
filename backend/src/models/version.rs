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
    pub id: String,

    pub r#type: VersionType,
    pub supported: bool,
    pub java: i32,

    pub builds: i64,

    pub created: NaiveDateTime,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct MinifiedVersionStats {
    pub r#type: VersionType,
    pub supported: bool,
    pub java: i32,

    pub created: NaiveDateTime,
    pub builds: IndexMap<ServerType, i64>,
}

impl MinifiedVersionStats {
    #[inline]
    pub async fn by_id(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        id: &str,
    ) -> Option<Self> {
        cache
            .cached(&format!("version::{}::stats", id), 3600, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        builds.type AS type,
                        COUNT(builds.id) AS builds,
                        minecraft_versions.type::text AS minecraft_version_type,
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
                .await
                .unwrap();

                if data.is_empty() {
                    return None;
                }

                let mut builds = IndexMap::new();

                for r#type in ServerType::variants() {
                    builds.insert(
                        r#type,
                        data.iter()
                            .find(|x| x.get::<ServerType, _>("type") == r#type)
                            .map(|x| x.get("builds"))
                            .unwrap_or_default(),
                    );
                }

                Some(MinifiedVersionStats {
                    r#type: data[0].get("minecraft_version_type"),
                    supported: data[0].get("minecraft_version_supported"),
                    java: data[0].get("minecraft_version_java"),
                    builds,
                    created: data[0].get("minecraft_version_created"),
                })
            })
            .await
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
    #[inline]
    pub async fn location(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        r#type: ServerType,
        id: &str,
    ) -> Option<String> {
        cache
            .cached(
                &format!("version_location::{}::{}", r#type, id),
                86400,
                || async {
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

                    if project.map(|x| x.is_some()).unwrap_or_default() {
                        Some("project_version_id".to_string())
                    } else if minecraft.map(|x| x.is_some()).unwrap_or_default() {
                        Some("version_id".to_string())
                    } else {
                        None
                    }
                },
            )
            .await
    }

    #[inline]
    pub async fn all(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        r#type: ServerType,
    ) -> IndexMap<String, Self> {
        cache
            .cached(&format!("versions::{}", r#type), 1800, || async {
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
                                WHERE builds.type = $1
                                GROUP BY project_versions.id
                            ) AS x
                            INNER JOIN builds ON builds.id = x.latest
                            ORDER BY x.created_oldest ASC
                            "#,
                        Build::columns_sql(None, None)
                    ))
                    .bind(r#type)
                    .fetch_all(database.read())
                    .await
                    .unwrap();

                    for (i, row) in data.iter().enumerate() {
                        let latest = super::build::Build::map(None, row);
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
                    .await
                    .unwrap();

                    for row in data {
                        let version = Version {
                            r#type: row.get("minecraft_version_type"),
                            supported: row.get("minecraft_version_supported"),
                            java: row.get("minecraft_version_java"),
                            builds: row.get("builds"),
                            created: row.get("minecraft_version_created"),
                            latest: super::build::Build::map(None, &row),
                        };

                        versions.insert(row.get("minecraft_version_id"), version);
                    }
                }

                versions
            })
            .await
    }
}
