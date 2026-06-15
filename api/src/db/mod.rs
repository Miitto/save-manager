use anyhow::Context;
use sqlx::Executor;

pub(crate) async fn setup_db(db: &sqlx::Pool<sqlx::Sqlite>) -> anyhow::Result<()> {
    db.execute(
        r#"CREATE TABLE IF NOT EXISTS users (
                "id" INTEGER PRIMARY KEY,
                "anonymous" BOOLEAN NOT NULL,
                "username" VARCHAR(256) NOT NULL
            )"#,
    )
    .await?;

    db.execute(
        r#"CREATE TABLE IF NOT EXISTS user_permissions (
                "user_id" INTEGER NOT NULL,
                "token" VARCHAR(256) NOT NULL
            )"#,
    )
    .await?;

    db.execute(
        r#"CREATE TABLE IF NOT EXISTS saves (
                "id" INTEGER PRIMARY KEY,
                "name" VARCHAR(256) NOT NULL,
                "game" INTEGER NOT NULL,
                "owner" INTEGER NOT NULL,
                FOREIGN KEY(owner) REFERENCES users(id)
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
