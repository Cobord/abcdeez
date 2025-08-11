use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    state::AppState,
};

use abcdeez_core::{
    // statistical_validation::{StatisticalValidator, ValidationReport, CrossValidationResults},
    // transfer_learning::{TransferLearningSystem, IsomorphicMapping},
    experiments::core::Experiment,
    statistics::core::{ExGaussianModel, SessionAnalyzer, StrategyType},
    // population::{PopulationAnalyzer, PopulationInsights},
};

// TODO: These modules need to be implemented in abcdeez_core
// For now, create placeholder types with minimal APIs used below
#[derive(Debug, Clone, Serialize)]
struct ValidationReport {
    sample_size_adequate: Option<bool>,
    assumptions_met: Option<bool>,
}

impl ValidationReport {
    fn is_valid(&self) -> bool {
        self.sample_size_adequate.unwrap_or(true) && self.assumptions_met.unwrap_or(true)
    }
}

struct StatisticalValidator(f64, f64);
impl StatisticalValidator {
    fn new(_alpha: f64, power: f64) -> Self { Self(0.0, power) }
    fn validate(&self, _experiment: &Experiment) -> Result<ValidationReport, AppError> {
        Ok(ValidationReport { sample_size_adequate: Some(true), assumptions_met: Some(true) })
    }
    fn calculate_power(&self, _effect_size: f64, _n: usize) -> f64 { self.1 }
    fn calculate_sample_size(&self, _effect_size: f64, _alpha: f64, _power: f64) -> usize { 100 }
    fn cross_validate(&self, _k: usize, _stratified: bool) -> Result<CrossValidationResults, AppError> {
        Ok(CrossValidationResults { mean_accuracy: 0.8, variance: 0.05 })
    }
}

#[derive(Debug, Clone, Serialize)]
struct CrossValidationResults { mean_accuracy: f64, variance: f64 }

struct TransferLearningSystem;
impl TransferLearningSystem {
    fn new() -> Self { TransferLearningSystem }
    fn calculate_transfer_potential(&self, _src: &str, _tgt: &str) -> f64 { 0.5 }
    fn find_isomorphic_mapping(&self, _src: &str, _tgt: &str) -> Vec<(String, String)> { vec![] }
}

struct PopulationAnalyzer;
impl PopulationAnalyzer {
    fn new() -> Self { PopulationAnalyzer }
    fn generate_insights(&self, _n: usize, _dist: Vec<(f64, usize)>, _curves: Vec<(usize, f64)>) -> PopulationInsights { PopulationInsights }
}

#[derive(Debug, Clone, Serialize)]
struct PopulationInsights;

// ============= Statistical Validation Endpoints =============

#[derive(Debug, Deserialize)]
pub struct ValidationRequest {
    _experiment_id: Uuid,
    alpha: Option<f64>,
    power: Option<f64>,
    effect_size: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    report: ValidationReport,
    is_valid: bool,
    recommendations: Vec<String>,
}

/// Run statistical validation on an experiment
pub async fn validate_experiment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ValidationRequest>,
) -> AppResult<Json<ValidationResponse>> {
    // Check researcher permissions
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    let mut _conn = state.db_pool.acquire().await?;
    
    // Convert to core experiment format (placeholder/minimal)
    let experiment = Experiment {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Research Experiment".to_string(),
        description: String::new(),
        config: abcdeez_core::experiments::core::ExperimentConfig {
            topology_type: "alphabet".to_string(),
            n_participants: 0,
            n_sessions_per_participant: 0,
            n_trials_per_session: 0,
            adaptive_scheduling: false,
            use_bayesian_model: false,
            use_strategy_mixture: false,
            use_hierarchical_model: false,
            use_transfer_learning: false,
            use_macro_learning: false,
            randomization: abcdeez_core::experiments::core::RandomizationConfig {
                randomize_conditions: false,
                randomize_trials: false,
                counterbalance: false,
                block_size: None,
            },
        },
        conditions: Vec::new(),
        participants: Vec::new(),
        sessions: Vec::new(),
        results: None,
        metadata: abcdeez_core::experiments::core::ExperimentMetadata {
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            researcher: String::new(),
            notes: String::new(),
        },
    };
    
    // Run statistical validation
    let validator = StatisticalValidator::new(
        req.alpha.unwrap_or(0.05),
        req.power.unwrap_or(0.8),
    );
    
    let report = validator.validate(&experiment)?;
    
    // Generate recommendations based on results
    let mut recommendations = Vec::new();
    if report.sample_size_adequate == Some(false) {
        recommendations.push("Increase sample size for better statistical power".to_string());
    }
    if report.assumptions_met == Some(false) {
        recommendations.push("Data violates statistical assumptions - consider non-parametric tests".to_string());
    }
    
    Ok(Json(ValidationResponse {
        is_valid: report.is_valid(),
        report,
        recommendations,
    }))
}

/// Calculate statistical power for an experiment
pub async fn calculate_power(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ValidationRequest>,
) -> AppResult<Json<PowerAnalysisResponse>> {
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    let validator = StatisticalValidator::new(
        req.alpha.unwrap_or(0.05),
        req.power.unwrap_or(0.8),
    );
    
    let power = validator.calculate_power(
        req.effect_size.unwrap_or(0.5),
        100, // sample size - should be from actual data
    );
    
    let required_n = validator.calculate_sample_size(
        req.effect_size.unwrap_or(0.5),
        req.alpha.unwrap_or(0.05),
        req.power.unwrap_or(0.8),
    );
    
    Ok(Json(PowerAnalysisResponse {
        statistical_power: power,
        required_sample_size: required_n,
        current_effect_size: req.effect_size.unwrap_or(0.5),
        alpha: req.alpha.unwrap_or(0.05),
    }))
}

#[derive(Debug, Serialize)]
pub struct PowerAnalysisResponse {
    statistical_power: f64,
    required_sample_size: usize,
    current_effect_size: f64,
    alpha: f64,
}

// ============= Ex-Gaussian Analysis Endpoints =============

#[derive(Debug, Deserialize)]
pub struct ExGaussianRequest {
    learner_id: Option<Uuid>,
    session_id: Option<Uuid>,
    from_date: Option<DateTime<Utc>>,
    to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ExGaussianResponse {
    parameters: abcdeez_core::statistics::core::ExGaussianParameters,
    goodness_of_fit: f64,
    outliers: Vec<f64>,
    visualization_data: Vec<(f64, f64)>, // For plotting
}

/// Fit Ex-Gaussian distribution to response times
pub async fn analyze_response_times(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ExGaussianRequest>,
) -> AppResult<Json<ExGaussianResponse>> {
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    let mut conn = state.db_pool.acquire().await?;
    
    // Build parameterized query to prevent SQL injection
    let mut sql = String::from(
        "SELECT response_time_ms FROM session_responses WHERE 1=1"
    );
    let mut bindings: Vec<String> = Vec::new();
    
    if let Some(learner_id) = req.learner_id {
        sql.push_str(" AND learner_id = ?");
        bindings.push(learner_id.to_string());
    }
    if let Some(session_id) = req.session_id {
        sql.push_str(" AND session_id = ?");
        bindings.push(session_id.to_string());
    }
    if let Some(from) = req.from_date {
        sql.push_str(" AND created_at >= ?");
        bindings.push(from.to_string());
    }
    if let Some(to) = req.to_date {
        sql.push_str(" AND created_at <= ?");
        bindings.push(to.to_string());
    }
    
    // Build and execute query with bindings
    let mut query = sqlx::query_scalar(&sql);
    for binding in &bindings {
        query = query.bind(binding);
    }
    
    let response_times: Vec<f64> = query
        .fetch_all(&mut *conn)
        .await?;
    
    if response_times.is_empty() {
        return Err(AppError::NotFound("No response times found".to_string()));
    }
    
    // Fit Ex-Gaussian model
    let model = ExGaussianModel::fit(&response_times);
    let parameters = model.params.clone();
    let gof = 1.0; // Placeholder goodness-of-fit
    
    // Detect outliers
    let outliers_idx = abcdeez_core::statistics::core::ResponseTimeDistribution::detect_outliers(&response_times, 3.0);
    let outliers = outliers_idx.into_iter().filter_map(|i| response_times.get(i).cloned()).collect();
    
    // Generate visualization data (PDF values)
    let min_rt = response_times.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max_rt = response_times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let mut viz_data = Vec::new();
    
    let step = (max_rt - min_rt) / 100.0;
    let mut x = *min_rt;
    while x <= *max_rt {
        let y = model.pdf(x);
        viz_data.push((x, y));
        x += step;
    }
    
    Ok(Json(ExGaussianResponse {
        parameters,
        goodness_of_fit: gof,
        outliers,
        visualization_data: viz_data,
    }))
}

// ============= Strategy Detection Endpoints =============

#[derive(Debug, Serialize)]
pub struct StrategyAnalysisResponse {
    detected_strategies: Vec<StrategyType>,
    confidence_scores: Vec<f64>,
    strategy_transitions: Vec<(String, String, f64)>, // from, to, probability
    recommendations: Vec<String>,
}

/// Detect learning strategies from session data
pub async fn detect_strategies(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(learner_id): Path<Uuid>,
) -> AppResult<Json<StrategyAnalysisResponse>> {
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    let mut conn = state.db_pool.acquire().await?;
    
    // Load learner's session data
    let sessions: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
        "SELECT * FROM sessions WHERE learner_id = ? ORDER BY created_at",
    )
    .bind(learner_id.as_bytes().as_slice())
    .fetch_all(&mut *conn)
    .await
    .unwrap_or_default();
    
    // Analyze strategies using SessionAnalyzer
    let _analyzer = SessionAnalyzer::new(Vec::new());
    let all_strategies = Vec::new();
    let confidence_scores = Vec::new();
    
    for _session in sessions {
        // Load responses for this session
        let _responses: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT * FROM session_responses WHERE session_id = ? ORDER BY created_at",
        )
        .bind("")
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default();
        
        // Detect strategy for this session
        // Placeholder: no core API present; skip detection
    }
    
    // Analyze strategy transitions
    let mut transitions = Vec::new();
    for i in 1..all_strategies.len() {
        let from = format!("{:?}", all_strategies[i-1]);
        let to = format!("{:?}", all_strategies[i]);
        transitions.push((from, to, 1.0)); // Simplified - could calculate actual probability
    }
    
    // Generate recommendations
    let mut recommendations = Vec::new();
    if all_strategies.iter().any(|s| matches!(s, StrategyType::Hybrid)) {
        recommendations.push("Learner shows struggling pattern - consider intervention".to_string());
    }
    if all_strategies.iter().any(|s| matches!(s, StrategyType::DirectAccess)) {
        recommendations.push("Learner showing optimal strategy - consider increasing difficulty".to_string());
    }
    
    Ok(Json(StrategyAnalysisResponse {
        detected_strategies: all_strategies,
        confidence_scores,
        strategy_transitions: transitions,
        recommendations,
    }))
}

// ============= Transfer Learning Analysis Endpoints =============

#[derive(Debug, Deserialize)]
pub struct TransferAnalysisRequest {
    source_domain: String,
    target_domain: String,
    _learner_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct TransferAnalysisResponse {
    transfer_potential: f64,
    shared_skills: Vec<String>,
    mapping: Vec<(String, String, f64)>, // source_node, target_node, similarity
    recommendations: Vec<String>,
}

/// Analyze transfer learning potential between domains
pub async fn analyze_transfer(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TransferAnalysisRequest>,
) -> AppResult<Json<TransferAnalysisResponse>> {
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    // Initialize transfer learning system
    let transfer_system = TransferLearningSystem::new();
    
    // Calculate transfer potential
    let transfer_potential = transfer_system.calculate_transfer_potential(
        &req.source_domain,
        &req.target_domain,
    );
    
    // Find isomorphic mappings
    let mapping = transfer_system.find_isomorphic_mapping(
        &req.source_domain,
        &req.target_domain,
    );
    
    // Identify shared skills
    let shared_skills = vec![
        "Pattern recognition".to_string(),
        "Sequential learning".to_string(),
        "Memory retention".to_string(),
    ]; // Simplified - would analyze actual skills
    
    // Generate recommendations
    let mut recommendations = Vec::new();
    if transfer_potential > 0.7 {
        recommendations.push("High transfer potential - leverage existing knowledge".to_string());
    } else if transfer_potential < 0.3 {
        recommendations.push("Low transfer potential - treat as independent domain".to_string());
    }
    
    Ok(Json(TransferAnalysisResponse {
        transfer_potential,
        shared_skills,
        mapping: mapping.into_iter().map(|(s, t)| (s, t, 0.8)).collect(), // Add similarity scores
        recommendations,
    }))
}

// ============= Population-Level Analytics Endpoints =============

#[derive(Debug, Serialize)]
pub struct PopulationAnalysisResponse {
    total_learners: usize,
    active_learners: usize,
    average_proficiency: f64,
    proficiency_distribution: Vec<(f64, usize)>, // proficiency_level, count
    learning_curves: Vec<(usize, f64)>, // session_number, avg_performance
    strategy_distribution: Vec<(String, usize)>, // strategy, count
    insights: PopulationInsights,
}

/// Analyze population-level learning patterns
pub async fn analyze_population(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(_params): Query<PopulationQuery>,
) -> AppResult<Json<PopulationAnalysisResponse>> {
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    let mut conn = state.db_pool.acquire().await?;
    
    // Get population statistics
    let total_learners: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT learner_id) FROM sessions"
    )
    .fetch_one(&mut *conn)
    .await?;
    
    let active_learners: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT learner_id) FROM sessions 
         WHERE created_at > datetime('now', '-30 days')"
    )
    .fetch_one(&mut *conn)
    .await?;
    
    // Get proficiency distribution
    let proficiency_data: Vec<(f64, i64)> = Vec::new();
    
    let proficiency_distribution: Vec<(f64, usize)> = proficiency_data
        .into_iter()
        .map(|(avg, count)| (avg, count as usize))
        .collect();
    
    // Calculate learning curves
    let learning_curve_data: Vec<(i64, f64)> = Vec::new();
    
    let learning_curves: Vec<(usize, f64)> = learning_curve_data
        .into_iter()
        .map(|(n, avg)| (n as usize, avg))
        .collect();
    
    // Initialize population analyzer
    let analyzer = PopulationAnalyzer::new();
    let insights = analyzer.generate_insights(
        total_learners as usize,
        proficiency_distribution.clone(),
        learning_curves.clone(),
    );
    
    Ok(Json(PopulationAnalysisResponse {
        total_learners: total_learners as usize,
        active_learners: active_learners as usize,
        average_proficiency: proficiency_distribution.iter()
            .map(|(p, c)| p * (*c as f64))
            .sum::<f64>() / total_learners as f64,
        proficiency_distribution,
        learning_curves,
        strategy_distribution: vec![], // Would need strategy detection implemented
        insights,
    }))
}

#[derive(Debug, Deserialize)]
pub struct PopulationQuery {
    _from_date: Option<DateTime<Utc>>,
    _to_date: Option<DateTime<Utc>>,
    _domain: Option<String>,
}

// ============= Cross-Validation Endpoints =============

#[derive(Debug, Deserialize)]
pub struct CrossValidationRequest {
    model_type: String, // "bayesian", "irt", etc.
    k_folds: Option<usize>,
    stratified: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CrossValidationResponse {
    results: CrossValidationResults,
    best_model_params: serde_json::Value,
    recommendations: Vec<String>,
}

/// Run cross-validation on learner models
pub async fn cross_validate_models(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CrossValidationRequest>,
) -> AppResult<Json<CrossValidationResponse>> {
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    let validator = StatisticalValidator::new(0.05, 0.8);
    
    // Run cross-validation (simplified - would use actual data)
    let results = validator.cross_validate(
        req.k_folds.unwrap_or(5),
        req.stratified.unwrap_or(true),
    )?;
    
    // Generate recommendations
    let mut recommendations = Vec::new();
    if results.mean_accuracy < 0.7 {
        recommendations.push("Model accuracy is low - consider feature engineering".to_string());
    }
    if results.variance > 0.1 {
        recommendations.push("High variance across folds - model may be overfitting".to_string());
    }
    
    Ok(Json(CrossValidationResponse {
        results,
        best_model_params: serde_json::json!({
            "type": req.model_type,
            "optimal_params": {}
        }),
        recommendations,
    }))
}

// ============= Batch Export for External Analysis =============

#[derive(Debug, Deserialize)]
pub struct BatchExportRequest {
    format: String, // "csv", "json", "parquet"
    include_raw_data: bool,
    include_models: bool,
    include_statistics: bool,
}

/// Export data for external statistical analysis (R, Python, etc.)
pub async fn export_for_analysis(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<BatchExportRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.permissions.contains(&"research".to_string()) {
        return Err(AppError::Forbidden);
    }

    let mut conn = state.db_pool.acquire().await?;
    
    let mut export_data = serde_json::json!({
        "metadata": {
            "export_date": Utc::now(),
            "format": req.format,
            "version": "1.0.0"
        }
    });
    
    if req.include_raw_data {
        export_data["sessions"] = serde_json::json!([]);
        
        export_data["responses"] = serde_json::json!([]);
    }
    
    if req.include_models {
        export_data["models"] = serde_json::json!([]);
    }
    
    if req.include_statistics {
        // Add statistical summaries
        export_data["statistics"] = serde_json::json!({
            "total_sessions": sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
                .fetch_one(&mut *conn).await?,
            "total_responses": sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM session_responses")
                .fetch_one(&mut *conn).await?,
            "average_performance": sqlx::query_scalar::<_, f64>("SELECT AVG(performance) FROM sessions")
                .fetch_one(&mut *conn).await?,
        });
    }
    
    Ok(Json(export_data))
}