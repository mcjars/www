use super::BaseModel;
use crate::prelude::IteratorExtension;
use compact_str::ToCompactString;
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow, types::chrono::NaiveDateTime};
use std::{collections::BTreeMap, path::Path};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct File {
    pub name: compact_str::CompactString,

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
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString> {
        let table = table.unwrap_or("files");

        BTreeMap::from([
            (
                compact_str::format_compact!("{table}.path[array_upper({table}.path, 1)]"),
                compact_str::format_compact!("{}current_entry", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.size::int8"),
                compact_str::format_compact!("{}total_size", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.md5"),
                compact_str::format_compact!("{}md5", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.sha1"),
                compact_str::format_compact!("{}sha1", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.sha224"),
                compact_str::format_compact!("{}sha224", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.sha256"),
                compact_str::format_compact!("{}sha256", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.sha384"),
                compact_str::format_compact!("{}sha384", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.sha512"),
                compact_str::format_compact!("{}sha512", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.last_access"),
                compact_str::format_compact!("{}last_access", prefix.unwrap_or_default()),
            ),
        ])
    }

    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error> {
        let prefix = prefix.unwrap_or_default();

        Ok(Self {
            name: row.try_get(compact_str::format_compact!("{prefix}current_entry").as_str())?,
            is_directory: row
                .try_get(compact_str::format_compact!("{prefix}is_directory").as_str())
                .unwrap_or_default(),
            size: row.try_get(compact_str::format_compact!("{prefix}total_size").as_str())?,
            md5: row.try_get(compact_str::format_compact!("{prefix}md5").as_str())?,
            sha1: row.try_get(compact_str::format_compact!("{prefix}sha1").as_str())?,
            sha224: row.try_get(compact_str::format_compact!("{prefix}sha224").as_str())?,
            sha256: row.try_get(compact_str::format_compact!("{prefix}sha256").as_str())?,
            sha384: row.try_get(compact_str::format_compact!("{prefix}sha384").as_str())?,
            sha512: row.try_get(compact_str::format_compact!("{prefix}sha512").as_str())?,
            last_access: row
                .try_get(compact_str::format_compact!("{prefix}last_access").as_str())?,
        })
    }
}

impl File {
    pub async fn by_path(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        path: &Path,
    ) -> Result<Option<Self>, anyhow::Error> {
        cache
            .cached(&format!("file::{}", path.display()), 3600, || async {
                let data = sqlx::query(sqlx::AssertSqlSafe(format!(
                    r#"
                    SELECT {}
                    FROM files
                    WHERE files.path = $1::varchar[]
                    "#,
                    Self::columns_sql(None, None)
                )))
                .bind(
                    path.components()
                        .filter(|c| c.as_os_str().to_str().is_some_and(|s| !s.is_empty()))
                        .map(|c| c.as_os_str().to_string_lossy().to_compact_string())
                        .collect::<Vec<_>>(),
                )
                .fetch_optional(database.read())
                .await?;

                data.map(|row| Self::map(None, &row)).transpose()
            })
            .await
    }

    pub async fn all_for_root(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        root: &Path,
    ) -> Result<Vec<Self>, anyhow::Error> {
        cache
            .cached(&format!("files::{}", root.display()), 3600, || async {
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
                    per_depth AS (
                        SELECT
                            path[pl.len + 1] AS current_entry,
                            array_length(path, 1) AS path_len,
                            MIN(path) AS min_path,
                            SUM(size) AS size_sum
                        FROM files, prefix_length pl
                        WHERE
                            array_length(path, 1) >= pl.len + 1
                            AND (
                                pl.len = 0
                                OR
                                path[1:pl.len] = pl.arr
                            )
                            AND path[pl.len + 1] IS NOT NULL
                        GROUP BY path[pl.len + 1], array_length(path, 1)
                    ),
                    agg AS (
                        SELECT DISTINCT ON (current_entry)
                            current_entry,
                            bool_or(path_len > pl.len + 1) OVER (PARTITION BY current_entry) AS is_directory,
                            (SUM(size_sum) OVER (PARTITION BY current_entry))::int8 AS total_size,
                            min_path AS rep_path
                        FROM per_depth, prefix_length pl
                        ORDER BY current_entry, path_len
                    )
                    SELECT
                        a.current_entry,
                        f.path,
                        CASE
                            WHEN a.is_directory THEN a.total_size
                            ELSE f.size
                        END AS total_size,
                        f.*,
                        a.is_directory
                    FROM agg a
                    JOIN files f ON f.path = a.rep_path
                    ORDER BY a.current_entry
                    "#
                )
                .bind(
                    root
                        .components()
                        .filter(|c| c.as_os_str().to_str().is_some_and(|s| !s.is_empty()))
                        .map(|c| c.as_os_str().to_string_lossy().to_compact_string())
                        .collect::<Vec<_>>()
                )
                .fetch_all(database.read())
                .await?
                .into_iter()
                .map(|row| Self::map(None, &row))
                .try_collect_vec()
            })
            .await
    }
}
