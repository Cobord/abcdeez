use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Learner {
    pub id: String,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub learner_id: String,
    pub topology_type: String,
    pub topology_data: Option<serde_json::Value>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Response {
    pub id: String,
    pub session_id: String,
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: Option<String>,
    pub correct: bool,
    pub response_time_ms: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelSnapshot {
    pub id: String,
    pub learner_id: String,
    pub timestamp: DateTime<Utc>,
    pub parameters: serde_json::Value,
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct CreateLearnerRequest {
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub learner_id: String,
    pub topology_type: String,
    pub topology_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitResponseRequest {
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: String,
    pub correct: bool,
    pub response_time_ms: i32,
}

#[derive(Debug, Serialize)]
pub struct PopulationStats {
    pub total_learners: i64,
    pub total_sessions: i64,
    pub total_responses: i64,
    pub avg_accuracy: f64,
    pub avg_response_time_ms: f64,
    pub active_sessions: i64,
}

#[derive(Debug, Serialize)]
pub struct Bottleneck {
    pub from_task: String,
    pub to_task: String,
    pub error_rate: f64,
    pub sample_size: i64,
}

#[derive(Debug, Serialize)]
pub struct ItemDifficulty {
    pub task_type: String,
    pub empirical_difficulty: f64,
    pub sample_size: i64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
}

#[derive(Debug, Serialize)]
pub struct StrategyCluster {
    pub strategy_type: String,
    pub learner_ids: Vec<String>,
    pub avg_performance: f64,
}

#[derive(Debug, Deserialize)]
pub struct CompareLearnerRequest {
    pub learner_ids: Vec<String>,
    pub metric: String, // accuracy, speed, improvement_rate
}

#[derive(Debug, Serialize)]
pub struct LearnerComparison {
    pub learner_id: String,
    pub metric_value: f64,
    pub percentile: f64,
}

#[derive(Debug, Deserialize)]
pub struct PredictPerformanceRequest {
    pub learner_id: String,
    pub future_trials: i32,
}

#[derive(Debug, Serialize)]
pub struct PerformancePrediction {
    pub trial_number: i32,
    pub predicted_accuracy: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
}

#[derive(Debug, Deserialize)]
pub struct GenerateScheduleRequest {
    pub learner_id: String,
    pub target_mastery: f64,
    pub max_sessions: i32,
}

#[derive(Debug, Serialize)]
pub struct TrainingSchedule {
    pub session_number: i32,
    pub recommended_tasks: Vec<String>,
    pub expected_improvement: f64,
}

// Music domain specific
#[derive(Debug, Serialize)]
pub struct MusicScale {
    pub name: String,
    pub root: String,
    pub scale_type: String,
    pub notes: Vec<String>,
    pub intervals: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateMusicTaskRequest {
    pub task_type: String, // interval, scale_degree, chord_progression, transposition
    pub difficulty: f64,
    pub scale_context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MusicTask {
    pub id: String,
    pub task_type: String,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub difficulty: f64,
    pub musical_context: serde_json::Value,
}