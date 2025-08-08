use sqlx::{postgres::PgPool, Pool, Postgres};
use anyhow::Result;
use uuid::Uuid;
use std::time::Duration;

pub type DbPool = PgPool;

// Database abstraction for common operations
pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    // Generic query execution with proper error handling
    pub async fn execute_query(&self, query: &str, params: &[&(dyn sqlx::Encode<'_, Postgres> + Send + Sync)]) -> Result<u64> {
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    // Fetch a single row
    pub async fn fetch_one(&self, query: &str) -> Result<sqlx::postgres::PgRow> {
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }

    // Fetch all rows
    pub async fn fetch_all(&self, query: &str) -> Result<Vec<sqlx::postgres::PgRow>> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }
}

pub async fn init_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    Ok(pool)
}

// Helper functions for converting between types
pub fn uuid_to_bytes(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

pub fn bytes_to_uuid(bytes: Vec<u8>) -> Result<Uuid> {
    let array: [u8; 16] = bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid UUID bytes"))?;
    Ok(Uuid::from_bytes(array))
}