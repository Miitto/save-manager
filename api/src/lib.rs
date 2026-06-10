use std::collections::HashSet;

use dioxus::prelude::*;

#[cfg(feature = "server")]
mod auth;
#[cfg(feature = "server")]
mod db;

type Result<T> = core::result::Result<T, ServerFnError>;

#[server]
pub async fn get_greeting(name: String) -> Result<String> {
    debug!("Received name: {}", name);
    Ok(format!("Hello, {}!", name))
}

#[post("/api/user/login", auth: auth::Session)]
pub async fn login() -> Result<()> {
    auth.login_user(2);
    Ok(())
}

#[post("/api/user/logout", auth: auth::Session)]
pub async fn logout() -> Result<()> {
    auth.logout_user();
    Ok(())
}

#[post("/api/user/name", auth: auth::Session)]
pub async fn get_username() -> Result<String> {
    Ok(auth
        .current_user
        .map(|u| u.username.clone())
        .unwrap_or_else(|| "Unknown".to_string()))
}

/// Get the current user's permissions, guarding the endpoint with the `Auth` validator.
/// If this returns false, we use the `or_unauthorized` extension to return a 401 error.
#[get("/api/user/permissions", auth: auth::Session)]
pub async fn get_permissions() -> Result<HashSet<String>> {
    use crate::auth::User;
    use axum_session_auth::{Auth, Rights};

    let user = auth.current_user.unwrap();

    Auth::<User, i64, sqlx::SqlitePool>::build([axum::http::Method::GET], false)
        .requires(Rights::any([
            Rights::permission("Category::View"),
            Rights::permission("Admin::View"),
        ]))
        .validate(&user, &axum::http::Method::GET, None)
        .await
        .or_unauthorized("You do not have permission to view categories")?;

    Ok(user.permissions)
}

#[cfg(feature = "server")]
pub fn launch_server(
    app: fn() -> std::result::Result<dioxus::prelude::VNode, dioxus::prelude::RenderError>,
) {
    use crate::auth::*;
    use axum_session::{SessionConfig, SessionLayer, SessionStore};
    use axum_session_auth::AuthConfig;
    use axum_session_sqlx::SessionSqlitePool;
    use sqlx::{Executor, sqlite::SqlitePoolOptions};

    dioxus::serve(|| async move {
        let db = SqlitePoolOptions::new()
            .max_connections(20)
            .connect_with("sqlite::memory:".parse()?)
            .await?;

        db::setup_db(&db).await?;

        // Insert in some test data for two users (one anonymous, one normal)
        db.execute(r#"INSERT INTO users (id, anonymous, username) SELECT 1, true, 'Guest' ON CONFLICT(id) DO UPDATE SET anonymous = EXCLUDED.anonymous, username = EXCLUDED.username"#,)
            .await?;
        db.execute(r#"INSERT INTO users (id, anonymous, username) SELECT 2, false, 'Test' ON CONFLICT(id) DO UPDATE SET anonymous = EXCLUDED.anonymous, username = EXCLUDED.username"#,)
            .await?;

        // Make sure our test user has the ability to view categories
        db.execute(r#"INSERT INTO user_permissions (user_id, token) SELECT 2, 'Category::View'"#)
            .await?;

        // Create an axum router that dioxus will attach the app to
        Ok(dioxus::server::router(app)
            .layer(
                AuthLayer::new(Some(db.clone()))
                    .with_config(AuthConfig::<i64>::default().with_anonymous_user_id(Some(1))),
            )
            .layer(SessionLayer::new(
                SessionStore::<SessionSqlitePool>::new(
                    Some(db.into()),
                    SessionConfig::default().with_table_name("test_table"),
                )
                .await?,
            )))
    });
}
