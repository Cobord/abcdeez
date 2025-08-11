// User state management

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserState {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub learner_id: Option<Uuid>,
    
    // Gamification fields
    pub level: u32,
    pub xp: u32,
    pub streak: u32,
    pub achievements: Vec<String>,
}