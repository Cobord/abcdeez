use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    models::{CreateLearnerRequest, Learner},
    AppState,
};

pub async fn create_learner(
    State(state): State<AppState>,
    Json(req): Json<CreateLearnerRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4().to_string();
    
    let learner = sqlx::query_as::<_, Learner>(
        r#"
        INSERT INTO learners (id, user_id, display_name, metadata)
        VALUES (?1, ?2, ?3, ?4)
        RETURNING *
        "#
    )
    .bind(&id)
    .bind(&req.user_id)
    .bind(&req.display_name)
    .bind(&req.metadata)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, Json(learner)))
}

pub async fn get_learner(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let learner = sqlx::query_as::<_, Learner>(
        "SELECT * FROM learners WHERE id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(learner))
}

pub async fn export_learner_data(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get learner
    let learner = sqlx::query_as::<_, Learner>(
        "SELECT * FROM learners WHERE id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Get all sessions
    let sessions = sqlx::query!(
        r#"
        SELECT s.*, 
               COUNT(r.id) as response_count,
               AVG(CASE WHEN r.correct THEN 1.0 ELSE 0.0 END) as accuracy,
               AVG(r.response_time_ms) as avg_rt
        FROM sessions s
        LEFT JOIN responses r ON s.id = r.session_id
        WHERE s.learner_id = ?1
        GROUP BY s.id
        ORDER BY s.start_time DESC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Get latest model snapshot
    let snapshot = sqlx::query!(
        r#"
        SELECT parameters, metrics
        FROM model_snapshots
        WHERE learner_id = ?1
        ORDER BY timestamp DESC
        LIMIT 1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Build export structure
    let export = serde_json::json!({
        "learner": learner,
        "sessions": sessions,
        "model_snapshot": snapshot,
        "export_timestamp": chrono::Utc::now(),
    });
    
    Ok(Json(export))
}