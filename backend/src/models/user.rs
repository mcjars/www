use super::BaseModel;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{Row, postgres::PgRow, types::chrono::NaiveDateTime};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub github_id: i32,

    pub admin: bool,

    pub name: Option<compact_str::CompactString>,
    pub email: compact_str::CompactString,
    pub login: compact_str::CompactString,

    pub last_login: NaiveDateTime,
    pub created: NaiveDateTime,
}

impl BaseModel for User {
    #[inline]
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString> {
        let table = table.unwrap_or("users");

        BTreeMap::from([
            (
                compact_str::format_compact!("{table}.id"),
                compact_str::format_compact!("{}id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.github_id"),
                compact_str::format_compact!("{}github_id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.admin"),
                compact_str::format_compact!("{}admin", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.name"),
                compact_str::format_compact!("{}name", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.email"),
                compact_str::format_compact!("{}email", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.login"),
                compact_str::format_compact!("{}login", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.last_login"),
                compact_str::format_compact!("{}last_login", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.created"),
                compact_str::format_compact!("{}created", prefix.unwrap_or_default()),
            ),
        ])
    }

    #[inline]
    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error> {
        let prefix = prefix.unwrap_or_default();

        Ok(Self {
            id: row.try_get(compact_str::format_compact!("{prefix}id").as_str())?,
            github_id: row.try_get(compact_str::format_compact!("{prefix}github_id").as_str())?,
            admin: row.try_get(compact_str::format_compact!("{prefix}admin").as_str())?,
            name: row.try_get(compact_str::format_compact!("{prefix}name").as_str())?,
            email: row.try_get(compact_str::format_compact!("{prefix}email").as_str())?,
            login: row.try_get(compact_str::format_compact!("{prefix}login").as_str())?,
            last_login: row.try_get(compact_str::format_compact!("{prefix}last_login").as_str())?,
            created: row.try_get(compact_str::format_compact!("{prefix}created").as_str())?,
        })
    }
}

impl User {
    #[inline]
    pub async fn new(
        database: &crate::database::Database,
        github_id: i32,
        name: Option<String>,
        email: String,
        login: String,
    ) -> Result<Self, anyhow::Error> {
        let row = sqlx::query(&format!(
            r#"
            INSERT INTO users (github_id, name, email, login, last_login, created)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (github_id) DO UPDATE SET 
                name = EXCLUDED.name,
                email = EXCLUDED.email,
                login = EXCLUDED.login,
                last_login = NOW()
            RETURNING {}
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(github_id)
        .bind(&name)
        .bind(&email)
        .bind(&login)
        .fetch_one(database.write())
        .await?;

        Self::map(None, &row)
    }

    #[inline]
    pub async fn by_session(
        database: &crate::database::Database,
        session: &str,
    ) -> Result<Option<(Self, UserSession)>, anyhow::Error> {
        let row = sqlx::query(&format!(
            r#"
            SELECT {}, {}
            FROM users
            JOIN user_sessions ON user_sessions.user_id = users.id
            WHERE user_sessions.session = $1
            "#,
            Self::columns_sql(None, None),
            UserSession::columns_sql(Some("session_"), None)
        ))
        .bind(session)
        .fetch_optional(database.read())
        .await?;

        row.map(|row| {
            Ok((
                Self::map(None, &row)?,
                UserSession::map(Some("session_"), &row)?,
            ))
        })
        .transpose()
    }

    #[inline]
    pub async fn by_login(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        login: &str,
    ) -> Result<Option<Self>, anyhow::Error> {
        cache
            .cached(&format!("user::{login}"), 3600, || async {
                let data = sqlx::query(&format!(
                    r#"
                    SELECT {}
                    FROM users
                    WHERE users.login ILIKE $1
                    "#,
                    Self::columns_sql(None, None)
                ))
                .bind(login.replace('%', "\\%").replace('_', "\\_"))
                .fetch_optional(database.read())
                .await?;

                data.map(|row| Self::map(None, &row)).transpose()
            })
            .await
    }

    #[inline]
    pub fn api_user(&self, hide_email: bool) -> ApiUser {
        ApiUser {
            id: self.id,
            github_id: self.github_id,
            admin: self.admin,
            name: self.name.clone(),
            avatar: compact_str::format_compact!(
                "https://avatars.githubusercontent.com/u/{}",
                self.github_id
            ),
            email: if hide_email {
                "hidden@email.com".into()
            } else {
                self.email.clone()
            },
            login: self.login.clone(),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[schema(rename_all = "camelCase", title = "User")]
pub struct ApiUser {
    pub id: i32,
    pub github_id: i32,
    pub admin: bool,
    pub name: Option<compact_str::CompactString>,
    pub avatar: compact_str::CompactString,
    pub email: compact_str::CompactString,
    pub login: compact_str::CompactString,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[schema(rename_all = "camelCase")]
pub struct UserSession {
    pub id: i32,

    #[schema(value_type = String)]
    pub ip: sqlx::types::ipnetwork::IpNetwork,
    pub user_agent: compact_str::CompactString,

    pub last_used: NaiveDateTime,
    pub created: NaiveDateTime,
}

impl BaseModel for UserSession {
    #[inline]
    fn columns(
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> BTreeMap<compact_str::CompactString, compact_str::CompactString> {
        let table = table.unwrap_or("user_sessions");

        BTreeMap::from([
            (
                compact_str::format_compact!("{table}.id"),
                compact_str::format_compact!("{}id", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.ip"),
                compact_str::format_compact!("{}ip", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.user_agent"),
                compact_str::format_compact!("{}user_agent", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.last_used"),
                compact_str::format_compact!("{}last_used", prefix.unwrap_or_default()),
            ),
            (
                compact_str::format_compact!("{table}.created"),
                compact_str::format_compact!("{}created", prefix.unwrap_or_default()),
            ),
        ])
    }

    #[inline]
    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, anyhow::Error> {
        let prefix = prefix.unwrap_or_default();

        Ok(Self {
            id: row.try_get(compact_str::format_compact!("{prefix}id").as_str())?,
            ip: row.try_get(compact_str::format_compact!("{prefix}ip").as_str())?,
            user_agent: row.try_get(compact_str::format_compact!("{prefix}user_agent").as_str())?,
            last_used: row.try_get(compact_str::format_compact!("{prefix}last_used").as_str())?,
            created: row.try_get(compact_str::format_compact!("{prefix}created").as_str())?,
        })
    }
}

impl UserSession {
    #[inline]
    pub async fn new(
        database: &crate::database::Database,
        user_id: i32,
        ip: sqlx::types::ipnetwork::IpNetwork,
        user_agent: &str,
    ) -> Result<(Self, String), anyhow::Error> {
        let mut hash = sha2::Sha256::new();
        hash.update(chrono::Utc::now().timestamp().to_be_bytes());
        hash.update(user_id.to_be_bytes());
        let hash = format!("{:x}", hash.finalize());

        let row = sqlx::query(&format!(
            r#"
            INSERT INTO user_sessions (user_id, session, ip, user_agent, last_used, created)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING {}
            "#,
            Self::columns_sql(None, None)
        ))
        .bind(user_id)
        .bind(&hash)
        .bind(ip)
        .bind(user_agent)
        .fetch_one(database.write())
        .await?;

        Ok((Self::map(None, &row)?, hash))
    }

    #[inline]
    pub async fn save(&self, database: &crate::database::Database) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET
                ip = $2,
                user_agent = $3,
                last_used = $4
            WHERE user_sessions.id = $1
            "#,
        )
        .bind(self.id)
        .bind(self.ip)
        .bind(&self.user_agent)
        .bind(self.last_used)
        .execute(database.write())
        .await?;

        Ok(())
    }

    #[inline]
    pub async fn delete_by_session(
        database: &crate::database::Database,
        session: &str,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            DELETE FROM user_sessions
            WHERE user_sessions.session = $1
            "#,
        )
        .bind(session)
        .execute(database.write())
        .await?;

        Ok(())
    }
}
