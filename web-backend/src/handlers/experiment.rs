use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::Experiment,
    state::AppState,
};

pub async fn list(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Experiment>>> {
    let query = "SELECT id, name, description, config, start_date, end_date, created_by, status FROM experiments ORDER BY status DESC";
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let rows = sqlx::query(query).fetch_all(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    let experiments: Vec<Experiment> = rows.into_iter().map(|row| {
        Experiment {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            config: serde_json::from_str(row.get::<&str, _>("config")).unwrap_or_default(),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            created_by: row.get("created_by"),
            status: row.get("status"),
        }
    }).collect();
    
    Ok(Json(experiments))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<(StatusCode, Json<Experiment>)> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled Experiment");
    let description = req.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let config = req.get("config").cloned().unwrap_or_default();
    
    let id = Uuid::new_v4();
    
    let query = "INSERT INTO experiments (id, name, description, config, status) VALUES (?, ?, ?, ?, ?)";
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    sqlx::query(query)
        .bind(id.to_string())
        .bind(name)
        .bind(&description)
        .bind(config.to_string())
        .bind("draft")
        .execute(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    let experiment = Experiment {
        id,
        name: name.to_string(),
        description,
        config,
        start_date: None,
        end_date: None,
        created_by: None,
        status: "draft".to_string(),
    };
    
    Ok((StatusCode::CREATED, Json(experiment)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Experiment>> {
    let query = "SELECT id, name, description, config, start_date, end_date, created_by, status FROM experiments WHERE id = ?";
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let rows = sqlx::query(query).bind(id.to_string()).fetch_all(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    if let Some(row) = rows.into_iter().next() {
        let experiment = Experiment {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            config: serde_json::from_str(row.get::<&str, _>("config")).unwrap_or_default(),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            created_by: row.get("created_by"),
            status: row.get("status"),
        };
        Ok(Json(experiment))
    } else {
        Err(AppError::NotFound("Experiment not found".to_string()))
    }
}

pub async fn join(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Verify experiment exists
    let check_query = "SELECT id FROM experiments WHERE id = ?";
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let rows = sqlx::query(check_query).bind(id.to_string()).fetch_all(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    if rows.is_empty() {
        return Err(AppError::NotFound("Experiment not found".to_string()));
    }
    
    // For now, joining an experiment just means acknowledging it exists
    // In a real implementation, you'd track participants, assign conditions, etc.
    Ok(StatusCode::NO_CONTENT)
}

pub async fn results(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify experiment exists
    let check_query = "SELECT name FROM experiments WHERE id = ?";
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let rows = sqlx::query(check_query).bind(id.to_string()).fetch_all(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    if rows.is_empty() {
        return Err(AppError::NotFound("Experiment not found".to_string()));
    }
    
    let experiment_name = rows[0].get::<String, _>("name");
    
    // Get aggregated results from task responses related to this experiment
    let results_query = "SELECT COUNT(*) as total_responses, AVG(CASE WHEN correct = 1 THEN 1.0 ELSE 0.0 END) as accuracy, AVG(response_time_ms) as avg_response_time FROM task_responses WHERE learner_id IN (SELECT learner_id FROM learners)";
    let result_rows = sqlx::query(results_query).fetch_all(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    let results = if let Some(row) = result_rows.into_iter().next() {
        serde_json::json!({
            "experiment_id": id,
            "experiment_name": experiment_name,
            "total_responses": row.get::<i64, _>("total_responses"),
            "overall_accuracy": row.get::<Option<f64>, _>("accuracy").unwrap_or(0.0),
            "average_response_time_ms": row.get::<Option<f64>, _>("avg_response_time").unwrap_or(0.0),
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    } else {
        serde_json::json!({
            "experiment_id": id,
            "experiment_name": experiment_name,
            "total_responses": 0,
            "overall_accuracy": 0.0,
            "average_response_time_ms": 0.0,
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    };
    
    Ok(Json(results))
}

pub async fn export(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify experiment exists
    let check_query = "SELECT name FROM experiments WHERE id = ?";
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let rows = sqlx::query(check_query).bind(id.to_string()).fetch_all(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    if rows.is_empty() {
        return Err(AppError::NotFound("Experiment not found".to_string()));
    }
    
    let experiment_name = rows[0].get::<String, _>("name");
    
    // Export all relevant data for this experiment
    let export_query = "SELECT tr.learner_id, tr.task_type, tr.prompt, tr.response, tr.correct, tr.response_time_ms, tr.created_at FROM task_responses tr JOIN learners l ON tr.learner_id = l.id ORDER BY tr.created_at";
    let export_rows = sqlx::query(export_query).fetch_all(&mut *conn).await.map_err(|e| AppError::DatabaseError(e))?;
    
    let data: Vec<serde_json::Value> = export_rows.into_iter().map(|row| {
        serde_json::json!({
            "learner_id": row.get::<String, _>("learner_id"),
            "task_type": row.get::<String, _>("task_type"),
            "prompt": row.get::<String, _>("prompt"),
            "response": row.get::<String, _>("response"),
            "correct": row.get::<bool, _>("correct"),
            "response_time_ms": row.get::<i64, _>("response_time_ms"),
            "timestamp": row.get::<String, _>("created_at")
        })
    }).collect();
    
    Ok(Json(serde_json::json!({
        "experiment_id": id,
        "experiment_name": experiment_name,
        "export_format": "json",
        "total_records": data.len(),
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "data": data
    })))
}
