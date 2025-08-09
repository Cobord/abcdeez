use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use sqlx::Row;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::{session::Session, learner::Learner},
    services::{audit::AuditService, adaptation_service::AdaptationService, learner_service::LearnerService},
    state::AppState,
};

use graph_learning_core::{
    Task, TaskGenerator, TaskType, Topology,
    bayesian::BayesianLearnerModel, learner::LearnerModel,
};

/// Generate next task using adaptive scheduling
pub async fn next(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<NextTaskRequest>,
) -> AppResult<(StatusCode, Json<TaskResponse>)> {
    // Get session and verify permissions
    let session = get_session_with_permission(&state, &claims, req.session_id).await?;
    
    // Get learner service
    let learner_service = LearnerService::new(
        Arc::new(state.db_pool.clone()),
        state.redis_conn.clone(),
    );
    
    // Get learner model
    let learner = learner_service
        .get_learner(session.learner_id)
        .await
        .map_err(|_| AppError::NotFound("Learner not found".to_string()))?;
    
    // Parse topology from session
    let topology: Topology = serde_json::from_value(session.topology_data)
        .unwrap_or_else(|_| Topology::alphabet());
    
    // Create task generator
    let mut task_generator = TaskGenerator::new(topology.clone());
    
    // Generate task based on request parameters
    let task = if req.use_adaptive.unwrap_or(false) {
        // Use adaptive scheduling with EIG
        let adaptation_service = AdaptationService::new(Arc::new(learner_service));
        
        let next_task = adaptation_service
            .select_next_task(session.learner_id, &topology)
            .await
            .unwrap_or_else(|_| {
                // Fallback to basic task generation
                task_generator.generate_task(None) // Random task
            });
        
        next_task
    } else {
        // Generate basic task with random type
        task_generator.generate_task(None)
    };
    
    // Calculate expected difficulty based on learner model
    let expected_difficulty = calculate_task_difficulty(&task, &learner);
    
    // Calculate information gain if requested
    let information_gain = if req.use_eig.unwrap_or(false) {
        Some(calculate_information_gain(&task, &learner, &topology))
    } else {
        None
    };
    
    // Create task metadata
    let task_metadata = TaskMetadata {
        task_id: Uuid::new_v4(),
        generated_at: chrono::Utc::now(),
        generator_version: "1.0".to_string(),
        difficulty_source: if req.use_adaptive.unwrap_or(false) {
            "adaptive"
        } else {
            "fixed"
        }.to_string(),
    };
    
    // Log task generation
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "generate".to_string(),
        "task".to_string(),
        task_metadata.task_id.to_string(),
        Some(serde_json::json!({
            "session_id": req.session_id,
            "task_type": format!("{:?}", task.task_type),
            "difficulty": expected_difficulty,
            "adaptive": req.use_adaptive.unwrap_or(false),
            "eig": req.use_eig.unwrap_or(false)
        })),
        None,
        None,
    )
    .await
    .ok();
    
    Ok((StatusCode::OK, Json(TaskResponse {
        task,
        expected_difficulty,
        information_gain,
        task_metadata,
    })))
}

/// Generate multiple tasks for batch processing
pub async fn generate(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<TaskGenerationRequest>,
) -> AppResult<(StatusCode, Json<BulkTaskResponse>)> {
    // Verify learner access
    let learner_service = LearnerService::new(
        Arc::new(state.db_pool.clone()),
        state.redis_conn.clone(),
    );
    
    let learner = learner_service
        .get_learner(req.learner_id)
        .await
        .map_err(|_| AppError::NotFound("Learner not found".to_string()))?;
    
    // Check permissions
    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }
    
    // Create topology from type
    let topology = create_topology_from_string(&req.topology_type)?;
    let mut task_generator = TaskGenerator::new(topology.clone());
    
    // Generate tasks
    let count = req.count.unwrap_or(10).min(50); // Limit to 50 tasks
    let target_difficulty = req.difficulty_target.unwrap_or(0.5);
    
    let mut tasks = Vec::new();
    let mut difficulty_counts = DifficultyDistribution {
        easy: 0,
        medium: 0,
        hard: 0,
        average: 0.0,
    };
    
    let mut total_difficulty = 0.0;
    
    for _ in 0..count {
        // Add some randomness to difficulty
        let difficulty_variance = 0.2;
        let actual_difficulty = (target_difficulty + 
            (rand::random::<f64>() - 0.5) * difficulty_variance)
            .max(0.1)
            .min(0.9);
        
        let task = task_generator.generate_task(None); // Generate random task, difficulty handled separately
        let expected_difficulty = calculate_task_difficulty(&task, &learner);
        
        // Update difficulty distribution
        total_difficulty += expected_difficulty;
        if expected_difficulty < 0.4 {
            difficulty_counts.easy += 1;
        } else if expected_difficulty < 0.7 {
            difficulty_counts.medium += 1;
        } else {
            difficulty_counts.hard += 1;
        }
        
        let task_metadata = TaskMetadata {
            task_id: Uuid::new_v4(),
            generated_at: chrono::Utc::now(),
            generator_version: "1.0".to_string(),
            difficulty_source: "batch".to_string(),
        };
        
        tasks.push(TaskResponse {
            task,
            expected_difficulty,
            information_gain: None,
            task_metadata,
        });
    }
    
    difficulty_counts.average = total_difficulty / count as f64;
    
    // Log batch generation
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "batch_generate".to_string(),
        "task".to_string(),
        req.learner_id.to_string(),
        Some(serde_json::json!({
            "learner_id": req.learner_id,
            "topology_type": req.topology_type,
            "count": count,
            "target_difficulty": target_difficulty,
            "actual_average": difficulty_counts.average
        })),
        None,
        None,
    )
    .await
    .ok();
    
    Ok((StatusCode::OK, Json(BulkTaskResponse {
        tasks,
        total_generated: count,
        difficulty_distribution: difficulty_counts,
    })))
}

/// Get difficulty analysis for tasks and learner
pub async fn difficulty(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<DifficultyQuery>,
) -> AppResult<(StatusCode, Json<DifficultyAnalysis>)> {
    let mut task_type_difficulties = std::collections::HashMap::new();
    
    // Base difficulties for different task types
    task_type_difficulties.insert("PairwiseOrder".to_string(), 0.3);
    task_type_difficulties.insert("Successor".to_string(), 0.4);
    task_type_difficulties.insert("Predecessor".to_string(), 0.4);
    task_type_difficulties.insert("KJump".to_string(), 0.6);
    task_type_difficulties.insert("Segment".to_string(), 0.5);
    task_type_difficulties.insert("Index".to_string(), 0.7);
    task_type_difficulties.insert("MissingItem".to_string(), 0.6);
    task_type_difficulties.insert("ShortestDistance".to_string(), 0.8);
    
    let mut item_difficulties = None;
    let mut learner_performance = None;
    let mut recommended_difficulty = 0.5;
    
    // If learner is specified, get personalized analysis
    if let Some(learner_id) = params.learner_id {
        let learner_service = LearnerService::new(
            Arc::new(state.db_pool.clone()),
            state.redis_conn.clone(),
        );
        
        if let Ok(learner) = learner_service.get_learner(learner_id).await {
            // Check permissions
            if let Some(learner_user_id) = learner.user_id {
                if learner_user_id != claims.sub {
                    return Err(AppError::Forbidden);
                }
            }
            
            // Get recent performance
            let recent_accuracy = get_recent_accuracy(&state, learner_id).await?;
            
            // Determine trend
            let trend = if recent_accuracy > 0.8 {
                "improving"
            } else if recent_accuracy > 0.6 {
                "stable"
            } else {
                "declining"
            };
            
            // Calculate recommended difficulty based on performance
            recommended_difficulty = if recent_accuracy > 0.8 {
                0.7 // Increase difficulty
            } else if recent_accuracy < 0.6 {
                0.4 // Decrease difficulty
            } else {
                0.5 // Maintain current level
            };
            
            learner_performance = Some(LearnerPerformance {
                current_accuracy: recent_accuracy,
                trend: trend.to_string(),
                struggle_areas: vec!["KJump".to_string(), "Index".to_string()],
                strong_areas: vec!["Successor".to_string(), "PairwiseOrder".to_string()],
            });
            
            // Get item-specific difficulties if requested
            if params.item_difficulty.unwrap_or(false) {
                item_difficulties = Some(get_item_difficulties(&state, learner_id).await?);
            }
        }
    }
    
    Ok((StatusCode::OK, Json(DifficultyAnalysis {
        task_type_difficulties,
        item_difficulties,
        learner_performance,
        recommended_difficulty,
    })))
}

/// Request a hint for current task
pub async fn request_hint(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<HintRequest>,
) -> AppResult<(StatusCode, Json<HintResponse>)> {
    // For now, generate basic hints
    // In full implementation, this would use the intervention system
    
    let hint_text = match req.task_type.as_str() {
        "PairwiseOrder" => "Think about the alphabetical order of the letters.",
        "Successor" => "What letter comes immediately after this one in the alphabet?",
        "Predecessor" => "What letter comes immediately before this one in the alphabet?",
        "KJump" => "Count the specified number of positions from the starting letter.",
        "Segment" => "List the letters in sequence starting from the given position.",
        "Index" => "Count the position of this letter in the alphabet (A=1, B=2, etc.).",
        _ => "Break down the problem into smaller steps.",
    };
    
    Ok((StatusCode::OK, Json(HintResponse {
        hint_text: hint_text.to_string(),
        hint_level: 1,
        hint_type: "conceptual".to_string(),
        timestamp: chrono::Utc::now(),
    })))
}

// Helper functions
async fn get_session_with_permission(
    state: &Arc<AppState>,
    claims: &Claims,
    session_id: Uuid,
) -> AppResult<Session> {
    let session_bytes = session_id.as_bytes();
    let session = sqlx::query_as::<_, Session>(
        "SELECT id, learner_id, topology_type, topology_data, start_time, end_time, status, summary FROM sessions WHERE id = ?"
    )
    .bind(&session_bytes[..])
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e))?
    .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

    // Verify session belongs to user by getting learner
    let learner_service = LearnerService::new(
        Arc::new(state.db_pool.clone()),
        state.redis_conn.clone(),
    );
    
    let learner = learner_service
        .get_learner(session.learner_id)
        .await
        .map_err(|_| AppError::NotFound("Learner not found".to_string()))?;

    if let Some(user_id) = learner.user_id {
        if user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    Ok(session)
}

fn calculate_task_difficulty(task: &Task, learner: &Learner) -> f64 {
    // Base difficulty from task
    let mut difficulty = task.difficulty;

    // Adjust based on learner's proficiency with this operation type
    if let Some(proficiency) = learner.learning_model.operation_proficiencies.get(&format!("{:?}", task.operation)) {
        // Higher proficiency = easier (lower effective difficulty)
        let proficiency_adjustment = 1.0 - (1.0 / (1.0 + (-proficiency.theta).exp()));
        difficulty = difficulty * (1.0 + proficiency_adjustment * 0.3);
    }

    // Adjust based on node-specific knowledge for position-based tasks
    match &task.task_type {
        TaskType::PairwiseOrder { a, b } => {
            let avg_uncertainty = get_node_uncertainty(&learner.learning_model, a) 
                + get_node_uncertainty(&learner.learning_model, b);
            difficulty += avg_uncertainty * 0.2;
        },
        TaskType::Successor { item } | TaskType::Predecessor { item } => {
            difficulty += get_node_uncertainty(&learner.learning_model, item) * 0.3;
        },
        _ => {}
    }

    difficulty.clamp(0.1, 1.0)
}

fn get_node_uncertainty(model: &LearnerModel, item: &str) -> f64 {
    // Find node by label and get uncertainty
    for (_, embedding) in &model.node_embeddings {
        // This is a simplified lookup - in practice we'd need topology access
        // For now, return average uncertainty
        return embedding.uncertainty;
    }
    0.5 // Default uncertainty
}

fn calculate_information_gain(task: &Task, learner: &Learner, topology: &Topology) -> f64 {
    // Create a temporary Bayesian model to calculate EIG
    let bayesian_model = BayesianLearnerModel::new(topology);
    
    // Calculate expected information gain for this task
    // This is a simplified version - full implementation would simulate both outcomes
    let base_entropy = bayesian_model.total_entropy();
    
    // Estimate entropy reduction based on task type and learner uncertainty
    let entropy_reduction = match &task.task_type {
        TaskType::PairwiseOrder { a, b } => {
            let uncertainty_a = get_node_uncertainty(&learner.learning_model, a);
            let uncertainty_b = get_node_uncertainty(&learner.learning_model, b);
            (uncertainty_a + uncertainty_b) * 0.5
        },
        TaskType::Successor { item } | TaskType::Predecessor { item } => {
            get_node_uncertainty(&learner.learning_model, item) * 0.7
        },
        TaskType::KJump { .. } => 0.8, // High information tasks
        TaskType::Segment { count, .. } => *count as f64 * 0.1,
        _ => 0.5
    };
    
    entropy_reduction.min(base_entropy)
}

fn create_topology_from_string(topology_type: &str) -> AppResult<Topology> {
    match topology_type {
        "alphabet" => Ok(Topology::alphabet()),
        "numbers" => {
            let numbers: Vec<String> = (1..=26).map(|n| n.to_string()).collect();
            Ok(Topology::new_linear(numbers))
        },
        "days_of_week" => {
            let days = vec![
                "Monday".to_string(), "Tuesday".to_string(), "Wednesday".to_string(),
                "Thursday".to_string(), "Friday".to_string(), "Saturday".to_string(),
                "Sunday".to_string()
            ];
            Ok(Topology::new_cyclic(days))
        },
        "music_notes" => {
            let notes = vec![
                "C".to_string(), "D".to_string(), "E".to_string(), "F".to_string(),
                "G".to_string(), "A".to_string(), "B".to_string()
            ];
            Ok(Topology::new_linear(notes))
        },
        _ => Err(AppError::BadRequest(format!("Unknown topology type: {}", topology_type)))
    }
}

async fn get_recent_accuracy(state: &AppState, learner_id: Uuid) -> AppResult<f64> {
    // Get recent task responses (last 20)
    let learner_bytes = learner_id.as_bytes();
    let responses = sqlx::query(
        "SELECT correct FROM task_responses 
         WHERE learner_id = ? 
         ORDER BY created_at DESC 
         LIMIT 20"
    )
    .bind(&learner_bytes[..])
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    if responses.is_empty() {
        return Ok(0.5); // Default accuracy
    }

    let correct_count = responses.iter().filter(|r| {
        r.try_get::<bool, _>("correct").unwrap_or(false)
    }).count();
    Ok(correct_count as f64 / responses.len() as f64)
}

async fn get_item_difficulties(state: &AppState, learner_id: Uuid) -> AppResult<std::collections::HashMap<String, f64>> {
    // Get item-specific performance data
    let mut item_difficulties = std::collections::HashMap::new();
    
    // This would normally query actual task response data
    // For now, return some sample data
    item_difficulties.insert("A".to_string(), 0.2);
    item_difficulties.insert("B".to_string(), 0.3);
    item_difficulties.insert("Z".to_string(), 0.9);
    
    Ok(item_difficulties)
}

// Request/Response models
#[derive(Debug, Deserialize)]
pub struct NextTaskRequest {
    pub session_id: Uuid,
    pub use_adaptive: Option<bool>,
    pub use_eig: Option<bool>,
    pub task_type: Option<String>,
    pub difficulty_override: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub task: Task,
    pub expected_difficulty: f64,
    pub information_gain: Option<f64>,
    pub task_metadata: TaskMetadata,
}

#[derive(Debug, Serialize)]
pub struct TaskMetadata {
    pub task_id: Uuid,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub generator_version: String,
    pub difficulty_source: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskGenerationRequest {
    pub learner_id: Uuid,
    pub topology_type: String,
    pub count: Option<usize>,
    pub difficulty_target: Option<f64>,
    pub task_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct BulkTaskResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_generated: usize,
    pub difficulty_distribution: DifficultyDistribution,
}

#[derive(Debug, Serialize)]
pub struct DifficultyDistribution {
    pub easy: usize,    // < 0.4
    pub medium: usize,  // 0.4 - 0.7
    pub hard: usize,    // > 0.7
    pub average: f64,
}

#[derive(Debug, Deserialize)]
pub struct DifficultyQuery {
    pub learner_id: Option<Uuid>,
    pub item_difficulty: Option<bool>,
    pub task_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct DifficultyAnalysis {
    pub task_type_difficulties: std::collections::HashMap<String, f64>,
    pub item_difficulties: Option<std::collections::HashMap<String, f64>>,
    pub learner_performance: Option<LearnerPerformance>,
    pub recommended_difficulty: f64,
}

#[derive(Debug, Serialize)]
pub struct LearnerPerformance {
    pub current_accuracy: f64,
    pub trend: String,
    pub struggle_areas: Vec<String>,
    pub strong_areas: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct HintRequest {
    pub task_type: String,
    pub current_item: Option<String>,
    pub hint_level: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct HintResponse {
    pub hint_text: String,
    pub hint_level: u8,
    pub hint_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}