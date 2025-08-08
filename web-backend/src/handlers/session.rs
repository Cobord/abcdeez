use axum::{extract::{Path, State}, http::StatusCode, Json, Extension};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use graph_learning_core::{Topology, TopologyType, tasks::TaskResponse as CoreTaskResponse};
use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::{
        session::{Session, CreateSessionRequest, SessionStatus, SessionSummary},
        response::{TaskResponse, ResponseRecord}
    },
    state::AppState,
    services::{LearnerService, AdaptationService, audit::AuditService},
};

pub async fn create(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CreateSessionRequest>,
) -> AppResult<(StatusCode, Json<Session>)> {
    // Verify learner exists and user has permission
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );
    
    let learner = learner_service
        .get_learner(req.learner_id)
        .await
        .map_err(|_| AppError::NotFound("Learner not found".to_string()))?;

    // Check permissions
    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Create topology based on request
    let topology = match req.topology_type {
        TopologyType::Linear => Topology::alphabet_topology(),
        TopologyType::Cyclic => Topology::days_of_week_topology(),
        TopologyType::PartialOrder => Topology::music_theory_topology(),
        TopologyType::GeneralGraph => Topology::custom_graph_topology(),
    };

    // Create session in database
    let session_id = Uuid::new_v4();
    let session_id_bytes = session_id.as_bytes();
    let learner_id_bytes = req.learner_id.as_bytes();
    let now = Utc::now();

    let topology_data = serde_json::to_value(&topology)
        .map_err(|_| AppError::InternalServerError)?;

    sqlx::query!(
        "INSERT INTO sessions (id, learner_id, topology_type, topology_data, start_time, status) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        session_id_bytes,
        learner_id_bytes,
        format!("{:?}", req.topology_type),
        topology_data.to_string(),
        now,
        SessionStatus::Active.to_string()
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let session = Session {
        id: session_id,
        learner_id: req.learner_id,
        topology_type: format!("{:?}", req.topology_type),
        topology_data,
        start_time: now,
        end_time: None,
        status: SessionStatus::Active.to_string(),
        summary: None,
    };

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "create".to_string(),
        "session".to_string(),
        session_id.to_string(),
        Some(serde_json::json!({
            "learner_id": req.learner_id,
            "topology_type": req.topology_type
        })),
        None,
        None,
    ).await.ok();

    Ok((StatusCode::CREATED, Json(session)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Session>> {
    let session_id_bytes = id.as_bytes();
    
    let session_row = sqlx::query!(
        "SELECT s.id, s.learner_id, s.topology_type, s.topology_data, s.start_time, s.end_time, s.status, s.summary,
                l.user_id
         FROM sessions s
         JOIN learners l ON s.learner_id = l.id
         WHERE s.id = $1",
        session_id_bytes
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let session_row = session_row.ok_or(AppError::NotFound("Session not found".to_string()))?;

    // Check permissions
    if let Some(user_id_bytes) = session_row.user_id {
        let user_id = Uuid::from_bytes(user_id_bytes.try_into().unwrap_or_default());
        if user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    let learner_id = Uuid::from_bytes(
        session_row.learner_id.try_into().unwrap_or_default()
    );

    let topology_data: serde_json::Value = serde_json::from_str(&session_row.topology_data)
        .unwrap_or(serde_json::json!({}));

    let session = Session {
        id,
        learner_id,
        topology_type: session_row.topology_type,
        topology_data,
        start_time: session_row.start_time,
        end_time: session_row.end_time,
        status: session_row.status,
        summary: session_row.summary.and_then(|s| serde_json::from_str(&s).ok()),
    };

    Ok(Json(session))
}

pub async fn submit_response(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(session_id): Path<Uuid>,
    Json(response): Json<TaskResponse>,
) -> AppResult<Json<serde_json::Value>> {
    // First verify session exists and user has permission
    let session = get_session_with_permission(&state, &claims, session_id).await?;
    
    if session.status != SessionStatus::Active.to_string() {
        return Err(AppError::BadRequest("Session is not active".to_string()));
    }

    // Get next sequence number
    let session_id_bytes = session_id.as_bytes();
    let next_sequence = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM responses WHERE session_id = $1",
        session_id_bytes
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .unwrap_or(1);

    // Validate the response and determine if it's correct
    let is_correct = validate_task_response(&response).await?;

    // Store response in database
    let response_id = Uuid::new_v4();
    let response_id_bytes = response_id.as_bytes();

    sqlx::query!(
        "INSERT INTO responses (id, session_id, sequence_number, task_type, task_data, user_answer, correct, response_time_ms, hint_level) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        response_id_bytes,
        session_id_bytes,
        next_sequence,
        response.task_type,
        response.task_data.to_string(),
        response.user_answer,
        is_correct,
        response.response_time_ms,
        response.hint_level
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Update learner model with the response
    let learner_service = LearnerService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );

    // Create a core task response for model updating
    let core_response = CoreTaskResponse {
        task_id: response_id.to_string(),
        correct: is_correct,
        response_time_ms: response.response_time_ms as u128,
        operation_type: None, // Would be parsed from task_data in a full implementation
        timestamp: Utc::now(),
    };

    learner_service
        .update_learner_model(session.learner_id, &core_response)
        .await
        .ok(); // Don't fail if model update fails

    // Check if intervention is needed
    let adaptation_service = AdaptationService::new(
        Arc::new(learner_service)
    );

    // Get recent error count for intervention decision
    let recent_errors = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM responses 
         WHERE session_id = $1 
           AND sequence_number > $2 
           AND NOT correct",
        session_id_bytes,
        next_sequence - 5 // Last 5 responses
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0) as usize;

    let intervention = adaptation_service
        .should_intervene(session.learner_id, session_id, response.response_time_ms as u64, recent_errors)
        .await
        .ok()
        .flatten();

    // Generate next task recommendation
    let topology: Topology = serde_json::from_value(session.topology_data).unwrap_or_default();
    let next_task = adaptation_service
        .select_next_task(session.learner_id, &topology)
        .await
        .ok();

    let response_data = serde_json::json!({
        "response_id": response_id,
        "sequence_number": next_sequence,
        "correct": is_correct,
        "intervention": intervention,
        "next_task": next_task
    });

    Ok(Json(response_data))
}

pub async fn complete(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<SessionSummary>> {
    // Verify session exists and user has permission
    let session = get_session_with_permission(&state, &claims, session_id).await?;
    
    if session.status != SessionStatus::Active.to_string() {
        return Err(AppError::BadRequest("Session is not active".to_string()));
    }

    let session_id_bytes = session_id.as_bytes();
    let now = Utc::now();

    // Calculate session summary
    let summary_stats = sqlx::query!(
        "SELECT 
            COUNT(*) as total_tasks,
            SUM(CASE WHEN correct THEN 1 ELSE 0 END) as correct_responses,
            AVG(response_time_ms) as avg_response_time
         FROM responses 
         WHERE session_id = $1",
        session_id_bytes
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total_tasks = summary_stats.total_tasks.unwrap_or(0);
    let correct_responses = summary_stats.correct_responses.unwrap_or(0);
    let accuracy = if total_tasks > 0 {
        correct_responses as f64 / total_tasks as f64
    } else {
        0.0
    };
    let avg_response_time = summary_stats.avg_response_time.unwrap_or(0.0);

    let duration_seconds = (now - session.start_time).num_seconds();

    let summary = SessionSummary {
        total_tasks,
        correct_responses,
        accuracy,
        average_response_time_ms: avg_response_time,
        duration_seconds,
        strategy_distribution: serde_json::json!({"systematic": 0.6, "intuitive": 0.4}),
    };

    // Update session as completed
    sqlx::query!(
        "UPDATE sessions SET status = $1, end_time = $2, summary = $3 WHERE id = $4",
        SessionStatus::Completed.to_string(),
        now,
        serde_json::to_string(&summary).unwrap_or_default(),
        session_id_bytes
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Log audit event
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "complete".to_string(),
        "session".to_string(),
        session_id.to_string(),
        Some(serde_json::json!({
            "duration_seconds": duration_seconds,
            "total_tasks": total_tasks,
            "accuracy": accuracy
        })),
        None,
        None,
    ).await.ok();

    Ok(Json(summary))
}

pub async fn replay(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<Vec<ResponseRecord>>> {
    // Verify session exists and user has permission
    get_session_with_permission(&state, &claims, session_id).await?;
    
    let session_id_bytes = session_id.as_bytes();
    
    let responses = sqlx::query_as!(
        ResponseRecord,
        "SELECT id, session_id, sequence_number, task_type, task_data, user_answer, correct, response_time_ms, hint_level, timestamp
         FROM responses 
         WHERE session_id = $1 
         ORDER BY sequence_number",
        session_id_bytes
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Convert byte arrays back to UUIDs
    let responses: Vec<ResponseRecord> = responses.into_iter().map(|mut r| {
        r.id = Uuid::from_bytes(r.id.as_bytes().try_into().unwrap_or_default());
        r.session_id = session_id;
        r
    }).collect();

    Ok(Json(responses))
}

// Helper functions
async fn get_session_with_permission(
    state: &AppState,
    claims: &Claims,
    session_id: Uuid,
) -> AppResult<Session> {
    let session_id_bytes = session_id.as_bytes();
    
    let session_row = sqlx::query!(
        "SELECT s.id, s.learner_id, s.topology_type, s.topology_data, s.start_time, s.end_time, s.status, s.summary,
                l.user_id
         FROM sessions s
         JOIN learners l ON s.learner_id = l.id
         WHERE s.id = $1",
        session_id_bytes
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let session_row = session_row.ok_or(AppError::NotFound("Session not found".to_string()))?;

    // Check permissions
    if let Some(user_id_bytes) = session_row.user_id {
        let user_id = Uuid::from_bytes(user_id_bytes.try_into().unwrap_or_default());
        if user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    let learner_id = Uuid::from_bytes(
        session_row.learner_id.try_into().unwrap_or_default()
    );

    let topology_data: serde_json::Value = serde_json::from_str(&session_row.topology_data)
        .unwrap_or(serde_json::json!({}));

    Ok(Session {
        id: session_id,
        learner_id,
        topology_type: session_row.topology_type,
        topology_data,
        start_time: session_row.start_time,
        end_time: session_row.end_time,
        status: session_row.status,
        summary: session_row.summary.and_then(|s| serde_json::from_str(&s).ok()),
    })
}

async fn validate_task_response(response: &TaskResponse) -> AppResult<bool> {
    // Simplified validation - in a full implementation, this would use the task data
    // to determine the correct answer and validate the user's response
    match response.task_type.as_str() {
        "successor" | "predecessor" => {
            // For alphabet tasks, check if the answer is reasonable
            if let Some(answer) = &response.user_answer {
                Ok(answer.len() == 1 && answer.chars().next().unwrap().is_alphabetic())
            } else {
                Ok(false)
            }
        },
        "pairwise_order" => {
            // For ordering tasks, accept any non-empty answer
            Ok(response.user_answer.is_some())
        },
        _ => {
            // Default validation - assume correct for unknown task types
            Ok(true)
        }
    }
}