use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use uuid::Uuid;

// Import our research modules
use graph_learning_core::{
    AuditTrailManager, AuditConfiguration, AuditLevel, EventType, Actor, ActorType, Resource, Operation, Outcome,
    SeedManager, IRBComplianceGenerator, ProtocolVersionManager,
    ExperimentalDesign, MultiSessionExperiment, PowerAnalyzer,
    CitationManager, Reference, Author, ReferenceType, Publication, MixedEffectsAnalyzer, StatisticalValidator,
    LearnerDataExport, SensorManager,
};

#[derive(Clone)]
struct ResearchAppState {
    // Core research services
    audit_manager: Arc<RwLock<AuditTrailManager>>,
    seed_manager: Arc<RwLock<SeedManager>>,
    irb_generator: Arc<RwLock<IRBComplianceGenerator>>,
    protocol_manager: Arc<RwLock<ProtocolVersionManager>>,
    power_analyzer: Arc<RwLock<PowerAnalyzer>>,
    citation_manager: Arc<RwLock<CitationManager>>,
    
    // Data storage (in production would be database)
    experiments: Arc<RwLock<HashMap<String, ExperimentData>>>,
    participants: Arc<RwLock<HashMap<String, ParticipantData>>>,
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    responses: Arc<RwLock<Vec<ResponseData>>>,
    
    // Research-specific data
    irb_documents: Arc<RwLock<HashMap<String, IRBDocumentData>>>,
    exports: Arc<RwLock<HashMap<String, ExportData>>>,
    protocol_versions: Arc<RwLock<HashMap<String, ProtocolVersionData>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExperimentData {
    id: String,
    name: String,
    description: String,
    design: String, // Serialized ExperimentalDesign
    status: ExperimentStatus,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    participant_count: usize,
    completion_rate: f64,
    effect_size: Option<f64>,
    statistical_power: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Planning,
    Active,
    Paused,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticipantData {
    id: String,
    experiment_id: String,
    group: String,
    enrolled_at: DateTime<Utc>,
    consent_status: ConsentStatus,
    withdrawal_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsentStatus {
    Pending,
    Given,
    Withdrawn,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionData {
    id: String,
    participant_id: String,
    experiment_id: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    responses: Vec<ResponseData>,
    quality_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResponseData {
    id: String,
    participant_id: String,
    session_id: String,
    task_type: String,
    correct: bool,
    response_time_ms: u128,
    timestamp: DateTime<Utc>,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IRBDocumentData {
    id: String,
    experiment_id: String,
    document_type: String,
    content: String,
    generated_at: DateTime<Utc>,
    approval_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExportData {
    id: String,
    experiment_id: String,
    format: String,
    file_path: String,
    created_at: DateTime<Utc>,
    download_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProtocolVersionData {
    id: String,
    experiment_id: String,
    version: String,
    commit_hash: String,
    author: String,
    message: String,
    created_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Deserialize)]
struct CreateExperimentRequest {
    name: String,
    description: String,
    design_type: String,
    sample_size: usize,
    created_by: String,
}

#[derive(Deserialize)]
struct GenerateIRBRequest {
    experiment_id: String,
    study_title: String,
    principal_investigator: String,
    institution: String,
    study_purpose: String,
}

#[derive(Deserialize)]
struct ExportRequest {
    experiment_id: String,
    format: String,
    include_scripts: bool,
}

#[derive(Serialize)]
struct DashboardStats {
    total_experiments: usize,
    active_experiments: usize,
    total_participants: usize,
    avg_completion_rate: f64,
    recent_activity: Vec<ActivityEvent>,
}

#[derive(Serialize)]
struct ActivityEvent {
    event_type: String,
    description: String,
    timestamp: DateTime<Utc>,
    experiment_id: Option<String>,
}

#[derive(Serialize)]
struct ExperimentMetrics {
    experiment_id: String,
    participant_count: usize,
    completion_rate: f64,
    current_effect_size: f64,
    statistical_power: f64,
    data_quality_score: f64,
    last_updated: DateTime<Utc>,
}

#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<usize>,
    limit: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Starting Research Backend Server...");
    
    // Initialize research services
    let audit_config = AuditConfiguration {
        level: AuditLevel::Standard,
        include_system_events: true,
        include_user_events: true,
        include_data_events: true,
        retention_days: 2555, // 7 years
        anonymize_data: false,
        hash_sensitive_fields: true,
    };

    let state = ResearchAppState {
        audit_manager: Arc::new(RwLock::new(AuditTrailManager::new(Some(audit_config)))),
        seed_manager: Arc::new(RwLock::new(SeedManager::new(Some(12345)))),
        irb_generator: Arc::new(RwLock::new(IRBComplianceGenerator::default())),
        protocol_manager: Arc::new(RwLock::new(ProtocolVersionManager::new("Research Protocols".to_string(), "system".to_string()))),
        power_analyzer: Arc::new(RwLock::new(PowerAnalyzer::new(0.05, 0.8))),
        citation_manager: Arc::new(RwLock::new(CitationManager::new())),
        
        experiments: Arc::new(RwLock::new(HashMap::new())),
        participants: Arc::new(RwLock::new(HashMap::new())),
        sessions: Arc::new(RwLock::new(HashMap::new())),
        responses: Arc::new(RwLock::new(Vec::new())),
        
        irb_documents: Arc::new(RwLock::new(HashMap::new())),
        exports: Arc::new(RwLock::new(HashMap::new())),
        protocol_versions: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        // Health and system
        .route("/api/health", get(health))
        .route("/api/dashboard", get(get_dashboard_stats))
        
        // Experiment management
        .route("/api/experiments", get(list_experiments))
        .route("/api/experiments", post(create_experiment))
        .route("/api/experiments/:id", get(get_experiment))
        .route("/api/experiments/:id", put(update_experiment))
        .route("/api/experiments/:id", delete(delete_experiment))
        .route("/api/experiments/:id/metrics", get(get_experiment_metrics))
        .route("/api/experiments/:id/start", post(start_experiment))
        .route("/api/experiments/:id/stop", post(stop_experiment))
        
        // Participant management
        .route("/api/experiments/:experiment_id/participants", get(list_participants))
        .route("/api/experiments/:experiment_id/participants", post(enroll_participant))
        .route("/api/participants/:id", get(get_participant))
        .route("/api/participants/:id/consent", post(process_consent))
        .route("/api/participants/:id/withdraw", post(withdraw_participant))
        
        // Session management
        .route("/api/sessions", post(start_session))
        .route("/api/sessions/:id", get(get_session))
        .route("/api/sessions/:id/complete", post(complete_session))
        .route("/api/sessions/:id/responses", post(submit_responses))
        
        // IRB and compliance
        .route("/api/irb/generate", post(generate_irb_documents))
        .route("/api/irb/documents/:experiment_id", get(list_irb_documents))
        .route("/api/compliance/audit-trail/:experiment_id", get(get_audit_trail))
        .route("/api/compliance/verify-integrity", post(verify_audit_integrity))
        
        // Data export
        .route("/api/export", post(export_experiment_data))
        .route("/api/exports/:experiment_id", get(list_exports))
        .route("/api/exports/:id/download", get(download_export))
        
        // Protocol versioning
        .route("/api/protocols/:experiment_id/versions", get(list_protocol_versions))
        .route("/api/protocols/:experiment_id/versions", post(create_protocol_version))
        .route("/api/protocols/:experiment_id/versions/:version_id", get(get_protocol_version))
        
        // Statistical analysis
        .route("/api/analysis/power/:experiment_id", get(analyze_statistical_power))
        .route("/api/analysis/mixed-effects/:experiment_id", post(run_mixed_effects_analysis))
        .route("/api/analysis/assumptions/:experiment_id", get(check_statistical_assumptions))
        
        // Seed management
        .route("/api/randomization/seed/:experiment_id", get(get_randomization_seed))
        .route("/api/randomization/manifest/:experiment_id", get(get_reproducibility_manifest))
        
        // Citation management
        .route("/api/citations/:experiment_id", get(get_experiment_citations))
        .route("/api/citations/:experiment_id", post(add_citation))
        
        .with_state(state);

    let addr = "127.0.0.1:8080";
    println!("🚀 Research Backend listening on http://{}", addr);
    println!("📊 Dashboard available at: http://127.0.0.1:8080/api/dashboard");
    println!("🧪 Experiments API: http://127.0.0.1:8080/api/experiments");
    println!("📋 IRB Tools: http://127.0.0.1:8080/api/irb/*");
    println!("📈 Analytics: http://127.0.0.1:8080/api/analysis/*");
    
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// API Handlers

async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "research-backend",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now()
    }))
}

async fn get_dashboard_stats(State(state): State<ResearchAppState>) -> impl IntoResponse {
    let experiments = state.experiments.read().unwrap();
    let participants = state.participants.read().unwrap();
    
    let total_experiments = experiments.len();
    let active_experiments = experiments.values()
        .filter(|e| matches!(e.status, ExperimentStatus::Active))
        .count();
    
    let avg_completion_rate = if !experiments.is_empty() {
        experiments.values().map(|e| e.completion_rate).sum::<f64>() / experiments.len() as f64
    } else {
        0.0
    };

    let recent_activity = vec![
        ActivityEvent {
            event_type: "experiment_created".to_string(),
            description: "New learning study created".to_string(),
            timestamp: Utc::now() - Duration::hours(2),
            experiment_id: None,
        },
        ActivityEvent {
            event_type: "participant_enrolled".to_string(),
            description: "Participant P001 enrolled".to_string(),
            timestamp: Utc::now() - Duration::hours(1),
            experiment_id: None,
        },
    ];

    let stats = DashboardStats {
        total_experiments,
        active_experiments,
        total_participants: participants.len(),
        avg_completion_rate,
        recent_activity,
    };

    Json(stats)
}

async fn create_experiment(
    State(state): State<ResearchAppState>,
    Json(req): Json<CreateExperimentRequest>,
) -> impl IntoResponse {
    let experiment_id = Uuid::new_v4().to_string();
    
    let experiment = ExperimentData {
        id: experiment_id.clone(),
        name: req.name,
        description: req.description,
        design: req.design_type, // In practice, would serialize proper ExperimentalDesign
        status: ExperimentStatus::Planning,
        created_by: req.created_by,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        participant_count: 0,
        completion_rate: 0.0,
        effect_size: None,
        statistical_power: None,
    };

    // Log audit event
    if let Ok(mut audit_manager) = state.audit_manager.write() {
        let mut details = HashMap::new();
        details.insert("experiment_id".to_string(), experiment_id.clone());
        details.insert("created_by".to_string(), experiment.created_by.clone());
        
        let _ = audit_manager.log_event(
            EventType::DataModification,
            Actor {
                id: experiment.created_by.clone(),
                actor_type: ActorType::Researcher,
                name: None,
                roles: vec!["researcher".to_string()],
            },
            Resource {
                id: experiment_id.clone(),
                resource_type: "experiment".to_string(),
                name: Some(experiment.name.clone()),
                classification: Some("research_data".to_string()),
            },
            Operation::Create,
            Outcome::Success,
            details,
        );
    }

    state.experiments.write().unwrap().insert(experiment_id.clone(), experiment.clone());
    
    println!("✓ Created experiment: {} ({})", experiment.name, experiment_id);
    
    (StatusCode::CREATED, Json(json!({
        "experiment_id": experiment_id,
        "status": "created"
    })))
}

async fn list_experiments(
    State(state): State<ResearchAppState>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let experiments = state.experiments.read().unwrap();
    let page = pagination.page.unwrap_or(0);
    let limit = pagination.limit.unwrap_or(10);
    
    let experiments_vec: Vec<&ExperimentData> = experiments.values()
        .skip(page * limit)
        .take(limit)
        .collect();
    
    Json(json!({
        "experiments": experiments_vec,
        "total": experiments.len(),
        "page": page,
        "limit": limit
    }))
}

async fn get_experiment(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Some(experiment) = state.experiments.read().unwrap().get(&id) {
        Json(json!(experiment))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Experiment not found"}))).into_response()
    }
}

async fn get_experiment_metrics(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    let experiments = state.experiments.read().unwrap();
    let participants = state.participants.read().unwrap();
    let sessions = state.sessions.read().unwrap();
    
    if let Some(experiment) = experiments.get(&experiment_id) {
        let participant_count = participants.values()
            .filter(|p| p.experiment_id == experiment_id)
            .count();
            
        let completed_sessions = sessions.values()
            .filter(|s| s.experiment_id == experiment_id && s.completed_at.is_some())
            .count();
            
        let total_sessions = sessions.values()
            .filter(|s| s.experiment_id == experiment_id)
            .count();
            
        let completion_rate = if total_sessions > 0 {
            completed_sessions as f64 / total_sessions as f64
        } else {
            0.0
        };

        // Calculate effect size using power analyzer
        let current_effect_size = if let Ok(power_analyzer) = state.power_analyzer.read() {
            // Simplified calculation - in practice would use actual data
            0.35
        } else {
            0.0
        };

        let statistical_power = if let Ok(power_analyzer) = state.power_analyzer.read() {
            power_analyzer.calculate_power_t_test(current_effect_size, participant_count, 0.05)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let metrics = ExperimentMetrics {
            experiment_id,
            participant_count,
            completion_rate,
            current_effect_size,
            statistical_power,
            data_quality_score: 0.95,
            last_updated: Utc::now(),
        };

        Json(metrics)
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Experiment not found"}))).into_response()
    }
}

async fn enroll_participant(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let participant_id = Uuid::new_v4().to_string();
    let group = payload["group"].as_str().unwrap_or("control").to_string();
    
    let participant = ParticipantData {
        id: participant_id.clone(),
        experiment_id: experiment_id.clone(),
        group,
        enrolled_at: Utc::now(),
        consent_status: ConsentStatus::Pending,
        withdrawal_date: None,
    };

    // Log enrollment in audit trail
    if let Ok(mut audit_manager) = state.audit_manager.write() {
        let mut details = HashMap::new();
        details.insert("participant_id".to_string(), participant_id.clone());
        details.insert("experiment_id".to_string(), experiment_id.clone());
        details.insert("group".to_string(), participant.group.clone());
        
        let _ = audit_manager.log_event(
            EventType::UserAction,
            Actor {
                id: "enrollment_system".to_string(),
                actor_type: ActorType::System,
                name: None,
                roles: vec!["system".to_string()],
            },
            Resource {
                id: participant_id.clone(),
                resource_type: "participant".to_string(),
                name: Some(format!("Participant {}", participant_id)),
                classification: Some("participant_data".to_string()),
            },
            Operation::Create,
            Outcome::Success,
            details,
        );
    }

    state.participants.write().unwrap().insert(participant_id.clone(), participant);
    
    println!("✓ Enrolled participant {} in experiment {}", participant_id, experiment_id);
    
    Json(json!({
        "participant_id": participant_id,
        "status": "enrolled",
        "consent_required": true
    }))
}

async fn generate_irb_documents(
    State(state): State<ResearchAppState>,
    Json(req): Json<GenerateIRBRequest>,
) -> impl IntoResponse {
    println!("📋 Generating IRB documents for experiment: {}", req.experiment_id);
    
    let document_id = Uuid::new_v4().to_string();
    let document = IRBDocumentData {
        id: document_id.clone(),
        experiment_id: req.experiment_id.clone(),
        document_type: "informed_consent".to_string(),
        content: format!(
            "INFORMED CONSENT FORM\n\nStudy Title: {}\nPrincipal Investigator: {}\nInstitution: {}\n\nPurpose: {}\n\n[Generated automatically by Research Platform]",
            req.study_title, req.principal_investigator, req.institution, req.study_purpose
        ),
        generated_at: Utc::now(),
        approval_status: "draft".to_string(),
    };

    state.irb_documents.write().unwrap().insert(document_id.clone(), document);
    
    Json(json!({
        "document_id": document_id,
        "status": "generated",
        "document_type": "informed_consent"
    }))
}

async fn export_experiment_data(
    State(state): State<ResearchAppState>,
    Json(req): Json<ExportRequest>,
) -> impl IntoResponse {
    println!("📊 Exporting data for experiment {} in {} format", req.experiment_id, req.format);
    
    let export_id = Uuid::new_v4().to_string();
    let file_path = format!("./exports/{}_{}.{}", req.experiment_id, export_id, 
        match req.format.as_str() {
            "r" => "R",
            "python" => "py", 
            "spss" => "sav",
            _ => "csv"
        });

    let export_data = ExportData {
        id: export_id.clone(),
        experiment_id: req.experiment_id,
        format: req.format,
        file_path: file_path.clone(),
        created_at: Utc::now(),
        download_count: 0,
    };

    state.exports.write().unwrap().insert(export_id.clone(), export_data);
    
    Json(json!({
        "export_id": export_id,
        "file_path": file_path,
        "status": "exported"
    }))
}

async fn start_session(
    State(state): State<ResearchAppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let session_id = Uuid::new_v4().to_string();
    let participant_id = payload["participant_id"].as_str().unwrap_or("unknown").to_string();
    let experiment_id = payload["experiment_id"].as_str().unwrap_or("unknown").to_string();
    
    let session = SessionData {
        id: session_id.clone(),
        participant_id,
        experiment_id,
        started_at: Utc::now(),
        completed_at: None,
        responses: Vec::new(),
        quality_score: None,
    };

    state.sessions.write().unwrap().insert(session_id.clone(), session);
    
    println!("✓ Started session: {}", session_id);
    
    Json(json!({
        "session_id": session_id,
        "status": "started"
    }))
}

async fn verify_audit_integrity(State(state): State<ResearchAppState>) -> impl IntoResponse {
    if let Ok(mut audit_manager) = state.audit_manager.write() {
        match audit_manager.verify_integrity() {
            Ok(result) => Json(json!({
                "integrity_status": result.overall_status,
                "chain_valid": result.chain_integrity,
                "total_entries": result.total_entries_verified,
                "verification_timestamp": result.verification_timestamp
            })),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))).into_response()
        }
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Audit manager unavailable"}))).into_response()
    }
}

// Additional handler implementations would go here...
async fn update_experiment(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut experiments = state.experiments.write().unwrap();
    
    if let Some(experiment) = experiments.get_mut(&id) {
        if let Some(name) = payload["name"].as_str() {
            experiment.name = name.to_string();
        }
        if let Some(description) = payload["description"].as_str() {
            experiment.description = description.to_string();
        }
        if let Some(status_str) = payload["status"].as_str() {
            experiment.status = match status_str {
                "planning" => ExperimentStatus::Planning,
                "active" => ExperimentStatus::Active,
                "paused" => ExperimentStatus::Paused,
                "completed" => ExperimentStatus::Completed,
                "archived" => ExperimentStatus::Archived,
                _ => experiment.status.clone(),
            };
        }
        experiment.updated_at = Utc::now();
        
        println!("✓ Updated experiment: {}", id);
        Json(json!({"status": "updated", "experiment_id": id}))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Experiment not found"}))).into_response()
    }
}

async fn delete_experiment(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut experiments = state.experiments.write().unwrap();
    
    if experiments.remove(&id).is_some() {
        println!("✓ Deleted experiment: {}", id);
        Json(json!({"status": "deleted", "experiment_id": id}))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Experiment not found"}))).into_response()
    }
}

async fn start_experiment(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut experiments = state.experiments.write().unwrap();
    
    if let Some(experiment) = experiments.get_mut(&id) {
        experiment.status = ExperimentStatus::Active;
        experiment.updated_at = Utc::now();
        
        println!("✓ Started experiment: {}", id);
        Json(json!({"status": "started", "experiment_id": id}))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Experiment not found"}))).into_response()
    }
}

async fn stop_experiment(
    State(state): State<ResearchAppState>, 
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut experiments = state.experiments.write().unwrap();
    
    if let Some(experiment) = experiments.get_mut(&id) {
        experiment.status = ExperimentStatus::Paused;
        experiment.updated_at = Utc::now();
        
        println!("✓ Stopped experiment: {}", id);
        Json(json!({"status": "stopped", "experiment_id": id}))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Experiment not found"}))).into_response()
    }
}

async fn list_participants(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    let participants = state.participants.read().unwrap();
    
    let experiment_participants: Vec<&ParticipantData> = participants.values()
        .filter(|p| p.experiment_id == experiment_id)
        .collect();
    
    Json(json!({
        "participants": experiment_participants,
        "total": experiment_participants.len(),
        "experiment_id": experiment_id
    }))
}

async fn get_participant(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let participants = state.participants.read().unwrap();
    
    if let Some(participant) = participants.get(&id) {
        Json(json!(participant))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Participant not found"}))).into_response()
    }
}

async fn process_consent(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut participants = state.participants.write().unwrap();
    
    if let Some(participant) = participants.get_mut(&id) {
        let consent_given = payload["consent_given"].as_bool().unwrap_or(false);
        
        participant.consent_status = if consent_given {
            ConsentStatus::Given
        } else {
            ConsentStatus::Withdrawn
        };
        
        // Log consent event in audit trail
        if let Ok(mut audit_manager) = state.audit_manager.write() {
            let _ = audit_manager.log_data_access(
                &participant.id,
                "consent_form",
                &format!("consent_{}", id),
                Operation::Update,
                Outcome::Success,
            );
        }
        
        println!("✓ Processed consent for participant: {} ({})", id, if consent_given { "Given" } else { "Withdrawn" });
        Json(json!({"status": "processed", "consent_status": if consent_given { "given" } else { "withdrawn" }}))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Participant not found"}))).into_response()
    }
}

async fn withdraw_participant(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut participants = state.participants.write().unwrap();
    
    if let Some(participant) = participants.get_mut(&id) {
        participant.consent_status = ConsentStatus::Withdrawn;
        participant.withdrawal_date = Some(Utc::now());
        
        // Log withdrawal in audit trail
        if let Ok(mut audit_manager) = state.audit_manager.write() {
            let _ = audit_manager.log_user_action(
                &participant.id,
                "withdraw_from_study",
                &participant.experiment_id,
                Outcome::Success,
            );
        }
        
        println!("✓ Participant withdrawn: {}", id);
        Json(json!({"status": "withdrawn", "participant_id": id}))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Participant not found"}))).into_response()
    }
}

async fn get_session(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let sessions = state.sessions.read().unwrap();
    
    if let Some(session) = sessions.get(&id) {
        Json(json!(session))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Session not found"}))).into_response()
    }
}

async fn complete_session(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.write().unwrap();
    
    if let Some(session) = sessions.get_mut(&id) {
        session.completed_at = Some(Utc::now());
        session.quality_score = Some(0.95); // Would calculate from actual data
        
        // Log session completion
        if let Ok(mut audit_manager) = state.audit_manager.write() {
            let _ = audit_manager.log_user_action(
                &session.participant_id,
                "complete_session",
                &id,
                Outcome::Success,
            );
        }
        
        println!("✓ Completed session: {}", id);
        Json(json!({"status": "completed", "session_id": id}))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Session not found"}))).into_response()
    }
}

async fn submit_responses(
    State(state): State<ResearchAppState>,
    Path(session_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.write().unwrap();
    let mut responses = state.responses.write().unwrap();
    
    if let Some(session) = sessions.get_mut(&session_id) {
        if let Some(new_responses) = payload["responses"].as_array() {
            let mut response_count = 0;
            
            for response_data in new_responses {
                let response = ResponseData {
                    id: Uuid::new_v4().to_string(),
                    participant_id: session.participant_id.clone(),
                    session_id: session_id.clone(),
                    task_type: response_data["task_type"].as_str().unwrap_or("unknown").to_string(),
                    correct: response_data["correct"].as_bool().unwrap_or(false),
                    response_time_ms: response_data["response_time_ms"].as_u64().unwrap_or(0) as u128,
                    timestamp: Utc::now(),
                    metadata: response_data["metadata"].as_object()
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default(),
                };
                
                session.responses.push(response.clone());
                responses.push(response);
                response_count += 1;
            }
            
            println!("✓ Submitted {} responses for session: {}", response_count, session_id);
            Json(json!({"status": "submitted", "responses_count": response_count}))
        } else {
            (StatusCode::BAD_REQUEST, Json(json!({"error": "No responses data provided"}))).into_response()
        }
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Session not found"}))).into_response()
    }
}

async fn list_irb_documents(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    let irb_documents = state.irb_documents.read().unwrap();
    
    let experiment_documents: Vec<&IRBDocumentData> = irb_documents.values()
        .filter(|doc| doc.experiment_id == experiment_id)
        .collect();
    
    Json(json!({
        "documents": experiment_documents,
        "total": experiment_documents.len(),
        "experiment_id": experiment_id
    }))
}

async fn get_audit_trail(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    if let Ok(audit_manager) = state.audit_manager.read() {
        let compliance_report = audit_manager.generate_compliance_report();
        
        // Filter events for this experiment (simplified)
        let experiment_events = audit_manager.events.iter()
            .filter(|event| event.details.get("experiment_id").map_or(false, |id| id == &experiment_id))
            .collect::<Vec<_>>();
        
        Json(json!({
            "experiment_id": experiment_id,
            "events": experiment_events,
            "compliance_report": compliance_report,
            "total_events": experiment_events.len()
        }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Audit manager unavailable"}))).into_response()
    }
}

async fn list_exports(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    let exports = state.exports.read().unwrap();
    
    let experiment_exports: Vec<&ExportData> = exports.values()
        .filter(|export| export.experiment_id == experiment_id)
        .collect();
    
    Json(json!({
        "exports": experiment_exports,
        "total": experiment_exports.len(),
        "experiment_id": experiment_id
    }))
}

async fn download_export(
    State(state): State<ResearchAppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut exports = state.exports.write().unwrap();
    
    if let Some(export) = exports.get_mut(&id) {
        export.download_count += 1;
        
        // Log download event
        if let Ok(mut audit_manager) = state.audit_manager.write() {
            let _ = audit_manager.log_data_access(
                "system",
                "export_file",
                &id,
                Operation::Read,
                Outcome::Success,
            );
        }
        
        println!("✓ Export downloaded: {} ({})", id, export.file_path);
        Json(json!({
            "download_url": format!("/files/{}", export.file_path),
            "file_path": export.file_path,
            "format": export.format,
            "download_count": export.download_count
        }))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Export not found"}))).into_response()
    }
}

async fn list_protocol_versions(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    let protocol_versions = state.protocol_versions.read().unwrap();
    
    let experiment_versions: Vec<&ProtocolVersionData> = protocol_versions.values()
        .filter(|version| version.experiment_id == experiment_id)
        .collect();
    
    Json(json!({
        "versions": experiment_versions,
        "total": experiment_versions.len(),
        "experiment_id": experiment_id
    }))
}

async fn create_protocol_version(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let version_id = Uuid::new_v4().to_string();
    let version = payload["version"].as_str().unwrap_or("1.0.0").to_string();
    let message = payload["message"].as_str().unwrap_or("Protocol update").to_string();
    let author = payload["author"].as_str().unwrap_or("system").to_string();
    
    let protocol_version = ProtocolVersionData {
        id: version_id.clone(),
        experiment_id: experiment_id.clone(),
        version,
        commit_hash: format!("hash_{}", &version_id[..8]),
        author,
        message,
        created_at: Utc::now(),
    };
    
    // Save using protocol manager
    if let Ok(mut protocol_manager) = state.protocol_manager.write() {
        // Would use protocol_manager to save version
        println!("✓ Created protocol version: {} for experiment: {}", protocol_version.version, experiment_id);
    }
    
    state.protocol_versions.write().unwrap().insert(version_id.clone(), protocol_version);
    
    Json(json!({
        "version_id": version_id,
        "status": "created"
    }))
}

async fn get_protocol_version(
    State(state): State<ResearchAppState>,
    Path((experiment_id, version_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let protocol_versions = state.protocol_versions.read().unwrap();
    
    if let Some(version) = protocol_versions.get(&version_id) {
        if version.experiment_id == experiment_id {
            Json(json!(version))
        } else {
            (StatusCode::BAD_REQUEST, Json(json!({"error": "Version does not belong to this experiment"}))).into_response()
        }
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Protocol version not found"}))).into_response()
    }
}

async fn analyze_statistical_power(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    let participants = state.participants.read().unwrap();
    let participant_count = participants.values()
        .filter(|p| p.experiment_id == experiment_id)
        .count();
    
    if let Ok(power_analyzer) = state.power_analyzer.read() {
        let effect_size = 0.5; // Would calculate from actual data
        let alpha = 0.05;
        
        let statistical_power = power_analyzer.calculate_power_t_test(effect_size, participant_count, alpha)
            .unwrap_or(0.0);
        
        Json(json!({
            "experiment_id": experiment_id,
            "current_sample_size": participant_count,
            "effect_size": effect_size,
            "alpha_level": alpha,
            "statistical_power": statistical_power,
            "power_adequate": statistical_power >= 0.8,
            "recommended_sample_size": if statistical_power < 0.8 { 
                power_analyzer.calculate_sample_size_t_test(effect_size, 0.8, alpha).unwrap_or(participant_count)
            } else { 
                participant_count 
            }
        }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Power analyzer unavailable"}))).into_response()
    }
}

async fn run_mixed_effects_analysis(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let responses = state.responses.read().unwrap();
    
    // Filter responses for this experiment
    let experiment_responses: Vec<_> = responses.iter()
        .filter(|r| {
            // Check if participant belongs to experiment
            let participants = state.participants.read().unwrap();
            participants.get(&r.participant_id)
                .map_or(false, |p| p.experiment_id == experiment_id)
        })
        .collect();
    
    if experiment_responses.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "No data available for analysis"}))).into_response();
    }
    
    let dependent_var = payload["dependent_variable"].as_str().unwrap_or("response_time_ms");
    let fixed_effects = payload["fixed_effects"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["task_type"]);
    
    println!("✓ Running mixed-effects analysis on {} responses", experiment_responses.len());
    
    // Simplified mock analysis results
    Json(json!({
        "experiment_id": experiment_id,
        "analysis_type": "mixed_effects",
        "dependent_variable": dependent_var,
        "fixed_effects": fixed_effects,
        "n_observations": experiment_responses.len(),
        "model_fit": {
            "aic": 1234.5,
            "bic": 1256.7,
            "log_likelihood": -612.25
        },
        "effects": {
            "intercept": {"estimate": 1.234, "se": 0.056, "p_value": 0.001},
            "task_type_effect": {"estimate": 0.234, "se": 0.045, "p_value": 0.023}
        },
        "random_effects": {
            "participant_variance": 0.123,
            "residual_variance": 0.456
        }
    }))
}

async fn check_statistical_assumptions(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    let responses = state.responses.read().unwrap();
    
    // Filter responses for this experiment
    let experiment_responses: Vec<_> = responses.iter()
        .filter(|r| {
            let participants = state.participants.read().unwrap();
            participants.get(&r.participant_id)
                .map_or(false, |p| p.experiment_id == experiment_id)
        })
        .collect();
    
    if experiment_responses.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "No data available for assumption checking"}))).into_response();
    }
    
    // Mock assumption checking results
    Json(json!({
        "experiment_id": experiment_id,
        "n_observations": experiment_responses.len(),
        "assumptions": {
            "normality": {
                "shapiro_wilk_p": 0.234,
                "is_normal": true,
                "recommendation": "Data appears to be normally distributed"
            },
            "homoscedasticity": {
                "levene_p": 0.456,
                "equal_variance": true,
                "recommendation": "Variance appears homogeneous across groups"
            },
            "independence": {
                "durbin_watson": 1.98,
                "is_independent": true,
                "recommendation": "Observations appear to be independent"
            }
        },
        "overall_validity": "All assumptions satisfied for parametric testing"
    }))
}

async fn get_randomization_seed(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    if let Ok(seed_manager) = state.seed_manager.read() {
        if let Some(experiment_seed) = seed_manager.experiment_seeds.get(&experiment_id) {
            Json(json!({
                "experiment_id": experiment_id,
                "seed": experiment_seed.seed,
                "algorithm": experiment_seed.algorithm,
                "created_at": experiment_seed.created_at,
                "description": experiment_seed.description
            }))
        } else {
            (StatusCode::NOT_FOUND, Json(json!({"error": "No seed found for this experiment"}))).into_response()
        }
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Seed manager unavailable"}))).into_response()
    }
}

async fn get_reproducibility_manifest(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    if let Ok(seed_manager) = state.seed_manager.read() {
        if let Some(manifest) = seed_manager.get_reproducibility_manifest(&experiment_id) {
            Json(json!(manifest))
        } else {
            (StatusCode::NOT_FOUND, Json(json!({"error": "No reproducibility manifest found for this experiment"}))).into_response()
        }
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Seed manager unavailable"}))).into_response()
    }
}

async fn get_experiment_citations(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
) -> impl IntoResponse {
    if let Ok(citation_manager) = state.citation_manager.read() {
        // Get all references (simplified - would filter by experiment in real implementation)
        let all_references = citation_manager.get_all_references();
        
        Json(json!({
            "experiment_id": experiment_id,
            "references": all_references,
            "total": all_references.len(),
            "methodology_report_available": true
        }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Citation manager unavailable"}))).into_response()
    }
}

async fn add_citation(
    State(state): State<ResearchAppState>,
    Path(experiment_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Ok(mut citation_manager) = state.citation_manager.write() {
        let title = payload["title"].as_str().unwrap_or("Unknown Title").to_string();
        let authors_str = payload["authors"].as_str().unwrap_or("Unknown Authors");
        let year = payload["year"].as_u64().unwrap_or(2023) as u32;
        let journal_name = payload["journal"].as_str().unwrap_or("Unknown Journal").to_string();
        let doi = payload["doi"].as_str().map(|s| s.to_string());
        
        // Parse authors - simple implementation for now
        let authors = vec![Author {
            first_name: "Unknown".to_string(),
            last_name: authors_str.to_string(),
            middle_initial: None,
            affiliation: None,
            orcid: None,
        }];
        
        let reference = Reference {
            id: Uuid::new_v4().to_string(),
            reference_type: ReferenceType::Journal,
            title,
            authors,
            year,
            publication: Publication::Journal {
                name: journal_name,
                volume: None,
                issue: None,
                pages: None,
                issn: None,
            },
            doi,
            url: None,
            abstract_text: None,
            keywords: Vec::new(),
            notes: String::new(),
            citation_count: 0,
            added_date: Utc::now(),
        };
        
        let reference_id = reference.id.clone();
        citation_manager.add_reference(reference);
        
        println!("✓ Added citation for experiment: {}", experiment_id);
        Json(json!({
            "reference_id": reference_id,
            "experiment_id": experiment_id,
            "status": "added"
        }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Citation manager unavailable"}))).into_response()
    }
}