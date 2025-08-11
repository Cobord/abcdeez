// WebSocket client for real-time communication with the backend

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::models::Task;

pub struct WebSocketClient {
    url: String,
    sender_tx: Option<mpsc::UnboundedSender<ClientMessage>>,
    receiver_rx: Option<mpsc::UnboundedReceiver<ServerMessage>>,
    connected: Arc<Mutex<bool>>,
}

impl WebSocketClient {
    pub fn new(base_url: &str) -> Result<Self> {
        // Convert HTTP URL to WebSocket URL
        let ws_url = if base_url.starts_with("http://") {
            base_url.replace("http://", "ws://")
        } else if base_url.starts_with("https://") {
            base_url.replace("https://", "wss://")
        } else {
            format!("ws://{}", base_url)
        };

        Ok(Self {
            url: ws_url,
            sender_tx: None,
            receiver_rx: None,
            connected: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn connect(&mut self, session_id: Uuid, token: &str) -> Result<()> {
        let url = format!("{}/api/sessions/{}/live", self.url, session_id);
        
        // Create channels for bidirectional communication
        let (sender_tx, sender_rx) = mpsc::unbounded_channel::<ClientMessage>();
        let (receiver_tx, receiver_rx) = mpsc::unbounded_channel::<ServerMessage>();

        self.sender_tx = Some(sender_tx);
        self.receiver_rx = Some(receiver_rx);

        let connected = self.connected.clone();
        let token = token.to_string();

        // Spawn WebSocket connection task
        tokio::spawn(async move {
            if let Err(e) = websocket_task(url, token, sender_rx, receiver_tx, connected).await {
                tracing::error!("WebSocket task error: {}", e);
            }
        });

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(tx) = &self.sender_tx {
            let _ = tx.send(ClientMessage::Disconnect);
        }
        self.sender_tx = None;
        self.receiver_rx = None;
        *self.connected.lock().await = false;
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    pub async fn send_message(&self, message: ClientMessage) -> Result<()> {
        if let Some(tx) = &self.sender_tx {
            tx.send(message)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebSocket not connected"))
        }
    }

    pub async fn receive_message(&mut self) -> Option<ServerMessage> {
        if let Some(rx) = &mut self.receiver_rx {
            rx.recv().await
        } else {
            None
        }
    }

    pub async fn submit_response(
        &self,
        task_type: String,
        task_data: serde_json::Value,
        user_answer: Option<String>,
        response_time_ms: u128,
    ) -> Result<()> {
        self.send_message(ClientMessage::SubmitResponse {
            task_type,
            task_data,
            user_answer,
            response_time_ms,
        })
        .await
    }

    pub async fn request_hint(&self, hint_level: Option<String>) -> Result<()> {
        self.send_message(ClientMessage::RequestHint { hint_level }).await
    }

    pub async fn pause_session(&self) -> Result<()> {
        self.send_message(ClientMessage::Pause).await
    }

    pub async fn resume_session(&self) -> Result<()> {
        self.send_message(ClientMessage::Resume).await
    }

    pub async fn send_heartbeat(&self) -> Result<()> {
        self.send_message(ClientMessage::Heartbeat).await
    }
}

async fn websocket_task(
    url: String,
    token: String,
    mut sender_rx: mpsc::UnboundedReceiver<ClientMessage>,
    receiver_tx: mpsc::UnboundedSender<ServerMessage>,
    connected: Arc<Mutex<bool>>,
) -> Result<()> {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{StreamExt, SinkExt};

    let (ws_stream, _) = connect_async(&url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send authentication message
    let auth_msg = ClientMessage::Authenticate { token };
    let json = serde_json::to_string(&auth_msg)?;
    write.send(Message::Text(json.into())).await?;

    *connected.lock().await = true;
    tracing::info!("WebSocket connected and authenticated");

    // Create heartbeat task
    let connected_heartbeat = connected.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if !*connected_heartbeat.lock().await {
                break;
            }
            // Heartbeat is sent through the main sender channel
        }
    });

    loop {
        tokio::select! {
            // Handle outgoing messages
            Some(msg) = sender_rx.recv() => {
                match msg {
                    ClientMessage::Disconnect => {
                        tracing::info!("Disconnecting WebSocket");
                        break;
                    }
                    _ => {
                        let json = serde_json::to_string(&msg)?;
                        write.send(Message::Text(json.into())).await?;
                    }
                }
            }
            // Handle incoming messages
            Some(msg) = read.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(server_msg) => {
                                // Handle authentication result
                                if let ServerMessage::AuthenticationResult { success, .. } = &server_msg {
                                    if !success {
                                        tracing::error!("WebSocket authentication failed");
                                        break;
                                    }
                                }
                                let _ = receiver_tx.send(server_msg);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse server message: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("WebSocket closed by server");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        write.send(Message::Pong(data)).await?;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            else => break,
        }
    }

    *connected.lock().await = false;
    heartbeat_task.abort();
    tracing::info!("WebSocket disconnected");

    Ok(())
}

// Message types matching the backend protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { 
        token: String 
    },
    StartTask { 
        payload: serde_json::Value 
    },
    SubmitResponse {
        task_type: String,
        task_data: serde_json::Value,
        user_answer: Option<String>,
        response_time_ms: u128,
    },
    RequestHint { 
        hint_level: Option<String> 
    },
    Pause,
    Resume,
    Heartbeat,
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    AuthenticationResult {
        success: bool,
        message: String,
        user_id: Option<Uuid>,
        timestamp: i64,
    },
    Task {
        payload: TaskMessage,
        timestamp: i64,
    },
    Hint {
        payload: HintMessage,
        timestamp: i64,
    },
    Feedback {
        payload: FeedbackMessage,
        timestamp: i64,
    },
    Intervention {
        payload: InterventionMessage,
        timestamp: i64,
    },
    StatsUpdate {
        payload: StatsMessage,
        timestamp: i64,
    },
    Error {
        message: String,
        timestamp: i64,
    },
    Heartbeat {
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub task: Task,
    pub session_progress: f64,
    pub adaptive_params: AdaptiveParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveParams {
    pub difficulty: f64,
    pub estimated_success_rate: f64,
    pub information_gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HintMessage {
    pub hint_text: String,
    pub hint_level: u32,
    pub max_hint_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackMessage {
    pub correct: bool,
    pub correct_answer: String,
    pub explanation: Option<String>,
    pub performance_trend: PerformanceTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionMessage {
    pub intervention_type: InterventionType,
    pub message: String,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionType {
    EncouragementMessage,
    DifficultyAdjustment,
    BreakSuggestion,
    HintSuggestion,
    StrategyTip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsMessage {
    pub current_accuracy: f64,
    pub current_streak: i32,
    pub tasks_completed: i32,
    pub session_duration_seconds: i64,
    pub proficiency_update: Option<ProficiencyUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProficiencyUpdate {
    pub operation: String,
    pub old_proficiency: f64,
    pub new_proficiency: f64,
}

// Helper function to create a WebSocket client from app state
pub async fn create_websocket_client(base_url: &str) -> Result<WebSocketClient> {
    WebSocketClient::new(base_url)
}