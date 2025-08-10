use chrono::{DateTime, Utc};
use abcdeez_core::LearnerModel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learner {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub total_practice_time_seconds: i64,
    pub metadata: Option<serde_json::Value>,
    // Integration with core learning model
    pub learning_model: LearnerModel,
}

impl Learner {
    pub fn new(
        id: Uuid,
        user_id: Option<Uuid>,
        display_name: Option<String>,
        learning_model: LearnerModel,
    ) -> Self {
        Self {
            id,
            user_id,
            display_name,
            created_at: Utc::now(),
            last_active: Some(Utc::now()),
            total_practice_time_seconds: 0,
            metadata: None,
            learning_model,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLearnerRequest {
    pub display_name: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLearnerRequest {
    pub display_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct LearnerStats {
    pub total_sessions: i64,
    pub total_tasks_completed: i64,
    pub overall_accuracy: f64,
    pub total_practice_time_seconds: i64,
    pub last_active: Option<DateTime<Utc>>,
    pub preferred_difficulty: f64,
    pub learning_curve: Vec<(DateTime<Utc>, f64)>,
}
