use super::{BaseModel, r#type::ServerType};
use crate::prelude::IteratorExtension;
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct WebhookOrganization {
    pub id: i32,
    pub name: compact_str::CompactString,
    pub verified: bool,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct Webhook {
    pub id: i32,
    pub organization: Option<WebhookOrganization>,

    pub label: Option<compact_str::CompactString>,
    pub url: compact_str::CompactString,
    pub types: Vec<ServerType>,
    pub enabled: bool,

    pub successful: i32,
    pub failed: i32,
}

impl BaseModel for Webhook {
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString> {
        let table = table.unwrap_or("webhooks");

        BTreeMap::from([
            (
                compact_str::format_compact!("{table}.id"),
                compact_str::format_compact!("{}id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.organization_id"),
                compact_str::format_compact!("{}organization_id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.label"),
                compact_str::format_compact!("{}label", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.url"),
                compact_str::format_compact!("{}url", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.types"),
                compact_str::format_compact!("{}types", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.enabled"),
                compact_str::format_compact!("{}enabled", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.successful"),
                compact_str::format_compact!("{}successful", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.failed"),
                compact_str::format_compact!("{}failed", prefix.unwrap_or_default()),
            ),
            (
                compact_str::CompactString::const_new("organizations.name"),
                compact_str::format_compact!("{}organization_name", prefix.unwrap_or_default()),
            ),
            (
                compact_str::CompactString::const_new("organizations.verified"),
                compact_str::format_compact!("{}organization_verified", prefix.unwrap_or_default()),
            ),
        ])
    }

    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error> {
        let prefix = prefix.unwrap_or_default();

        let organization_id: Option<i32> =
            row.try_get(compact_str::format_compact!("{prefix}organization_id").as_str())?;
        let organization_name: Option<compact_str::CompactString> =
            row.try_get(compact_str::format_compact!("{prefix}organization_name").as_str())?;

        Ok(Self {
            id: row.try_get(compact_str::format_compact!("{prefix}id").as_str())?,
            organization: match (organization_id, organization_name) {
                (Some(id), Some(name)) => Some(WebhookOrganization {
                    id,
                    name,
                    verified: row.try_get(
                        compact_str::format_compact!("{prefix}organization_verified").as_str(),
                    )?,
                }),
                _ => None,
            },

            label: row.try_get(compact_str::format_compact!("{prefix}label").as_str())?,
            url: row.try_get(compact_str::format_compact!("{prefix}url").as_str())?,
            types: serde_json::from_value(
                row.try_get(compact_str::format_compact!("{prefix}types").as_str())?,
            )?,
            enabled: row.try_get(compact_str::format_compact!("{prefix}enabled").as_str())?,

            successful: row.try_get(compact_str::format_compact!("{prefix}successful").as_str())?,
            failed: row.try_get(compact_str::format_compact!("{prefix}failed").as_str())?,
        })
    }
}

impl Webhook {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(
        database: &crate::database::Database,
        organization_id: Option<i32>,
        label: Option<&str>,
        url: &str,
        types: &[ServerType],
        enabled: bool,
    ) -> Result<i32, anyhow::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO webhooks (organization_id, label, url, types, enabled)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING webhooks.id
            "#,
        )
        .bind(organization_id)
        .bind(label)
        .bind(url)
        .bind(serde_json::to_value(types)?)
        .bind(enabled)
        .fetch_one(database.write())
        .await?;

        Ok(row.try_get(0)?)
    }

    pub async fn save(
        &self,
        database: &crate::database::Database,
        reset_counters: bool,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            UPDATE webhooks
            SET
                organization_id = $2,
                label = $3,
                url = $4,
                types = $5,
                enabled = $6,
                successful = CASE WHEN $7 THEN 0 ELSE webhooks.successful END,
                failed = CASE WHEN $7 THEN 0 ELSE webhooks.failed END
            WHERE webhooks.id = $1
            "#,
        )
        .bind(self.id)
        .bind(
            self.organization
                .as_ref()
                .map(|organization| organization.id),
        )
        .bind(&self.label)
        .bind(&self.url)
        .bind(serde_json::to_value(&self.types)?)
        .bind(self.enabled)
        .bind(reset_counters)
        .execute(database.write())
        .await?;

        Ok(())
    }

    pub async fn all_with_pagination(
        database: &crate::database::Database,
        page: i64,
        per_page: i64,
        search: Option<&str>,
    ) -> Result<crate::models::Pagination<Self>, anyhow::Error> {
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            r#"
            SELECT {}, COUNT(*) OVER() AS total_count
            FROM webhooks
            LEFT JOIN organizations ON webhooks.organization_id = organizations.id
            WHERE
                $3 IS NULL
                OR webhooks.label ILIKE '%' || $3 || '%'
                OR webhooks.url ILIKE '%' || $3 || '%'
                OR organizations.name ILIKE '%' || $3 || '%'
            ORDER BY webhooks.id DESC
            LIMIT $1 OFFSET $2
            "#,
            Self::columns_sql(None, None)
        )))
        .bind(per_page)
        .bind((page - 1) * per_page)
        .bind(search)
        .fetch_all(database.read())
        .await?;

        Ok(crate::models::Pagination {
            total: rows
                .first()
                .map_or(Ok(0), |row| row.try_get("total_count"))?,
            per_page,
            page,
            data: rows
                .into_iter()
                .map(|row| Self::map(None, &row))
                .try_collect_vec()?,
        })
    }

    pub async fn by_id(
        database: &crate::database::Database,
        id: i32,
    ) -> Result<Option<Self>, anyhow::Error> {
        if id < 1 {
            return Ok(None);
        }

        let data = sqlx::query(sqlx::AssertSqlSafe(format!(
            r#"
            SELECT {}
            FROM webhooks
            LEFT JOIN organizations ON webhooks.organization_id = organizations.id
            WHERE webhooks.id = $1
            "#,
            Self::columns_sql(None, None)
        )))
        .bind(id)
        .fetch_optional(database.read())
        .await?;

        data.map(|row| Self::map(None, &row)).transpose()
    }

    pub async fn delete(
        &self,
        database: &crate::database::Database,
    ) -> Result<bool, anyhow::Error> {
        Ok(sqlx::query(
            r#"
            DELETE FROM webhooks
            WHERE webhooks.id = $1
            "#,
        )
        .bind(self.id)
        .execute(database.write())
        .await?
        .rows_affected()
            == 1)
    }
}
