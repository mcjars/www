use super::{BaseModel, r#type::ServerType};
use crate::prelude::IteratorExtension;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{Row, postgres::PgRow, types::chrono::NaiveDateTime};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct Organization {
    pub id: i32,
    pub owner: super::user::ApiUser,

    pub verified: bool,
    pub public: bool,

    pub name: compact_str::CompactString,
    pub icon: compact_str::CompactString,
    pub types: Vec<ServerType>,

    #[serde(skip)]
    pub subuser_pending: bool,
    pub created: NaiveDateTime,
}

impl BaseModel for Organization {
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString> {
        let table = table.unwrap_or("organizations");

        let mut columns = BTreeMap::from([
            (
                compact_str::format_compact!("{table}.id"),
                compact_str::format_compact!("{}id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.owner_id"),
                compact_str::format_compact!("{}owner_id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.verified"),
                compact_str::format_compact!("{}verified", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.public"),
                compact_str::format_compact!("{}public", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.name"),
                compact_str::format_compact!("{}name", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.icon"),
                compact_str::format_compact!("{}icon", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.types"),
                compact_str::format_compact!("{}types", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.created"),
                compact_str::format_compact!("{}created", prefix.unwrap_or_default()),
            ),
        ]);

        columns.extend(super::user::User::columns(Some("owner_"), None));

        columns
    }

    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error> {
        let prefix = prefix.unwrap_or_default();

        Ok(Self {
            id: row.try_get(compact_str::format_compact!("{prefix}id").as_str())?,
            owner: super::user::User::map(Some("owner_"), row)?.api_user(false),

            verified: row.try_get(compact_str::format_compact!("{prefix}verified").as_str())?,
            public: row.try_get(compact_str::format_compact!("{prefix}public").as_str())?,
            name: row.try_get(compact_str::format_compact!("{prefix}name").as_str())?,
            icon: row.try_get(compact_str::format_compact!("{prefix}icon").as_str())?,
            types: serde_json::from_value(
                row.try_get(compact_str::format_compact!("{prefix}types").as_str())?,
            )
            .unwrap(),

            subuser_pending: row.try_get("pending").unwrap_or(false),
            created: row.try_get(compact_str::format_compact!("{prefix}created").as_str())?,
        })
    }
}

impl Organization {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(
        database: &crate::database::Database,
        owner_id: i32,
        name: &str,
    ) -> Result<(), anyhow::Error> {
        sqlx::query("INSERT INTO organizations (owner_id, name) VALUES ($1, $2)")
            .bind(owner_id)
            .bind(name)
            .execute(database.write())
            .await?;

        Ok(())
    }

    pub async fn save(&self, database: &crate::database::Database) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            UPDATE organizations
            SET
                owner_id = $2,
                verified = $3,
                public = $4,
                name = $5,
                icon = $6,
                types = $7
            WHERE organizations.id = $1
            "#,
        )
        .bind(self.id)
        .bind(self.owner.id)
        .bind(self.verified)
        .bind(self.public)
        .bind(&self.name)
        .bind(&self.icon)
        .bind(serde_json::to_value(&self.types).unwrap())
        .execute(database.write())
        .await?;

        Ok(())
    }

    pub async fn count_by_owner(database: &crate::database::Database, user_id: i32) -> i64 {
        sqlx::query(
            r#"
            SELECT COUNT(*)
            FROM organizations
            WHERE organizations.owner_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(database.read())
        .await
        .map_or(0, |row| row.try_get(0).unwrap_or(0))
    }

    pub async fn all_by_owner(
        database: &crate::database::Database,
        user_id: i32,
    ) -> Result<Vec<Self>, anyhow::Error> {
        sqlx::query(&format!(
            r#"
            SELECT DISTINCT {}, organization_subusers.pending
            FROM organizations
            LEFT JOIN users ON organizations.owner_id = users.id
            LEFT JOIN organization_subusers ON organizations.id = organization_subusers.organization_id
            WHERE
                organizations.owner_id = $1
                OR organization_subusers.user_id = $1
            ORDER BY organizations.id DESC
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(user_id)
        .fetch_all(database.read())
        .await?
        .into_iter()
        .map(|row| Self::map(None, &row))
        .try_collect_vec()
    }

    pub async fn by_id(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        id: i32,
    ) -> Result<Option<Self>, anyhow::Error> {
        if id < 1 {
            return Ok(None);
        }

        cache
            .cached(&format!("organization::{id}"), 300, || async {
                let data = sqlx::query(&format!(
                    r#"
                    SELECT {}
                    FROM organizations
                    LEFT JOIN users ON organizations.owner_id = users.id
                    WHERE organizations.id = $1
                    "#,
                    Self::columns_sql(None, None)
                ))
                .bind(id)
                .fetch_optional(database.read())
                .await?;

                data.map(|row| Self::map(None, &row)).transpose()
            })
            .await
    }

    pub async fn by_key(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        key: &str,
    ) -> Result<Option<Self>, anyhow::Error> {
        cache
            .cached(&format!("organization::key::{key}"), 300, || async {
                let data = sqlx::query(&format!(
                    r#"
                    SELECT {}
                    FROM organizations
                    LEFT JOIN users ON organizations.owner_id = users.id
                    LEFT JOIN organization_keys ON organizations.id = organization_keys.organization_id
                    WHERE organization_keys.key = $1
                    "#,
                    Self::columns_sql(None, None)
                ))
                .bind(key)
                .fetch_optional(database.read())
                .await?;

                data.map(|row| Self::map(None, &row)).transpose()
            })
            .await
    }

    pub async fn by_id_and_user(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        user_id: i32,
        user_admin: bool,
        organization_id: i32,
    ) -> Result<Option<Self>, anyhow::Error> {
        cache
            .cached(&format!("organization::{organization_id}::user::{user_id}"), 60, || async {
                let data = sqlx::query(&format!(
                    r#"
                    SELECT {}
                    FROM organizations
                    LEFT JOIN users ON organizations.owner_id = users.id
                    LEFT JOIN organization_subusers ON organizations.id = organization_subusers.organization_id
                    WHERE
                        (
                            organizations.owner_id = $1
                            OR organization_subusers.user_id = $1
                            OR $2
                        )
                        AND organizations.id = $3
                    LIMIT 1
                    "#,
                    Self::columns_sql(None, None)
                ))
                .bind(user_id)
                .bind(user_admin)
                .bind(organization_id)
                .fetch_optional(database.read())
                .await?;

                data.map(|row| Self::map(None, &row)).transpose()
            })
            .await
    }

    pub async fn delete_by_id(
        database: &crate::database::Database,
        id: i32,
    ) -> Result<bool, anyhow::Error> {
        Ok(sqlx::query(
            r#"
            DELETE FROM organizations
            WHERE organizations.id = $1
            "#,
        )
        .bind(id)
        .execute(database.write())
        .await?
        .rows_affected()
            == 1)
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct OrganizationKey {
    pub id: i32,
    #[serde(skip)]
    #[schema(ignore)]
    pub organization_id: i32,

    pub name: compact_str::CompactString,

    pub created: NaiveDateTime,
}

impl BaseModel for OrganizationKey {
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString> {
        let table = table.unwrap_or("organization_keys");

        BTreeMap::from([
            (
                compact_str::format_compact!("{table}.id"),
                compact_str::format_compact!("{}id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.name"),
                compact_str::format_compact!("{}name", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.organization_id"),
                compact_str::format_compact!("{}organization_id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.created"),
                compact_str::format_compact!("{}created", prefix.unwrap_or_default()),
            ),
        ])
    }

    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error> {
        let prefix = prefix.unwrap_or_default();

        Ok(Self {
            id: row.try_get(compact_str::format_compact!("{prefix}id").as_str())?,
            organization_id: row
                .try_get(compact_str::format_compact!("{prefix}organization_id").as_str())?,

            name: row.try_get(compact_str::format_compact!("{prefix}name").as_str())?,

            created: row.try_get(compact_str::format_compact!("{prefix}created").as_str())?,
        })
    }
}

impl OrganizationKey {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(
        database: &crate::database::Database,
        organization_id: i32,
        name: &str,
    ) -> Result<(bool, String), anyhow::Error> {
        let mut hash = sha2::Sha256::new();
        hash.update(chrono::Utc::now().timestamp().to_be_bytes());
        hash.update(organization_id.to_be_bytes());
        let hash = format!("{:x}", hash.finalize());

        Ok((
            sqlx::query(
                r#"
                INSERT INTO organization_keys (organization_id, name, key)
                VALUES ($1, $2, $3)
                ON CONFLICT (organization_id, name) DO NOTHING
                "#,
            )
            .bind(organization_id)
            .bind(name)
            .bind(&hash)
            .execute(database.write())
            .await?
            .rows_affected()
                == 1,
            hash,
        ))
    }

    pub async fn count_by_organization(
        database: &crate::database::Database,
        organization_id: i32,
    ) -> i64 {
        sqlx::query(
            r#"
            SELECT COUNT(*)
            FROM organization_keys
            WHERE organization_keys.organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_one(database.read())
        .await
        .map_or(0, |row| row.try_get(0).unwrap_or(0))
    }

    pub async fn all_by_organization(
        database: &crate::database::Database,
        organization_id: i32,
    ) -> Result<Vec<Self>, anyhow::Error> {
        sqlx::query(&format!(
            r#"
            SELECT {} FROM organization_keys
            WHERE organization_keys.organization_id = $1
            ORDER BY organization_keys.id DESC
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(organization_id)
        .fetch_all(database.read())
        .await?
        .into_iter()
        .map(|row| Self::map(None, &row))
        .try_collect_vec()
    }

    pub async fn by_id(
        database: &crate::database::Database,
        id: i32,
    ) -> Result<Option<Self>, anyhow::Error> {
        if id < 1 {
            return Ok(None);
        }

        let data = sqlx::query(&format!(
            "SELECT {} FROM organization_keys WHERE organization_keys.id = $1",
            Self::columns_sql(None, None)
        ))
        .bind(id)
        .fetch_optional(database.read())
        .await?;

        data.map(|row| Self::map(None, &row)).transpose()
    }

    pub async fn delete_by_id(
        database: &crate::database::Database,
        id: i32,
    ) -> Result<bool, anyhow::Error> {
        Ok(sqlx::query(
            r#"
            DELETE FROM organization_keys
            WHERE organization_keys.id = $1
            "#,
        )
        .bind(id)
        .execute(database.write())
        .await?
        .rows_affected()
            == 1)
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct OrganizationSubuser {
    #[serde(skip)]
    #[schema(ignore)]
    pub organization_id: i32,

    pub user: super::user::ApiUser,
    pub pending: bool,

    pub created: NaiveDateTime,
}

impl BaseModel for OrganizationSubuser {
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString> {
        let table = table.unwrap_or("organization_subusers");

        let mut columns = BTreeMap::from([
            (
                compact_str::format_compact!("{table}.organization_id"),
                compact_str::format_compact!("{}organization_id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.pending"),
                compact_str::format_compact!("{}pending", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.created"),
                compact_str::format_compact!("{}created", prefix.unwrap_or_default()),
            ),
        ]);

        columns.extend(super::user::User::columns(Some("user_"), None));

        columns
    }

    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error> {
        let prefix = prefix.unwrap_or_default();
        let pending = row.try_get(compact_str::format_compact!("{prefix}pending").as_str())?;

        Ok(Self {
            organization_id: row
                .try_get(compact_str::format_compact!("{prefix}organization_id").as_str())?,
            user: super::user::User::map(Some("user_"), row)?.api_user(pending),
            created: row.try_get(compact_str::format_compact!("{prefix}created").as_str())?,
            pending,
        })
    }
}

impl OrganizationSubuser {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(
        database: &crate::database::Database,
        organization_id: i32,
        user_id: i32,
    ) -> Result<bool, anyhow::Error> {
        Ok(sqlx::query(
            r#"
            INSERT INTO organization_subusers (organization_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (organization_id, user_id) DO NOTHING
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .execute(database.write())
        .await?
        .rows_affected()
            == 1)
    }

    pub async fn save(&self, database: &crate::database::Database) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            UPDATE organization_subusers
            SET
                pending = $3
            WHERE
                organization_subusers.organization_id = $1
                AND organization_subusers.user_id = $2
            "#,
        )
        .bind(self.organization_id)
        .bind(self.user.id)
        .bind(self.pending)
        .execute(database.write())
        .await?;

        Ok(())
    }

    pub async fn count_by_organization(
        database: &crate::database::Database,
        organization_id: i32,
    ) -> i64 {
        sqlx::query(
            r#"
            SELECT COUNT(*)
            FROM organization_subusers
            WHERE organization_subusers.organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_one(database.read())
        .await
        .map_or(0, |row| row.try_get(0).unwrap_or(0))
    }

    pub async fn all_by_organization(
        database: &crate::database::Database,
        organization_id: i32,
    ) -> Result<Vec<Self>, anyhow::Error> {
        sqlx::query(&format!(
            r#"
            SELECT {}
            FROM organization_subusers
            LEFT JOIN users ON organization_subusers.user_id = users.id
            WHERE organization_subusers.organization_id = $1
            ORDER BY organization_subusers.created DESC
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(organization_id)
        .fetch_all(database.read())
        .await?
        .into_iter()
        .map(|row| Self::map(None, &row))
        .try_collect_vec()
    }

    pub async fn by_ids(
        database: &crate::database::Database,
        organization_id: i32,
        user_id: i32,
    ) -> Result<Option<Self>, anyhow::Error> {
        let data = sqlx::query(&format!(
            r#"
            SELECT {}
            FROM organization_subusers
            LEFT JOIN users ON organization_subusers.user_id = users.id
            WHERE
                organization_subusers.organization_id = $1
                AND organization_subusers.user_id = $2
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(organization_id)
        .bind(user_id)
        .fetch_optional(database.read())
        .await?;

        data.map(|row| Self::map(None, &row)).transpose()
    }

    pub async fn delete_by_ids(
        database: &crate::database::Database,
        organization_id: i32,
        user_id: i32,
    ) -> Result<bool, anyhow::Error> {
        Ok(sqlx::query(
            r#"
            DELETE FROM organization_subusers
            WHERE
                organization_subusers.organization_id = $1
                AND organization_subusers.user_id = $2
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .execute(database.write())
        .await?
        .rows_affected()
            == 1)
    }
}
