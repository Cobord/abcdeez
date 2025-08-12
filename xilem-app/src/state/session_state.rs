// Session state management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{PendingResponse, ResponseMetrics, Task};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: Uuid,
    pub learner_id: Uuid,
    pub topology: abcdeez_core::core::Topology,
    pub domain: Option<String>, // Domain name (e.g., "alphabet", "numbers")
    pub start_time: DateTime<Utc>,
    pub tasks_completed: usize,
    pub current_task: Option<Task>,
    pub pending_response: Option<PendingResponse>,
    pub hint_level: u32,
    pub struggle_indicators: StruggleState,
    pub performance_buffer: Vec<ResponseMetrics>,
    pub responses_total: u32,
    pub responses_correct: u32,
    pub total_hints_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StruggleState {
    pub consecutive_errors: u32,
    pub consecutive_correct: u32,
    pub response_time_ms: u128,
    pub hint_requested: bool,
    pub difficulty_adjustment_needed: bool,
}

impl Default for StruggleState {
    fn default() -> Self {
        Self {
            consecutive_errors: 0,
            consecutive_correct: 0,
            response_time_ms: 0,
            hint_requested: false,
            difficulty_adjustment_needed: false,
        }
    }
}