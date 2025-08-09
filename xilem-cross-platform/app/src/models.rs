use chrono::{DateTime, Utc};
use graph_learning_core::{
    tasks::TaskResponse as CoreTaskResponse, LearnerMetrics as CoreLearnerMetrics,
    LearnerModel as CoreLearnerModel, Task as CoreTask, Topology, TopologyType,
};
use serde::{Deserialize, Serialize};

// UI-specific wrapper types that bridge between the core library and the UI

// Core data structures for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub apple_user_id: Option<String>,
    pub github_user_id: Option<String>,
    pub oauth_provider_id: Option<String>,
    pub auth_provider: String,
    pub is_private_email: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Wrapper for core LearnerModel with UI-specific fields
#[derive(Debug, Clone)]
pub struct Learner {
    pub id: String,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub core_model: CoreLearnerModel, // The actual learner model from the library
    pub metadata: Option<serde_json::Value>,
}

// Session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub learner_id: String,
    pub topology_type: String,
    pub topology: Option<Topology>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub summary: Option<serde_json::Value>,
    pub responses: Vec<CoreTaskResponse>,
}

// Domain types for UI
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    Alphabet,
    DaysOfWeek,
    Music,
    Mathematics,
    Custom(String),
}

impl Domain {
    pub fn as_str(&self) -> &str {
        match self {
            Domain::Alphabet => "alphabet",
            Domain::DaysOfWeek => "days_of_week",
            Domain::Music => "music",
            Domain::Mathematics => "mathematics",
            Domain::Custom(s) => s,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Domain::Alphabet => "Alphabet (A-Z)",
            Domain::DaysOfWeek => "Days of the Week",
            Domain::Music => "Music Theory",
            Domain::Mathematics => "Mathematics",
            Domain::Custom(s) => s,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Domain::Alphabet => "Learn letter positions, sequences, and relationships",
            Domain::DaysOfWeek => "Master the order and relationships between days",
            Domain::Music => "Understand intervals, scales, and chord progressions",
            Domain::Mathematics => "Practice arithmetic operations and number patterns",
            Domain::Custom(_) => "Custom learning domain",
        }
    }

    pub fn to_topology_type(&self) -> TopologyType {
        match self {
            Domain::Alphabet => TopologyType::Linear,
            Domain::DaysOfWeek => TopologyType::Cyclic,
            Domain::Music => TopologyType::PartialOrder,
            Domain::Mathematics => TopologyType::GeneralGraph,
            Domain::Custom(_) => TopologyType::Linear,
        }
    }
}

// UI-specific task wrapper that includes the core task
#[derive(Debug, Clone)]
pub struct UITask {
    pub core_task: CoreTask,
    pub display_prompt: String,
    pub display_options: Vec<String>,
    pub hint: Option<String>,
    pub feedback_message: Option<String>,
}

impl UITask {
    pub fn from_core_task(task: CoreTask) -> Self {
        let (display_prompt, display_options) = Self::format_task_for_ui(&task);

        UITask {
            core_task: task,
            display_prompt,
            display_options,
            hint: None,
            feedback_message: None,
        }
    }

    fn format_task_for_ui(task: &CoreTask) -> (String, Vec<String>) {
        // Format the task prompt and options based on task type
        let prompt = task.prompt.clone();
        let options = task.options.clone();

        (prompt, options)
    }
}

// Performance metrics with UI-specific fields
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub core_metrics: Option<CoreLearnerMetrics>,
    pub total_responses: usize,
    pub correct_responses: usize,
    pub average_response_time_ms: f64,
    pub accuracy_rate: f64,
    pub recent_accuracy: f64, // Last 10 responses
    pub improvement_rate: f64,
    pub streak_count: usize,
    pub best_streak: usize,
}

impl PerformanceMetrics {
    pub fn update(&mut self, correct: bool, response_time_ms: i32) {
        self.total_responses += 1;
        if correct {
            self.correct_responses += 1;
            self.streak_count += 1;
            if self.streak_count > self.best_streak {
                self.best_streak = self.streak_count;
            }
        } else {
            self.streak_count = 0;
        }

        // Update average response time
        let old_avg = self.average_response_time_ms;
        self.average_response_time_ms = (old_avg * (self.total_responses - 1) as f64
            + response_time_ms as f64)
            / self.total_responses as f64;

        // Update accuracy rate
        self.accuracy_rate = self.correct_responses as f64 / self.total_responses as f64;
    }

    pub fn update_from_core_metrics(&mut self, metrics: &CoreLearnerMetrics) {
        self.core_metrics = Some(metrics.clone());
    }
}

// Request/Response DTOs for API communication
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub learner_id: String,
    pub topology_type: String,
    pub topology_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponseRequest {
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: String,
    pub response_time_ms: i64,
}

// OAuth-related structures
#[derive(Debug, Serialize, Deserialize)]
pub struct AppleSignInRequest {
    pub identity_token: String,
    pub authorization_code: Option<String>,
    pub user_info: Option<AppleUserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleUserInfo {
    pub name: Option<AppleUserName>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleUserName {
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallbackRequest {
    pub provider: String,
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthAuthUrlResponse {
    pub authorization_url: String,
    pub state: String,
    pub provider: String,
}

// Export data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub learner: Learner,
    pub sessions: Vec<Session>,
    pub metrics: PerformanceMetrics,
    pub export_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_seed: Option<u64>,
}

// Serialization helpers for Learner (since it contains non-serializable CoreLearnerModel)
impl Serialize for Learner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Learner", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("display_name", &self.display_name)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Learner {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // For now, we can't deserialize a Learner with a CoreLearnerModel
        // This would need to be handled by reconstructing from stored data
        unimplemented!("Learner deserialization requires topology information")
    }
}

// Export format enum for data export functionality
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Replay,
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Json
    }
}

// Additional API models for integration

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLearnerRequest {
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponseResponse {
    pub response_id: String,
    pub sequence_number: u32,
    pub correct: bool,
    pub intervention: Option<serde_json::Value>,
    pub next_task: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub overall_accuracy: f64,
    pub average_response_time: f64,
    pub total_sessions: u32,
    pub total_tasks: u32,
    pub learning_metrics: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Paused,
    Cancelled,
}

// Update Session struct to use the enum
impl Session {
    pub fn new(id: String, learner_id: String) -> Self {
        Self {
            id,
            learner_id,
            topology_type: "linear".to_string(),
            topology: None,
            start_time: chrono::Utc::now(),
            end_time: None,
            status: "active".to_string(),
            summary: None,
            responses: Vec::new(),
        }
    }
}
