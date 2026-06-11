//! The code here is pulled from the `axum-session-auth` crate examples, requiring little to no
//! modification to work with dioxus fullstack.

use async_trait::async_trait;
use axum_session_auth::*;
use axum_session_sqlx::SessionSqlitePool;
use dioxus::fullstack::HttpError;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::collections::HashSet;

pub(crate) type Session = axum_session_auth::AuthSession<User, i64, SessionSqlitePool, SqlitePool>;
pub(crate) type AuthLayer =
    axum_session_auth::AuthSessionLayer<User, i64, SessionSqlitePool, SqlitePool>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct User {
    pub id: crate::UserId,
    pub anonymous: bool,
    pub username: String,
    pub permissions: HashSet<String>,
}

#[derive(sqlx::FromRow, Clone)]
pub(crate) struct SqlPermissionTokens {
    pub token: String,
}

pub(crate) trait RequireUser {
    fn require_user(&self) -> Result<&User, dioxus::server::ServerFnError>;
}

impl RequireUser for Session {
    fn require_user(&self) -> Result<&User, dioxus::server::ServerFnError> {
        self.current_user.as_ref().ok_or_else(|| {
            HttpError::new(
                StatusCode::UNAUTHORIZED,
                "You are not authenticated".to_string(),
            )
            .into()
        })
    }
}

#[async_trait]
impl Authentication<User, i64, SqlitePool> for User {
    async fn load_user(userid: i64, pool: Option<&SqlitePool>) -> Result<User, anyhow::Error> {
        let db = pool.unwrap();

        #[derive(sqlx::FromRow, Clone)]
        struct SqlUser {
            id: crate::UserId,
            anonymous: bool,
            username: String,
        }

        let sqluser = sqlx::query_as::<_, SqlUser>("SELECT * FROM users WHERE id = $1")
            .bind(userid)
            .fetch_one(db)
            .await
            .unwrap();

        //lets just get all the tokens the user can use, we will only use the full permissions if modifying them.
        let sql_user_perms = sqlx::query_as::<_, SqlPermissionTokens>(
            "SELECT token FROM user_permissions WHERE user_id = $1;",
        )
        .bind(userid)
        .fetch_all(db)
        .await
        .unwrap();

        Ok(User {
            id: sqluser.id,
            anonymous: sqluser.anonymous,
            username: sqluser.username,
            permissions: sql_user_perms.into_iter().map(|x| x.token).collect(),
        })
    }

    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}

#[async_trait]
impl HasPermission<SqlitePool> for User {
    async fn has(&self, perm: &str, _pool: &Option<&SqlitePool>) -> bool {
        self.permissions.contains(perm)
    }
}
