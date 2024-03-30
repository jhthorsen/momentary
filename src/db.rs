use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn build_db(database_url: &String) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new().connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
