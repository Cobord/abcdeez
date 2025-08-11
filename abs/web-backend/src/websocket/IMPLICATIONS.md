# WebSocket Directory - Integration Guide

## Overview

The websocket directory provides real-time communication infrastructure for the educational platform, enabling live learning sessions, adaptive interventions, analytics dashboards, and immediate feedback. It implements secure authentication, session management, and bi-directional communication patterns optimized for educational interactions.

## How Core and Apps Should Use This Functionality

### Core Integration Patterns

#### WebSocket Authentication Flow
```rust
// Secure authentication before any WebSocket operations
// 1. Client connects to WebSocket endpoint
// 2. Client sends Authenticate message with JWT token
// 3. Server validates token and establishes session
// 4. Bi-directional communication begins

use crate::websocket::{ClientMessage, ServerMessage};

// Client-side authentication
let auth_message = ClientMessage::Authenticate {
    token: jwt_token.to_string(),
};
websocket.send(serde_json::to_string(&auth_message)?).await?;
```

#### Real-time Learning Session
```rust
// Interactive learning session with adaptive responses
let session_ws = connect_to_session_websocket(session_id, auth_token).await?;

// Start a learning task
let start_task = ClientMessage::StartTask {
    payload: serde_json::json!({"topic": "algebra"}),
};
session_ws.send(serde_json::to_string(&start_task)?).await?;

// Handle real-time responses from server
while let Some(message) = session_ws.next().await {
    match serde_json::from_str::<ServerMessage>(&message?) {
        Ok(ServerMessage::Task { payload, .. }) => handle_new_task(payload).await?,
        Ok(ServerMessage::Intervention { payload, .. }) => apply_intervention(payload).await?,
        Ok(ServerMessage::Feedback { payload, .. }) => show_feedback(payload).await?,
        _ => {}
    }
}
```

### Application Usage Patterns

#### Analytics Dashboard Integration
```rust
// Real-time analytics for administrators and researchers
let analytics_ws = connect_to_analytics_websocket(auth_token).await?;

while let Some(metrics) = analytics_ws.next().await {
    let live_metrics: LiveMetrics = serde_json::from_str(&metrics?)?;
    update_dashboard(live_metrics).await?;
}
```

#### Adaptive Learning Intervention
```rust
// Server-side adaptive response based on learner performance
pub async fn handle_submit_response(
    response: &SubmitResponse,
    adaptation_service: &AdaptationService,
    sender: &mut WebSocketSender,
) -> Result<(), Box<dyn Error>> {
    // Process response and determine if intervention needed
    let performance_analysis = analyze_response(response).await?;
    
    if let Some(intervention) = adaptation_service
        .determine_intervention(&performance_analysis)
        .await?
    {
        let intervention_message = ServerMessage::Intervention {
            payload: map_intervention_to_message(intervention),
            timestamp: Utc::now().timestamp(),
        };
        sender.send(serde_json::to_string(&intervention_message)?).await?;
    }
    
    Ok(())
}
```

## State Machines and Lifecycle Patterns

### WebSocket Connection Lifecycle
```
Connect → Authenticate → Session Establishment → Active Communication → Heartbeat Maintenance → Graceful Closure
```

### Learning Session State Machine
```
Session Start → Task Generation → Response Collection → Feedback Delivery → Intervention Check → Next Task/End
```

### Authentication State Flow
```
Connection → Auth Message → Token Validation → Claims Extraction → Permission Check → Session Authorization
```

### Analytics Broadcast Lifecycle
```
Connect → Authenticate → Permission Check → Metrics Collection → Broadcast Loop → Disconnection
```

## Integration Patterns and Best Practices

### Message Type Safety
```rust
// Strongly typed message handling prevents runtime errors
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { token: String },
    StartTask { payload: serde_json::Value },
    SubmitResponse { 
        task_type: String, 
        user_answer: Option<String>,
        response_time_ms: i64 
    },
    RequestHint { hint_level: Option<String> },
}

// Pattern matching ensures all message types are handled
match client_message {
    ClientMessage::StartTask { payload } => handle_start_task(payload).await?,
    ClientMessage::SubmitResponse { task_type, user_answer, response_time_ms } => {
        handle_response_submission(task_type, user_answer, response_time_ms).await?
    },
    // ... other cases
}
```

### Error Handling Strategy
```rust
// Graceful error handling with client notification
pub async fn handle_websocket_error(
    error: WebSocketError,
    sender: &mut WebSocketSender,
) -> Result<(), Box<dyn Error>> {
    let error_message = ServerMessage::Error {
        message: format!("Error: {}", error),
        timestamp: Utc::now().timestamp(),
    };
    
    // Attempt to notify client of error
    if sender.send(serde_json::to_string(&error_message)?).await.is_err() {
        tracing::error!("Failed to send error message to client");
    }
    
    Ok(())
}
```

### Session State Management
```rust
// Maintain session state throughout WebSocket connection
pub struct SessionState {
    current_task: Option<Task>,
    task_start_time: Option<DateTime<Utc>>,
    recent_errors: Vec<bool>,
    performance_metrics: PerformanceTracker,
}

impl SessionState {
    pub fn update_performance(&mut self, correct: bool, response_time: Duration) {
        self.recent_errors.push(correct);
        self.performance_metrics.record_response(correct, response_time);
        
        // Maintain sliding window of recent performance
        if self.recent_errors.len() > RECENT_ERRORS_BUFFER_SIZE {
            self.recent_errors.remove(0);
        }
    }
}
```

## Architectural Decisions and Constraints

### Security-First Design
- JWT token authentication required before any WebSocket operations
- Session ownership verification prevents unauthorized access
- Role-based access control for analytics and administrative functions
- Secure token handling without URL parameter exposure

### Real-time Performance Optimization
- Efficient message serialization using serde_json with minimal allocation
- Heartbeat mechanism maintains connection health and detects failures
- Async/await patterns throughout for non-blocking concurrent operations
- Connection pooling and resource management for scalability

### Educational Domain Specialization
- Adaptive intervention system integrated directly into WebSocket message flow
- Real-time learning analytics with immediate feedback loops
- Response time tracking and cognitive load assessment
- Specialized message types for educational interactions (hints, feedback, interventions)

### Fault Tolerance and Recovery
- Connection timeout handling with graceful degradation
- Automatic reconnection support on client side
- Error state recovery with session state preservation
- Comprehensive logging for debugging and monitoring

## Security and Performance Considerations

### Security Measures
```rust
// Multi-layer security validation
pub async fn validate_websocket_access(
    token: &str,
    session_id: Uuid,
    state: &AppState,
) -> Result<Claims, AuthError> {
    // 1. Validate JWT token structure and signature
    let claims = validate_jwt_token(token, &state.config.jwt_secret)?;
    
    // 2. Check token expiration and not-before claims
    validate_token_timing(&claims)?;
    
    // 3. Verify session ownership
    verify_session_belongs_to_user(session_id, &claims.sub, state).await?;
    
    // 4. Check user permissions for specific operations
    validate_user_permissions(&claims)?;
    
    Ok(claims)
}
```

### Performance Optimization
- Message batching for high-frequency updates to reduce network overhead
- Efficient JSON serialization with minimal memory allocation
- Connection state caching to avoid repeated database queries
- Background task processing to minimize response latency

### Resource Management
- Connection limits and rate limiting to prevent abuse
- Memory management for session state with bounded collections
- Graceful shutdown procedures to clean up resources
- Database connection pooling for concurrent session access

## Common Usage Patterns and Examples

### Interactive Learning Session
```rust
// Complete learning session workflow
pub async fn run_learning_session(
    session_id: Uuid,
    auth_token: &str,
) -> Result<SessionResults, Box<dyn Error>> {
    let mut ws = connect_websocket(session_id, auth_token).await?;
    let mut session_results = SessionResults::new();
    
    // Start the session
    ws.send_message(ClientMessage::StartTask {
        payload: serde_json::json!({}),
    }).await?;
    
    while let Some(message) = ws.receive_message().await? {
        match message {
            ServerMessage::Task { payload, .. } => {
                let user_response = collect_user_response(&payload.task).await?;
                
                ws.send_message(ClientMessage::SubmitResponse {
                    task_type: payload.task.task_type.clone(),
                    task_data: serde_json::to_value(&payload.task)?,
                    user_answer: Some(user_response.answer),
                    response_time_ms: user_response.duration.as_millis() as i64,
                }).await?;
            },
            
            ServerMessage::Feedback { payload, .. } => {
                session_results.record_feedback(payload);
                display_feedback_to_user(&payload).await?;
            },
            
            ServerMessage::Intervention { payload, .. } => {
                apply_learning_intervention(&payload).await?;
            },
            
            ServerMessage::Error { message, .. } => {
                return Err(format!("Session error: {}", message).into());
            },
            
            _ => {} // Handle other message types
        }
    }
    
    Ok(session_results)
}
```

### Real-time Analytics Dashboard
```rust
// Live analytics dashboard with WebSocket updates
pub async fn start_analytics_dashboard(
    auth_token: &str,
) -> Result<(), Box<dyn Error>> {
    let mut analytics_ws = connect_analytics_websocket(auth_token).await?;
    
    while let Some(metrics_message) = analytics_ws.receive().await? {
        let metrics: LiveMetrics = serde_json::from_str(&metrics_message)?;
        
        // Update dashboard components
        update_active_learners_count(metrics.active_learners).await?;
        update_task_completion_rate(metrics.tasks_per_minute).await?;
        update_difficulty_distribution(&metrics.difficulty_distribution).await?;
        
        // Trigger dashboard refresh
        refresh_dashboard_ui().await?;
    }
    
    Ok(())
}
```

### Adaptive Intervention System
```rust
// Server-side adaptive intervention logic
pub async fn process_learner_response(
    response: &SubmitResponse,
    session_state: &mut SessionState,
    adaptation_service: &AdaptationService,
) -> Option<InterventionAction> {
    // Update learner model with response data
    let performance_update = PerformanceUpdate {
        correct: validate_response(response),
        response_time: Duration::from_millis(response.response_time_ms as u64),
        task_difficulty: session_state.current_task.as_ref()?.difficulty,
    };
    
    session_state.update_performance(
        performance_update.correct,
        performance_update.response_time,
    );
    
    // Determine if intervention is needed
    let struggle_level = assess_struggle_level(&session_state.recent_errors);
    let response_pattern = analyze_response_patterns(&session_state.performance_metrics);
    
    adaptation_service.determine_intervention(AdaptationContext {
        struggle_level,
        response_pattern,
        session_duration: session_state.session_duration(),
        recent_accuracy: session_state.recent_accuracy(),
    }).await
}
```

## Configuration Examples

### WebSocket Configuration
```toml
[websocket]
# Connection settings
heartbeat_interval_seconds = 30
analytics_broadcast_interval_seconds = 5
connection_timeout_seconds = 300

# Authentication
jwt_leeway_seconds = 60
auth_timeout_seconds = 30

# Performance settings
max_concurrent_connections = 1000
message_buffer_size = 1024
recent_errors_buffer_size = 10

# Educational settings
default_predicted_response_time_ms = 2000
intervention_threshold_errors = 3
hint_cooldown_seconds = 10

[websocket.analytics]
# Analytics dashboard settings
metrics_update_interval_seconds = 5
max_metrics_history = 100
enable_real_time_charts = true
```

This WebSocket infrastructure provides the real-time communication foundation that enables immediate feedback, adaptive learning interventions, and live analytics while maintaining security, performance, and educational domain specialization.