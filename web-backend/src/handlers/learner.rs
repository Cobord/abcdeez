use axum::{extract::{Path, State}, http::StatusCode, Json, Extension};
use std::sync::Arc;
use uuid::Uuid;

use graph_learning_core::{Topology, TopologyType};
use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::{learner::{Learner, CreateLearnerRequest, UpdateLearnerRequest, LearnerStats}},
    state::AppState,
    services::{LearnerService, audit::AuditService},
};

pub async fn create(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CreateLearnerRequest>,
) -> AppResult<(StatusCode, Json<Learner>)> {
    // Default to alphabet topology if not specified
    let topology = Topology::alphabet_topology();
    
    // Use the authenticated user's ID if no user_id provided
    let user_id = req.user_id.or(Some(claims.sub));
    
    // Create learner using the service
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );
    
    let learner = learner_service
        .create_learner(user_id, req.display_name, &topology)
        .await
        .map_err(|e| AppError::InternalServerError)?;

    // Convert to the API model format
    let api_learner = Learner {
        id: learner.id,
        user_id: learner.user_id,
        display_name: learner.display_name,
        created_at: learner.created_at,
        last_active: learner.last_active,
        total_practice_time_seconds: learner.total_practice_time_seconds,
        metadata: learner.metadata,
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
    ).await.ok();

    Ok((StatusCode::CREATED, Json(api_learner)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Learner>> {
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );
    
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
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );
    
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
    
    sqlx::query!(
        "UPDATE learners SET display_name = COALESCE($1, display_name), 
                           metadata = COALESCE($2, metadata),
                           last_active = $3
         WHERE id = $4",
        req.display_name,
        req.metadata.map(|m| m.to_string()),
        now,
        learner_bytes
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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
    ).await.ok();

    Ok(Json(api_learner))
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<LearnerStats>> {
    // Check permissions first
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );
    
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
    let session_stats = sqlx::query!(
        "SELECT COUNT(*) as total_sessions FROM sessions WHERE learner_id = $1",
        learner_bytes
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Get response statistics
    let response_stats = sqlx::query!(
        "SELECT 
            COUNT(*) as total_tasks,
            AVG(CASE WHEN correct THEN 1.0 ELSE 0.0 END) as accuracy
         FROM responses r
         JOIN sessions s ON r.session_id = s.id
         WHERE s.learner_id = $1",
        learner_bytes
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Get learning curve (daily accuracy over last 30 days)
    let learning_curve_data = sqlx::query!(
        "SELECT 
            DATE(r.timestamp) as date,
            AVG(CASE WHEN r.correct THEN 1.0 ELSE 0.0 END) as daily_accuracy
         FROM responses r
         JOIN sessions s ON r.session_id = s.id
         WHERE s.learner_id = $1 
           AND r.timestamp > $2
         GROUP BY DATE(r.timestamp)
         ORDER BY date",
        learner_bytes,
        chrono::Utc::now() - chrono::Duration::days(30)
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let learning_curve = learning_curve_data
        .into_iter()
        .map(|row| {
            let date = row.date.and_hms_opt(12, 0, 0).unwrap().and_utc();
            let accuracy = row.daily_accuracy.unwrap_or(0.0);
            (date, accuracy)
        })
        .collect();

    let stats = LearnerStats {
        total_sessions: session_stats.total_sessions.unwrap_or(0),
        total_tasks_completed: response_stats.total_tasks.unwrap_or(0),
        overall_accuracy: response_stats.accuracy.unwrap_or(0.0),
        total_practice_time_seconds: learner.total_practice_time_seconds,
        last_active: learner.last_active,
        preferred_difficulty: 0.5, // TODO: Calculate from learner model
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
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );
    
    let learner = learner_service
        .get_learner(id)
        .await
        .map_err(|e| AppError::NotFound("Learner not found".to_string()))?;

    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Export data using the service
    let export_data = learner_service
        .export_learner_data(id)
        .await
        .map_err(|e| AppError::InternalServerError)?;

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
    ).await.ok();

    Ok(Json(export_data))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Check permissions first
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );
    
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
    ).await.ok();

    Ok(StatusCode::NO_CONTENT)
}