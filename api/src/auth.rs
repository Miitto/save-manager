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
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub(crate) struct User {
    pub id: crate::UserId,
    pub username: String,
    pub password: String,
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

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(userid)
            .fetch_one(db)
            .await
            .unwrap();

        Ok(user)
    }

    fn is_authenticated(&self) -> bool {
        true
    }

    fn is_active(&self) -> bool {
        true
    }

    fn is_anonymous(&self) -> bool {
        false
    }
}
