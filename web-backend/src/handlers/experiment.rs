use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::Experiment,
    state::AppState,
};

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<Experiment>>> {
    // TODO: List experiments
    Ok(Json(vec![]))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<(StatusCode, Json<Experiment>)> {
    // TODO: Create experiment
    Err(AppError::InternalServerError)
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Experiment>> {
    // TODO: Get experiment details
    Err(AppError::NotFound("Experiment not found".to_string()))
}

pub async fn join(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // TODO: Join experiment
    Ok(StatusCode::NO_CONTENT)
}

pub async fn results(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Get experiment results
    Ok(Json(serde_json::json!({"results": "placeholder"})))
}

pub async fn export(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Export experiment data
    Ok(Json(serde_json::json!({"export": "placeholder"})))
}