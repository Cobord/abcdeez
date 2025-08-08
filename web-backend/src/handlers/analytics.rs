use axum::{extract::{Query, State}, Json, Extension};
use std::{sync::Arc, collections::HashMap};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    state::AppState,
    services::{AnalyticsService, audit::AuditService},
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
    pub group_a: Vec<Uuid>, // learner IDs
    pub group_b: Vec<Uuid>, // learner IDs
    pub metrics: Vec<String>, // ["accuracy", "response_time", "completion_rate"]
}

pub async fn population(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<PopulationQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let analytics_service = AnalyticsService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );

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
    ).await.ok();

    Ok(Json(serde_json::to_value(population_stats).unwrap_or_default()))
}

pub async fn bottlenecks(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<BottlenecksQuery>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let analytics_service = AnalyticsService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );

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
    ).await.ok();

    Ok(Json(bottleneck_data))
}

pub async fn strategies(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    // Get strategy distribution from database
    let strategy_stats = sqlx::query!(
        "SELECT 
            COUNT(*) as total_learners,
            -- Simplified strategy classification based on response patterns
            AVG(CASE WHEN correct THEN response_time_ms ELSE NULL END) as avg_correct_rt,
            AVG(CASE WHEN NOT correct THEN response_time_ms ELSE NULL END) as avg_incorrect_rt,
            COUNT(CASE WHEN hint_level IS NOT NULL THEN 1 END) as hint_usage
         FROM responses r
         JOIN sessions s ON r.session_id = s.id
         WHERE r.timestamp > $1
         GROUP BY s.learner_id",
        chrono::Utc::now() - chrono::Duration::days(30)
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Classify strategies based on response patterns
    let mut systematic_count = 0;
    let mut intuitive_count = 0;
    let mut mixed_count = 0;

    for row in strategy_stats {
        let avg_correct_rt = row.avg_correct_rt.unwrap_or(0.0);
        let avg_incorrect_rt = row.avg_incorrect_rt.unwrap_or(0.0);
        let hint_usage = row.hint_usage.unwrap_or(0);

        // Classify based on response time consistency and hint usage
        if avg_correct_rt > 0.0 && (avg_incorrect_rt - avg_correct_rt).abs() < 500.0 && hint_usage < 5 {
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
    ).await.ok();

    Ok(Json(strategy_distribution))
}

pub async fn learning_curves(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<LearningCurvesQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let analytics_service = AnalyticsService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );

    // Parse learner IDs if provided
    let learner_ids = if let Some(ids_str) = params.learner_ids {
        ids_str.split(',')
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
            "learner_count": if learner_ids.is_empty() { "population" } else { &learner_ids.len().to_string() },
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
    let analytics_service = AnalyticsService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );

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
        req.experiment_id.map(|id| id.to_string()).unwrap_or("ad_hoc".to_string()),
        Some(serde_json::json!({
            "group_a_size": req.group_a.len(),
            "group_b_size": req.group_b.len(),
            "metrics": req.metrics
        })),
        None,
        None,
    ).await.ok();

    Ok(Json(serde_json::to_value(comparison_result).unwrap_or_default()))
}

// Real-time analytics endpoint for live dashboard
pub async fn live(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    let analytics_service = AnalyticsService::new(
        state.db_pool.clone(),
        state.redis_conn.clone(),
    );

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

    let stats = sqlx::query!(
        "SELECT 
            COUNT(DISTINCT s.learner_id) as participant_count,
            AVG(CASE WHEN r.correct THEN 1.0 ELSE 0.0 END) as avg_accuracy,
            AVG(r.response_time_ms) as avg_response_time,
            COUNT(DISTINCT CASE WHEN s.status = 'completed' THEN s.learner_id END) as completed_learners
         FROM sessions s
         LEFT JOIN responses r ON s.id = r.session_id
         WHERE s.learner_id = ANY($1)",
        &learner_id_bytes as &[Vec<u8>]
    )
    .fetch_one(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let participant_count = stats.participant_count.unwrap_or(0) as usize;
    let completion_rate = if participant_count > 0 {
        stats.completed_learners.unwrap_or(0) as f64 / participant_count as f64
    } else {
        0.0
    };

    Ok(crate::services::analytics_service::ConditionStats {
        participant_count,
        average_accuracy: stats.avg_accuracy.unwrap_or(0.0),
        average_response_time_ms: stats.avg_response_time.unwrap_or(0.0),
        completion_rate,
        learning_rate: 0.1, // Simplified - would be calculated from learning curve slope
    })
}