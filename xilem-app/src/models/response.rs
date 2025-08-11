// Response models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: i32,
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: Option<String>,
    pub correct: bool,
    pub response_time_ms: u128,
    pub hint_level: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: i32,
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: Option<String>,
    pub correct: bool,
    pub response_time_ms: u128,
    pub hint_level: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    pub correct: bool,
    pub response_time_ms: u128,
    pub timestamp: DateTime<Utc>,
    pub difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub session_id: Uuid,
    pub task_type: String,
    pub response: String,
    pub is_correct: bool,
    pub response_time_ms: u128,
    pub timestamp: DateTime<Utc>,
}