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
    // Generate simple sine wave audio for the requested note
    tracing::debug!("Audio requested for note: {}", note);

    // Map note to frequency (simplified mapping)
    let frequency = match note.as_str() {
        "C" | "C4" => 261.63,
        "D" | "D4" => 293.66,
        "E" | "E4" => 329.63,
        "F" | "F4" => 349.23,
        "G" | "G4" => 392.00,
        "A" | "A4" => 440.00,
        "B" | "B4" => 493.88,
        _ => 440.00, // Default to A4
    };

    // Generate 1 second of audio at 44.1kHz sample rate
    let sample_rate = 44100.0;
    let duration = 1.0; // seconds
    let samples = (sample_rate * duration) as usize;

    let mut audio_data = Vec::with_capacity(samples * 2); // 16-bit audio

    for i in 0..samples {
        let t = i as f64 / sample_rate;
        let sample = (2.0 * std::f64::consts::PI * frequency * t).sin();
        let sample_i16 = (sample * 32767.0) as i16;

        // Convert to little-endian bytes
        audio_data.push((sample_i16 & 0xFF) as u8);
        audio_data.push((sample_i16 >> 8) as u8);
    }

    Ok(audio_data)
}
