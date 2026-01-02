use compact_str::CompactStringExt;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::postgres::PgRow;
use std::collections::BTreeMap;

pub mod build;
pub mod config;
pub mod file;
pub mod organization;
pub mod r#type;
pub mod user;
pub mod version;

pub trait BaseModel: Serialize + DeserializeOwned {
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString>;

    #[inline]
    fn columns_sql(prefix: Option<&str>, table: Option<&str>) -> compact_str::CompactString {
        Self::columns(prefix, table)
            .iter()
            .map(|(key, value)| compact_str::format_compact!("{key} as {value}"))
            .collect::<Vec<compact_str::CompactString>>()
            .join_compact(", ")
    }

    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error>;
}
