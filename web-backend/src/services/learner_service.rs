use crate::cache::ConnectionManager;
use crate::db::DbPool;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::learner::Learner;
use abcdeez_core::{
    ResponseData,
    data::export::{self, SessionData},
    tasks::TaskResponse as CoreTaskResponse,
    BayesianLearnerModel, LearnerDataExport, LearnerMetrics, LearnerModel as CoreLearnerModel,
    Topology,
};

#[derive(Clone)]
pub struct LearnerService {
    pub db: Arc<DbPool>,
    pub cache: ConnectionManager,
}

impl LearnerService {
    pub fn new(db: Arc<DbPool>, cache: ConnectionManager) -> Self {
        Self { db, cache }
    }

    pub async fn create_learner(
        &self,
        user_id: Option<Uuid>,
        display_name: Option<String>,
    ) -> Result<Learner> {
        let learner_id = Uuid::new_v4();
        let now = Utc::now();

        // Create a default learning model
        let topology = Topology::alphabet(); // Use the correct method name
        let learning_model = CoreLearnerModel::new(learner_id.to_string(), &topology);

        let learner_bytes = learner_id.as_bytes().to_vec();
        let user_id_bytes = user_id.map(|id| id.as_bytes().to_vec());
        let learning_model_json = serde_json::to_string(&learning_model)?;

        // Insert learner into database
        let mut conn = self.db.acquire().await?;
        sqlx::query(
            "INSERT INTO learners (id, user_id, display_name, created_at, last_active, total_practice_time_seconds, metadata, learning_model)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&learner_bytes)
        .bind(&user_id_bytes)
        .bind(&display_name)
        .bind(now)
        .bind(now)
        .bind(0i64)
        .bind(serde_json::json!({}).to_string())
        .bind(&learning_model_json)
        .execute(&mut *conn)
        .await?;

        Ok(Learner::new(
            learner_id,
            user_id,
            display_name,
            learning_model,
        ))
    }

    pub async fn get_learner(&self, learner_id: Uuid) -> Result<Learner> {
        let learner_bytes = learner_id.as_bytes().to_vec();

        let mut conn = self.db.acquire().await?;
        let row = sqlx::query(
            "SELECT id, user_id, display_name, created_at, last_active, total_practice_time_seconds, metadata, learning_model
             FROM learners WHERE id = ?"
        )
        .bind(&learner_bytes)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound("Learner not found".to_string()),
            _ => AppError::DatabaseError(e),
        })?;

        // Default topology if parsing fails
        let topology = Topology::alphabet();
        let model_json: Option<String> = row.get::<Option<String>, _>("learning_model");
        let learning_model = if let Some(json_str) = model_json {
            serde_json::from_str(&json_str)
                .unwrap_or_else(|_| CoreLearnerModel::new(learner_id.to_string(), &topology))
        } else {
            CoreLearnerModel::new(learner_id.to_string(), &topology)
        };

        Ok(Learner {
            id: learner_id,
            user_id: {
                let user_id_opt: Option<Vec<u8>> = row.get::<Option<Vec<u8>>, _>("user_id");
                user_id_opt.and_then(|bytes| {
                    let array: [u8; 16] = bytes.try_into().ok()?;
                    Some(Uuid::from_bytes(array))
                })
            },
            display_name: row.get::<Option<String>, _>("display_name"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            last_active: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_active"),
            total_practice_time_seconds: row.get::<i64, _>("total_practice_time_seconds"),
            metadata: row
                .get::<Option<String>, _>("metadata")
                .and_then(|s| serde_json::from_str(&s).ok()),
            learning_model,
        })
    }

    pub async fn update_learner_response(
        &mut self,
        learner_id: Uuid,
        response: &CoreTaskResponse,
        topology: &Topology,
    ) -> Result<()> {
        // Get the current learner
        let mut learner = self.get_learner(learner_id).await?;

        // Create/update Bayesian model for more sophisticated learning
        let mut bayesian_model = self
            .get_or_create_bayesian_model(learner_id, topology)
            .await?;

        // Convert response to ResponseData format for Bayesian update
        let response_data = ResponseData {
            task: response.task.clone(),
            correct: response.correct,
            response_time: response.response_time_ms as f64,
        };

        // Update Bayesian model with response (this is the key enhancement)
        bayesian_model.update_with_response(response_data);

        // Save the updated Bayesian model
        self.save_bayesian_model(learner_id, &bayesian_model)
            .await?;

        // Also update the traditional learning model
        learner
            .learning_model
            .update_memory_strength(&response.task.correct_answer, response.correct);

        // Update operation proficiency with more sophisticated logic
        learner
            .learning_model
            .update_operation_proficiency(&response.task.operation, response.correct);

        // Save updated learner with enhanced total practice time calculation
        let learner_bytes = learner_id.as_bytes().to_vec();
        let learning_model_json = serde_json::to_string(&learner.learning_model)?;
        let now = Utc::now();

        // Add response time to total practice time (convert from ms to seconds)
        let additional_practice_time = response.response_time_ms as u64 / 1000;
        learner.total_practice_time_seconds += additional_practice_time as i64;

        let mut conn = self.db.acquire().await?;
        sqlx::query(
            "UPDATE learners SET learning_model = ?, last_active = ?, total_practice_time_seconds = ? WHERE id = ?"
        )
        .bind(&learning_model_json)
        .bind(now)
        .bind(learner.total_practice_time_seconds)
        .bind(&learner_bytes)
        .execute(&mut *conn)
        .await?;

        // Cache both models
        let cache_key = format!("learner_model:{}", learner_id);
        self.cache_learner_model(&cache_key, &learner.learning_model)
            .await?;

        let bayesian_cache_key = format!("bayesian_model:{}", learner_id);
        self.cache_bayesian_model(&bayesian_cache_key, &bayesian_model)
            .await?;

        Ok(())
    }

    pub async fn get_learner_model(&self, learner_id: Uuid) -> Result<CoreLearnerModel> {
        let learner = self.get_learner(learner_id).await?;
        Ok(learner.learning_model)
    }

    /// Get the Bayesian model for a learner, creating one if it doesn't exist
    pub async fn get_or_create_bayesian_model(
        &self,
        learner_id: Uuid,
        topology: &Topology,
    ) -> Result<BayesianLearnerModel> {
        // Try to load from cache first
        let cache_key = format!("bayesian_model:{}", learner_id);
        if let Ok(cached_model) = self.get_cached_bayesian_model(&cache_key).await {
            return Ok(cached_model);
        }

        // Try to load from database
        if let Ok(model) = self.load_bayesian_model_from_db(learner_id).await {
            // Cache it for future use
            self.cache_bayesian_model(&cache_key, &model).await?;
            return Ok(model);
        }

        // Create new Bayesian model if none exists
        let model = BayesianLearnerModel::new(topology);

        // Save to database and cache
        self.save_bayesian_model(learner_id, &model).await?;
        self.cache_bayesian_model(&cache_key, &model).await?;

        Ok(model)
    }

    /// Get Bayesian model for Enhanced Information Gain calculations
    pub async fn get_bayesian_model(
        &self,
        learner_id: Uuid,
        topology: &Topology,
    ) -> Result<BayesianLearnerModel> {
        self.get_or_create_bayesian_model(learner_id, topology)
            .await
    }

    /// Load Bayesian model from database
    async fn load_bayesian_model_from_db(&self, learner_id: Uuid) -> Result<BayesianLearnerModel> {
        let learner_id_bytes = learner_id.as_bytes();

        let mut conn = self.db.acquire().await?;
        let result = sqlx::query(
            "SELECT model_data FROM bayesian_models WHERE learner_id = ? ORDER BY created_at DESC LIMIT 1"
        )
        .bind(&learner_id_bytes[..])
        .fetch_optional(&mut *conn)
        .await?;

        if let Some(row) = result {
            let model_data: String = row.try_get("model_data")?;
            let model: BayesianLearnerModel = serde_json::from_str(&model_data)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize Bayesian model: {}", e))?;
            Ok(model)
        } else {
            Err(anyhow::anyhow!("No Bayesian model found for learner"))
        }
    }

    /// Save Bayesian model to database
    async fn save_bayesian_model(
        &self,
        learner_id: Uuid,
        model: &BayesianLearnerModel,
    ) -> Result<()> {
        let learner_id_bytes = learner_id.as_bytes();
        let model_data = serde_json::to_string(model)
            .map_err(|e| anyhow::anyhow!("Failed to serialize Bayesian model: {}", e))?;

        let mut conn = self.db.acquire().await?;

        // Insert new model (keeping history for analysis)
        let model_id = uuid::Uuid::new_v4();
        let model_id_bytes = model_id.as_bytes();

        sqlx::query(
            "INSERT INTO bayesian_models (id, learner_id, model_data, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&model_id_bytes[..])
        .bind(&learner_id_bytes[..])
        .bind(&model_data)
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .execute(&mut *conn)
        .await?;

        tracing::info!("Saved Bayesian model for learner {}", learner_id);
        Ok(())
    }

    /// Cache and persist Bayesian model
    async fn cache_bayesian_model(
        &self,
        cache_key: &str,
        model: &BayesianLearnerModel,
    ) -> Result<()> {
        // Extract learner ID from cache key
        let learner_id_str = cache_key
            .strip_prefix("bayesian_model:")
            .ok_or_else(|| anyhow::anyhow!("Invalid cache key format"))?;
        let learner_id = uuid::Uuid::parse_str(learner_id_str)?;

        // Save to database for persistence
        self.save_bayesian_model(learner_id, model).await?;
        // The model will be regenerated as needed
        Ok(())
    }

    /// Get cached Bayesian model from database
    async fn get_cached_bayesian_model(&self, cache_key: &str) -> Result<BayesianLearnerModel> {
        // Extract learner ID from cache key
        let learner_id_str = cache_key
            .strip_prefix("bayesian_model:")
            .ok_or_else(|| anyhow::anyhow!("Invalid cache key format"))?;
        let learner_id = uuid::Uuid::parse_str(learner_id_str)?;

        // Try to load from database
        self.load_bayesian_model_from_db(learner_id).await
    }

    async fn cache_learner_model(&self, cache_key: &str, model: &CoreLearnerModel) -> Result<()> {
        let model_json = serde_json::to_string(model)?;
        let mut conn = self.cache.clone();

        crate::cache::cmd("SETEX")
            .arg(cache_key)
            .arg(3600) // 1 hour TTL
            .arg(model_json)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| anyhow::anyhow!("Cache error: {}", e))?;

        Ok(())
    }

    pub async fn delete_learner(&self, learner_id: Uuid) -> Result<()> {
        let learner_bytes = learner_id.as_bytes().to_vec();

        let mut conn = self.db.acquire().await?;
        sqlx::query("DELETE FROM learners WHERE id = ?")
            .bind(&learner_bytes)
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn get_learner_sessions(&self, learner_id: Uuid) -> Result<serde_json::Value> {
        let learner_bytes = learner_id.as_bytes().to_vec();

        let mut conn = self.db.acquire().await?;
        let sessions = sqlx::query(
            "SELECT id, topology_type, topology_data, start_time, end_time, status, summary
             FROM sessions WHERE learner_id = ? ORDER BY start_time",
        )
        .bind(&learner_bytes)
        .fetch_all(&mut *conn)
        .await?;

        // Convert sessions to JSON
        let session_data: Vec<serde_json::Value> = sessions
            .into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row.get::<Vec<u8>, _>("id");
                let session_id_str =
                    Uuid::from_bytes(bytes.try_into().unwrap_or_default()).to_string();

                serde_json::json!({
                    "id": session_id_str,
                    "topology_type": row.get::<String, _>("topology_type"),
                    "start_time": row.get::<DateTime<Utc>, _>("start_time"),
                    "end_time": row.get::<Option<DateTime<Utc>>, _>("end_time"),
                    "status": row.get::<String, _>("status"),
                })
            })
            .collect();

        let learner = self.get_learner(learner_id).await?;
        let metrics = LearnerMetrics::from_model(&learner.learning_model);

        Ok(serde_json::json!({
            "learner_id": learner_id,
            "sessions": session_data,
            "model_metrics": metrics,
        }))
    }

    pub async fn save_model_snapshot(&self, learner_id: Uuid, learner: &Learner) -> Result<()> {
        let snapshot_id = Uuid::new_v4();
        let snapshot_id_bytes = snapshot_id.as_bytes().to_vec();
        let learner_bytes = learner_id.as_bytes().to_vec();
        let now = Utc::now();
        let parameters = serde_json::to_string(&learner.learning_model)?;
        let metrics = serde_json::to_string(&LearnerMetrics::from_model(&learner.learning_model))?;

        let mut conn = self.db.acquire().await?;
        sqlx::query(
            "INSERT INTO model_snapshots (id, learner_id, timestamp, parameters, metrics)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&snapshot_id_bytes)
        .bind(&learner_bytes)
        .bind(now)
        .bind(&parameters)
        .bind(&metrics)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Export complete learner data using core LearnerDataExport functionality
    pub async fn export_learner_data(
        &self,
        learner_id: Uuid,
        experiment_id: Option<String>,
    ) -> Result<LearnerDataExport> {
        // Get the learner and their model
        let learner = self.get_learner(learner_id).await?;

        // Get sessions data from database
        let learner_bytes = learner_id.as_bytes().to_vec();
        let mut conn = self.db.acquire().await?;

        let session_rows = sqlx::query(
            "SELECT id, topology_type, topology_data, start_time, end_time, status, summary
             FROM sessions WHERE learner_id = ? ORDER BY start_time",
        )
        .bind(&learner_bytes)
        .fetch_all(&mut *conn)
        .await?;

        // Convert database sessions to core SessionData format
        let mut sessions = Vec::new();
        for row in session_rows {
            let session_id_bytes: Vec<u8> = row.get("id");
            let session_id =
                Uuid::from_bytes(session_id_bytes.clone().try_into().unwrap_or_default())
                    .to_string();

            // Get responses for this session
            let responses = self.get_session_responses(&session_id_bytes).await?;

            let topology_type: String = row.get("topology_type");
            let start_time: DateTime<Utc> = row.get("start_time");
            let end_time: Option<DateTime<Utc>> = row.get("end_time");

            let summary = self.calculate_session_summary(&responses).await?;
            sessions.push(SessionData {
                session_id,
                start_time,
                end_time,
                topology_type: topology_type.clone(),
                responses,
                summary,
            });
        }

        // Use core functionality to create the complete export
        // Note: We'll need to adapt this since we don't have TaskSession directly
        // For now, create a simplified version that matches our database structure
        let export = self
            .create_export_from_data(
                learner.learning_model,
                learner_id.to_string(),
                sessions,
                experiment_id,
            )
            .await?;

        Ok(export)
    }

    /// Get responses for a session
    async fn get_session_responses(
        &self,
        session_id_bytes: &[u8],
    ) -> Result<Vec<CoreTaskResponse>> {
        let mut conn = self.db.acquire().await?;
        let response_rows = sqlx::query(
            "SELECT task_type, task_data, correct_answer, user_answer, correct, response_time_ms, timestamp
             FROM responses WHERE session_id = ? ORDER BY timestamp"
        )
        .bind(&session_id_bytes[..])
        .fetch_all(&mut *conn)
        .await?;

        let mut responses = Vec::new();
        for row in response_rows {
            let task_type: String = row.get("task_type");
            let task_data: String = row.get("task_data");
            let correct_answer: String = row.get("correct_answer");
            let user_answer: String = row.get("user_answer");
            let correct: bool = row.get("correct");
            let response_time_ms: i32 = row.get("response_time_ms");
            let timestamp: DateTime<Utc> = row.get("timestamp");

            // Reconstruct task from stored data
            let task = self
                .reconstruct_task_from_data(&task_type, &task_data, &correct_answer)
                .await?;

            responses.push(CoreTaskResponse {
                task,
                user_answer,
                correct,
                response_time_ms: response_time_ms as u128,
                timestamp,
            });
        }

        Ok(responses)
    }

    /// Calculate session summary from responses
    async fn calculate_session_summary(
        &self,
        responses: &[CoreTaskResponse],
    ) -> Result<export::SessionSummary> {
        let total_tasks = responses.len();
        let correct_count = responses.iter().filter(|r| r.correct).count();
        let accuracy = if total_tasks > 0 {
            correct_count as f64 / total_tasks as f64
        } else {
            0.0
        };

        let rts: Vec<f64> = responses
            .iter()
            .map(|r| r.response_time_ms as f64)
            .collect();

        let mean_rt_ms = if !rts.is_empty() {
            rts.iter().sum::<f64>() / rts.len() as f64
        } else {
            0.0
        };

        let median_rt_ms = if !rts.is_empty() {
            let mut sorted_rts = rts.clone();
            sorted_rts.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted_rts[sorted_rts.len() / 2]
        } else {
            0.0
        };

        Ok(export::SessionSummary {
            total_tasks,
            correct_count,
            accuracy,
            mean_rt_ms,
            median_rt_ms,
            strategy_detected: None, // Could be enhanced with strategy detection
        })
    }

    /// Reconstruct task from stored database data
    async fn reconstruct_task_from_data(
        &self,
        task_type: &str,
        task_data: &str,
        correct_answer: &str,
    ) -> Result<abcdeez_core::Task> {
        // Parse the task data JSON to reconstruct the original task
        let task_json: serde_json::Value = serde_json::from_str(task_data)?;

        let task_type_enum = match task_type {
            "Successor" => {
                let item = task_json["item"].as_str().unwrap_or("A").to_string();
                abcdeez_core::TaskType::Successor { item }
            }
            "Predecessor" => {
                let item = task_json["item"].as_str().unwrap_or("A").to_string();
                abcdeez_core::TaskType::Predecessor { item }
            }
            "PairwiseOrder" => {
                let a = task_json["a"].as_str().unwrap_or("A").to_string();
                let b = task_json["b"].as_str().unwrap_or("B").to_string();
                abcdeez_core::TaskType::PairwiseOrder { a, b }
            }
            "KJump" => {
                let start = task_json["start"].as_str().unwrap_or("A").to_string();
                let k = task_json["k"].as_i64().unwrap_or(1) as i32;
                abcdeez_core::TaskType::KJump { start, k }
            }
            "Segment" => {
                let start = task_json["start"].as_str().unwrap_or("A").to_string();
                let count = task_json["count"].as_u64().unwrap_or(3) as usize;
                let reverse = task_json["reverse"].as_bool().unwrap_or(false);
                abcdeez_core::TaskType::Segment {
                    start,
                    count,
                    reverse,
                }
            }
            _ => abcdeez_core::TaskType::Successor {
                item: "A".to_string(),
            },
        };

        let difficulty = task_json["difficulty"].as_f64().unwrap_or(0.5);
        let prompt = task_json["prompt"].as_str().unwrap_or("").to_string();
        let options = task_json["options"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        Ok(abcdeez_core::Task {
            task_type: task_type_enum.clone(),
            prompt,
            correct_answer: correct_answer.to_string(),
            options,
            difficulty,
            operation: match &task_type_enum {
                abcdeez_core::TaskType::Successor { .. } => {
                    abcdeez_core::OperationType::Successor
                }
                abcdeez_core::TaskType::Predecessor { .. } => {
                    abcdeez_core::OperationType::Predecessor
                }
                abcdeez_core::TaskType::PairwiseOrder { .. } => {
                    abcdeez_core::OperationType::PairwiseOrder
                }
                abcdeez_core::TaskType::KJump { k, .. } => {
                    abcdeez_core::OperationType::KJump(*k)
                }
                abcdeez_core::TaskType::Segment { count, reverse, .. } => {
                    abcdeez_core::OperationType::Segment(*count, *reverse)
                }
                _ => abcdeez_core::OperationType::Successor,
            },
        })
    }

    /// Create export from our database data format
    async fn create_export_from_data(
        &self,
        model: CoreLearnerModel,
        learner_id: String,
        sessions: Vec<SessionData>,
        experiment_id: Option<String>,
    ) -> Result<LearnerDataExport> {
        let export_timestamp = Utc::now();

        // Calculate performance trajectories
        let mut performance_trajectories = Vec::new();
        let mut running_correct = 0;
        let mut running_total = 0;

        for session in &sessions {
            for response in &session.responses {
                running_total += 1;
                if response.correct {
                    running_correct += 1;
                }

                performance_trajectories.push(export::PerformancePoint {
                    trial_number: running_total,
                    timestamp: response.timestamp,
                    accuracy: running_correct as f64 / running_total as f64,
                    mean_rt: response.response_time_ms as f64,
                    task_type: format!("{:?}", response.task.task_type),
                    difficulty: response.task.difficulty,
                });
            }
        }

        // Analyze errors
        let error_patterns = self.analyze_errors(&sessions).await?;

        // Create model snapshot
        let model_snapshot = export::ModelSnapshot {
            timestamp: export_timestamp,
            node_embeddings: model
                .node_embeddings
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        export::NodeEmbeddingExport {
                            position: v.position,
                            uncertainty: v.uncertainty,
                        },
                    )
                })
                .collect(),
            operation_proficiencies: model
                .operation_proficiencies
                .iter()
                .map(|(k, v)| (k.clone(), 1.0 / (1.0 + (-v.theta).exp()))) // sigmoid
                .collect(),
            memory_strengths: model
                .memory_strengths
                .iter()
                .map(|(k, v)| (k.clone(), v.strength))
                .collect(),
            chunk_boundaries: model
                .chunk_boundaries
                .iter()
                .map(|b| export::ChunkBoundaryExport {
                    position: b.position,
                    strength: b.strength,
                })
                .collect(),
            total_practice_time_seconds: model.total_practice_time.as_secs(),
        };

        let metadata = export::ExportMetadata {
            export_version: "1.0.0".to_string(),
            software_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            experiment_id,
            notes: None,
        };

        Ok(LearnerDataExport {
            learner_id,
            export_timestamp,
            sessions,
            performance_trajectories,
            error_patterns,
            model_parameters: model_snapshot,
            metadata,
        })
    }

    /// Analyze errors across sessions  
    async fn analyze_errors(
        &self,
        sessions: &[SessionData],
    ) -> Result<export::ErrorAnalysis> {
        let mut total_errors = 0;
        let mut total_tasks = 0;
        let mut confusion_counts: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        let mut error_by_task_type: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();
        let mut error_by_difficulty: std::collections::HashMap<String, Vec<bool>> =
            std::collections::HashMap::new();

        for session in sessions {
            for response in &session.responses {
                total_tasks += 1;

                let task_type = format!("{:?}", response.task.task_type);
                let difficulty_bucket = format!("{:.1}", response.task.difficulty);

                error_by_task_type
                    .entry(task_type.clone())
                    .or_insert((0, 0))
                    .1 += 1;

                error_by_difficulty
                    .entry(difficulty_bucket)
                    .or_insert_with(Vec::new)
                    .push(response.correct);

                if !response.correct {
                    total_errors += 1;

                    error_by_task_type.entry(task_type).or_insert((0, 0)).0 += 1;

                    let confusion = (
                        response.task.correct_answer.clone(),
                        response.user_answer.clone(),
                    );
                    *confusion_counts.entry(confusion).or_insert(0) += 1;
                }
            }
        }

        let error_rate = if total_tasks > 0 {
            total_errors as f64 / total_tasks as f64
        } else {
            0.0
        };

        let mut common_confusions: Vec<(String, String, usize)> = confusion_counts
            .into_iter()
            .map(|((expected, actual), count)| (expected, actual, count))
            .collect();
        common_confusions.sort_by_key(|c| std::cmp::Reverse(c.2));
        common_confusions.truncate(10);

        let error_by_task_type_rates = error_by_task_type
            .into_iter()
            .map(|(k, (errors, total))| {
                (
                    k,
                    if total > 0 {
                        errors as f64 / total as f64
                    } else {
                        0.0
                    },
                )
            })
            .collect();

        let error_by_difficulty_rates: Vec<(f64, f64)> = error_by_difficulty
            .into_iter()
            .map(|(bucket, results)| {
                let errors = results.iter().filter(|&&c| !c).count();
                let rate = if !results.is_empty() {
                    errors as f64 / results.len() as f64
                } else {
                    0.0
                };
                (bucket.parse::<f64>().unwrap_or(0.0), rate)
            })
            .collect();

        Ok(export::ErrorAnalysis {
            total_errors,
            error_rate,
            common_confusions,
            error_by_task_type: error_by_task_type_rates,
            error_by_difficulty: error_by_difficulty_rates,
        })
    }
}
