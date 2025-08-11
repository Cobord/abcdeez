/// Example Service Template - Database Agnostic Implementation
/// 
/// This file demonstrates best practices for creating services that work with
/// both SQLite and PostgreSQL databases using the abstraction layer.

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    config::Config,
    // Import the database abstraction types and functions
    db::{
        DbPool, DbUuid, DbJson,
        uuid_to_db, uuid_from_db,
        json_to_db, json_from_db,
        param_placeholder, json_extract
    },
    error::{AppError, AppResult},
};

/// Example entity model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExampleEntity {
    pub id: Uuid,
    pub name: String,
    pub metadata: serde_json::Value,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Example service that works with both SQLite and PostgreSQL
pub struct ExampleService {
    db_pool: Arc<DbPool>,  // Uses DbPool type alias, not Pool<Sqlite> or Pool<Postgres>
    config: Arc<Config>,
}

impl ExampleService {
    /// Create a new service instance
    pub fn new(db_pool: Arc<DbPool>, config: Arc<Config>) -> Self {
        Self { db_pool, config }
    }

    /// Create a new entity - demonstrates UUID and JSON handling
    pub async fn create_entity(&self, name: String, tags: Vec<String>) -> AppResult<Uuid> {
        let entity_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Prepare metadata as JSON
        let metadata = json!({
            "version": "1.0",
            "created_by": "example_service",
            "features": ["database_agnostic", "portable"]
        });

        // Convert to database-specific formats
        let metadata_db = json_to_db(&metadata)?;
        let tags_db = json_to_db(&tags)?;
        
        let mut conn = self.db_pool.acquire().await?;
        
        sqlx::query(
            "INSERT INTO example_entities (id, name, metadata, tags, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(uuid_to_db(entity_id))  // Convert UUID to DB format (bytes for SQLite, UUID for Postgres)
        .bind(&name)
        .bind(metadata_db)  // JSON as TEXT for SQLite, JSONB for Postgres
        .bind(tags_db)
        .bind(now)
        .bind(now)
        .execute(&mut *conn)
        .await?;
        
        info!("Created entity {} with name: {}", entity_id, name);
        Ok(entity_id)
    }

    /// Get an entity by ID - demonstrates UUID conversion from database
    pub async fn get_entity(&self, entity_id: Uuid) -> AppResult<ExampleEntity> {
        let mut conn = self.db_pool.acquire().await?;
        
        let row = sqlx::query(
            "SELECT id, name, metadata, tags, created_at, updated_at
             FROM example_entities
             WHERE id = ?"
        )
        .bind(uuid_to_db(entity_id))
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entity {} not found", entity_id)))?;
        
        // Extract data with proper type conversions
        #[cfg(feature = "sqlite")]
        let entity = {
            let id_bytes: Vec<u8> = row.try_get("id")?;
            let metadata_str: String = row.try_get("metadata")?;
            let tags_str: String = row.try_get("tags")?;
            
            ExampleEntity {
                id: uuid_from_db(id_bytes)?,
                name: row.try_get("name")?,
                metadata: json_from_db(metadata_str)?,
                tags: json_from_db(tags_str)?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            }
        };
        
        #[cfg(feature = "postgres")]
        let entity = ExampleEntity {
            id: row.try_get("id")?,  // PostgreSQL has native UUID support
            name: row.try_get("name")?,
            metadata: row.try_get("metadata")?,  // PostgreSQL has native JSONB support
            tags: json_from_db(row.try_get("tags")?)?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        
        Ok(entity)
    }

    /// Update entity metadata - demonstrates partial JSON updates
    pub async fn update_metadata(
        &self,
        entity_id: Uuid,
        new_metadata: serde_json::Value,
    ) -> AppResult<()> {
        let mut conn = self.db_pool.acquire().await?;
        
        let metadata_db = json_to_db(&new_metadata)?;
        let now = Utc::now();
        
        let result = sqlx::query(
            "UPDATE example_entities 
             SET metadata = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(metadata_db)
        .bind(now)
        .bind(uuid_to_db(entity_id))
        .execute(&mut *conn)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Entity {} not found", entity_id)));
        }
        
        Ok(())
    }

    /// Search by tag - demonstrates JSON field queries
    pub async fn find_by_tag(&self, tag: &str) -> AppResult<Vec<Uuid>> {
        let mut conn = self.db_pool.acquire().await?;
        
        // Different JSON query syntax for different databases
        #[cfg(feature = "sqlite")]
        let query = format!(
            "SELECT id FROM example_entities 
             WHERE json_extract(tags, '$') LIKE '%{}%'",
            tag
        );
        
        #[cfg(feature = "postgres")]
        let query = format!(
            "SELECT id FROM example_entities 
             WHERE tags @> '\"{}\"'::jsonb",
            tag
        );
        
        let rows = sqlx::query(&query)
            .fetch_all(&mut *conn)
            .await?;
        
        let mut ids = Vec::new();
        for row in rows {
            #[cfg(feature = "sqlite")]
            {
                let id_bytes: Vec<u8> = row.try_get("id")?;
                ids.push(uuid_from_db(id_bytes)?);
            }
            
            #[cfg(feature = "postgres")]
            {
                ids.push(row.try_get("id")?);
            }
        }
        
        Ok(ids)
    }

    /// Bulk insert - demonstrates efficient batch operations
    pub async fn bulk_insert(&self, entities: Vec<(String, Vec<String>)>) -> AppResult<Vec<Uuid>> {
        let mut tx = self.db_pool.begin().await?;
        let mut created_ids = Vec::new();
        let now = Utc::now();
        
        for (name, tags) in entities {
            let entity_id = Uuid::new_v4();
            let metadata = json!({ "bulk_insert": true });
            
            sqlx::query(
                "INSERT INTO example_entities (id, name, metadata, tags, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(uuid_to_db(entity_id))
            .bind(&name)
            .bind(json_to_db(&metadata)?)
            .bind(json_to_db(&tags)?)
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await?;
            
            created_ids.push(entity_id);
        }
        
        tx.commit().await?;
        info!("Bulk inserted {} entities", created_ids.len());
        
        Ok(created_ids)
    }

    /// Complex query with joins - demonstrates more advanced patterns
    pub async fn get_entity_with_relations(&self, entity_id: Uuid) -> AppResult<serde_json::Value> {
        let mut conn = self.db_pool.acquire().await?;
        
        // Build parameterized query based on database type
        #[cfg(feature = "sqlite")]
        let query = "
            SELECT 
                e.id, e.name, e.metadata,
                COUNT(r.id) as relation_count
            FROM example_entities e
            LEFT JOIN example_relations r ON r.entity_id = e.id
            WHERE e.id = ?
            GROUP BY e.id, e.name, e.metadata
        ";
        
        #[cfg(feature = "postgres")]
        let query = "
            SELECT 
                e.id, e.name, e.metadata,
                COUNT(r.id) as relation_count
            FROM example_entities e
            LEFT JOIN example_relations r ON r.entity_id = e.id
            WHERE e.id = $1
            GROUP BY e.id, e.name, e.metadata
        ";
        
        let row = sqlx::query(query)
            .bind(uuid_to_db(entity_id))
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Entity {} not found", entity_id)))?;
        
        // Build response with proper type handling
        #[cfg(feature = "sqlite")]
        let result = {
            let id_bytes: Vec<u8> = row.try_get("id")?;
            let metadata_str: String = row.try_get("metadata")?;
            
            json!({
                "id": uuid_from_db(id_bytes)?,
                "name": row.try_get::<String, _>("name")?,
                "metadata": json_from_db::<serde_json::Value>(metadata_str)?,
                "relation_count": row.try_get::<i32, _>("relation_count")?
            })
        };
        
        #[cfg(feature = "postgres")]
        let result = json!({
            "id": row.try_get::<Uuid, _>("id")?,
            "name": row.try_get::<String, _>("name")?,
            "metadata": row.try_get::<serde_json::Value, _>("metadata")?,
            "relation_count": row.try_get::<i64, _>("relation_count")?
        });
        
        Ok(result)
    }
}

// Migration SQL for this example service
pub const SQLITE_MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS example_entities (
    id BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    metadata TEXT NOT NULL,  -- JSON stored as TEXT
    tags TEXT NOT NULL,      -- JSON array stored as TEXT
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_example_entities_name ON example_entities(name);
CREATE INDEX idx_example_entities_created ON example_entities(created_at);
"#;

pub const POSTGRES_MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS example_entities (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    metadata JSONB NOT NULL,  -- Native JSONB type
    tags JSONB NOT NULL,       -- Native JSONB array
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_example_entities_name ON example_entities(name);
CREATE INDEX idx_example_entities_created ON example_entities(created_at);
CREATE INDEX idx_example_entities_metadata ON example_entities USING gin(metadata);
CREATE INDEX idx_example_entities_tags ON example_entities USING gin(tags);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_abstraction() {
        // Test would use the appropriate database based on feature flags
        #[cfg(feature = "sqlite")]
        println!("Testing with SQLite");
        
        #[cfg(feature = "postgres")]
        println!("Testing with PostgreSQL");
        
        // Your test implementation here
    }
}