use serde::{Serialize, de::DeserializeOwned};
use sqlx::postgres::PgRow;
use std::collections::BTreeMap;

pub mod build;
pub mod config;
pub mod organization;
pub mod r#type;
pub mod user;
pub mod version;

pub trait BaseModel: Serialize + DeserializeOwned {
    fn columns(prefix: Option<&str>, table: Option<&str>) -> BTreeMap<String, String>;

    #[inline]
    fn columns_sql(prefix: Option<&str>, table: Option<&str>) -> String {
        Self::columns(prefix, table)
            .iter()
            .map(|(key, value)| format!("{key} as {value}"))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn map(prefix: Option<&str>, row: &PgRow) -> Self;
}
