use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::{
        response::{ResponseRecord, TaskResponse},
        session::{CreateSessionRequest, Session, SessionStatus, SessionSummary},
    },
    monitoring::business::global_business_metrics,
    services::{audit::AuditService, AdaptationService, LearnerService},
    state::AppState,
};
use graph_learning_core::{tasks::TaskResponse as CoreTaskResponse, Topology};

pub async fn create(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CreateSessionRequest>,
) -> AppResult<(StatusCode, Json<Session>)> {
    // Verify learner exists and user has permission
    let learner_service =
        LearnerService::new(Arc::new(state.db_pool.clone()), state.redis_conn.clone());

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

    // Create topology based on request - either use provided topology_data or create from type
    let topology_data = if let Some(provided_topology) = req.topology_data {
        // Use provided topology data from UI
        provided_topology
    } else {
        // Create topology server-side based on string type
        let topology = match req.topology_type.to_lowercase().as_str() {
            "linear" | "alphabet" => Topology::alphabet(),
            "cyclic" | "days_of_week" => Topology::days_of_week(),
            "partial_order" | "partialorder" => Topology::alphabet(), // Use alphabet as fallback
            "general_graph" | "generalgraph" | "music" | "mathematics" => Topology::alphabet(), // Use alphabet as fallback
            _ => Topology::alphabet(), // Default fallback
        };
        serde_json::to_value(&topology).map_err(|_| AppError::InternalServerError)?
    };

    // Create session in database
    let session_id = Uuid::new_v4();
    let session_id_bytes = session_id.as_bytes();
    let learner_id_bytes = req.learner_id.as_bytes();
    let now = Utc::now();

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    sqlx::query(
        "INSERT INTO sessions (id, learner_id, topology_type, topology_data, start_time, status)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&session_id_bytes[..])
    .bind(&learner_id_bytes[..])
    .bind(&req.topology_type)
    .bind(topology_data.to_string())
    .bind(now)
    .bind(SessionStatus::Active.to_string())
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let session = Session {
        id: session_id,
        learner_id: req.learner_id,
        topology_type: req.topology_type.clone(),
        topology_data,
        start_time: now,
        end_time: None,
        status: SessionStatus::Active.to_string(),
        summary: None,
    };

    // Record session creation
    crate::monitoring::global_metrics().record_session_creation();

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
    )
    .await
    .ok();

    Ok((StatusCode::CREATED, Json(session)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Session>> {
    let session_id_bytes = id.as_bytes();

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let session_row = sqlx::query(
        "SELECT s.id, s.learner_id, s.topology_type, s.topology_data, s.start_time, s.end_time, s.status, s.summary,
                l.user_id
         FROM sessions s
         JOIN learners l ON s.learner_id = l.id
         WHERE s.id = ?"
    )
    .bind(&session_id_bytes[..])
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let session_row = session_row.ok_or(AppError::NotFound("Session not found".to_string()))?;

    // Check permissions
    if let Some(user_id_bytes) = session_row.get::<Option<Vec<u8>>, _>("user_id") {
        let user_id = Uuid::from_bytes(user_id_bytes.try_into().unwrap_or_default());
        if user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    let learner_id_bytes: Vec<u8> = session_row.get("learner_id");
    let learner_id = Uuid::from_bytes(learner_id_bytes.try_into().unwrap_or_default());

    let topology_data_str: String = session_row.get("topology_data");
    let topology_data: serde_json::Value =
        serde_json::from_str(&topology_data_str).unwrap_or(serde_json::json!({}));

    let session = Session {
        id,
        learner_id,
        topology_type: session_row.get("topology_type"),
        topology_data,
        start_time: session_row.get("start_time"),
        end_time: session_row.get("end_time"),
        status: session_row.get("status"),
        summary: session_row
            .get::<Option<String>, _>("summary")
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
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
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let next_sequence: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM responses WHERE session_id = ?",
    )
    .bind(&session_id_bytes[..])
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Validate the response and determine if it's correct
    let is_correct = validate_task_response(&response).await?;

    // Store response in database
    let response_id = Uuid::new_v4();
    let response_id_bytes = response_id.as_bytes();

    sqlx::query(
        "INSERT INTO responses (id, session_id, sequence_number, task_type, task_data, user_answer, correct, response_time_ms, hint_level)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&response_id_bytes[..])
    .bind(&session_id_bytes[..])
    .bind(next_sequence)
    .bind(&response.task_type)
    .bind(response.task_data.to_string())
    .bind(&response.user_answer)
    .bind(is_correct)
    .bind(response.response_time_ms)
    .bind(response.hint_level)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Track business metrics for task performance
    global_business_metrics().record_task_performance(
        claims.sub,
        &response.task_type,
        is_correct,
        response.response_time_ms as u64,
        response.hint_level.is_some(),
    ).await;

    // Update learner model with the response
    let learner_service =
        LearnerService::new(Arc::new(state.db_pool.clone()), state.redis_conn.clone());

    // Create a placeholder task for the response (correct_answer would be determined from task_data)
    let task = graph_learning_core::Task {
        task_type: graph_learning_core::TaskType::Successor {
            item: "A".to_string(),
        },
        prompt: "What comes next?".to_string(),
        correct_answer: "B".to_string(), // This would normally be derived from task_data
        options: vec![],
        difficulty: 0.5,
        operation: graph_learning_core::OperationType::Successor,
    };

    // Create a core task response for model updating
    let core_response = CoreTaskResponse {
        task,
        user_answer: response.user_answer.clone(),
        correct: is_correct,
        response_time_ms: response.response_time_ms as u128,
        timestamp: Utc::now(),
    };

    // Extract topology from session data for Bayesian updates
    let topology: Topology =
        serde_json::from_value(session.topology_data).unwrap_or_else(|_| Topology::alphabet());

    let mut learner_service_mut = learner_service.clone();
    learner_service_mut
        .update_learner_response(session.learner_id, &core_response, &topology)
        .await
        .ok(); // Don't fail if model update fails

    // Check if intervention is needed
    let adaptation_service = AdaptationService::new(Arc::new(learner_service));

    // Get recent error count for intervention decision
    let recent_errors: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM responses
         WHERE session_id = ?
           AND sequence_number > ?
           AND NOT correct",
    )
    .bind(&session_id_bytes[..])
    .bind(next_sequence - 5)
    .fetch_one(&mut *conn)
    .await
    .unwrap_or(0);
    let recent_errors = recent_errors as usize;

    let intervention = adaptation_service
        .should_intervene(
            session.learner_id,
            session_id,
            response.response_time_ms as u64,
            recent_errors,
        )
        .await
        .ok()
        .flatten();

    // Generate next task recommendation using the same topology
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
) -> AppResult<StatusCode> {
    // Verify session exists and user has permission
    let session = get_session_with_permission(&state, &claims, session_id).await?;

    if session.status != SessionStatus::Active.to_string() {
        return Err(AppError::BadRequest("Session is not active".to_string()));
    }

    let session_id_bytes = session_id.as_bytes();
    let now = Utc::now();

    // Calculate session summary
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let summary_stats = sqlx::query(
        "SELECT
            COUNT(*) as total_tasks,
            SUM(CASE WHEN correct THEN 1 ELSE 0 END) as correct_responses,
            AVG(response_time_ms) as avg_response_time
         FROM responses
         WHERE session_id = ?",
    )
    .bind(&session_id_bytes[..])
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let total_tasks: i64 = summary_stats.get::<i64, _>("total_tasks");
    let correct_responses: i64 = summary_stats.get::<i64, _>("correct_responses");
    let accuracy = if total_tasks > 0 {
        correct_responses as f64 / total_tasks as f64
    } else {
        0.0
    };
    let avg_response_time: f64 = summary_stats
        .get::<Option<f64>, _>("avg_response_time")
        .unwrap_or(0.0);

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
    sqlx::query("UPDATE sessions SET status = ?, end_time = ?, summary = ? WHERE id = ?")
        .bind(SessionStatus::Completed.to_string())
        .bind(now)
        .bind(serde_json::to_string(&summary).unwrap_or_default())
        .bind(&session_id_bytes[..])
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

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
    )
    .await
    .ok();

    Ok(StatusCode::NO_CONTENT)
}

pub async fn replay(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<Vec<ResponseRecord>>> {
    // Verify session exists and user has permission
    get_session_with_permission(&state, &claims, session_id).await?;

    let session_id_bytes = session_id.as_bytes();

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let response_rows = sqlx::query(
        "SELECT id, session_id, sequence_number, task_type, task_data, user_answer, correct, response_time_ms, hint_level, timestamp
         FROM responses
         WHERE session_id = ?
         ORDER BY sequence_number"
    )
    .bind(&session_id_bytes[..])
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Convert byte arrays back to UUIDs and build ResponseRecord structs
    let responses: Vec<ResponseRecord> = response_rows
        .into_iter()
        .map(|row| {
            let id_bytes: Vec<u8> = row.get::<Vec<u8>, _>("id");
            let response_id = Uuid::from_bytes(id_bytes.try_into().unwrap_or_default());

            ResponseRecord {
                id: response_id,
                session_id,
                sequence_number: row.get::<i32, _>("sequence_number"),
                task_type: row.get::<String, _>("task_type"),
                task_data: {
                    let s: String = row.get::<String, _>("task_data");
                    serde_json::from_str(&s).unwrap_or(serde_json::json!({}))
                },
                user_answer: row.get::<Option<String>, _>("user_answer"),
                correct: row.get::<bool, _>("correct"),
                response_time_ms: row.get::<i64, _>("response_time_ms"),
                hint_level: row.get::<Option<i32>, _>("hint_level"),
                timestamp: row.get::<chrono::DateTime<chrono::Utc>, _>("timestamp"),
            }
        })
        .collect();

    Ok(Json(responses))
}

pub async fn responses(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<Vec<ResponseRecord>>> {
    // This is an alias/delegation to the replay function for UI compatibility
    replay(State(state), claims, Path(session_id)).await
}

// Helper functions
async fn get_session_with_permission(
    state: &AppState,
    claims: &Claims,
    session_id: Uuid,
) -> AppResult<Session> {
    let session_id_bytes = session_id.as_bytes();

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let session_row = sqlx::query(
        "SELECT s.id, s.learner_id, s.topology_type, s.topology_data, s.start_time, s.end_time, s.status, s.summary,
                l.user_id
         FROM sessions s
         JOIN learners l ON s.learner_id = l.id
         WHERE s.id = ?"
    )
    .bind(&session_id_bytes[..])
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let session_row = session_row.ok_or(AppError::NotFound("Session not found".to_string()))?;

    // Check permissions
    if let Some(user_id_bytes) = session_row.get::<Option<Vec<u8>>, _>("user_id") {
        let user_id = Uuid::from_bytes(user_id_bytes.try_into().unwrap_or_default());
        if user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    let learner_id_bytes: Vec<u8> = session_row.get("learner_id");
    let learner_id = Uuid::from_bytes(learner_id_bytes.try_into().unwrap_or_default());

    let topology_data_str: String = session_row.get("topology_data");
    let topology_data: serde_json::Value =
        serde_json::from_str(&topology_data_str).unwrap_or(serde_json::json!({}));

    Ok(Session {
        id: session_id,
        learner_id,
        topology_type: session_row.get("topology_type"),
        topology_data,
        start_time: session_row.get("start_time"),
        end_time: session_row.get("end_time"),
        status: session_row.get("status"),
        summary: session_row
            .get::<Option<String>, _>("summary")
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
    })
}

async fn validate_task_response(response: &TaskResponse) -> AppResult<bool> {
    // Simplified validation - in a full implementation, this would use the task data
    // to determine the correct answer and validate the user's response
    match response.task_type.as_str() {
        "successor" | "predecessor" => {
            // For alphabet tasks, check if the answer is reasonable
            Ok(response.user_answer.len() == 1
                && response
                    .user_answer
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_alphabetic()))
        }
        "pairwise_order" => {
            // For ordering tasks, accept any non-empty answer
            Ok(!response.user_answer.is_empty())
        }
        _ => {
            // Default validation - assume correct for unknown task types
            Ok(true)
        }
    }
}
