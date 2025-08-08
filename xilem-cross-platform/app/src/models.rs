use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use graph_learning_core::{
    LearnerModel as CoreLearnerModel,
    LearnerMetrics as CoreLearnerMetrics,
    Task as CoreTask,
    TaskType as CoreTaskType,
    tasks::TaskResponse as CoreTaskResponse,
    OperationType,
    Topology,
    TopologyType,
};

// UI-specific wrapper types that bridge between the core library and the UI

// Core data structures for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: Option<String>,
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
    pub recent_accuracy: f64,  // Last 10 responses
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
        self.average_response_time_ms = 
            (old_avg * (self.total_responses - 1) as f64 + response_time_ms as f64) 
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
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub learner_id: String,
    pub topology_type: String,
    pub topology_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitResponseRequest {
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: String,
    pub correct: bool,
    pub response_time_ms: i32,
}

// Export data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub learner: Learner,
    pub sessions: Vec<Session>,
    pub metrics: PerformanceMetrics,
    pub export_time: DateTime<Utc>,
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