use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{
    error::{AppError, AppResult},
    state::AppState,
};

pub async fn next(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Get next adaptive task
    Ok(Json(serde_json::json!({"task": "placeholder"})))
}

pub async fn generate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Generate specific task type
    Ok(Json(serde_json::json!({"task": "generated"})))
}

pub async fn difficulty(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<f64>> {
    // TODO: Get current difficulty calibration
    Ok(Json(0.5))
}

pub async fn request_hint(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Generate hint for current task
    Ok(Json(serde_json::json!({"hint": "Try thinking about it differently"})))
}