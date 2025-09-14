use super::BaseModel;
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow, types::chrono::NaiveDateTime};
use std::{collections::BTreeMap, path::Path};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct File {
    pub name: String,

    pub is_directory: bool,
    pub size: i64,

    pub md5: [u8; 16],
    pub sha1: [u8; 20],
    pub sha224: [u8; 28],
    pub sha256: [u8; 32],
    #[serde(with = "serde_arrays")]
    pub sha384: [u8; 48],
    #[serde(with = "serde_arrays")]
    pub sha512: [u8; 64],

    pub last_access: Option<NaiveDateTime>,
}

impl BaseModel for File {
    #[inline]
    fn columns(prefix: Option<&str>, table: Option<&str>) -> BTreeMap<String, String> {
        let table = table.unwrap_or("files");

        BTreeMap::from([
            (
                format!("{table}.path[array_upper({table}.path, 1)]"),
                format!("{}current_entry", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.size::int8"),
                format!("{}total_size", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.md5"),
                format!("{}md5", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.sha1"),
                format!("{}sha1", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.sha224"),
                format!("{}sha224", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.sha256"),
                format!("{}sha256", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.sha384"),
                format!("{}sha384", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.sha512"),
                format!("{}sha512", prefix.unwrap_or_default()),
            ),
            (
                format!("{table}.last_access"),
                format!("{}last_access", prefix.unwrap_or_default()),
            ),
        ])
    }

    #[inline]
    fn map(prefix: Option<&str>, row: &PgRow) -> Self {
        let prefix = prefix.unwrap_or_default();

        Self {
            name: row.get(format!("{prefix}current_entry").as_str()),
            is_directory: row
                .try_get(format!("{prefix}is_directory").as_str())
                .unwrap_or_default(),
            size: row.get(format!("{prefix}total_size").as_str()),
            md5: row.get(format!("{prefix}md5").as_str()),
            sha1: row.get(format!("{prefix}sha1").as_str()),
            sha224: row.get(format!("{prefix}sha224").as_str()),
            sha256: row.get(format!("{prefix}sha256").as_str()),
            sha384: row.get(format!("{prefix}sha384").as_str()),
            sha512: row.get(format!("{prefix}sha512").as_str()),
            last_access: row.get(format!("{prefix}last_access").as_str()),
        }
    }
}

impl File {
    #[inline]
    pub async fn by_path(database: &crate::database::Database, path: &Path) -> Option<Self> {
        sqlx::query(&format!(
            r#"
            SELECT {}
            FROM files
            WHERE files.path = $1::varchar[]
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(
            path.components()
                .filter(|c| c.as_os_str().to_str().is_some_and(|s| !s.is_empty()))
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>(),
        )
        .fetch_optional(database.read())
        .await
        .unwrap()
        .map(|row| Self::map(None, &row))
    }

    #[inline]
    pub async fn all_for_root(database: &crate::database::Database, root: &Path) -> Vec<Self> {
        sqlx::query(
            r#"
            WITH prefix AS (
                SELECT $1::varchar[] AS arr
            ),
            prefix_length AS (
                SELECT
                    COALESCE(array_length(arr, 1), 0) AS len,
                    arr
                FROM prefix
            ),
            path_info AS (
                SELECT
                    path[prefix_length.len + 1] AS current_entry,
                    array_length(path, 1) AS path_len,
                    files.*
                FROM files, prefix_length
                WHERE
                    array_length(path, 1) >= prefix_length.len + 1
                    AND (
                        prefix_length.len = 0
                        OR
                        path[1:prefix_length.len] = prefix_length.arr
                    )
            ),
            directory_check AS (
                SELECT
                    current_entry,
                    MAX(CASE WHEN path_len > (SELECT len FROM prefix_length) + 1 THEN 1 ELSE 0 END)::boolean AS is_directory,
                    SUM(size) AS total_size
                FROM path_info
                WHERE current_entry IS NOT NULL
                GROUP BY current_entry
            )
            SELECT DISTINCT ON (pi.current_entry)
                pi.current_entry,
                pi.path,
                CASE
                    WHEN dc.is_directory THEN dc.total_size
                    ELSE pi.size
                END AS total_size,
                pi.*,
                dc.is_directory
            FROM path_info pi
            JOIN directory_check dc ON pi.current_entry = dc.current_entry
            WHERE pi.current_entry IS NOT NULL
            ORDER BY pi.current_entry, pi.path_len
            "#
        )
        .bind(
            root
                .components()
                .filter(|c| c.as_os_str().to_str().is_some_and(|s| !s.is_empty()))
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect::<Vec<String>>()
        )
        .fetch_all(database.read())
        .await
        .unwrap()
        .into_iter()
        .map(|row| Self::map(None, &row))
        .collect()
    }
}
