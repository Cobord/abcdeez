use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

use crate::{error::AppResult, state::AppState};

pub async fn scales(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // Return common music scales
    Ok(Json(vec![
        serde_json::json!({"name": "Major", "intervals": [2, 2, 1, 2, 2, 2, 1]}),
        serde_json::json!({"name": "Minor", "intervals": [2, 1, 2, 2, 1, 2, 2]}),
        serde_json::json!({"name": "Dorian", "intervals": [2, 1, 2, 2, 2, 1, 2]}),
        serde_json::json!({"name": "Phrygian", "intervals": [1, 2, 2, 2, 1, 2, 2]}),
    ]))
}

pub async fn progressions(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // Return common chord progressions
    Ok(Json(vec![
        serde_json::json!({"name": "I-IV-V", "chords": ["I", "IV", "V"]}),
        serde_json::json!({"name": "I-V-vi-IV", "chords": ["I", "V", "vi", "IV"]}),
        serde_json::json!({"name": "ii-V-I", "chords": ["ii", "V", "I"]}),
    ]))
}

pub async fn tasks(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // Generate a simple music theory task
    Ok(Json(serde_json::json!({
        "type": "interval_identification",
        "question": "What interval is between C and G?",
        "options": ["Perfect Fourth", "Perfect Fifth", "Major Third", "Minor Sixth"],
        "correct_answer": "Perfect Fifth",
        "difficulty": 0.3
    })))
}

pub async fn audio(
    State(_state): State<Arc<AppState>>,
    Path(note): Path<String>,
) -> AppResult<Vec<u8>> {
    // Return placeholder audio data
    // In production, this would load actual audio files
    tracing::debug!("Audio requested for note: {}", note);
    Ok(vec![0u8; 100]) // Placeholder audio data
}
