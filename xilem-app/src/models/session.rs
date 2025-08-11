// Session models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub learner_id: Uuid,
    pub topology_type: String,
    pub topology_data: serde_json::Value,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String, // "active", "completed", "abandoned"
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Abandoned,
}

impl SessionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SessionStatus::Active => "active",
            SessionStatus::Completed => "completed",
            SessionStatus::Abandoned => "abandoned",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => SessionStatus::Active,
            "completed" => SessionStatus::Completed,
            "abandoned" => SessionStatus::Abandoned,
            _ => SessionStatus::Abandoned,
        }
    }
}