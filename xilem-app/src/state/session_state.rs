// Session state management

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{PendingResponse, ResponseMetrics, Task};

#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: Uuid,
    pub learner_id: Uuid,
    pub topology: abcdeez_core::core::Topology,
    pub start_time: DateTime<Utc>,
    pub tasks_completed: usize,
    pub current_task: Option<Task>,
    pub pending_response: Option<PendingResponse>,
    pub hint_level: u32,
    pub struggle_indicators: StruggleState,
    pub performance_buffer: Vec<ResponseMetrics>,
}

#[derive(Debug, Clone)]
pub struct StruggleState {
    pub consecutive_errors: u32,
    pub response_time_ms: u128,
    pub hint_requested: bool,
    pub difficulty_adjustment_needed: bool,
}

impl Default for StruggleState {
    fn default() -> Self {
        Self {
            consecutive_errors: 0,
            response_time_ms: 0,
            hint_requested: false,
            difficulty_adjustment_needed: false,
        }
    }
}