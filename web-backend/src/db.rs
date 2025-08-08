use sqlx::{Pool, Sqlite, Postgres, Any};
use std::time::Duration;

pub type DbPool = Pool<Any>;

pub async fn init_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = if database_url.starts_with("sqlite") {
        sqlx::any::AnyPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?
    } else {
        sqlx::any::AnyPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?
    };

    Ok(pool)
}