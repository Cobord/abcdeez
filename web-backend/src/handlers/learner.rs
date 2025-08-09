use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::learner::{CreateLearnerRequest, Learner, LearnerStats, UpdateLearnerRequest},
    models::session::Session,
    services::{audit::AuditService, LearnerService},
    state::AppState,
};
use graph_learning_core::Topology;

pub async fn create(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CreateLearnerRequest>,
) -> AppResult<(StatusCode, Json<Learner>)> {
    // Default to alphabet topology if not specified
    let _topology = Topology::alphabet();

    // Use the authenticated user's ID if no user_id provided
    let user_id = req.user_id.or(Some(claims.sub));

    // Create learner using the service
    let learner_service =
        LearnerService::new(state.db_pool.clone().into(), state.redis_conn.clone());

    let learner = learner_service
        .create_learner(user_id, req.display_name.clone())
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // Convert to the API model format
    let api_learner = Learner {
        id: learner.id,
        user_id: learner.user_id,
        display_name: learner.display_name,
        created_at: learner.created_at,
        last_active: learner.last_active,
        total_practice_time_seconds: learner.total_practice_time_seconds,
        metadata: learner.metadata,
        learning_model: learner.learning_model,
    };

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "create".to_string(),
        "learner".to_string(),
        learner.id.to_string(),
        Some(serde_json::json!({
            "display_name": req.display_name
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok((StatusCode::CREATED, Json(api_learner)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Learner>> {
    let learner_service =
        LearnerService::new(state.db_pool.clone().into(), state.redis_conn.clone());

    let learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::NotFound("Learner not found".to_string()))?;

    // Check if the user has permission to access this learner
    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Convert to the API model format
    let api_learner = Learner {
        id: learner.id,
        user_id: learner.user_id,
        display_name: learner.display_name,
        created_at: learner.created_at,
        last_active: learner.last_active,
        total_practice_time_seconds: learner.total_practice_time_seconds,
        metadata: learner.metadata,
        learning_model: learner.learning_model,
    };

    Ok(Json(api_learner))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLearnerRequest>,
) -> AppResult<Json<Learner>> {
    // First get the existing learner to check permissions
    let learner_service =
        LearnerService::new(state.db_pool.clone().into(), state.redis_conn.clone());

    let learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::NotFound("Learner not found".to_string()))?;

    // Check permissions
    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Update in database
    let learner_bytes = id.as_bytes();
    let now = chrono::Utc::now();

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    sqlx::query(
        "UPDATE learners SET display_name = COALESCE(?, display_name),
                           metadata = COALESCE(?, metadata),
                           last_active = ?
         WHERE id = ?",
    )
    .bind(req.display_name.clone())
    .bind(req.metadata.clone().map(|m| m.to_string()))
    .bind(now)
    .bind(&learner_bytes[..])
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Get updated learner
    let updated_learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::InternalServerError)?;

    let api_learner = Learner {
        id: updated_learner.id,
        user_id: updated_learner.user_id,
        display_name: updated_learner.display_name,
        created_at: updated_learner.created_at,
        last_active: updated_learner.last_active,
        total_practice_time_seconds: updated_learner.total_practice_time_seconds,
        metadata: updated_learner.metadata,
        learning_model: updated_learner.learning_model,
    };

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "update".to_string(),
        "learner".to_string(),
        id.to_string(),
        Some(serde_json::json!({
            "display_name": req.display_name,
            "metadata": req.metadata
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(api_learner))
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<LearnerStats>> {
    // Check permissions first
    let learner_service =
        LearnerService::new(state.db_pool.clone().into(), state.redis_conn.clone());

    let learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::NotFound("Learner not found".to_string()))?;

    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Get statistics from database
    let learner_bytes = id.as_bytes();

    // Get session count
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let session_stats =
        sqlx::query("SELECT COUNT(*) as total_sessions FROM sessions WHERE learner_id = ?")
            .bind(&learner_bytes[..])
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| AppError::DatabaseError(e))?;

    // Get response statistics
    let response_stats = sqlx::query(
        "SELECT
            COUNT(*) as total_tasks,
            AVG(CASE WHEN correct THEN 1.0 ELSE 0.0 END) as accuracy
         FROM responses r
         JOIN sessions s ON r.session_id = s.id
         WHERE s.learner_id = ?",
    )
    .bind(&learner_bytes[..])
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Get learning curve (daily accuracy over last 30 days)
    let learning_curve_data = sqlx::query(
        "SELECT
            DATE(r.timestamp) as date,
            AVG(CASE WHEN r.correct THEN 1.0 ELSE 0.0 END) as daily_accuracy
         FROM responses r
         JOIN sessions s ON r.session_id = s.id
         WHERE s.learner_id = ?
           AND r.timestamp > ?
         GROUP BY DATE(r.timestamp)
         ORDER BY date",
    )
    .bind(&learner_bytes[..])
    .bind(chrono::Utc::now() - chrono::Duration::days(30))
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let learning_curve = learning_curve_data
        .into_iter()
        .map(|row| {
            let date_str: String = row.get("date");
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive())
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_utc();
            let accuracy: Option<f64> = row.get("daily_accuracy");
            (date, accuracy.unwrap_or(0.0))
        })
        .collect();

    let stats = LearnerStats {
        total_sessions: session_stats.get::<i64, _>("total_sessions"),
        total_tasks_completed: response_stats.get::<i64, _>("total_tasks"),
        overall_accuracy: response_stats
            .get::<Option<f64>, _>("accuracy")
            .unwrap_or(0.0),
        total_practice_time_seconds: learner.total_practice_time_seconds,
        last_active: learner.last_active,
        preferred_difficulty: calculate_preferred_difficulty(&response_stats),
        learning_curve,
    };

    Ok(Json(stats))
}

pub async fn export(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Check permissions first
    let learner_service =
        LearnerService::new(state.db_pool.clone().into(), state.redis_conn.clone());

    let learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::NotFound("Learner not found".to_string()))?;

    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Export complete learner data using the service
    let export_data = match learner_service
        .export_learner_data(id, None) // No experiment_id for now
        .await
    {
        Ok(export) => {
            // Convert to JSON for the API response
            serde_json::to_value(&export).map_err(|_| AppError::InternalServerError)?
        }
        Err(_) => {
            // Fallback to basic export structure if full export fails
            serde_json::json!({
                "learner_id": id,
                "display_name": learner.display_name,
                "created_at": learner.created_at,
                "total_practice_time_seconds": learner.total_practice_time_seconds,
                "export_timestamp": chrono::Utc::now(),
                "note": "Full export functionality failed, providing basic data"
            })
        }
    };

    // Log audit event for GDPR compliance
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "export".to_string(),
        "learner".to_string(),
        id.to_string(),
        None,
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(export_data))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Check permissions first
    let learner_service =
        LearnerService::new(state.db_pool.clone().into(), state.redis_conn.clone());

    let learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::NotFound("Learner not found".to_string()))?;

    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Delete using the service (GDPR compliance)
    learner_service
        .delete_learner(id)
        .await
        .map_err(|e| AppError::InternalServerError)?;

    // Log audit event for GDPR compliance
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "delete".to_string(),
        "learner".to_string(),
        id.to_string(),
        None,
        None,
        None,
    )
    .await
    .ok();

    Ok(StatusCode::NO_CONTENT)
}

pub async fn sessions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<Session>>> {
    // Check permissions first
    let learner_service =
        LearnerService::new(state.db_pool.clone().into(), state.redis_conn.clone());

    let learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::NotFound("Learner not found".to_string()))?;

    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Get sessions for this learner
    let learner_bytes = id.as_bytes();
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let session_rows = sqlx::query(
        "SELECT id, learner_id, topology_type, topology_data, status, 
                start_time, end_time, summary
         FROM sessions 
         WHERE learner_id = ? 
         ORDER BY start_time DESC",
    )
    .bind(&learner_bytes[..])
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let sessions: Vec<Session> = session_rows
        .into_iter()
        .map(|row| {
            let id_bytes: Vec<u8> = row.get("id");
            let learner_id_bytes: Vec<u8> = row.get("learner_id");

            Session {
                id: Uuid::from_bytes(id_bytes.try_into().unwrap_or_default()),
                learner_id: Uuid::from_bytes(learner_id_bytes.try_into().unwrap_or_default()),
                topology_type: row.get("topology_type"),
                topology_data: row
                    .get::<Option<String>, _>("topology_data")
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({})),
                status: row.get("status"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                summary: row
                    .get::<Option<String>, _>("summary")
                    .and_then(|s| serde_json::from_str(&s).ok()),
            }
        })
        .collect();

    Ok(Json(sessions))
}

// Helper function to calculate preferred difficulty based on learner performance
fn calculate_preferred_difficulty(response_stats: &sqlx::sqlite::SqliteRow) -> f64 {
    let accuracy = response_stats
        .get::<Option<f64>, _>("accuracy")
        .unwrap_or(0.5);

    // Calculate preferred difficulty based on accuracy:
    // - Too easy (>90% accuracy): increase difficulty
    // - Too hard (<50% accuracy): decrease difficulty
    // - Sweet spot (70-85% accuracy): maintain current level
    if accuracy > 0.90 {
        0.7 // Increase difficulty for high performers
    } else if accuracy < 0.50 {
        0.3 // Decrease difficulty for struggling learners
    } else if accuracy >= 0.70 && accuracy <= 0.85 {
        0.5 // Maintain current difficulty in sweet spot
    } else if accuracy > 0.85 {
        0.6 // Moderate increase
    } else {
        0.4 // Moderate decrease
    }
}
