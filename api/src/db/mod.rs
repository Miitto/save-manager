use anyhow::Context;
use bcrypt::{DEFAULT_COST, hash};
use dioxus::prelude::{debug, error};
use sqlx::{
    ConnectOptions, Executor,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;

pub type Pool = sqlx::Pool<sqlx::Sqlite>;

const DATABASE_URL: &str = "file:db.db";

pub(crate) async fn create_pool() -> anyhow::Result<Pool> {
    let connection_options = sqlx::sqlite::SqliteConnectOptions::from_str(DATABASE_URL)?
        .create_if_missing(true)
        .foreign_keys(true)
        .log_statements(log::LevelFilter::Trace)
        .log_slow_statements(log::LevelFilter::Warn, std::time::Duration::from_secs(1));

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(connection_options)
        .await?;

    Ok(pool)
}

pub(crate) async fn setup_db(db: &Pool) -> anyhow::Result<()> {
    db.execute(
        r#"CREATE TABLE IF NOT EXISTS users (
                "id" INTEGER PRIMARY KEY,
                "username" VARCHAR(256) NOT NULL UNIQUE,
                "password" VARCHAR(256) NOT NULL
            )"#,
    )
    .await?;

    db.execute(
        r#"CREATE TABLE IF NOT EXISTS saves (
                "id" INTEGER PRIMARY KEY,
                "name" VARCHAR(256) NOT NULL,
                "game" INTEGER NOT NULL,
                "owner" INTEGER NOT NULL,
                FOREIGN KEY(owner) REFERENCES users(id),
                UNIQUE(name, game, owner)
        )"#,
    )
    .await
    .context("Failed to create saves table")?;

    db.execute(
        r#"CREATE TABLE IF NOT EXISTS versions (
                "id" INTEGER PRIMARY KEY,
                "save_id" INTEGER NOT NULL,
                "version" INTEGER NOT NULL,
                "label" VARCHAR(256) NOT NULL,
                "timestamp" INTEGER NOT NULL DEFAULT (unixepoch('now')),
                "by" INTEGER NOT NULL,
                FOREIGN KEY(save_id) REFERENCES saves(id),
                FOREIGN KEY(by) REFERENCES users(id)
        )"#,
    )
    .await
    .context("Failed to create versions table")?;

    db.execute(
        r#"CREATE TABLE IF NOT EXISTS user_save_access (
                "user_id" INTEGER NOT NULL,
                "save_id" INTEGER NOT NULL,
                "access" INTEGER NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id),
                FOREIGN KEY(save_id) REFERENCES saves(id)
        )"#,
    )
    .await
    .context("Failed to create user_save_access table")?;

    Ok(())
}
