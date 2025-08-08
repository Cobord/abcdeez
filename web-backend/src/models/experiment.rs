use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Experiment {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_by: Option<Uuid>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExperimentParticipant {
    pub experiment_id: Uuid,
    pub learner_id: Uuid,
    pub condition: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Draft,
    Active,
    Completed,
    Archived,
}

impl ToString for ExperimentStatus {
    fn to_string(&self) -> String {
        match self {
            ExperimentStatus::Draft => "draft".to_string(),
            ExperimentStatus::Active => "active".to_string(),
            ExperimentStatus::Completed => "completed".to_string(),
            ExperimentStatus::Archived => "archived".to_string(),
        }
    }
}