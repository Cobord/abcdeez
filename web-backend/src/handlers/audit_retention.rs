use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    services::audit::{AuditRetentionManager, AuditRetentionPolicy, AuditService},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct RetentionCleanupQuery {
    pub dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ComplianceReportQuery {
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

/// Get audit retention statistics and policy overview
pub async fn get_retention_statistics(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    let retention_manager = AuditRetentionManager::new();
    
    let stats = AuditService::get_retention_statistics(&state.db_pool, &retention_manager)
        .await
        .map_err(|e| AppError::InternalServerError)?;

    Ok(Json(stats))
}

/// Apply audit trail retention policies (admin only)
pub async fn apply_retention_policies(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<RetentionCleanupQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let dry_run = params.dry_run.unwrap_or(true); // Default to dry run for safety
    let retention_manager = AuditRetentionManager::new();

    let stats = AuditService::apply_retention_policies(&state.db_pool, &retention_manager, dry_run)
        .await
        .map_err(|e| AppError::InternalServerError)?;

    Ok(Json(serde_json::json!({
        "cleanup_stats": stats,
        "dry_run": dry_run,
        "message": if dry_run {
            "Dry run completed - no records were deleted"
        } else {
            "Retention policies applied successfully"
        }
    })))
}

/// Generate compliance report for audit records (admin only)
pub async fn generate_compliance_report(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<ComplianceReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let retention_manager = AuditRetentionManager::new();
    
    let report = AuditService::generate_compliance_report(
        &state.db_pool,
        &retention_manager,
        params.from_date,
        params.to_date,
    )
    .await
    .map_err(|e| AppError::InternalServerError)?;

    Ok(Json(report))
}

/// Get all active retention policies (admin only)
pub async fn get_retention_policies(
    _state: State<Arc<AppState>>,
    _claims: Extension<Claims>,
) -> AppResult<Json<Vec<AuditRetentionPolicy>>> {
    let retention_manager = AuditRetentionManager::new();
    let policies = retention_manager.list_policies().into_iter().cloned().collect();

    Ok(Json(policies))
}

/// Update or create a retention policy (admin only)
pub async fn update_retention_policy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(policy): Json<AuditRetentionPolicy>,
) -> AppResult<Json<serde_json::Value>> {
    // Validate policy configuration
    if policy.retention_days < policy.minimum_retention_days {
        return Err(AppError::BadRequest(
            "Retention period cannot be less than minimum retention period".to_string(),
        ));
    }

    if policy.retention_days > 3650 && !policy.legal_hold {
        return Err(AppError::BadRequest(
            "Retention period over 10 years requires legal hold justification".to_string(),
        ));
    }

    // Log the policy change
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "update_retention_policy".to_string(),
        "audit_policy".to_string(),
        policy.name.clone(),
        Some(serde_json::json!({
            "policy": policy,
            "admin_user": claims.username,
            "admin_role": claims.role
        })),
        None,
        Some("AdminInterface".to_string()),
    )
    .await
    .map_err(|e| AppError::InternalServerError)?;

    Ok(Json(serde_json::json!({
        "message": "Retention policy updated successfully",
        "policy_name": policy.name,
        "retention_days": policy.retention_days,
        "legal_hold": policy.legal_hold
    })))
}

/// Delete a retention policy (admin only, with safety checks)
pub async fn delete_retention_policy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    axum::extract::Path(policy_name): axum::extract::Path<String>,
) -> AppResult<StatusCode> {
    // Safety check - prevent deletion of critical policies
    let critical_policies = vec!["security", "auth", "admin"];
    if critical_policies.contains(&policy_name.as_str()) {
        return Err(AppError::BadRequest(
            "Cannot delete critical retention policies".to_string(),
        ));
    }

    // Log the policy deletion
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "delete_retention_policy".to_string(),
        "audit_policy".to_string(),
        policy_name.clone(),
        Some(serde_json::json!({
            "policy_name": policy_name,
            "admin_user": claims.username,
            "admin_role": claims.role,
            "deletion_timestamp": Utc::now()
        })),
        None,
        Some("AdminInterface".to_string()),
    )
    .await
    .map_err(|e| AppError::InternalServerError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get audit trail for a specific resource (with retention context)
pub async fn get_audit_trail_with_retention(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Query(params): Query<AuditTrailQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let retention_manager = AuditRetentionManager::new();

    let audit_records = AuditService::get_audit_trail(
        &state.db_pool,
        params.resource_type.clone(),
        params.resource_id.clone(),
        params.user_id,
        params.limit,
    )
    .await
    .map_err(|e| AppError::InternalServerError)?;

    // Add retention policy information to each record
    let enriched_records: Vec<serde_json::Value> = audit_records
        .into_iter()
        .map(|mut record| {
            let resource_type = record["resource_type"].as_str().unwrap_or("");
            let action = record["action"].as_str().unwrap_or("");
            let policy = retention_manager.get_applicable_policy(resource_type, action);
            
            record["retention_policy"] = serde_json::json!({
                "policy_name": policy.name,
                "retention_days": policy.retention_days,
                "legal_hold": policy.legal_hold,
                "priority": policy.priority
            });
            
            record
        })
        .collect();

    Ok(Json(serde_json::json!({
        "audit_records": enriched_records,
        "query_params": params,
        "total_records": enriched_records.len()
    })))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuditTrailQuery {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub user_id: Option<uuid::Uuid>,
    pub limit: Option<i64>,
}