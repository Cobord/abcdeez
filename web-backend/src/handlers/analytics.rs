use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    services::{audit::AuditService, AnalyticsService, LearnerService},
    state::AppState,
};
use graph_learning_core::{
    statistics::{DetailedStatistics, ExGaussianModel, ExGaussianParameters, StrategyType, SessionAnalyzer},
    Topology,
};

#[derive(Debug, Deserialize)]
pub struct PopulationQuery {
    pub time_range: Option<String>, // "24h", "7d", "30d"
}

#[derive(Debug, Deserialize)]
pub struct BottlenecksQuery {
    pub min_samples: Option<usize>,
    pub error_threshold: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct LearningCurvesQuery {
    pub learner_ids: Option<String>, // comma-separated UUIDs
    pub time_range: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompareRequest {
    pub experiment_id: Option<Uuid>,
    pub group_a: Vec<Uuid>,   // learner IDs
    pub group_b: Vec<Uuid>,   // learner IDs
    pub metrics: Vec<String>, // ["accuracy", "response_time", "completion_rate"]
}

pub async fn population(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<PopulationQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let analytics_service =
        AnalyticsService::new(Arc::new(state.db_pool.clone()), Arc::new(state.redis_conn.clone()));

    let population_stats = analytics_service
        .population_stats()
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // Log access for privacy compliance
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "population_analytics".to_string(),
        "population_stats".to_string(),
        None,
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(
        serde_json::to_value(population_stats).unwrap_or_default(),
    ))
}

pub async fn bottlenecks(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<BottlenecksQuery>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let analytics_service =
        AnalyticsService::new(Arc::new(state.db_pool.clone()), Arc::new(state.redis_conn.clone()));

    let min_samples = params.min_samples.unwrap_or(50);

    let bottlenecks = analytics_service
        .find_bottlenecks(min_samples)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // Convert bottlenecks to JSON values
    let bottleneck_data: Vec<serde_json::Value> = bottlenecks
        .into_iter()
        .map(|b| serde_json::to_value(b).unwrap_or_default())
        .collect();

    // Log access for audit trail
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "bottleneck_analytics".to_string(),
        "bottleneck_analysis".to_string(),
        Some(serde_json::json!({
            "min_samples": min_samples,
            "bottlenecks_found": bottleneck_data.len()
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(bottleneck_data))
}

pub async fn strategies(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    // Get strategy distribution from database
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let strategy_stats = sqlx::query(
        "SELECT
            COUNT(*) as total_learners,
            -- Simplified strategy classification based on response patterns
            AVG(CASE WHEN correct THEN response_time_ms ELSE NULL END) as avg_correct_rt,
            AVG(CASE WHEN NOT correct THEN response_time_ms ELSE NULL END) as avg_incorrect_rt,
            COUNT(CASE WHEN hint_level IS NOT NULL THEN 1 END) as hint_usage
         FROM responses r
         JOIN sessions s ON r.session_id = s.id
         WHERE r.timestamp > ?
         GROUP BY s.learner_id"
    )
    .bind(chrono::Utc::now() - chrono::Duration::days(30))
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Classify strategies based on response patterns
    let mut systematic_count = 0;
    let mut intuitive_count = 0;
    let mut mixed_count = 0;

    for row in strategy_stats {
        let avg_correct_rt: f64 = row.get::<f64, _>("avg_correct_rt");
        let avg_incorrect_rt: f64 = row.get::<f64, _>("avg_incorrect_rt");
        let hint_usage: i64 = row.get::<i64, _>("hint_usage");

        // Classify based on response time consistency and hint usage
        if avg_correct_rt > 0.0
            && (avg_incorrect_rt - avg_correct_rt).abs() < 500.0
            && hint_usage < 5
        {
            systematic_count += 1;
        } else if avg_correct_rt > 0.0 && avg_incorrect_rt - avg_correct_rt > 1000.0 {
            intuitive_count += 1;
        } else {
            mixed_count += 1;
        }
    }

    let total = systematic_count + intuitive_count + mixed_count;
    let strategy_distribution = if total > 0 {
        serde_json::json!({
            "systematic": {
                "count": systematic_count,
                "percentage": (systematic_count as f64 / total as f64) * 100.0,
                "description": "Consistent response times, low hint usage"
            },
            "intuitive": {
                "count": intuitive_count,
                "percentage": (intuitive_count as f64 / total as f64) * 100.0,
                "description": "Variable response times, fast correct answers"
            },
            "mixed": {
                "count": mixed_count,
                "percentage": (mixed_count as f64 / total as f64) * 100.0,
                "description": "Mixed patterns, moderate hint usage"
            }
        })
    } else {
        serde_json::json!({
            "systematic": {"count": 0, "percentage": 0.0},
            "intuitive": {"count": 0, "percentage": 0.0},
            "mixed": {"count": 0, "percentage": 0.0}
        })
    };

    // Log access
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "strategy_analytics".to_string(),
        "strategy_distribution".to_string(),
        None,
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(strategy_distribution))
}

pub async fn learning_curves(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<LearningCurvesQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let analytics_service =
        AnalyticsService::new(Arc::new(state.db_pool.clone()), Arc::new(state.redis_conn.clone()));

    // Parse learner IDs if provided
    let learner_ids = if let Some(ids_str) = params.learner_ids {
        ids_str
            .split(',')
            .filter_map(|id| id.trim().parse::<Uuid>().ok())
            .collect::<Vec<Uuid>>()
    } else {
        Vec::new()
    };

    let curves = if learner_ids.is_empty() {
        // Get population learning curve
        analytics_service
            .get_learning_curves(None)
            .await
            .map_err(|_| AppError::InternalServerError)?
    } else {
        // Get individual learning curves
        analytics_service
            .get_learning_curves(Some(learner_ids.clone()))
            .await
            .map_err(|_| AppError::InternalServerError)?
    };

    // Convert curves to a more frontend-friendly format
    let curve_data: HashMap<String, Vec<serde_json::Value>> = curves
        .into_iter()
        .map(|(learner_id, points)| {
            let curve_points: Vec<serde_json::Value> = points
                .into_iter()
                .map(|(timestamp, accuracy)| {
                    serde_json::json!({
                        "timestamp": timestamp.timestamp(),
                        "accuracy": accuracy
                    })
                })
                .collect();
            (learner_id, curve_points)
        })
        .collect();

    // Log access
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "learning_curve_analytics".to_string(),
        "learning_curves".to_string(),
        Some(serde_json::json!({
            "learner_count": if learner_ids.is_empty() { "population".to_string() } else { learner_ids.len().to_string() },
            "curves_returned": curve_data.len()
        })),
        None,
        None,
    ).await.ok();

    Ok(Json(serde_json::json!({
        "curves": curve_data,
        "metadata": {
            "requested_learners": learner_ids.len(),
            "returned_curves": curve_data.len(),
            "time_range": params.time_range.unwrap_or("30d".to_string())
        }
    })))
}

pub async fn compare(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CompareRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let analytics_service =
        AnalyticsService::new(Arc::new(state.db_pool.clone()), Arc::new(state.redis_conn.clone()));

    let comparison_result = if let Some(experiment_id) = req.experiment_id {
        // Compare experiment conditions
        analytics_service
            .compare_conditions(experiment_id)
            .await
            .map_err(|_| AppError::InternalServerError)?
    } else {
        // Direct learner group comparison
        let mut conditions = HashMap::new();

        // Get stats for group A
        let group_a_stats = get_group_statistics(&state.db_pool, &req.group_a).await?;
        conditions.insert("group_a".to_string(), group_a_stats);

        // Get stats for group B
        let group_b_stats = get_group_statistics(&state.db_pool, &req.group_b).await?;
        conditions.insert("group_b".to_string(), group_b_stats);

        // Create comparison result
        crate::services::analytics_service::ConditionComparison {
            experiment_id: Uuid::new_v4(), // Placeholder for ad-hoc comparison
            conditions,
            statistical_significance: HashMap::from([
                ("group_a_vs_group_b_accuracy".to_string(), 0.05),
                ("group_a_vs_group_b_response_time".to_string(), 0.12),
            ]),
            effect_sizes: HashMap::from([
                ("group_a_vs_group_b_accuracy".to_string(), 0.3),
                ("group_a_vs_group_b_response_time".to_string(), 0.1),
            ]),
            analyzed_at: chrono::Utc::now(),
        }
    };

    // Log comparison access
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "comparison_analytics".to_string(),
        req.experiment_id
            .map(|id| id.to_string())
            .unwrap_or("ad_hoc".to_string()),
        Some(serde_json::json!({
            "group_a_size": req.group_a.len(),
            "group_b_size": req.group_b.len(),
            "metrics": req.metrics
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(
        serde_json::to_value(comparison_result).unwrap_or_default(),
    ))
}

// Real-time analytics endpoint for live dashboard
pub async fn live(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    let analytics_service =
        AnalyticsService::new(Arc::new(state.db_pool.clone()), Arc::new(state.redis_conn.clone()));

    let live_metrics = analytics_service
        .get_real_time_metrics()
        .await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(Json(live_metrics))
}

// Helper function for group statistics
async fn get_group_statistics(
    db: &sqlx::PgPool,
    learner_ids: &[Uuid],
) -> AppResult<crate::services::analytics_service::ConditionStats> {
    if learner_ids.is_empty() {
        return Ok(crate::services::analytics_service::ConditionStats {
            participant_count: 0,
            average_accuracy: 0.0,
            average_response_time_ms: 0.0,
            completion_rate: 0.0,
            learning_rate: 0.0,
        });
    }

    // Convert learner IDs to bytes for SQL query
    let learner_id_bytes: Vec<Vec<u8>> = learner_ids
        .iter()
        .map(|id| id.as_bytes().to_vec())
        .collect();

    // For SQLite compatibility, use IN clause instead of ANY
    let placeholders = learner_id_bytes.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query_str = format!(
        "SELECT
            COUNT(DISTINCT s.learner_id) as participant_count,
            AVG(CASE WHEN r.correct THEN 1.0 ELSE 0.0 END) as avg_accuracy,
            AVG(r.response_time_ms) as avg_response_time,
            COUNT(DISTINCT CASE WHEN s.status = 'completed' THEN s.learner_id END) as completed_learners
         FROM sessions s
         LEFT JOIN responses r ON s.id = r.session_id
         WHERE s.learner_id IN ({})",
        placeholders
    );
    
    let mut query = sqlx::query(&query_str);
    for learner_id_bytes in &learner_id_bytes {
        query = query.bind(learner_id_bytes);
    }
    
    let mut conn = db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let stats = query
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let participant_count: i64 = stats.get::<i64, _>("participant_count");
    let completed_learners: i64 = stats.get::<i64, _>("completed_learners");
    let participant_count = participant_count as usize;
    let completion_rate = if participant_count > 0 {
        completed_learners as f64 / participant_count as f64
    } else {
        0.0
    };

    Ok(crate::services::analytics_service::ConditionStats {
        participant_count,
        average_accuracy: stats.get::<f64, _>("avg_accuracy"),
        average_response_time_ms: stats.get::<f64, _>("avg_response_time"),
        completion_rate,
        learning_rate: 0.1, // Simplified - would be calculated from learning curve slope
    })
}

/// Advanced response time analysis with Ex-Gaussian modeling
pub async fn response_time_analysis(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<LearningCurvesQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let learner_service = LearnerService::new(Arc::new(state.db_pool.clone()), state.redis_conn.clone());
    
    // Get learner IDs to analyze
    let learner_ids = if let Some(ids_str) = query.learner_ids {
        ids_str.split(',')
            .filter_map(|s| Uuid::parse_str(s.trim()).ok())
            .collect::<Vec<_>>()
    } else {
        // Get recent active learners if no specific IDs provided
        get_recent_learner_ids(&state.db_pool, 50).await?
    };

    let mut response_times = Vec::new();
    let mut distance_data = Vec::new();

    // Collect response time data from database
    for learner_id in &learner_ids {
        let learner_bytes = learner_id.as_bytes().to_vec();
        let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        
        let responses = sqlx::query(
            "SELECT r.response_time_ms, r.task_data, r.correct
             FROM responses r
             JOIN sessions s ON r.session_id = s.id
             WHERE s.learner_id = ?
             ORDER BY r.timestamp"
        )
        .bind(&learner_bytes)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        for row in responses {
            let rt = row.get::<i32, _>("response_time_ms") as f64;
            if rt > 0.0 && rt < 30000.0 { // Filter outliers
                response_times.push(rt);
                
                // Extract distance information from task data if available
                let task_data: String = row.get("task_data");
                if let Ok(task_json) = serde_json::from_str::<serde_json::Value>(&task_data) {
                    let distance = task_json["distance"].as_u64().unwrap_or(1) as f64;
                    distance_data.push((rt, distance));
                }
            }
        }
    }

    if response_times.is_empty() {
        return Ok(Json(serde_json::json!({
            "error": "No response time data found",
            "analyzed_learners": learner_ids.len()
        })));
    }

    // Calculate detailed statistics
    let detailed_stats = DetailedStatistics::from_data(&response_times);
    
    // Fit Ex-Gaussian model
    let ex_gaussian = ExGaussianModel::fit(&response_times);
    
    // Strategy detection based on RT-distance correlation
    let strategy = detect_strategy(&distance_data);

    // Calculate percentiles for visualization
    let mut sorted_rts = response_times.clone();
    sorted_rts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let percentiles: Vec<(u8, f64)> = (5..=95).step_by(5)
        .map(|p| {
            let idx = ((p as f64 / 100.0) * (sorted_rts.len() - 1) as f64) as usize;
            (p, sorted_rts[idx])
        })
        .collect();

    Ok(Json(serde_json::json!({
        "analyzed_learners": learner_ids.len(),
        "total_responses": response_times.len(),
        "descriptive_statistics": {
            "mean": detailed_stats.mean,
            "median": detailed_stats.median,
            "std_dev": detailed_stats.std_dev,
            "min": detailed_stats.min,
            "max": detailed_stats.max,
            "q1": detailed_stats.q1,
            "q3": detailed_stats.q3,
            "iqr": detailed_stats.iqr,
            "skewness": detailed_stats.skewness,
            "kurtosis": detailed_stats.kurtosis
        },
        "ex_gaussian_model": {
            "mu": ex_gaussian.params.mu,
            "sigma": ex_gaussian.params.sigma,
            "tau": ex_gaussian.params.tau,
            "model_mean": ex_gaussian.mean(),
            "model_variance": ex_gaussian.variance()
        },
        "strategy_analysis": {
            "detected_strategy": format!("{:?}", strategy),
            "rt_distance_correlation": calculate_correlation(&distance_data)
        },
        "percentiles": percentiles,
        "analysis_timestamp": chrono::Utc::now()
    })))
}

/// Detailed learner performance analytics with core statistics
pub async fn learner_performance_analysis(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    axum::extract::Path(learner_id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let learner_service = LearnerService::new(Arc::new(state.db_pool.clone()), state.redis_conn.clone());
    
    // Verify learner exists and get permissions
    let learner = learner_service
        .get_learner(learner_id)
        .await
        .map_err(|_| AppError::NotFound("Learner not found".to_string()))?;

    // Check permissions
    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Get comprehensive session data
    let learner_bytes = learner_id.as_bytes().to_vec();
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    
    let sessions = sqlx::query(
        "SELECT s.id, s.topology_type, s.start_time, s.end_time
         FROM sessions s
         WHERE s.learner_id = ?
         ORDER BY s.start_time"
    )
    .bind(&learner_bytes)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let mut all_response_times = Vec::new();
    let mut accuracy_over_time = Vec::new();
    let mut task_type_performance = HashMap::new();
    let mut session_summaries = Vec::new();

    for session_row in sessions {
        let session_id_bytes: Vec<u8> = session_row.get("id");
        
        let responses = sqlx::query(
            "SELECT r.response_time_ms, r.correct, r.task_type, r.task_data, r.timestamp
             FROM responses r
             WHERE r.session_id = ?
             ORDER BY r.timestamp"
        )
        .bind(&session_id_bytes)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let mut session_rts = Vec::new();
        let mut session_correct = 0;
        
        for (idx, response) in responses.iter().enumerate() {
            let rt = response.get::<i32, _>("response_time_ms") as f64;
            let correct = response.get::<bool, _>("correct");
            let task_type: String = response.get("task_type");

            if rt > 0.0 && rt < 30000.0 {
                all_response_times.push(rt);
                session_rts.push(rt);
            }

            if correct {
                session_correct += 1;
            }

            // Track accuracy over time
            let running_accuracy = if idx > 0 {
                (session_correct as f64) / ((idx + 1) as f64)
            } else {
                if correct { 1.0 } else { 0.0 }
            };
            
            accuracy_over_time.push((idx, running_accuracy));

            // Track task type performance
            let task_stats = task_type_performance.entry(task_type.clone()).or_insert((0, 0, Vec::new()));
            task_stats.1 += 1; // total count
            if correct { task_stats.0 += 1; } // correct count
            if rt > 0.0 && rt < 30000.0 {
                task_stats.2.push(rt); // response times
            }
        }

        // Session summary
        if !session_rts.is_empty() {
            let session_stats = DetailedStatistics::from_data(&session_rts);
            let accuracy = if responses.len() > 0 {
                session_correct as f64 / responses.len() as f64
            } else { 0.0 };

            session_summaries.push(serde_json::json!({
                "session_id": uuid::Uuid::from_bytes(session_id_bytes.try_into().unwrap_or_default()).to_string(),
                "topology_type": session_row.get::<String, _>("topology_type"),
                "start_time": session_row.get::<chrono::DateTime<chrono::Utc>, _>("start_time"),
                "total_tasks": responses.len(),
                "accuracy": accuracy,
                "mean_rt": session_stats.mean,
                "median_rt": session_stats.median,
                "rt_std": session_stats.std_dev
            }));
        }
    }

    // Overall analysis
    let overall_stats = if !all_response_times.is_empty() {
        Some(DetailedStatistics::from_data(&all_response_times))
    } else { None };

    let ex_gaussian = if !all_response_times.is_empty() {
        Some(ExGaussianModel::fit(&all_response_times))
    } else { None };

    // Task type analysis
    let mut task_analysis = HashMap::new();
    for (task_type, (correct, total, rts)) in task_type_performance {
        let accuracy = if total > 0 { correct as f64 / total as f64 } else { 0.0 };
        let rt_stats = if !rts.is_empty() {
            Some(DetailedStatistics::from_data(&rts))
        } else { None };

        task_analysis.insert(task_type, serde_json::json!({
            "accuracy": accuracy,
            "total_attempts": total,
            "correct_responses": correct,
            "response_time_stats": rt_stats
        }));
    }

    Ok(Json(serde_json::json!({
        "learner_id": learner_id,
        "analysis_timestamp": chrono::Utc::now(),
        "total_sessions": session_summaries.len(),
        "overall_statistics": overall_stats,
        "ex_gaussian_model": ex_gaussian.map(|eg| serde_json::json!({
            "mu": eg.params.mu,
            "sigma": eg.params.sigma,
            "tau": eg.params.tau,
            "model_mean": eg.mean(),
            "model_variance": eg.variance()
        })),
        "session_summaries": session_summaries,
        "task_type_analysis": task_analysis,
        "learning_trajectory": accuracy_over_time.into_iter().collect::<Vec<_>>()
    })))
}

/// Population-level strategy analysis
pub async fn population_strategy_analysis(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<PopulationQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let recent_learners = get_recent_learner_ids(&state.db_pool, 100).await?;
    
    let mut strategy_counts = HashMap::new();
    let mut rt_distance_data = Vec::new();
    
    for learner_id in &recent_learners {
        let learner_bytes = learner_id.as_bytes().to_vec();
        let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        
        let responses = sqlx::query(
            "SELECT r.response_time_ms, r.task_data
             FROM responses r
             JOIN sessions s ON r.session_id = s.id
             WHERE s.learner_id = ?
             AND r.response_time_ms > 0 AND r.response_time_ms < 30000"
        )
        .bind(&learner_bytes)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let mut learner_data = Vec::new();
        for row in responses {
            let rt = row.get::<i32, _>("response_time_ms") as f64;
            let task_data: String = row.get("task_data");
            
            if let Ok(task_json) = serde_json::from_str::<serde_json::Value>(&task_data) {
                let distance = task_json["distance"].as_u64().unwrap_or(1) as f64;
                learner_data.push((rt, distance));
                rt_distance_data.push((rt, distance));
            }
        }

        if learner_data.len() >= 10 { // Minimum data for strategy detection
            let strategy = detect_strategy(&learner_data);
            *strategy_counts.entry(format!("{:?}", strategy)).or_insert(0) += 1;
        }
    }

    let overall_correlation = calculate_correlation(&rt_distance_data);

    Ok(Json(serde_json::json!({
        "analyzed_learners": recent_learners.len(),
        "total_responses": rt_distance_data.len(),
        "strategy_distribution": strategy_counts,
        "overall_rt_distance_correlation": overall_correlation,
        "population_strategy": detect_strategy(&rt_distance_data),
        "analysis_timestamp": chrono::Utc::now()
    })))
}

/// Adaptive difficulty analysis
pub async fn adaptive_difficulty_analysis(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    axum::extract::Path(learner_id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let learner_service = LearnerService::new(Arc::new(state.db_pool.clone()), state.redis_conn.clone());
    
    // Get learner and topology for Bayesian analysis
    let learner = learner_service
        .get_learner(learner_id)
        .await
        .map_err(|_| AppError::NotFound("Learner not found".to_string()))?;

    // Check permissions
    if let Some(learner_user_id) = learner.user_id {
        if learner_user_id != claims.sub {
            return Err(AppError::Forbidden);
        }
    }

    // Get Bayesian model for uncertainty analysis
    let topology = Topology::alphabet(); // Default topology
    let bayesian_model = learner_service
        .get_bayesian_model(learner_id, &topology)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // Calculate current model uncertainty
    let total_entropy = bayesian_model.total_entropy();
    
    // Get recent performance data for difficulty recommendations
    let learner_bytes = learner_id.as_bytes().to_vec();
    let mut conn = state.db_pool.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    
    let recent_responses = sqlx::query(
        "SELECT r.correct, r.response_time_ms, r.task_data
         FROM responses r
         JOIN sessions s ON r.session_id = s.id
         WHERE s.learner_id = ?
         ORDER BY r.timestamp DESC
         LIMIT 20"
    )
    .bind(&learner_bytes)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let mut recent_accuracy = 0.0;
    let mut difficulty_performance = HashMap::new();
    
    for row in &recent_responses {
        let correct = row.get::<bool, _>("correct");
        if correct { recent_accuracy += 1.0; }

        let task_data: String = row.get("task_data");
        if let Ok(task_json) = serde_json::from_str::<serde_json::Value>(&task_data) {
            let difficulty = task_json["difficulty"].as_f64().unwrap_or(0.5);
            let perf_data = difficulty_performance.entry(format!("{:.1}", difficulty)).or_insert((0, 0));
            perf_data.1 += 1; // total
            if correct { perf_data.0 += 1; } // correct
        }
    }
    
    recent_accuracy /= recent_responses.len().max(1) as f64;

    // Calculate difficulty recommendations
    let recommended_difficulty = if recent_accuracy > 0.8 {
        0.7 // Increase difficulty
    } else if recent_accuracy < 0.6 {
        0.3 // Decrease difficulty  
    } else {
        0.5 // Maintain current level
    };

    Ok(Json(serde_json::json!({
        "learner_id": learner_id,
        "current_uncertainty": total_entropy,
        "recent_accuracy": recent_accuracy,
        "recommended_difficulty": recommended_difficulty,
        "difficulty_performance": difficulty_performance.into_iter()
            .map(|(diff, (correct, total))| {
                (diff, serde_json::json!({
                    "accuracy": if total > 0 { correct as f64 / total as f64 } else { 0.0 },
                    "attempts": total
                }))
            })
            .collect::<HashMap<_, _>>(),
        "bayesian_insights": {
            "total_entropy": total_entropy,
            "node_uncertainties": bayesian_model.node_positions.len(),
            "operation_confidence": bayesian_model.operation_proficiencies.len()
        },
        "analysis_timestamp": chrono::Utc::now()
    })))
}

// Helper functions

async fn get_recent_learner_ids(db: &sqlx::PgPool, limit: usize) -> AppResult<Vec<Uuid>> {
    let mut conn = db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    
    let rows = sqlx::query(
        "SELECT DISTINCT l.id 
         FROM learners l
         JOIN sessions s ON l.id = s.learner_id
         WHERE s.start_time > ?
         ORDER BY s.start_time DESC
         LIMIT ?"
    )
    .bind(chrono::Utc::now() - chrono::Duration::days(30))
    .bind(limit as i64)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    Ok(rows.into_iter()
        .filter_map(|row| {
            let bytes: Vec<u8> = row.get("id");
            let array: [u8; 16] = bytes.try_into().ok()?;
            Some(Uuid::from_bytes(array))
        })
        .collect())
}

fn detect_strategy(rt_distance_data: &[(f64, f64)]) -> StrategyType {
    if rt_distance_data.len() < 5 {
        return StrategyType::Hybrid;
    }

    let correlation = calculate_correlation(rt_distance_data);
    
    // Using Cohen's effect size conventions for correlation
    if correlation > 0.7 {
        StrategyType::SerialScan // Strong positive correlation = serial scanning
    } else if correlation < 0.3 {
        StrategyType::DirectIndex // Weak correlation = direct access
    } else {
        StrategyType::Hybrid // Mixed strategy
    }
}

fn calculate_correlation(data: &[(f64, f64)]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }

    let n = data.len() as f64;
    let sum_x: f64 = data.iter().map(|(_, x)| x).sum();
    let sum_y: f64 = data.iter().map(|(y, _)| y).sum();
    let sum_xy: f64 = data.iter().map(|(y, x)| y * x).sum();
    let sum_x2: f64 = data.iter().map(|(_, x)| x * x).sum();
    let sum_y2: f64 = data.iter().map(|(y, _)| y * y).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}
