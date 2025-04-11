use super::{BaseModel, r#type::ServerType, version::VersionType};
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow, types::chrono::NaiveDateTime};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum InstallationStep {
    #[serde(rename = "download")]
    Download(#[schema(inline)] InstallationStepDownload),
    #[serde(rename = "unzip")]
    Unzip(#[schema(inline)] InstallationStepUnzip),
    #[serde(rename = "remove")]
    Remove(#[schema(inline)] InstallationStepRemove),
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct InstallationStepDownload {
    pub url: String,
    pub file: String,
    pub size: u64,
}
#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct InstallationStepUnzip {
    pub file: String,
    pub location: String,
}
#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct InstallationStepRemove {
    pub location: String,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct Build {
    pub id: i32,

    #[serde(rename(serialize = "versionId"), alias = "versionId")]
    #[schema(rename = "versionId", example = "1.17.1")]
    pub version_id: Option<String>,
    #[serde(rename(serialize = "projectVersionId"), alias = "projectVersionId")]
    #[schema(rename = "projectVersionId")]
    pub project_version_id: Option<String>,

    pub r#type: ServerType,
    pub experimental: bool,

    pub name: String,
    #[serde(rename(serialize = "buildNumber"), alias = "buildNumber")]
    #[schema(rename = "buildNumber")]
    pub build_number: i32,
    #[serde(rename(serialize = "jarUrl"), alias = "jarUrl")]
    #[schema(rename = "jarUrl")]
    pub jar_url: Option<String>,
    #[serde(rename(serialize = "jarSize"), alias = "jarSize")]
    #[schema(rename = "jarSize")]
    pub jar_size: Option<i32>,
    #[serde(rename(serialize = "zipUrl"), alias = "zipUrl")]
    #[schema(rename = "zipUrl")]
    pub zip_url: Option<String>,
    #[serde(rename(serialize = "zipSize"), alias = "zipSize")]
    #[schema(rename = "zipSize")]
    pub zip_size: Option<i32>,

    pub installation: Vec<Vec<InstallationStep>>,
    pub changes: Vec<String>,

    pub created: Option<NaiveDateTime>,
}

impl BaseModel for Build {
    #[inline]
    fn columns(prefix: Option<&str>, table: Option<&str>) -> BTreeMap<String, String> {
        let table = table.unwrap_or("builds");

        BTreeMap::from([
            (
                format!("{}.id", table),
                format!("{}id", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.version_id", table),
                format!("{}version_id", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.project_version_id", table),
                format!("{}project_version_id", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.type", table),
                format!("{}type", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.experimental", table),
                format!("{}experimental", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.build_number", table),
                format!("{}build_number", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.jar_url", table),
                format!("{}jar_url", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.jar_size", table),
                format!("{}jar_size", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.zip_url", table),
                format!("{}zip_url", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.zip_size", table),
                format!("{}zip_size", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.installation", table),
                format!("{}installation", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.changes", table),
                format!("{}changes", prefix.unwrap_or_default()),
            ),
            (
                format!("{}.created", table),
                format!("{}created", prefix.unwrap_or_default()),
            ),
        ])
    }

    #[inline]
    fn map(prefix: Option<&str>, row: &PgRow) -> Self {
        let prefix = prefix.unwrap_or_default();

        Self {
            id: row.get(format!("{}id", prefix).as_str()),
            version_id: row.try_get(format!("{}version_id", prefix).as_str()).ok(),
            project_version_id: row
                .try_get(format!("{}project_version_id", prefix).as_str())
                .ok(),
            r#type: row.get(format!("{}type", prefix).as_str()),
            experimental: row.get(format!("{}experimental", prefix).as_str()),
            name: if row.get::<i32, _>(format!("{}build_number", prefix).as_str()) == 1
                && row
                    .try_get::<String, _>(format!("{}project_version_id", prefix).as_str())
                    .is_ok()
            {
                row.get(format!("{}project_version_id", prefix).as_str())
            } else {
                format!(
                    "#{}",
                    row.get::<i32, _>(format!("{}build_number", prefix).as_str())
                )
            },
            build_number: row.get(format!("{}build_number", prefix).as_str()),
            jar_url: row.get(format!("{}jar_url", prefix).as_str()),
            jar_size: row.get(format!("{}jar_size", prefix).as_str()),
            zip_url: row.get(format!("{}zip_url", prefix).as_str()),
            zip_size: row.get(format!("{}zip_size", prefix).as_str()),
            installation: serde_json::from_value(
                row.get(format!("{}installation", prefix).as_str()),
            )
            .unwrap(),
            changes: serde_json::from_value(row.get(format!("{}changes", prefix).as_str()))
                .unwrap(),
            created: row.get(format!("{}created", prefix).as_str()),
        }
    }
}

impl Build {
    #[inline]
    pub fn installation_size(&self) -> u64 {
        self.installation
            .iter()
            .flat_map(|step| step.iter())
            .filter_map(|step| match step {
                InstallationStep::Download(step) => Some(step.size),
                _ => None,
            })
            .sum()
    }

    #[inline]
    pub async fn by_v1_identifier(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        identifier: &str,
    ) -> Option<(Self, Self, super::version::MinifiedVersion)> {
        cache.cached(&format!("build::{}", identifier), 3600, || async {
        let hash: Option<&str> = match identifier.len() {
            32 => Some("md5"),
            40 => Some("sha1"),
            56 => Some("sha224"),
            64 => Some("sha256"),
            96 => Some("sha384"),
            128 => Some("sha512"),
            _ => {
                if let Ok(id) = identifier.parse::<i32>() {
                    if id < 1 {
                        return None;
                    } else {
                        None
                    }
                } else {
                    return None;
                }
            }
        };

        let query = sqlx::query(&format!(
            r#"
            WITH spec_build AS (
                SELECT {}
                FROM {}
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
                'RELEASE' AS version_type,
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
            Self::columns_sql(None, None),
            if let Some(hash) = hash {
                format!("build_hashes INNER JOIN builds ON builds.id = build_hashes.build_id WHERE {} = decode($1, 'hex')", hash)
            } else {
                "builds WHERE builds.id = $1::int".to_string()
            },
            Self::columns_sql(None, Some("b")),
            Self::columns_sql(None, Some("b"))
        ))
        .bind(identifier)
        .fetch_all(database.read())
        .await
        .unwrap();

        if query.len() != 2 {
            return None;
        }

        Some((
            Self::map(None, &query[0]),
            Self::map(None, &query[1]),
            super::version::MinifiedVersion {
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
        ))})
        .await
    }

    #[inline]
    pub async fn by_build_number(
        database: &crate::database::Database,
        r#type: ServerType,
        version_location: &str,
        version_id: &str,
        build_number: Option<i32>,
    ) -> Option<Self> {
        let data = sqlx::query(&format!(
            r#"
            SELECT {}
            FROM builds
            WHERE
                {} = $1
                AND type = $2
                {}
            ORDER BY id DESC
            "#,
            Self::columns_sql(None, None),
            version_location,
            if build_number.is_some() {
                "AND build_number = $3"
            } else {
                ""
            }
        ))
        .bind(version_id)
        .bind(r#type)
        .bind(build_number)
        .fetch_one(database.read())
        .await;

        match data {
            Ok(data) => Some(Self::map(None, &data)),
            Err(_) => None,
        }
    }

    #[inline]
    pub async fn all_for_version(
        database: &crate::database::Database,
        r#type: ServerType,
        version_location: &str,
        version_id: &str,
    ) -> Vec<Self> {
        sqlx::query(&format!(
            r#"
            SELECT {}
            FROM builds
            WHERE
                {} = $1
                AND type = $2
            ORDER BY id DESC
            "#,
            Self::columns_sql(None, None),
            version_location
        ))
        .bind(version_id)
        .bind(r#type)
        .fetch_all(database.read())
        .await
        .unwrap()
        .into_iter()
        .map(|row| Self::map(None, &row))
        .collect()
    }

    #[inline]
    pub async fn all_for_minecraft_version(
        database: &crate::database::Database,
        version_id: &str,
    ) -> Vec<Self> {
        sqlx::query(&format!(
            r#"
            SELECT {}
            FROM builds
            WHERE version_id = $1
            ORDER BY id DESC
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(version_id)
        .fetch_all(database.read())
        .await
        .unwrap()
        .into_iter()
        .map(|row| Self::map(None, &row))
        .collect()
    }
}
