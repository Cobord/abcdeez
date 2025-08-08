use axum::{extract::{Path, State}, Json};
use std::sync::Arc;

use crate::{
    error::AppResult,
    state::AppState,
};

pub async fn scales(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // TODO: List available scales
    Ok(Json(vec![]))
}

pub async fn progressions(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // TODO: List chord progressions
    Ok(Json(vec![]))
}

pub async fn tasks(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Generate music task
    Ok(Json(serde_json::json!({"task": "music_placeholder"})))
}

pub async fn audio(
    State(state): State<Arc<AppState>>,
    Path(note): Path<String>,
) -> AppResult<Vec<u8>> {
    // TODO: Get audio sample for note
    Ok(vec![])
}