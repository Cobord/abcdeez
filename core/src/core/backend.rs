use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{
    data::export::LearnerDataExport,
    learning::learner::LearnerMetrics,
    tasks::core::{Task, TaskResponse},
};

/// Configuration for backend connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BackendConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub experiment_id: String,
    pub batch_size: usize,
    pub sync_interval_secs: u64,
    pub retry_attempts: u32,
    pub timeout_secs: u64,
}

impl Default for BackendConfig {
    fn default() -> Self {
        BackendConfig {
            api_url: "https://api.example.com/v1".to_string(),
            api_key: None,
            experiment_id: "default_experiment".to_string(),
            batch_size: 50,
            sync_interval_secs: 60,
            retry_attempts: 3,
            timeout_secs: 30,
        }
    }
}

impl BackendConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(url) = std::env::var("LEARNING_API_URL") {
            config.api_url = url;
        }
        if let Ok(key) = std::env::var("LEARNING_API_KEY") {
            config.api_key = Some(key);
        }
        if let Ok(id) = std::env::var("EXPERIMENT_ID") {
            config.experiment_id = id;
        }

        config
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).context("Failed to read backend config file")?;

        // Try JSON first, then TOML
        serde_json::from_str(&content)
            .or_else(|_| toml::from_str(&content))
            .context("Failed to parse backend config")
    }
}

/// Client for communicating with the backend
#[cfg(not(target_arch = "wasm32"))]
pub struct BackendClient {
    config: BackendConfig,
    client: reqwest::blocking::Client,
    response_buffer: Vec<TaskResponse>,
    last_sync: DateTime<Utc>,
}

#[cfg(not(target_arch = "wasm32"))]
impl BackendClient {
    pub fn new(config: BackendConfig) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(BackendClient {
            config,
            client,
            response_buffer: Vec::new(),
            last_sync: Utc::now(),
        })
    }

    /// Test connection to backend
    pub fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/health", self.config.api_url);
        let request = self.build_request(self.client.get(&url))?;
        let response = request.send()?;
        Ok(response.status().is_success())
    }

    fn build_request(&self, mut request: reqwest::blocking::RequestBuilder) -> Result<reqwest::blocking::RequestBuilder> {
        if let Some(key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        Ok(request)
    }

    /// Register a new participant
    pub fn register_participant(&self, participant_id: &str, group: &str) -> Result<SessionToken> {
        let url = format!("{}/participants", self.config.api_url);

        let registration = ParticipantRegistration {
            participant_id: participant_id.to_string(),
            experiment_id: self.config.experiment_id.clone(),
            group: group.to_string(),
            timestamp: Utc::now(),
            metadata: Default::default(),
        };

        let request = self.build_request(self.client.post(&url).json(&registration))?;
        let response = request.send()?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to register participant: {}", response.status());
        }

        response.json().context("Failed to parse session token")
    }

    /// Start a new session
    pub fn start_session(&self, token: &SessionToken) -> Result<String> {
        let url = format!("{}/sessions/start", self.config.api_url);

        let session_start = SessionStart {
            token: token.token.clone(),
            timestamp: Utc::now(),
        };

        let request = self.build_request(self.client.post(&url).json(&session_start))?;
        let response = request.send()?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to start session: {}", response.status());
        }

        let session_response: SessionResponse = response.json().context("Failed to parse session response")?;
        Ok(session_response.session_id)
    }

    /// Buffer a task response for later submission
    pub fn buffer_response(&mut self, response: TaskResponse) {
        self.response_buffer.push(response);

        // Auto-sync if buffer is full
        if self.response_buffer.len() >= self.config.batch_size {
            if let Err(e) = self.sync_responses() {
                eprintln!("Warning: Failed to sync responses: {}", e);
            }
        }
    }

    /// Sync buffered responses to backend
    pub fn sync_responses(&mut self) -> Result<()> {
        if self.response_buffer.is_empty() {
            return Ok(());
        }

        let url = format!("{}/responses/batch", self.config.api_url);

        let batch = ResponseBatch {
            experiment_id: self.config.experiment_id.clone(),
            responses: self.response_buffer.clone(),
            timestamp: Utc::now(),
        };

        // Retry logic
        let mut last_error = None;
        for attempt in 0..self.config.retry_attempts {
            if attempt > 0 {
                std::thread::sleep(Duration::from_secs(2_u64.pow(attempt)));
            }

            let request = match self.build_request(self.client.post(&url).json(&batch)) {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(e.to_string());
                    continue;
                }
            };

            match request.send() {
                Ok(response) if response.status().is_success() => {
                    self.response_buffer.clear();
                    self.last_sync = Utc::now();
                    return Ok(());
                }
                Ok(response) => {
                    last_error = Some(format!("HTTP {}", response.status()));
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        }

        anyhow::bail!(
            "Failed to sync after {} attempts: {}",
            self.config.retry_attempts,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )
    }

    /// Check if sync is needed based on time or buffer size
    pub fn should_sync(&self) -> bool {
        let time_since_sync = Utc::now() - self.last_sync;
        time_since_sync.num_seconds() as u64 >= self.config.sync_interval_secs
            || self.response_buffer.len() >= self.config.batch_size
    }

    /// Upload complete session data
    pub fn upload_session(&self, data: &LearnerDataExport) -> Result<()> {
        let url = format!("{}/sessions/complete", self.config.api_url);
        let request = self.build_request(self.client.post(&url).json(&data))?;
        let response = request.send()?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to upload session: {}", response.status());
        }

        Ok(())
    }

    /// Send real-time metrics update
    pub fn send_metrics(&self, participant_id: &str, metrics: &LearnerMetrics) -> Result<()> {
        let url = format!("{}/metrics", self.config.api_url);

        let update = MetricsUpdate {
            participant_id: participant_id.to_string(),
            experiment_id: self.config.experiment_id.clone(),
            metrics: metrics.clone(),
            timestamp: Utc::now(),
        };

        if let Ok(request) = self.build_request(self.client.post(&url).json(&update)) {
            let _ = request.send();
        }

        Ok(())
    }

    /// Report an error or issue
    pub fn report_error(&self, participant_id: &str, error: &str, context: Option<String>) {
        let url = format!("{}/errors", self.config.api_url);

        let error_report = ErrorReport {
            participant_id: participant_id.to_string(),
            experiment_id: self.config.experiment_id.clone(),
            error: error.to_string(),
            context,
            timestamp: Utc::now(),
        };

        if let Ok(request) = self.build_request(self.client.post(&url).json(&error_report)) {
            let _ = request.send();
        }
    }

    /// Get pending tasks from backend (for yoked control)
    pub fn get_next_task(&self, participant_id: &str) -> Result<Option<Task>> {
        let url = format!("{}/tasks/next", self.config.api_url);

        let request_body = TaskRequest {
            participant_id: participant_id.to_string(),
            experiment_id: self.config.experiment_id.clone(),
        };

        let request = self.build_request(self.client.post(&url).json(&request_body))?;
        let response = request.send()?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        if !response.status().is_success() {
            anyhow::bail!("Failed to get next task: {}", response.status());
        }

        let task: Task = response.json().context("Failed to parse task")?;
        Ok(Some(task))
    }

    /// Get experiment configuration from backend
    pub fn get_experiment_config(&self) -> Result<ExperimentConfig> {
        let url = format!(
            "{}/experiments/{}",
            self.config.api_url, self.config.experiment_id
        );

        let request = self.build_request(self.client.get(&url))?;
        let response = request.send()?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get experiment config: {}", response.status());
        }

        response.json().context("Failed to parse experiment config")
    }
}

// Request/Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticipantRegistration {
    participant_id: String,
    experiment_id: String,
    group: String,
    timestamp: DateTime<Utc>,
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionStart {
    token: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionResponse {
    session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResponseBatch {
    experiment_id: String,
    responses: Vec<TaskResponse>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricsUpdate {
    participant_id: String,
    experiment_id: String,
    metrics: LearnerMetrics,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorReport {
    participant_id: String,
    experiment_id: String,
    error: String,
    context: Option<String>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskRequest {
    participant_id: String,
    experiment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub experiment_id: String,
    pub name: String,
    pub description: String,
    pub groups: Vec<String>,
    pub trials_per_session: usize,
    pub sessions: usize,
    pub task_types: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

// Async version using tokio (optional)
#[cfg(all(feature = "async", not(target_arch = "wasm32")))]
pub mod async_client {
    use super::*;
    use tokio::time::sleep;

    pub struct AsyncBackendClient {
        config: BackendConfig,
        client: reqwest::Client,
        response_buffer: tokio::sync::Mutex<Vec<TaskResponse>>,
        last_sync: tokio::sync::Mutex<DateTime<Utc>>,
    }

    impl AsyncBackendClient {
        pub fn new(config: BackendConfig) -> Result<Self> {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .build()
                .context("Failed to create HTTP client")?;

            Ok(AsyncBackendClient {
                config,
                client,
                response_buffer: tokio::sync::Mutex::new(Vec::new()),
                last_sync: tokio::sync::Mutex::new(Utc::now()),
            })
        }

        pub async fn test_connection(&self) -> Result<bool> {
            let url = format!("{}/health", self.config.api_url);

            let mut request = self.client.get(&url);
            if let Some(key) = &self.config.api_key {
                request = request.header("Authorization", format!("Bearer {}", key));
            }

            let response = request.send().await?;
            Ok(response.status().is_success())
        }

        pub async fn buffer_response(&self, response: TaskResponse) {
            let mut buffer = self.response_buffer.lock().await;
            buffer.push(response);

            if buffer.len() >= self.config.batch_size {
                drop(buffer); // Release lock before sync
                if let Err(e) = self.sync_responses().await {
                    eprintln!("Warning: Failed to sync responses: {}", e);
                }
            }
        }

        pub async fn sync_responses(&self) -> Result<()> {
            let mut buffer = self.response_buffer.lock().await;
            if buffer.is_empty() {
                return Ok(());
            }

            let url = format!("{}/responses/batch", self.config.api_url);

            let batch = ResponseBatch {
                experiment_id: self.config.experiment_id.clone(),
                responses: buffer.clone(),
                timestamp: Utc::now(),
            };

            // Retry logic with exponential backoff
            let mut last_error = None;
            for attempt in 0..self.config.retry_attempts {
                if attempt > 0 {
                    sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                }

                let mut request = self.client.post(&url).json(&batch);
                if let Some(key) = &self.config.api_key {
                    request = request.header("Authorization", format!("Bearer {}", key));
                }

                match request.send().await {
                    Ok(response) if response.status().is_success() => {
                        buffer.clear();
                        *self.last_sync.lock().await = Utc::now();
                        return Ok(());
                    }
                    Ok(response) => {
                        last_error = Some(format!("HTTP {}", response.status()));
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                    }
                }
            }

            anyhow::bail!(
                "Failed to sync after {} attempts: {}",
                self.config.retry_attempts,
                last_error.unwrap_or_else(|| "Unknown error".to_string())
            )
        }

        pub async fn start_background_sync(self: std::sync::Arc<Self>) {
            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_secs(self.config.sync_interval_secs)).await;
                    if let Err(e) = self.sync_responses().await {
                        eprintln!("Background sync failed: {}", e);
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        std::env::set_var("LEARNING_API_URL", "https://test.com");
        std::env::set_var("LEARNING_API_KEY", "test_key");
        std::env::set_var("EXPERIMENT_ID", "exp_001");

        let config = BackendConfig::from_env();
        assert_eq!(config.api_url, "https://test.com");
        assert_eq!(config.api_key, Some("test_key".to_string()));
        assert_eq!(config.experiment_id, "exp_001");

        std::env::remove_var("LEARNING_API_URL");
        std::env::remove_var("LEARNING_API_KEY");
        std::env::remove_var("EXPERIMENT_ID");
    }
}
