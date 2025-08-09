#[cfg(feature = "sqlite")]
use sqlx::{Row,sqlite::SqlitePool};
#[cfg(feature = "postgres")]
use sqlx::{Row, postgres::PgPool, Pool, Postgres};

use anyhow::Result;
use std::time::Duration;
use uuid::Uuid;

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
        let result = sqlx::query(query).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    // Fetch a single row
    pub async fn fetch_one(&self, query: &str) -> Result<DbRow> {
        let row = sqlx::query(query).fetch_one(&self.pool).await?;
        Ok(row)
    }

    // Fetch all rows
    pub async fn fetch_all(&self, query: &str) -> Result<Vec<DbRow>> {
        let rows = sqlx::query(query).fetch_all(&self.pool).await?;
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
        .max_connections(
            std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
        )
        .min_connections(
            std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        )
        .acquire_timeout(Duration::from_secs(
            std::env::var("DB_ACQUIRE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        ))
        .max_lifetime(Duration::from_secs(
            std::env::var("DB_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1800), // 30 minutes
        ))
        .idle_timeout(Duration::from_secs(
            std::env::var("DB_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(600), // 10 minutes
        ))
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
        .max_connections(
            std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
        )
        .min_connections(
            std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        )
        .acquire_timeout(Duration::from_secs(
            std::env::var("DB_ACQUIRE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        ))
        .max_lifetime(Duration::from_secs(
            std::env::var("DB_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1800), // 30 minutes
        ))
        .idle_timeout(Duration::from_secs(
            std::env::var("DB_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(600), // 10 minutes
        ))
        .connect(database_url)
        .await?;

    Ok(pool)
}

// Helper functions for converting between types
pub fn uuid_to_bytes(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

pub fn bytes_to_uuid(bytes: Vec<u8>) -> Result<Uuid> {
    let array: [u8; 16] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid UUID bytes"))?;
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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Migration status and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationInfo {
    pub version: i64,
    pub description: String,
    pub applied_at: Option<DateTime<Utc>>,
    pub rollback_sql: Option<String>,
    pub checksum: String,
    pub can_rollback: bool,
}

/// Migration management with rollback capabilities
pub struct MigrationManager {
    pool: DbPool,
    migrations_path: String,
    rollback_migrations: HashMap<i64, String>,
}

impl MigrationManager {
    pub fn new(pool: DbPool, migrations_path: String) -> Self {
        Self {
            pool,
            migrations_path,
            rollback_migrations: HashMap::new(),
        }
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> Result<Vec<MigrationInfo>, sqlx::migrate::MigrateError> {
        // Ensure migration tracking table exists
        self.ensure_migration_tracking().await?;

        // Run SQLx migrations
        #[cfg(feature = "sqlite")]
        let migrations = sqlx::migrate!("./migrations");
        #[cfg(feature = "postgres")]
        let migrations = sqlx::migrate!("./migrations-postgres");

        migrations.run(&self.pool).await?;

        // Return applied migrations info
        self.get_migration_history().await
    }

    /// Rollback to a specific migration version
    pub async fn rollback_to(
        &self,
        target_version: i64,
    ) -> Result<Vec<MigrationInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let current_migrations = self.get_applied_migrations().await?;
        let mut rollback_info = Vec::new();

        // Find migrations to rollback (in reverse order)
        let mut to_rollback: Vec<_> = current_migrations
            .into_iter()
            .filter(|m| m.version > target_version)
            .collect();
        to_rollback.sort_by(|a, b| b.version.cmp(&a.version)); // Reverse order

        if to_rollback.is_empty() {
            tracing::info!(
                "No migrations to rollback. Already at or before version {}",
                target_version
            );
            return Ok(rollback_info);
        }

        // Execute rollbacks
        let mut tx = self.pool.begin().await?;

        for migration in to_rollback {
            if !migration.can_rollback {
                return Err(format!(
                    "Migration {} cannot be rolled back safely",
                    migration.version
                )
                .into());
            }

            if let Some(rollback_sql) = &migration.rollback_sql {
                tracing::info!(
                    "Rolling back migration {}: {}",
                    migration.version,
                    migration.description
                );

                // Execute rollback SQL
                sqlx::query(rollback_sql)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        format!("Rollback failed for migration {}: {}", migration.version, e)
                    })?;

                // Remove from migration tracking
                #[cfg(feature = "sqlite")]
                sqlx::query("DELETE FROM _sqlx_migrations WHERE version = ?")
                    .bind(migration.version)
                    .execute(&mut *tx)
                    .await?;

                #[cfg(feature = "postgres")]
                sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
                    .bind(migration.version)
                    .execute(&mut *tx)
                    .await?;

                rollback_info.push(migration);
            } else {
                return Err(format!(
                    "No rollback SQL available for migration {}",
                    migration.version
                )
                .into());
            }
        }

        tx.commit().await?;

        tracing::info!(
            "Successfully rolled back {} migrations to version {}",
            rollback_info.len(),
            target_version
        );
        Ok(rollback_info)
    }

    /// Rollback the last N migrations
    pub async fn rollback_last(
        &self,
        count: usize,
    ) -> Result<Vec<MigrationInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let applied = self.get_applied_migrations().await?;

        if applied.is_empty() {
            return Ok(Vec::new());
        }

        let mut sorted = applied;
        sorted.sort_by(|a, b| b.version.cmp(&a.version));

        if count >= sorted.len() {
            // Rollback all migrations
            self.rollback_to(0).await
        } else {
            let target_version = sorted[count].version;
            self.rollback_to(target_version).await
        }
    }

    /// Get migration status and history
    pub async fn get_migration_history(
        &self,
    ) -> Result<Vec<MigrationInfo>, sqlx::migrate::MigrateError> {
        self.ensure_migration_tracking().await?;

        #[cfg(feature = "sqlite")]
        let query = "SELECT version, description, installed_on, checksum FROM _sqlx_migrations ORDER BY version";
        #[cfg(feature = "postgres")]
        let query = "SELECT version, description, installed_on, checksum FROM _sqlx_migrations ORDER BY version";

        let rows = sqlx::query(query).fetch_all(&self.pool).await?;

        let mut migrations = Vec::new();
        for row in rows {
            let version: i64 = row.try_get("version")?;
            let description: String = row.try_get("description")?;
            let installed_on: DateTime<Utc> = row.try_get("installed_on")?;
            let checksum: String = row.try_get("checksum")?;

            migrations.push(MigrationInfo {
                version,
                description,
                applied_at: Some(installed_on),
                rollback_sql: self.rollback_migrations.get(&version).cloned(),
                checksum,
                can_rollback: self.rollback_migrations.contains_key(&version),
            });
        }

        Ok(migrations)
    }

    /// Check if database is up to date
    pub async fn is_up_to_date(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let applied = self.get_applied_migrations().await?;
        let available = self.get_available_migrations().await?;

        if applied.is_empty() && available.is_empty() {
            return Ok(true);
        }

        let latest_applied = applied.iter().map(|m| m.version).max().unwrap_or(0);
        let latest_available = available.iter().map(|m| m.version).max().unwrap_or(0);

        Ok(latest_applied >= latest_available)
    }

    /// Validate migration integrity
    pub async fn validate_migrations(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        let applied = self.get_applied_migrations().await?;

        // Check for missing rollback scripts for recent migrations
        let recent_threshold = Utc::now() - chrono::Duration::days(30);
        for migration in &applied {
            if let Some(applied_at) = migration.applied_at {
                if applied_at > recent_threshold && !migration.can_rollback {
                    issues.push(format!(
                        "Migration {} (applied {}) lacks rollback capability",
                        migration.version,
                        applied_at.format("%Y-%m-%d")
                    ));
                }
            }
        }

        // Check for checksum mismatches (simplified)
        for migration in &applied {
            if migration.checksum.is_empty() {
                issues.push(format!(
                    "Migration {} has empty checksum",
                    migration.version
                ));
            }
        }

        Ok(issues)
    }

    /// Create a backup before migrations
    pub async fn create_backup(
        &self,
        backup_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = format!("backups/{}_{}.sql", backup_name, timestamp);

        #[cfg(feature = "sqlite")]
        {
            // For SQLite, we can use VACUUM INTO or copy the file
            let db_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:test.db".to_string());
            let db_path = db_url
                .strip_prefix("sqlite:")
                .unwrap_or("test.db");

            std::fs::create_dir_all("backups")?;
            std::fs::copy(db_path, &backup_path)?;

            tracing::info!("SQLite database backed up to: {}", backup_path);
        }

        #[cfg(feature = "postgres")]
        {
            // For PostgreSQL, we would use pg_dump
            tracing::warn!("PostgreSQL backup not implemented - use pg_dump manually");
            return Err("PostgreSQL backup not implemented".into());
        }

        Ok(backup_path)
    }

    // Private helper methods
    async fn ensure_migration_tracking(&self) -> Result<(), sqlx::migrate::MigrateError> {
        // SQLx automatically creates the _sqlx_migrations table
        Ok(())
    }

    async fn get_applied_migrations(
        &self,
    ) -> Result<Vec<MigrationInfo>, sqlx::migrate::MigrateError> {
        self.get_migration_history().await
    }

    async fn get_available_migrations(
        &self,
    ) -> Result<Vec<MigrationInfo>, Box<dyn std::error::Error + Send + Sync>> {
        // This would scan the migrations directory and return available migrations
        // For now, return empty - would need filesystem scanning in real implementation
        Ok(Vec::new())
    }

    /// Register rollback SQL for a migration
    pub fn register_rollback(&mut self, version: i64, rollback_sql: String) {
        self.rollback_migrations.insert(version, rollback_sql);
    }
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

/// Enhanced migration runner with rollback support
pub async fn run_migrations_with_rollback(
    pool: &DbPool,
) -> Result<MigrationManager, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "sqlite")]
    let migrations_path = "./migrations".to_string();
    #[cfg(feature = "postgres")]
    let migrations_path = "./migrations-postgres".to_string();

    let mut manager = MigrationManager::new(pool.clone(), migrations_path);

    // Register known rollback scripts
    register_rollback_scripts(&mut manager);

    // Run migrations
    manager.migrate().await?;

    Ok(manager)
}

/// Register rollback SQL for known migrations
pub fn register_rollback_scripts(manager: &mut MigrationManager) {
    // Example rollback scripts - these would be maintained alongside forward migrations

    // Migration 001: Initial tables
    manager.register_rollback(
        1,
        r#"
        DROP TABLE IF EXISTS responses;
        DROP TABLE IF EXISTS sessions; 
        DROP TABLE IF EXISTS learners;
        DROP TABLE IF EXISTS users;
    "#
        .to_string(),
    );

    // Migration 002: Add metadata columns
    manager.register_rollback(
        2,
        r#"
        ALTER TABLE users DROP COLUMN IF EXISTS metadata;
        ALTER TABLE learners DROP COLUMN IF EXISTS metadata;
    "#
        .to_string(),
    );

    // Migration 003: Add audit tables
    manager.register_rollback(
        3,
        r#"
        DROP TABLE IF EXISTS audit_events;
        DROP TABLE IF EXISTS audit_security_events;
    "#
        .to_string(),
    );

    // Migration 004: Add indexes
    manager.register_rollback(
        4,
        r#"
        DROP INDEX IF EXISTS idx_users_email;
        DROP INDEX IF EXISTS idx_sessions_learner_id;
        DROP INDEX IF EXISTS idx_responses_session_id;
    "#
        .to_string(),
    );

    // Migration 005: Add gamification
    manager.register_rollback(
        5,
        r#"
        DROP TABLE IF EXISTS user_achievements;
        DROP TABLE IF EXISTS leaderboards;
        ALTER TABLE users DROP COLUMN IF EXISTS xp_points;
        ALTER TABLE users DROP COLUMN IF EXISTS level;
    "#
        .to_string(),
    );
}
