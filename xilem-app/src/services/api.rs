// API client for communication with web-backend

use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::models::{Response, Session, Task, User};

const DEFAULT_API_URL: &str = "http://localhost:3000/api";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
            auth_token: None,
        })
    }

    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    pub fn clear_auth_token(&mut self) {
        self.auth_token = None;
    }

    fn auth_header(&self) -> Option<String> {
        self.auth_token.as_ref().map(|t| format!("Bearer {}", t))
    }

    // Authentication endpoints
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse> {
        let url = format!("{}/auth/register", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Registration failed: {}",
                response.text().await?
            ))
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse> {
        let url = format!("{}/auth/login", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Login failed: {}", response.text().await?))
        }
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse> {
        let url = format!("{}/auth/refresh", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "refresh_token": refresh_token
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Token refresh failed: {}",
                response.text().await?
            ))
        }
    }

    pub async fn logout(&self) -> Result<()> {
        if let Some(token) = &self.auth_token {
            let url = format!("{}/auth/logout", self.base_url);
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            if !response.status().is_success() {
                tracing::warn!("Logout failed: {}", response.text().await?);
            }
        }
        Ok(())
    }

    // Session management
    pub async fn create_session(&self, request: CreateSessionRequest) -> Result<SessionResponse> {
        let url = format!("{}/sessions", self.base_url);
        let mut req = self.client.post(&url).json(&request);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to create session: {}",
                response.text().await?
            ))
        }
    }

    pub async fn get_session(&self, session_id: Uuid) -> Result<Session> {
        let url = format!("{}/sessions/{}", self.base_url, session_id);
        let mut req = self.client.get(&url);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to get session: {}",
                response.text().await?
            ))
        }
    }

    pub async fn submit_response(&self, session_id: Uuid, response: ResponseSubmission) -> Result<FeedbackResponse> {
        let url = format!("{}/sessions/{}/responses", self.base_url, session_id);
        let mut req = self.client.post(&url).json(&response);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to submit response: {}",
                response.text().await?
            ))
        }
    }

    pub async fn complete_session(&self, session_id: Uuid) -> Result<SessionSummary> {
        let url = format!("{}/sessions/{}/complete", self.base_url, session_id);
        let mut req = self.client.post(&url);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to complete session: {}",
                response.text().await?
            ))
        }
    }

    // Task generation
    pub async fn generate_task(&self, request: GenerateTaskRequest) -> Result<Task> {
        let url = format!("{}/tasks/generate", self.base_url);
        let mut req = self.client.get(&url).query(&request);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to generate task: {}",
                response.text().await?
            ))
        }
    }

    pub async fn get_hint(&self, request: HintRequest) -> Result<HintResponse> {
        let url = format!("{}/tasks/hint", self.base_url);
        let mut req = self.client.get(&url).query(&request);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to get hint: {}",
                response.text().await?
            ))
        }
    }

    // Learner management
    pub async fn create_learner(&self, request: CreateLearnerRequest) -> Result<LearnerResponse> {
        let url = format!("{}/learners", self.base_url);
        let mut req = self.client.post(&url).json(&request);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to create learner: {}",
                response.text().await?
            ))
        }
    }

    pub async fn get_learner_stats(&self, learner_id: Uuid) -> Result<LearnerStats> {
        let url = format!("{}/learners/{}/stats", self.base_url, learner_id);
        let mut req = self.client.get(&url);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to get learner stats: {}",
                response.text().await?
            ))
        }
    }

    // Sync endpoints
    pub async fn sync_push(&self, responses: Vec<Response>) -> Result<SyncResponse> {
        let url = format!("{}/sync/push", self.base_url);
        let mut req = self.client.post(&url).json(&responses);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to sync push: {}",
                response.text().await?
            ))
        }
    }

    pub async fn sync_pull(&self) -> Result<SyncPullResponse> {
        let url = format!("{}/sync/pull", self.base_url);
        let mut req = self.client.get(&url);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to sync pull: {}",
                response.text().await?
            ))
        }
    }

    // Gamification
    pub async fn get_gamification_profile(&self) -> Result<GamificationProfile> {
        let url = format!("{}/gamification/profile", self.base_url);
        let mut req = self.client.get(&url);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to get gamification profile: {}",
                response.text().await?
            ))
        }
    }

    pub async fn get_leaderboard(&self) -> Result<LeaderboardResponse> {
        let url = format!("{}/gamification/leaderboard", self.base_url);
        let mut req = self.client.get(&url);

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!(
                "Failed to get leaderboard: {}",
                response.text().await?
            ))
        }
    }
}

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub learner_id: Uuid,
    pub topology_type: String,
    pub topology_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session: Session,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseSubmission {
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: Option<String>,
    pub response_time_ms: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackResponse {
    pub correct: bool,
    pub feedback: String,
    pub next_task: Option<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: Uuid,
    pub accuracy: f64,
    pub mean_response_time_ms: f64,
    pub tasks_completed: usize,
    pub xp_gained: i32,
    pub achievements_unlocked: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTaskRequest {
    pub learner_id: Uuid,
    pub topology: String,
    pub difficulty: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HintRequest {
    pub learner_id: Uuid,
    pub task_type: String,
    pub hint_level: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HintResponse {
    pub hint: String,
    pub hint_level: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLearnerRequest {
    pub display_name: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearnerResponse {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearnerStats {
    pub total_sessions: i32,
    pub total_tasks_completed: i32,
    pub overall_accuracy: f64,
    pub total_practice_time_seconds: i64,
    pub last_active: Option<DateTime<Utc>>,
    pub preferred_difficulty: f64,
    pub learning_curve: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub synced_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPullResponse {
    pub updates: Vec<serde_json::Value>,
    pub last_sync: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GamificationProfile {
    pub level: i32,
    pub xp: i32,
    pub xp_to_next_level: i32,
    pub achievements: Vec<Achievement>,
    pub current_streak: i32,
    pub longest_streak: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub weekly: Vec<LeaderboardEntry>,
    pub all_time: Vec<LeaderboardEntry>,
    pub user_rank: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i32,
    pub username: String,
    pub score: i32,
    pub level: i32,
}