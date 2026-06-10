use sqlx::Executor;

pub(crate) async fn setup_db(db: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
    db.execute(r#"CREATE TABLE IF NOT EXISTS users ( "id" INTEGER PRIMARY KEY, "anonymous" BOOLEAN NOT NULL, "username" VARCHAR(256) NOT NULL )"#,)
            .await?;

    db.execute(r#"CREATE TABLE IF NOT EXISTS user_permissions ( "user_id" INTEGER NOT NULL, "token" VARCHAR(256) NOT NULL)"#,)
            .await?;

    Ok(())
}
