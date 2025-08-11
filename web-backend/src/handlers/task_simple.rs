use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    handlers::common::ValidationHelper,
    middleware::Claims,
    state::AppState,
};

use abcdeez_core::{
    core::Topology,
    tasks::core::TaskGenerator,
};

// Task difficulty constants
const DIFFICULTY_PAIRWISE_ORDER: f64 = 0.3;
const DIFFICULTY_SUCCESSOR: f64 = 0.4;
const DIFFICULTY_PREDECESSOR: f64 = 0.4;
const DIFFICULTY_K_JUMP: f64 = 0.6;
const DIFFICULTY_SEGMENT: f64 = 0.5;
const DIFFICULTY_INDEX: f64 = 0.7;
const DEFAULT_RECOMMENDED_DIFFICULTY: f64 = 0.5;

/// Simple task generation endpoint that works with current Axum version
pub async fn generate_simple(
    State(_state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<SimpleTaskRequest>,
) -> AppResult<Json<SimpleTaskResponse>> {
    // Get topology type with validation
    let topology_type = params.topology_type.unwrap_or_else(|| "alphabet".to_string());
    ValidationHelper::validate_not_empty(&topology_type, "topology_type")?;

    // Create topology from type
    let topology = create_topology_from_string(&topology_type)?;
    let mut task_generator = TaskGenerator::new(topology);

    // Generate a task
    // The core API accepts an optional TaskType; difficulty is not used directly.
    let task = task_generator.generate_task(None);

    let response = SimpleTaskResponse {
        task_type: format!("{:?}", task.task_type),
        prompt: task.prompt,
        options: task.options,
        correct_answer: task.correct_answer,
        difficulty: task.difficulty,
        operation: format!("{:?}", task.operation),
    };

    Ok(Json(response))
}

/// Get task difficulty analysis
pub async fn get_difficulty(
    State(_state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<DifficultyQuery>,
) -> AppResult<Json<DifficultyResponse>> {
    // Base difficulties for different task types using constants
    let task_type_difficulties = get_task_difficulty_map();

    let response = DifficultyResponse {
        task_type_difficulties,
        recommended_difficulty: params.learner_id.map(|_| DEFAULT_RECOMMENDED_DIFFICULTY),
    };

    Ok(Json(response))
}

/// Simple hint generation endpoint
pub async fn generate_hint(
    State(_state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<HintRequest>,
) -> AppResult<Json<HintResponse>> {
    ValidationHelper::validate_not_empty(&params.task_type, "task_type")?;

    let hint_text = get_hint_for_task_type(&params.task_type);

    let response = HintResponse {
        hint_text,
        hint_level: params.hint_level.unwrap_or(1),
        hint_type: "conceptual".to_string(),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

// Helper functions
fn get_task_difficulty_map() -> HashMap<String, f64> {
    let mut task_type_difficulties = HashMap::new();
    task_type_difficulties.insert("PairwiseOrder".to_string(), DIFFICULTY_PAIRWISE_ORDER);
    task_type_difficulties.insert("Successor".to_string(), DIFFICULTY_SUCCESSOR);
    task_type_difficulties.insert("Predecessor".to_string(), DIFFICULTY_PREDECESSOR);
    task_type_difficulties.insert("KJump".to_string(), DIFFICULTY_K_JUMP);
    task_type_difficulties.insert("Segment".to_string(), DIFFICULTY_SEGMENT);
    task_type_difficulties.insert("Index".to_string(), DIFFICULTY_INDEX);
    task_type_difficulties
}

fn get_hint_for_task_type(task_type: &str) -> String {
    match task_type {
        "PairwiseOrder" => "Think about the alphabetical order of the letters.",
        "Successor" => "What letter comes immediately after this one in the alphabet?",
        "Predecessor" => "What letter comes immediately before this one in the alphabet?",
        "KJump" => "Count the specified number of positions from the starting letter.",
        "Segment" => "List the letters in sequence starting from the given position.",
        "Index" => "Count the position of this letter in the alphabet (A=1, B=2, etc.).",
        _ => "Break down the problem into smaller steps.",
    }
    .to_string()
}

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
