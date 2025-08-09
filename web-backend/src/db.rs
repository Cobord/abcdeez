#[cfg(feature = "sqlite")]
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
#[cfg(feature = "postgres")]
use sqlx::{postgres::PgPool, Pool, Postgres};

use anyhow::Result;
use uuid::Uuid;
use std::time::Duration;

#[cfg(feature = "sqlite")]
pub type DbPool = SqlitePool;
#[cfg(feature = "postgres")]
pub type DbPool = PgPool;

#[cfg(feature = "sqlite")]
pub type DbRow = sqlx::sqlite::SqliteRow;
#[cfg(feature = "postgres")]
pub type DbRow = sqlx::postgres::PgRow;

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
    pub async fn execute_query(&self, query: &str) -> Result<u64> {
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    // Fetch a single row
    pub async fn fetch_one(&self, query: &str) -> Result<DbRow> {
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }

    // Fetch all rows
    pub async fn fetch_all(&self, query: &str) -> Result<Vec<DbRow>> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }
}

#[cfg(feature = "sqlite")]
pub async fn init_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    // Ensure database URL is for SQLite
    let db_url = if !database_url.starts_with("sqlite:") {
        if database_url.contains("://") {
            // It's some other database URL, convert to SQLite
            "sqlite:test.db"
        } else {
            // Assume it's a file path
            &format!("sqlite:{}", database_url)
        }
    } else {
        database_url
    };

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(db_url)
        .await?;

    // Enable foreign key constraints for SQLite
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}

#[cfg(feature = "postgres")]
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

// Database-agnostic query builders
pub fn bind_uuid_param(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

// Parameter placeholder - SQLite uses ?, PostgreSQL uses $1, $2, etc.
#[cfg(feature = "sqlite")]
pub fn param_placeholder(_index: usize) -> &'static str {
    "?"
}

#[cfg(feature = "postgres")]
pub fn param_placeholder(index: usize) -> String {
    format!("${}", index)
}

// JSON field access - different syntax between databases
#[cfg(feature = "sqlite")]
pub fn json_extract(field: &str, path: &str) -> String {
    format!("JSON_EXTRACT({}, '{}')", field, path)
}

#[cfg(feature = "postgres")]
pub fn json_extract(field: &str, path: &str) -> String {
    format!("{}->>'{}' ", field, path)
}

// Database migration handling
#[cfg(feature = "sqlite")]
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

#[cfg(feature = "postgres")]
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations-postgres").run(pool).await
}