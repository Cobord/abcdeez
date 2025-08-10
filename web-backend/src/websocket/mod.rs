use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, Request, State,
    },
    http::StatusCode,
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::{
    middleware::Claims,
    services::{AdaptationService, AnalyticsService, LearnerService},
    state::AppState,
};
use abcdeez_core::{
    hints::{HintLevel, InterventionAction},
    Task,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    // Authentication must be the first message sent
    Authenticate {
        token: String,
    },
    StartTask {
        payload: serde_json::Value,
    },
    SubmitResponse {
        task_type: String,
        task_data: serde_json::Value,
        user_answer: Option<String>,
        response_time_ms: i64,
    },
    RequestHint {
        hint_level: Option<String>,
    },
    Pause,
    Resume,
    Heartbeat,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // Authentication response
    AuthenticationResult {
        success: bool,
        message: String,
        user_id: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskMessage {
    pub task: Task,
    pub difficulty: f64,
    pub expected_duration_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HintMessage {
    pub hint_text: String,
    pub hint_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackMessage {
    pub correct: bool,
    pub expected_answer: Option<String>,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterventionMessage {
    pub intervention_type: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsMessage {
    pub session_accuracy: f64,
    pub recent_accuracy: f64,
    pub response_count: usize,
    pub average_response_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveMetrics {
    pub active_learners: usize,
    pub tasks_per_minute: f64,
    pub average_accuracy: f64,
    pub difficulty_distribution: serde_json::Value,
    pub strategy_distribution: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    // Token removed from query parameters for security
    // Authentication will be handled via first message or subprotocol
}

pub async fn session_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Query(_query): Query<WebSocketQuery>,
    request: Request,
) -> Result<Response, (StatusCode, String)> {
    // Extract correlation ID from request extensions if available
    let correlation_id = request
        .extensions()
        .get::<crate::middleware::CorrelationId>()
        .map(|c| c.0.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Authentication will be handled after WebSocket connection is established
    // This is more secure than passing tokens in query parameters

    Ok(ws.on_upgrade(move |socket| {
        handle_session_socket_with_correlation(socket, state, session_id, correlation_id)
    }))
}

pub async fn analytics_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(_query): Query<WebSocketQuery>,
) -> Result<Response, (StatusCode, String)> {
    // Authentication will be handled after WebSocket connection is established
    // This is more secure than passing tokens in query parameters

    Ok(ws.on_upgrade(move |socket| handle_analytics_socket(socket, state)))
}

async fn handle_session_socket_with_correlation(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    correlation_id: String,
) {
    tracing::info!(
        correlation_id = %correlation_id,
        session_id = %session_id,
        "WebSocket session connection established"
    );

    handle_session_socket_internal(socket, state, session_id, Some(correlation_id)).await;
}

// Keep the original function for backward compatibility
async fn handle_session_socket(socket: WebSocket, state: Arc<AppState>, session_id: Uuid) {
    handle_session_socket_internal(socket, state, session_id, None).await;
}

async fn handle_session_socket_internal(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    correlation_id: Option<String>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for authentication message
    let authenticated_claims = match authenticate_websocket(&mut sender, &mut receiver, &state)
        .await
    {
        Ok(claims) => claims,
        Err(e) => {
            if let Some(ref corr_id) = correlation_id {
                tracing::warn!(correlation_id = %corr_id, "WebSocket authentication failed: {}", e);
            } else {
                tracing::warn!("WebSocket authentication failed: {}", e);
            }
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::AuthenticationResult {
                        success: false,
                        message: "Authentication failed".to_string(),
                        user_id: None,
                        timestamp: chrono::Utc::now().timestamp(),
                    })
                    .unwrap(),
                ))
                .await;
            let _ = sender.close().await;
            return;
        }
    };

    // Verify session belongs to authenticated user
    if let Err(e) =
        verify_session_access(&state, session_id, &authenticated_claims.sub.to_string()).await
    {
        if let Some(ref corr_id) = correlation_id {
            tracing::warn!(correlation_id = %corr_id, "Session access denied: {}", e);
        } else {
            tracing::warn!("Session access denied: {}", e);
        }
        let _ = sender
            .send(Message::Text(
                serde_json::to_string(&ServerMessage::Error {
                    message: "Session access denied".to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .unwrap(),
            ))
            .await;
        let _ = sender.close().await;
        return;
    }

    // Send authentication success
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&ServerMessage::AuthenticationResult {
                success: true,
                message: "Authentication successful".to_string(),
                user_id: Some(authenticated_claims.sub.to_string()),
                timestamp: chrono::Utc::now().timestamp(),
            })
            .unwrap(),
        ))
        .await;

    if let Some(ref corr_id) = correlation_id {
        tracing::info!(
            correlation_id = %corr_id,
            "WebSocket authenticated for session {} by user {}",
            session_id,
            authenticated_claims.username
        );
    } else {
        tracing::info!(
            "WebSocket authenticated for session {} by user {}",
            session_id,
            authenticated_claims.username
        );
    }

    // Get session info and verify it exists
    let session_info = match get_session_info(&state, session_id).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Failed to get session info: {}", e);
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: "Session not found".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                    })
                    .unwrap_or_default(),
                ))
                .await;
            return;
        }
    };

    // Create services
    let learner_service = Arc::new(LearnerService::new(
        state.db_pool.clone().into(),
        state.redis_conn.clone(),
    ));
    let adaptation_service = AdaptationService::new(learner_service.clone());

    // Set up heartbeat
    let mut heartbeat_interval = interval(Duration::from_secs(30));

    // Session state tracking
    let mut current_task: Option<Task> = None;
    let mut task_start_time: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut recent_errors: Vec<bool> = Vec::new();

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            if let Err(e) = handle_client_message(
                                &client_msg,
                                &mut sender,
                                &state,
                                &adaptation_service,
                                session_id,
                                session_info.learner_id,
                                &mut current_task,
                                &mut task_start_time,
                                &mut recent_errors,
                            ).await {
                                tracing::error!("Error handling client message: {}", e);
                            }
                        }
                    },
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("WebSocket closed for session {}", session_id);
                        break;
                    },
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error for session {}: {}", session_id, e);
                        break;
                    },
                    None => break,
                    _ => {}
                }
            },

            // Send periodic heartbeat
            _ = heartbeat_interval.tick() => {
                if let Err(_) = sender.send(Message::Text(
                    serde_json::to_string(&ServerMessage::Heartbeat {
                        timestamp: chrono::Utc::now().timestamp(),
                    }).unwrap_or_default()
                )).await {
                    break;
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed for session {}", session_id);
}

async fn handle_analytics_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for authentication message
    let authenticated_claims =
        match authenticate_websocket(&mut sender, &mut receiver, &state).await {
            Ok(claims) => claims,
            Err(e) => {
                tracing::warn!("Analytics WebSocket authentication failed: {}", e);
                let _ = sender
                    .send(Message::Text(
                        serde_json::to_string(&ServerMessage::AuthenticationResult {
                            success: false,
                            message: "Authentication failed".to_string(),
                            user_id: None,
                            timestamp: chrono::Utc::now().timestamp(),
                        })
                        .unwrap(),
                    ))
                    .await;
                let _ = sender.close().await;
                return;
            }
        };

    // Check for analytics permission
    if !authenticated_claims
        .permissions
        .contains(&"analytics_access".to_string())
        && authenticated_claims.role != "admin"
        && authenticated_claims.role != "researcher"
    {
        tracing::warn!(
            "Analytics access denied for user: {}",
            authenticated_claims.username
        );
        let _ = sender
            .send(Message::Text(
                serde_json::to_string(&ServerMessage::AuthenticationResult {
                    success: false,
                    message: "Analytics access denied".to_string(),
                    user_id: Some(authenticated_claims.sub.to_string()),
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .unwrap(),
            ))
            .await;
        let _ = sender.close().await;
        return;
    }

    // Send authentication success
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&ServerMessage::AuthenticationResult {
                success: true,
                message: "Analytics access granted".to_string(),
                user_id: Some(authenticated_claims.sub.to_string()),
                timestamp: chrono::Utc::now().timestamp(),
            })
            .unwrap(),
        ))
        .await;

    tracing::info!(
        "Analytics WebSocket connected for user {}",
        authenticated_claims.username
    );

    // Create analytics service
    let analytics_service = AnalyticsService::new_with_config(
        state.db_pool.clone().into(),
        state.redis_conn.clone().into(),
        state.config.clone(),
    );

    // Set up broadcast interval (every 5 seconds)
    let mut broadcast_interval = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            // Handle incoming messages (mostly just heartbeats)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("Analytics WebSocket closed");
                        break;
                    },
                    Some(Err(e)) => {
                        tracing::error!("Analytics WebSocket error: {}", e);
                        break;
                    },
                    None => break,
                    _ => {}
                }
            },

            // Broadcast live metrics
            _ = broadcast_interval.tick() => {
                match analytics_service.get_real_time_metrics().await {
                    Ok(metrics) => {
                        let live_metrics = LiveMetrics {
                            active_learners: metrics["active_sessions"].as_u64().unwrap_or(0) as usize,
                            tasks_per_minute: metrics["responses_last_minute"].as_f64().unwrap_or(0.0),
                            average_accuracy: 0.75, // Would be calculated from real data
                            difficulty_distribution: serde_json::json!({
                                "easy": 30,
                                "medium": 50,
                                "hard": 20
                            }),
                            strategy_distribution: serde_json::json!({
                                "systematic": 60,
                                "intuitive": 40
                            }),
                            timestamp: chrono::Utc::now().timestamp(),
                        };

                        if let Ok(msg_text) = serde_json::to_string(&live_metrics) {
                            if sender.send(Message::Text(msg_text)).await.is_err() {
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to get real-time metrics: {}", e);
                    }
                }
            }
        }
    }

    tracing::info!("Analytics WebSocket connection closed");
}

async fn handle_client_message(
    message: &ClientMessage,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    adaptation_service: &AdaptationService,
    session_id: Uuid,
    learner_id: Uuid,
    current_task: &mut Option<Task>,
    task_start_time: &mut Option<chrono::DateTime<chrono::Utc>>,
    recent_errors: &mut Vec<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timestamp = chrono::Utc::now().timestamp();

    match message {
        ClientMessage::StartTask { payload: _ } => {
            // Get session topology
            let topology = get_session_topology(state, session_id).await?;

            // Generate next task
            let task = adaptation_service
                .select_next_task(learner_id, &topology)
                .await?;

            let predicted_rt = 2000.0; // Would be calculated from learner model

            let task_message = TaskMessage {
                task: task.clone(),
                difficulty: task.difficulty,
                expected_duration_ms: predicted_rt,
            };

            *current_task = Some(task);
            *task_start_time = Some(chrono::Utc::now());

            let response = ServerMessage::Task {
                payload: task_message,
                timestamp,
            };

            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }

        ClientMessage::SubmitResponse {
            task_type,
            task_data,
            user_answer,
            response_time_ms,
        } => {
            // Validate response (simplified)
            let is_correct = user_answer.is_some(); // Simplified validation

            // Update recent errors
            recent_errors.push(is_correct);
            if recent_errors.len() > 10 {
                recent_errors.remove(0);
            }

            // Send feedback
            let feedback = FeedbackMessage {
                correct: is_correct,
                expected_answer: None, // Would be calculated from task
                explanation: if is_correct {
                    Some("Correct!".to_string())
                } else {
                    Some("That's not quite right.".to_string())
                },
            };

            sender
                .send(Message::Text(serde_json::to_string(
                    &ServerMessage::Feedback {
                        payload: feedback,
                        timestamp,
                    },
                )?))
                .await?;

            // Check for intervention
            let error_count = recent_errors.iter().filter(|&&correct| !correct).count();
            let elapsed_ms = if let Some(start_time) = task_start_time {
                (chrono::Utc::now() - *start_time).num_milliseconds() as u64
            } else {
                *response_time_ms as u64
            };

            if let Ok(Some(intervention)) = adaptation_service
                .should_intervene(learner_id, session_id, elapsed_ms, error_count)
                .await
            {
                let intervention_msg = match intervention {
                    InterventionAction::ProvideHint(level) => InterventionMessage {
                        intervention_type: "hint".to_string(),
                        message: "Here's a hint to help you".to_string(),
                        suggestion: Some(format!("Hint level: {:?}", level)),
                    },
                    InterventionAction::ProvideWorkedExample(example) => InterventionMessage {
                        intervention_type: "worked_example".to_string(),
                        message: "Here's a worked example to help you understand".to_string(),
                        suggestion: Some(example),
                    },
                    InterventionAction::IncreaseDifficulty => InterventionMessage {
                        intervention_type: "difficulty".to_string(),
                        message: "Great progress! Let's try something more challenging".to_string(),
                        suggestion: Some("Difficulty increased".to_string()),
                    },
                    InterventionAction::DecreaseDifficulty => InterventionMessage {
                        intervention_type: "difficulty".to_string(),
                        message: "Let's try something a bit easier".to_string(),
                        suggestion: Some("Difficulty reduced".to_string()),
                    },
                    InterventionAction::SuggestBreak => InterventionMessage {
                        intervention_type: "break".to_string(),
                        message:
                            "You've been practicing for a while. Consider taking a short break!"
                                .to_string(),
                        suggestion: None,
                    },
                    InterventionAction::SkipTask => InterventionMessage {
                        intervention_type: "skip".to_string(),
                        message: "Let's move on to a different task".to_string(),
                        suggestion: Some("Task skipped".to_string()),
                    },
                };

                sender
                    .send(Message::Text(serde_json::to_string(
                        &ServerMessage::Intervention {
                            payload: intervention_msg,
                            timestamp,
                        },
                    )?))
                    .await?;
            }

            // Send stats update
            let accuracy = if recent_errors.is_empty() {
                0.0
            } else {
                recent_errors.iter().filter(|&&correct| correct).count() as f64
                    / recent_errors.len() as f64
            };

            let stats = StatsMessage {
                session_accuracy: accuracy,
                recent_accuracy: accuracy, // Simplified
                response_count: recent_errors.len(),
                average_response_time_ms: *response_time_ms as f64,
            };

            sender
                .send(Message::Text(serde_json::to_string(
                    &ServerMessage::StatsUpdate {
                        payload: stats,
                        timestamp,
                    },
                )?))
                .await?;
        }

        ClientMessage::RequestHint { hint_level } => {
            if let Some(task) = current_task {
                let hint_level_enum = match hint_level.as_deref() {
                    Some("subtle") => HintLevel::Confirmation,
                    Some("strong") => HintLevel::Worked,
                    _ => HintLevel::Partial,
                };

                let hint_text = adaptation_service
                    .generate_hint(learner_id, task, hint_level_enum)
                    .await
                    .unwrap_or_else(|_| "Try thinking about the pattern".to_string());

                let hint_msg = HintMessage {
                    hint_text,
                    hint_level: hint_level.clone().unwrap_or("mild".to_string()),
                };

                sender
                    .send(Message::Text(serde_json::to_string(
                        &ServerMessage::Hint {
                            payload: hint_msg,
                            timestamp,
                        },
                    )?))
                    .await?;
            }
        }

        ClientMessage::Pause => {
            tracing::info!("Session {} paused", session_id);
        }

        ClientMessage::Resume => {
            tracing::info!("Session {} resumed", session_id);
        }

        ClientMessage::Heartbeat => {
            // Client heartbeat received, no action needed
        }

        ClientMessage::Authenticate { token: _ } => {
            // Authentication should have been handled before entering this function
            tracing::warn!("Unexpected authenticate message received after authentication");
        }
    }

    Ok(())
}

// Helper functions
#[derive(Debug)]
struct SessionInfo {
    learner_id: Uuid,
    topology_type: String,
}

async fn get_session_info(
    state: &AppState,
    session_id: Uuid,
) -> Result<SessionInfo, Box<dyn std::error::Error + Send + Sync>> {
    let session_id_bytes = session_id.as_bytes();

    // Use a raw query with proper binding
    let mut conn = state.db_pool.acquire().await?;
    let row = sqlx::query(
        "SELECT learner_id, topology_type FROM sessions WHERE id = ? AND status = 'active'",
    )
    .bind(&session_id_bytes[..])
    .fetch_one(&mut *conn)
    .await?;

    let learner_id_bytes: Vec<u8> = row.try_get("learner_id")?;
    let learner_id = Uuid::from_bytes(learner_id_bytes.try_into().unwrap_or_default());
    let topology_type: String = row.try_get("topology_type")?;

    Ok(SessionInfo {
        learner_id,
        topology_type,
    })
}

async fn get_session_topology(
    state: &AppState,
    session_id: Uuid,
) -> Result<abcdeez_core::Topology, Box<dyn std::error::Error + Send + Sync>> {
    let session_id_bytes = session_id.as_bytes();

    // Use a raw query with proper binding
    let mut conn = state.db_pool.acquire().await?;
    let topology_data: String =
        sqlx::query_scalar("SELECT topology_data FROM sessions WHERE id = ?")
            .bind(&session_id_bytes[..])
            .fetch_one(&mut *conn)
            .await?;
    let topology: abcdeez_core::Topology = serde_json::from_str(&topology_data)
        .unwrap_or_else(|_| abcdeez_core::Topology::alphabet());

    Ok(topology)
}

/// Authenticate WebSocket connection by waiting for the first authentication message
async fn authenticate_websocket(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    receiver: &mut futures_util::stream::SplitStream<WebSocket>,
    state: &Arc<AppState>,
) -> Result<Claims, String> {
    // Wait for the first message which must be authentication
    if let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let client_msg: ClientMessage = serde_json::from_str(&text)
                    .map_err(|_| "Invalid message format".to_string())?;

                match client_msg {
                    ClientMessage::Authenticate { token } => {
                        // Validate JWT token
                        let mut validation = Validation::new(Algorithm::HS256);
                        validation.validate_exp = true;
                        validation.validate_nbf = true;
                        validation.leeway = 60;

                        let token_data = decode::<Claims>(
                            &token,
                            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
                            &validation,
                        )
                        .map_err(|e| format!("Invalid token: {:?}", e))?;

                        Ok(token_data.claims)
                    }
                    _ => Err("First message must be authentication".to_string()),
                }
            }
            Ok(Message::Close(_)) => Err("Connection closed before authentication".to_string()),
            _ => Err("Invalid message type for authentication".to_string()),
        }
    } else {
        Err("No authentication message received".to_string())
    }
}

/// Verify that a session belongs to the authenticated user
async fn verify_session_access(
    state: &Arc<AppState>,
    session_id: Uuid,
    user_id: &str,
) -> Result<(), String> {
    let session_id_bytes = session_id.as_bytes();
    let user_id_bytes = user_id.as_bytes();

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let session_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions s 
         JOIN learners l ON s.learner_id = l.id 
         WHERE s.id = ? AND l.user_id = ?)",
    )
    .bind(&session_id_bytes[..])
    .bind(&user_id_bytes[..])
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| format!("Database query error: {}", e))?;

    if session_exists {
        Ok(())
    } else {
        Err("Session does not belong to user".to_string())
    }
}
