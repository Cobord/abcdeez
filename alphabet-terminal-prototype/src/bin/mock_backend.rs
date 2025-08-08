use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    participants: Arc<Mutex<HashMap<String, ParticipantData>>>,
    responses: Arc<Mutex<Vec<ResponseData>>>,
    sessions: Arc<Mutex<HashMap<String, SessionData>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticipantData {
    id: String,
    experiment_id: String,
    group: String,
    registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResponseData {
    participant_id: String,
    task_type: String,
    correct: bool,
    rt_ms: u128,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionData {
    id: String,
    participant_id: String,
    started_at: DateTime<Utc>,
    responses: Vec<ResponseData>,
}

#[tokio::main]
async fn main() {
    println!("Starting mock backend server...");
    
    let state = AppState {
        participants: Arc::new(Mutex::new(HashMap::new())),
        responses: Arc::new(Mutex::new(Vec::new())),
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };
    
    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/participants", post(register_participant))
        .route("/v1/sessions/start", post(start_session))
        .route("/v1/responses/batch", post(receive_batch))
        .route("/v1/sessions/complete", post(complete_session))
        .route("/v1/metrics", post(receive_metrics))
        .route("/v1/errors", post(log_error))
        .route("/v1/tasks/next", post(next_task))
        .route("/v1/experiments/:id", get(get_experiment))
        .route("/v1/stats", get(get_stats))
        .with_state(state);
    
    let addr = "127.0.0.1:3000";
    println!("Mock backend listening on http://{}", addr);
    println!("Configure your TUI with:");
    println!("  export LEARNING_API_URL=http://127.0.0.1:3000/v1");
    println!("  export EXPERIMENT_ID=test_experiment");
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "timestamp": Utc::now() }))
}

async fn register_participant(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let participant_id = payload["participant_id"].as_str().unwrap_or("unknown");
    let experiment_id = payload["experiment_id"].as_str().unwrap_or("default");
    let group = payload["group"].as_str().unwrap_or("control");
    
    let participant = ParticipantData {
        id: participant_id.to_string(),
        experiment_id: experiment_id.to_string(),
        group: group.to_string(),
        registered_at: Utc::now(),
    };
    
    state.participants.lock().unwrap()
        .insert(participant_id.to_string(), participant.clone());
    
    println!("✓ Registered participant: {} in group {}", participant_id, group);
    
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);
    
    Json(json!({
        "token": token,
        "expires_at": expires_at
    }))
}

async fn start_session(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let session_id = Uuid::new_v4().to_string();
    
    let session = SessionData {
        id: session_id.clone(),
        participant_id: "unknown".to_string(),
        started_at: Utc::now(),
        responses: Vec::new(),
    };
    
    state.sessions.lock().unwrap()
        .insert(session_id.clone(), session);
    
    println!("✓ Started session: {}", session_id);
    
    Json(json!({
        "session_id": session_id
    }))
}

async fn receive_batch(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(responses) = payload["responses"].as_array() {
        let mut all_responses = state.responses.lock().unwrap();
        
        for response in responses {
            if let (Some(correct), Some(rt)) = (
                response["correct"].as_bool(),
                response["response_time_ms"].as_u64()
            ) {
                let resp = ResponseData {
                    participant_id: "unknown".to_string(),
                    task_type: "unknown".to_string(),
                    correct,
                    rt_ms: rt as u128,
                    timestamp: Utc::now(),
                };
                all_responses.push(resp);
            }
        }
        
        println!("✓ Received batch of {} responses", responses.len());
        println!("  Total responses stored: {}", all_responses.len());
    }
    
    StatusCode::OK
}

async fn complete_session(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    println!("✓ Received complete session data");
    
    if let Some(learner_id) = payload["learner_id"].as_str() {
        println!("  Learner: {}", learner_id);
    }
    
    if let Some(sessions) = payload["sessions"].as_array() {
        println!("  Sessions: {}", sessions.len());
        
        for session in sessions {
            if let Some(responses) = session["responses"].as_array() {
                println!("    - {} responses", responses.len());
            }
        }
    }
    
    StatusCode::OK
}

async fn receive_metrics(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(participant_id) = payload["participant_id"].as_str() {
        println!("📊 Metrics update from {}", participant_id);
        
        if let Some(metrics) = payload["metrics"].as_object() {
            if let Some(bidirectionality) = metrics["bidirectionality_index"].as_f64() {
                println!("  Bidirectionality: {:.3}", bidirectionality);
            }
            if let Some(memory) = metrics["avg_memory_strength"].as_f64() {
                println!("  Memory strength: {:.1}%", memory * 100.0);
            }
        }
    }
    
    StatusCode::OK
}

async fn log_error(
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let (Some(participant_id), Some(error)) = (
        payload["participant_id"].as_str(),
        payload["error"].as_str()
    ) {
        println!("⚠ Error from {}: {}", participant_id, error);
    }
    
    StatusCode::OK
}

async fn next_task(
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Return no content for adaptive mode (let client generate tasks)
    StatusCode::NO_CONTENT
}

async fn get_experiment(
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(json!({
        "experiment_id": id,
        "name": "Test Experiment",
        "description": "Mock experiment for testing",
        "groups": ["Adaptive", "Linear", "Yoked"],
        "trials_per_session": 100,
        "sessions": 7,
        "task_types": ["PairwiseOrder", "KJump", "Segment"],
        "created_at": Utc::now(),
        "metadata": {}
    }))
}

async fn get_stats(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let participants = state.participants.lock().unwrap();
    let responses = state.responses.lock().unwrap();
    let sessions = state.sessions.lock().unwrap();
    
    let total_correct: usize = responses.iter()
        .filter(|r| r.correct)
        .count();
    
    let avg_rt: f64 = if !responses.is_empty() {
        responses.iter()
            .map(|r| r.rt_ms as f64)
            .sum::<f64>() / responses.len() as f64
    } else {
        0.0
    };
    
    Json(json!({
        "participants": participants.len(),
        "sessions": sessions.len(),
        "total_responses": responses.len(),
        "total_correct": total_correct,
        "overall_accuracy": if responses.is_empty() { 0.0 } else { total_correct as f64 / responses.len() as f64 },
        "average_rt_ms": avg_rt,
        "server_time": Utc::now()
    }))
}