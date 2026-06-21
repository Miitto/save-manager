use dioxus::prelude::*;

#[cfg(feature = "server")]
mod auth;
#[cfg(feature = "server")]
mod db;

#[cfg(feature = "server")]
use bcrypt::{DEFAULT_COST, hash, verify};

mod save;
pub use save::*;

mod version;
pub use version::*;

pub type UserId = i32;

type Result<T> = core::result::Result<T, ServerFnError>;

#[server]
pub async fn get_greeting(name: String) -> Result<String> {
    debug!("Received name: {}", name);
    Ok(format!("Hello, {}!", name))
}

#[post("/api/user/login", auth: auth::Session, db: ServerDb)]
pub async fn login(username: String, password: String) -> Result<crate::UserPreview> {
    if auth.current_user.is_some() {
        return Err(
            HttpError::new(StatusCode::BAD_REQUEST, "Already logged in".to_string()).into(),
        );
    }

    let user: crate::auth::User =
        sqlx::query_as::<_, crate::auth::User>("SELECT * FROM users WHERE username = $1")
            .bind(&username)
            .fetch_one(&db.0)
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {:?}", e);
                HttpError::new(
                    StatusCode::UNAUTHORIZED,
                    "Invalid username or password".to_string(),
                )
            })?;

    if !verify(&password, &user.password).unwrap_or(false) {
        warn!("Invalid password for user: {}", username);
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        )
        .into());
    }

    auth.login_user(user.id.into());

    auth.remember_user(true);

    debug!("User logged in: {:?}", user);

    Ok(UserPreview {
        id: user.id,
        username: user.username,
    })
}

#[post("/api/user/register", auth: auth::Session, db: ServerDb)]
pub async fn register(username: String, password: String) -> Result<crate::UserPreview> {
    if auth.current_user.is_some() {
        return Err(
            HttpError::new(StatusCode::BAD_REQUEST, "Already logged in".to_string()).into(),
        );
    }

    let hashed_password = hash(&password, DEFAULT_COST).map_err(|e| {
        error!("Failed to hash password: {:?}", e);
        HttpError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        )
    })?;

    let user = sqlx::query_as::<_, crate::auth::User>(
        "INSERT INTO users (id, username, password) VALUES (NULL, $1, $2) RETURNING *",
    )
    .bind(&username)
    .bind(&hashed_password)
    .fetch_one(&db.0)
    .await
    .map_err(|e| {
        error!("Failed to create user: {:?}", e);
        HttpError::new(
            StatusCode::BAD_REQUEST,
            "Username already exists".to_string(),
        )
    })?;

    auth.login_user(user.id.into());

    auth.remember_user(true);

    debug!("User registered: {:?}", user);

    Ok(UserPreview {
        id: user.id,
        username: user.username,
    })
}

#[post("/api/user/logout", auth: auth::Session)]
pub async fn logout() -> Result<()> {
    if auth.current_user.is_none() {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "Not logged in".to_string()).into());
    }

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
/*
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
*/

#[cfg(feature = "server")]
pub type DbPool = sqlx::Pool<sqlx::Sqlite>;

#[cfg(feature = "server")]
pub type ServerDb = axum::Extension<DbPool>;

#[cfg(feature = "server")]
pub fn launch_server(
    app: fn() -> std::result::Result<dioxus::prelude::VNode, dioxus::prelude::RenderError>,
) {
    use crate::auth::*;
    use axum_session::{SessionConfig, SessionLayer, SessionStore};
    use axum_session_auth::AuthConfig;
    use axum_session_sqlx::SessionSqlitePool;
    use dioxus::logger::tracing::Level;
    use sqlx::{
        ConnectOptions, Executor,
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    };
    use std::str::FromStr;

    dioxus::logger::init(Level::DEBUG).expect("Failed to initialize logger");

    dioxus::serve(|| async move {
        let connection_options = SqliteConnectOptions::from_str("file:db.db")?
            .create_if_missing(true)
            .foreign_keys(true)
            .log_statements(log::LevelFilter::Trace)
            .log_slow_statements(log::LevelFilter::Warn, std::time::Duration::from_secs(1));
        let db = SqlitePoolOptions::new()
            .max_connections(20)
            .connect_with(connection_options)
            .await?;

        if let Err(e) = db::setup_db(&db).await {
            error!("Failed to set up database: {:?}", e);
            return Err(e);
        }

        let init_test_data = async || -> anyhow::Result<()> {
            debug!("Inserting test data into database");

            let password_hash =
                hash("TestPass", DEFAULT_COST).context("Failed to hash password")?;

            sqlx::query(r#"INSERT INTO users (id, username, password) SELECT 1, 'Test', $1 ON CONFLICT(id) DO UPDATE SET username = EXCLUDED.username, password = EXCLUDED.password"#)
                .bind(password_hash).execute(&db)
            .await.context("Failed to create test user")?;

            db.execute(r#"INSERT INTO saves (id, name, game, owner) SELECT 1, 'Test Save', 0, 1 ON CONFLICT(id) DO UPDATE SET name = EXCLUDED.name, game = EXCLUDED.game"#,)
            .await.context("Failed to create save")?;

            db.execute(r#"INSERT INTO versions (id, save_id, version, label, timestamp, by) SELECT 1, 1, 1, 'Initial Version', strftime('%s','now'), 1 ON CONFLICT(id) DO UPDATE SET save_id = EXCLUDED.save_id, version = EXCLUDED.version, label = EXCLUDED.label, timestamp = EXCLUDED.timestamp, by = EXCLUDED.by"#,)
            .await.context("Failed to create version")?;

            debug!("Test data inserted successfully");

            Ok(())
        };

        init_test_data().await?;

        // Create an axum router that dioxus will attach the app to
        Ok(dioxus::server::router(app)
            .layer(AuthLayer::new(Some(db.clone())).with_config(AuthConfig::<i64>::default()))
            .layer(SessionLayer::new(
                SessionStore::<SessionSqlitePool>::new(
                    Some(db.clone().into()),
                    SessionConfig::default().with_table_name("test_table"),
                )
                .await?,
            ))
            .layer(axum::middleware::from_fn(
                move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                    let db = db.clone();
                    async move {
                        req.extensions_mut().insert(db);

                        next.run(req).await
                    }
                },
            )))
    });
}
