use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: String, // Made non-optional as UI always provides this
    pub response_time_ms: i64,
    pub hint_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResponseRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: i32,
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: Option<String>,
    pub correct: bool,
    pub response_time_ms: i64,
    pub hint_level: Option<i32>,
    pub timestamp: DateTime<Utc>,
}
