use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use serde_json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use graph_learning_core::{LearnerModel as CoreLearnerModel, Topology, tasks::TaskResponse as CoreTaskResponse};
use crate::models::{learner::Learner, user::User};
use crate::error::AppError;

#[derive(Clone)]
pub struct LearnerService {
    pub db: Arc<PgPool>,
    pub cache: Arc<ConnectionManager>,
}

impl LearnerService {
    pub fn new(db: Arc<PgPool>, cache: Arc<ConnectionManager>) -> Self {
        Self { db, cache }
    }

    pub async fn create_learner(&self, user_id: Option<Uuid>, display_name: Option<String>, topology: &Topology) -> Result<Learner> {
        let learner_id = Uuid::new_v4();
        let now = Utc::now();

        // Create core learner model
        let core_model = CoreLearnerModel::new(learner_id.to_string(), topology);

        // Insert into database
        let learner_bytes = learner_id.as_bytes();
        let user_id_bytes = user_id.as_ref().map(|id| id.as_bytes());

        sqlx::query!(
            "INSERT INTO learners (id, user_id, display_name, created_at, last_active, total_practice_time_seconds, metadata) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            learner_bytes,
            user_id_bytes,
            display_name,
            now,
            now,
            0i64,
            serde_json::json!({}).to_string()
        )
        .execute(&**self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Cache the learner model
        self.cache_learner_model(&learner_id, &core_model).await?;

        Ok(Learner {
            id: learner_id,
            user_id,
            display_name,
            created_at: now,
            last_active: Some(now),
            total_practice_time_seconds: 0,
            core_model,
            metadata: Some(serde_json::json!({})),
        })
    }

    pub async fn get_learner(&self, learner_id: Uuid) -> Result<Learner> {
        // Try cache first
        if let Ok(cached) = self.get_cached_learner(&learner_id).await {
            return Ok(cached);
        }

        // Fallback to database
        let learner_bytes = learner_id.as_bytes();
        let row = sqlx::query!(
            "SELECT id, user_id, display_name, created_at, last_active, total_practice_time_seconds, metadata 
             FROM learners WHERE id = $1",
            learner_bytes
        )
        .fetch_one(&**self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound("Learner not found".to_string()),
            _ => AppError::DatabaseError(e.to_string()),
        })?;

        let user_id = row.user_id.map(|bytes| Uuid::from_bytes(
            bytes.try_into().unwrap_or_default()
        ));

        // For now, create a default core model - in production you'd load from model_snapshots
        let topology = Topology::alphabet_topology(); // Default topology
        let core_model = CoreLearnerModel::new(learner_id.to_string(), &topology);

        let learner = Learner {
            id: learner_id,
            user_id,
            display_name: row.display_name,
            created_at: row.created_at,
            last_active: row.last_active,
            total_practice_time_seconds: row.total_practice_time_seconds,
            core_model,
            metadata: row.metadata.and_then(|s| serde_json::from_str(&s).ok()),
        };

        // Cache for next time
        self.cache_learner(&learner).await?;

        Ok(learner)
    }

    pub async fn update_learner_model(&self, learner_id: Uuid, response: &CoreTaskResponse) -> Result<()> {
        // Get current learner
        let mut learner = self.get_learner(learner_id).await?;

        // Update the core model based on the response
        // This is where the actual learning algorithm updates would happen
        learner.core_model.update_memory_strength(&response.task_id, response.correct);
        
        if let Some(operation) = &response.operation_type {
            learner.core_model.update_operation_proficiency(operation, response.correct);
        }

        // Update practice time
        learner.total_practice_time_seconds += (response.response_time_ms / 1000) as i64;

        // Save updated model to cache
        self.cache_learner_model(&learner_id, &learner.core_model).await?;

        // Update database
        let learner_bytes = learner_id.as_bytes();
        sqlx::query!(
            "UPDATE learners SET last_active = $1, total_practice_time_seconds = $2 WHERE id = $3",
            Utc::now(),
            learner.total_practice_time_seconds,
            learner_bytes
        )
        .execute(&**self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Create model snapshot for analysis
        self.create_model_snapshot(&learner).await?;

        Ok(())
    }

    pub async fn get_learner_model(&self, learner_id: Uuid) -> Result<CoreLearnerModel> {
        let learner = self.get_learner(learner_id).await?;
        Ok(learner.core_model)
    }

    pub async fn delete_learner(&self, learner_id: Uuid) -> Result<()> {
        let learner_bytes = learner_id.as_bytes();
        
        // Delete from cache
        let cache_key = format!("learner:{}", learner_id);
        let mut conn = self.cache.clone();
        redis::cmd("DEL")
            .arg(&cache_key)
            .query_async(&mut conn)
            .await
            .ok(); // Ignore cache errors

        // Delete from database (cascading deletes will handle related records)
        sqlx::query!("DELETE FROM learners WHERE id = $1", learner_bytes)
            .execute(&**self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn export_learner_data(&self, learner_id: Uuid) -> Result<serde_json::Value> {
        let learner = self.get_learner(learner_id).await?;
        
        // Get all sessions for this learner
        let learner_bytes = learner_id.as_bytes();
        let sessions = sqlx::query!(
            "SELECT id, topology_type, topology_data, start_time, end_time, status, summary 
             FROM sessions WHERE learner_id = $1 ORDER BY start_time",
            learner_bytes
        )
        .fetch_all(&**self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get all responses for these sessions
        let session_ids: Vec<Vec<u8>> = sessions.iter()
            .map(|s| s.id.clone())
            .collect();

        let responses = if !session_ids.is_empty() {
            sqlx::query!(
                "SELECT session_id, sequence_number, task_type, task_data, user_answer, correct, response_time_ms, hint_level, timestamp
                 FROM responses WHERE session_id = ANY($1) ORDER BY session_id, sequence_number",
                &session_ids as &[Vec<u8>]
            )
            .fetch_all(&**self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        } else {
            vec![]
        };

        // Export data using the core library's export functionality
        let export_data = serde_json::json!({
            "learner_id": learner_id,
            "display_name": learner.display_name,
            "created_at": learner.created_at,
            "total_practice_time_seconds": learner.total_practice_time_seconds,
            "sessions": sessions.len(),
            "total_responses": responses.len(),
            "model_metrics": graph_learning_core::LearnerMetrics::from_model(&learner.core_model),
            "export_timestamp": Utc::now()
        });

        Ok(export_data)
    }

    // Private helper methods
    async fn cache_learner(&self, learner: &Learner) -> Result<()> {
        let cache_key = format!("learner:{}", learner.id);
        let learner_data = serde_json::to_string(&serde_json::json!({
            "id": learner.id,
            "user_id": learner.user_id,
            "display_name": learner.display_name,
            "created_at": learner.created_at,
            "last_active": learner.last_active,
            "total_practice_time_seconds": learner.total_practice_time_seconds,
            "metadata": learner.metadata
        }))?;

        let mut conn = self.cache.clone();
        redis::cmd("SETEX")
            .arg(&cache_key)
            .arg(3600) // 1 hour TTL
            .arg(learner_data)
            .query_async(&mut conn)
            .await
            .ok(); // Ignore cache errors

        Ok(())
    }

    async fn cache_learner_model(&self, learner_id: &Uuid, model: &CoreLearnerModel) -> Result<()> {
        let cache_key = format!("learner_model:{}", learner_id);
        let model_data = serde_json::to_string(model)?;

        let mut conn = self.cache.clone();
        redis::cmd("SETEX")
            .arg(&cache_key)
            .arg(3600) // 1 hour TTL
            .arg(model_data)
            .query_async(&mut conn)
            .await
            .ok(); // Ignore cache errors

        Ok(())
    }

    async fn get_cached_learner(&self, learner_id: &Uuid) -> Result<Learner> {
        let cache_key = format!("learner:{}", learner_id);
        let mut conn = self.cache.clone();
        
        let cached_data: String = redis::cmd("GET")
            .arg(&cache_key)
            .query_async(&mut conn)
            .await
            .map_err(|_| AppError::NotFound("Not in cache".to_string()))?;

        let data: serde_json::Value = serde_json::from_str(&cached_data)?;
        
        // Get the model from cache too
        let model_cache_key = format!("learner_model:{}", learner_id);
        let model_data: String = redis::cmd("GET")
            .arg(&model_cache_key)
            .query_async(&mut conn)
            .await
            .map_err(|_| AppError::NotFound("Model not in cache".to_string()))?;

        let core_model: CoreLearnerModel = serde_json::from_str(&model_data)?;

        Ok(Learner {
            id: *learner_id,
            user_id: data["user_id"].as_str().map(|s| s.parse().ok()).flatten(),
            display_name: data["display_name"].as_str().map(|s| s.to_string()),
            created_at: serde_json::from_value(data["created_at"].clone())?,
            last_active: data["last_active"].as_str()
                .map(|s| s.parse().ok())
                .flatten(),
            total_practice_time_seconds: data["total_practice_time_seconds"].as_i64().unwrap_or(0),
            core_model,
            metadata: Some(data["metadata"].clone()),
        })
    }

    async fn create_model_snapshot(&self, learner: &Learner) -> Result<()> {
        let snapshot_id = Uuid::new_v4();
        let learner_id_bytes = learner.id.as_bytes();
        let snapshot_id_bytes = snapshot_id.as_bytes();
        
        let parameters = serde_json::to_string(&learner.core_model)?;
        let metrics = serde_json::to_string(&graph_learning_core::LearnerMetrics::from_model(&learner.core_model))?;

        sqlx::query!(
            "INSERT INTO model_snapshots (id, learner_id, timestamp, parameters, metrics) 
             VALUES ($1, $2, $3, $4, $5)",
            snapshot_id_bytes,
            learner_id_bytes,
            Utc::now(),
            parameters,
            metrics
        )
        .execute(&**self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}