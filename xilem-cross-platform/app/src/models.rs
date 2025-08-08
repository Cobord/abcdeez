use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Core data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learner {
    pub id: String,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub session_id: String,
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: Option<String>,
    pub correct: bool,
    pub response_time_ms: i32,
    pub timestamp: DateTime<Utc>,
}

// Domain types
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    Alphabet,
    Music,
    Mathematics,
    Custom(String),
}

impl Domain {
    pub fn as_str(&self) -> &str {
        match self {
            Domain::Alphabet => "alphabet",
            Domain::Music => "music",
            Domain::Mathematics => "mathematics",
            Domain::Custom(s) => s,
        }
    }
    
    pub fn display_name(&self) -> &str {
        match self {
            Domain::Alphabet => "Alphabet Recognition",
            Domain::Music => "Music Theory",
            Domain::Mathematics => "Mathematics",
            Domain::Custom(s) => s,
        }
    }
}

// Task types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphabetTask {
    pub letter: char,
    pub position: usize,
    pub options: Vec<char>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTask {
    pub task_type: String,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub difficulty: f64,
    pub musical_context: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum Task {
    Alphabet(AlphabetTask),
    Music(MusicTask),
    Custom(serde_json::Value),
}

// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_responses: usize,
    pub correct_responses: usize,
    pub average_response_time_ms: f64,
    pub accuracy_rate: f64,
    pub recent_accuracy: f64,  // Last 10 responses
    pub improvement_rate: f64,
}

impl PerformanceMetrics {
    pub fn update(&mut self, correct: bool, response_time_ms: i32) {
        self.total_responses += 1;
        if correct {
            self.correct_responses += 1;
        }
        
        // Update average response time
        let old_avg = self.average_response_time_ms;
        self.average_response_time_ms = 
            (old_avg * (self.total_responses - 1) as f64 + response_time_ms as f64) 
            / self.total_responses as f64;
        
        // Update accuracy rate
        self.accuracy_rate = self.correct_responses as f64 / self.total_responses as f64;
    }
}

// Request/Response DTOs
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
    pub responses: Vec<TaskResponse>,
    pub metrics: PerformanceMetrics,
    pub export_time: DateTime<Utc>,
}