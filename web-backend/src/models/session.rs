use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use graph_learning_core::{TopologyType};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub learner_id: Uuid,
    pub topology_type: String,
    pub topology_data: serde_json::Value,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub learner_id: Uuid,
    pub topology_type: String, // Accept as string from UI
    pub topology_data: Option<serde_json::Value>, // Accept optional topology data from UI
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Completed,
    Abandoned,
}

impl ToString for SessionStatus {
    fn to_string(&self) -> String {
        match self {
            SessionStatus::Active => "active".to_string(),
            SessionStatus::Completed => "completed".to_string(),
            SessionStatus::Abandoned => "abandoned".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionSummary {
    pub total_tasks: i64,
    pub correct_responses: i64,
    pub accuracy: f64,
    pub average_response_time_ms: f64,
    pub duration_seconds: i64,
    pub strategy_distribution: serde_json::Value,
}