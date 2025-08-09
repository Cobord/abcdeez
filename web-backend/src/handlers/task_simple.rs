use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::{learner::Learner, session::Session},
    services::{adaptation_service::AdaptationService, learner_service::LearnerService},
    state::AppState,
};

use graph_learning_core::{
    bayesian::BayesianLearnerModel, learner::LearnerModel, Task, TaskGenerator, TaskType, Topology,
};

/// Simple task generation endpoint that works with current Axum version
pub async fn generate_simple(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<SimpleTaskRequest>,
) -> AppResult<Json<SimpleTaskResponse>> {
    // Get topology type
    let topology_type = params
        .topology_type
        .unwrap_or_else(|| "alphabet".to_string());

    // Create topology from type
    let topology = create_topology_from_string(&topology_type)?;
    let mut task_generator = TaskGenerator::new(topology.clone());

    // Generate a task
    let task = task_generator.generate_task(None);

    Ok(Json(SimpleTaskResponse {
        task_type: format!("{:?}", task.task_type),
        prompt: task.prompt,
        options: task.options,
        correct_answer: task.correct_answer,
        difficulty: task.difficulty,
        operation: format!("{:?}", task.operation),
    }))
}

/// Get task difficulty analysis
pub async fn get_difficulty(
    State(_state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<DifficultyQuery>,
) -> AppResult<Json<DifficultyResponse>> {
    // Base difficulties for different task types
    let mut task_type_difficulties = std::collections::HashMap::new();
    task_type_difficulties.insert("PairwiseOrder".to_string(), 0.3);
    task_type_difficulties.insert("Successor".to_string(), 0.4);
    task_type_difficulties.insert("Predecessor".to_string(), 0.4);
    task_type_difficulties.insert("KJump".to_string(), 0.6);
    task_type_difficulties.insert("Segment".to_string(), 0.5);
    task_type_difficulties.insert("Index".to_string(), 0.7);

    Ok(Json(DifficultyResponse {
        task_type_difficulties,
        recommended_difficulty: params.learner_id.map(|_| 0.5),
    }))
}

/// Simple hint generation endpoint
pub async fn generate_hint(
    State(_state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<HintRequest>,
) -> AppResult<Json<HintResponse>> {
    let hint_text = match params.task_type.as_str() {
        "PairwiseOrder" => "Think about the alphabetical order of the letters.",
        "Successor" => "What letter comes immediately after this one in the alphabet?",
        "Predecessor" => "What letter comes immediately before this one in the alphabet?",
        "KJump" => "Count the specified number of positions from the starting letter.",
        "Segment" => "List the letters in sequence starting from the given position.",
        "Index" => "Count the position of this letter in the alphabet (A=1, B=2, etc.).",
        _ => "Break down the problem into smaller steps.",
    };

    Ok(Json(HintResponse {
        hint_text: hint_text.to_string(),
        hint_level: 1,
        hint_type: "conceptual".to_string(),
        timestamp: chrono::Utc::now(),
    }))
}

// Helper functions
fn create_topology_from_string(topology_type: &str) -> AppResult<Topology> {
    match topology_type {
        "alphabet" => Ok(Topology::alphabet()),
        "numbers" => {
            let numbers: Vec<String> = (1..=26).map(|n| n.to_string()).collect();
            Ok(Topology::new_linear(numbers))
        }
        "days_of_week" => {
            let days = vec![
                "Monday".to_string(),
                "Tuesday".to_string(),
                "Wednesday".to_string(),
                "Thursday".to_string(),
                "Friday".to_string(),
                "Saturday".to_string(),
                "Sunday".to_string(),
            ];
            Ok(Topology::new_cyclic(days))
        }
        "music_notes" => {
            let notes = vec![
                "C".to_string(),
                "D".to_string(),
                "E".to_string(),
                "F".to_string(),
                "G".to_string(),
                "A".to_string(),
                "B".to_string(),
            ];
            Ok(Topology::new_linear(notes))
        }
        _ => Err(AppError::BadRequest(format!(
            "Unknown topology type: {}",
            topology_type
        ))),
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct SimpleTaskRequest {
    pub topology_type: Option<String>,
    pub difficulty: Option<f64>,
    pub learner_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct SimpleTaskResponse {
    pub task_type: String,
    pub prompt: String,
    pub options: Vec<String>,
    pub correct_answer: String,
    pub difficulty: f64,
    pub operation: String,
}

#[derive(Debug, Deserialize)]
pub struct DifficultyQuery {
    pub learner_id: Option<Uuid>,
    pub topology_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DifficultyResponse {
    pub task_type_difficulties: std::collections::HashMap<String, f64>,
    pub recommended_difficulty: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct HintRequest {
    pub task_type: String,
    pub current_item: Option<String>,
    pub hint_level: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct HintResponse {
    pub hint_text: String,
    pub hint_level: u8,
    pub hint_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
