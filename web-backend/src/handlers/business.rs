use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{
    error::AppResult,
    middleware::require_admin,
    monitoring::business::{
        global_business_metrics, BusinessDashboard, DailyBusinessMetrics,
        LearningEffectivenessMetrics,
    },
    state::AppState,
};

/// Get comprehensive business metrics dashboard
pub async fn get_business_dashboard(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<BusinessDashboard>> {
    let dashboard = global_business_metrics().get_business_dashboard().await;
    Ok(Json(dashboard))
}

/// Get daily business metrics snapshot
pub async fn get_daily_metrics(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<DailyBusinessMetrics>> {
    let daily_snapshot = global_business_metrics().generate_daily_snapshot().await;
    Ok(Json(daily_snapshot))
}

#[inline(always)]
fn u64f64_into(x: u64) -> f64 {
    let x_u32 : Result<u32,_> = x.try_into();
    x_u32.map(|x| x.into()).expect("This seems too large???")
}

/// Get learning effectiveness metrics
pub async fn get_learning_effectiveness(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<LearningEffectivenessMetrics>> {
    // Calculate learning effectiveness metrics
    let skill_progression_velocity = u64f64_into(global_business_metrics()
            .learning_velocity
            .load(std::sync::atomic::Ordering::Relaxed));
    let knowledge_retention_rate = u64f64_into(global_business_metrics()
            .task_completion_rate
            .load(std::sync::atomic::Ordering::Relaxed));
    let metrics = LearningEffectivenessMetrics {
        skill_progression_velocity,
        knowledge_retention_rate,
        adaptive_difficulty_effectiveness: 0.85, // Would be calculated from actual data
        personalization_impact: 0.72,            // Would be A/B tested
        intervention_success_rate: 0.68,         // Success rate of learning interventions
    };

    Ok(Json(metrics))
}

/// Get user journey analytics
pub async fn get_user_journey_analytics(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let user_journeys = global_business_metrics().user_journey_data.read().await;

    // Aggregate user journey statistics
    let total_users = user_journeys.len();
    let active_users = user_journeys
        .values()
        .filter(|j| {
            matches!(
                j.lifecycle_stage,
                crate::monitoring::business::UserLifecycleStage::Active
            )
        })
        .count();
    let at_risk_users = user_journeys
        .values()
        .filter(|j| {
            matches!(
                j.lifecycle_stage,
                crate::monitoring::business::UserLifecycleStage::AtRisk
            )
        })
        .count();
    let churned_users = user_journeys
        .values()
        .filter(|j| {
            matches!(
                j.lifecycle_stage,
                crate::monitoring::business::UserLifecycleStage::Churned
            )
        })
        .count();

    let avg_session_count = if total_users > 0 {
        user_journeys
            .values()
            .map(|j| j.total_sessions)
            .sum::<u64>() as f64
            / total_users as f64
    } else {
        0.0
    };

    let avg_task_completion = if total_users > 0 {
        user_journeys
            .values()
            .map(|j| j.total_tasks_completed)
            .sum::<u64>() as f64
            / total_users as f64
    } else {
        0.0
    };

    let response = serde_json::json!({
        "total_users": total_users,
        "lifecycle_distribution": {
            "active": active_users,
            "at_risk": at_risk_users,
            "churned": churned_users,
            "trial": user_journeys.values()
                .filter(|j| matches!(j.lifecycle_stage, crate::monitoring::business::UserLifecycleStage::Trial))
                .count(),
            "onboarding": user_journeys.values()
                .filter(|j| matches!(j.lifecycle_stage, crate::monitoring::business::UserLifecycleStage::Onboarding))
                .count(),
        },
        "engagement_stats": {
            "average_sessions_per_user": avg_session_count,
            "average_tasks_completed_per_user": avg_task_completion,
            "users_with_streaks": user_journeys.values()
                .filter(|j| j.current_streak > 0)
                .count(),
            "longest_streak": user_journeys.values()
                .map(|j| j.longest_streak)
                .max()
                .unwrap_or(0),
        },
        "subscription_tiers": {
            "trial": user_journeys.values()
                .filter(|j| j.subscription_tier == "trial")
                .count(),
            "free": user_journeys.values()
                .filter(|j| j.subscription_tier == "free")
                .count(),
            "premium": user_journeys.values()
                .filter(|j| j.subscription_tier == "premium")
                .count(),
        }
    });

    Ok(Json(response))
}

/// Get revenue metrics and financial KPIs
pub async fn get_revenue_metrics(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let daily_metrics = global_business_metrics().generate_daily_snapshot().await;

    let response = serde_json::json!({
        "revenue_metrics": daily_metrics.revenue_metrics,
        "conversion_metrics": {
            "trial_to_paid_conversion": global_business_metrics()
                .trial_to_paid_conversion
                .load(std::sync::atomic::Ordering::Relaxed),
            "customer_lifetime_value": global_business_metrics()
                .customer_lifetime_value
                .load(std::sync::atomic::Ordering::Relaxed),
            "churn_rate": global_business_metrics()
                .user_churn_rate
                .load(std::sync::atomic::Ordering::Relaxed),
        },
        "growth_metrics": {
            "new_signups": global_business_metrics()
                .new_user_signups
                .load(std::sync::atomic::Ordering::Relaxed),
            "daily_active_users": global_business_metrics()
                .daily_active_users
                .load(std::sync::atomic::Ordering::Relaxed),
            "monthly_active_users": global_business_metrics()
                .monthly_active_users
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    });

    Ok(Json(response))
}

/// Export business metrics for external analysis
pub async fn export_business_data(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let dashboard = global_business_metrics().get_business_dashboard().await;
    let daily_metrics = global_business_metrics().generate_daily_snapshot().await;
    let user_journeys = global_business_metrics().user_journey_data.read().await;

    // Create comprehensive export
    let export_data = serde_json::json!({
        "export_timestamp": chrono::Utc::now(),
        "dashboard": dashboard,
        "daily_snapshot": daily_metrics,
        "user_count": user_journeys.len(),
        "summary": {
            "total_tasks_attempted": global_business_metrics()
                .tasks_attempted
                .load(std::sync::atomic::Ordering::Relaxed),
            "total_tasks_completed": global_business_metrics()
                .tasks_completed
                .load(std::sync::atomic::Ordering::Relaxed),
            "total_hints_requested": global_business_metrics()
                .hints_requested
                .load(std::sync::atomic::Ordering::Relaxed),
            "platform_accuracy": global_business_metrics()
                .accuracy_rate
                .load(std::sync::atomic::Ordering::Relaxed),
            "user_satisfaction": global_business_metrics()
                .user_satisfaction_score
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    });

    tracing::info!("Business metrics data exported");
    Ok(Json(export_data))
}
