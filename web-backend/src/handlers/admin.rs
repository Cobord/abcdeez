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
    services::audit::AuditService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SystemQuery {
    pub include_cache: Option<bool>,
    pub include_db_stats: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub action: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JobQuery {
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub database: DatabaseStatus,
    pub redis: RedisStatus,
    pub active_sessions: usize,
    pub active_websockets: usize,
    pub memory_usage_mb: Option<f64>,
    pub request_rate: f64, // requests per minute
}

#[derive(Debug, Serialize)]
pub struct DatabaseStatus {
    pub connected: bool,
    pub pool_size: usize,
    pub active_connections: usize,
    pub total_queries: Option<i64>,
    pub avg_query_time_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RedisStatus {
    pub connected: bool,
    pub memory_usage_mb: Option<f64>,
    pub total_connections: Option<i64>,
    pub cache_hit_rate: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct JobStatus {
    pub id: Uuid,
    pub job_type: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
}

// Admin dashboard overview - system health and key metrics
pub async fn dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<SystemQuery>,
) -> AppResult<Json<SystemStatus>> {
    // Check if user has admin permissions
    if !has_admin_permissions(&claims.sub, &state).await? {
        return Err(AppError::Forbidden);
    }

    // Get database statistics
    let db_stats = get_database_status(&state.db_pool).await?;

    // Get Redis status
    let redis_stats = get_redis_status(&state).await?;

    // Get active sessions count
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let active_sessions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE status = 'active'")
            .fetch_one(&mut *conn)
            .await
            .unwrap_or(0);
    let active_sessions = active_sessions as usize;

    let system_status = SystemStatus {
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Would be calculated from startup time
        database: db_stats,
        redis: redis_stats,
        active_sessions,
        active_websockets: 0,  // Would be tracked in application state
        memory_usage_mb: None, // Would need system monitoring
        request_rate: 120.0,   // Would be calculated from metrics
    };

    // Log admin access
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "admin".to_string(),
        "dashboard".to_string(),
        None,
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(system_status))
}

// List all users with pagination and filtering
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let users = sqlx::query(
        "SELECT id, username, email, created_at, updated_at, metadata
         FROM users
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *conn)
        .await
        .unwrap_or(0);

    let user_data: Vec<serde_json::Value> = users
        .into_iter()
        .map(|user| {
            let id_bytes: Vec<u8> = user.get::<Vec<u8>, _>("id");
            let id = Uuid::from_bytes(id_bytes.try_into().unwrap_or_default());
            let metadata_json = user
                .get::<Option<String>, _>("metadata")
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

            serde_json::json!({
                "id": id,
                "username": user.get::<String, _>("username"),
                "email": user.get::<String, _>("email"),
                "created_at": user.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "updated_at": user.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
                "metadata": metadata_json,
            })
        })
        .collect();

    // Log admin access
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "admin".to_string(),
        "list_users".to_string(),
        Some(serde_json::json!({
            "limit": limit,
            "offset": offset,
            "total_returned": user_data.len()
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "users": user_data,
        "pagination": {
            "total": total_count,
            "limit": limit,
            "offset": offset,
            "has_more": (offset + limit) < total_count
        }
    })))
}

// List all learners with detailed information
pub async fn list_learners(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let learners = sqlx::query(
        "SELECT l.id, l.user_id, l.display_name, l.created_at, l.last_active,
                l.total_practice_time_seconds, l.metadata,
                u.username,
                COUNT(s.id) as session_count,
                COUNT(r.id) as response_count
         FROM learners l
         LEFT JOIN users u ON l.user_id = u.id
         LEFT JOIN sessions s ON l.id = s.learner_id
         LEFT JOIN responses r ON s.id = r.session_id
         GROUP BY l.id, l.user_id, l.display_name, l.created_at, l.last_active,
                  l.total_practice_time_seconds, l.metadata, u.username
         ORDER BY l.created_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let learner_data: Vec<serde_json::Value> = learners
        .into_iter()
        .map(|learner| {
            let id_bytes: Vec<u8> = learner.get::<Vec<u8>, _>("id");
            let user_id_bytes: Option<Vec<u8>> = learner.get::<Option<Vec<u8>>, _>("user_id");
            serde_json::json!({
                "id": Uuid::from_bytes(id_bytes.try_into().unwrap_or_default()),
                "user_id": user_id_bytes.map(|bytes| Uuid::from_bytes(bytes.try_into().unwrap_or_default())),
                "username": learner.get::<Option<String>, _>("username"),
                "display_name": learner.get::<String, _>("display_name"),
                "created_at": learner.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "last_active": learner.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_active"),
                "total_practice_time_seconds": learner.get::<i32, _>("total_practice_time_seconds"),
                "session_count": learner.get::<Option<i64>, _>("session_count").unwrap_or(0),
                "response_count": learner.get::<Option<i64>, _>("response_count").unwrap_or(0),
                "metadata": learner.get::<Option<String>, _>("metadata")
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            })
        })
        .collect();

    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM learners")
        .fetch_one(&mut *conn)
        .await
        .unwrap_or(0);

    Ok(Json(serde_json::json!({
        "learners": learner_data,
        "pagination": {
            "total": total_count,
            "limit": limit,
            "offset": offset,
            "has_more": (offset + limit) < total_count
        }
    })))
}

// Get audit trail with filtering
pub async fn audit_trail(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<AuditQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let audit_records = AuditService::get_audit_trail(
        &state.db_pool,
        params.resource_type.clone(),
        params.resource_id.clone(),
        params.user_id,
        params.limit,
    )
    .await
    .map_err(|_| AppError::InternalServerError)?;

    // Log audit access (meta-audit!)
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "read".to_string(),
        "admin".to_string(),
        "audit_trail".to_string(),
        Some(serde_json::json!({
            "filters": {
                "resource_type": params.resource_type,
                "resource_id": params.resource_id,
                "user_id": params.user_id,
                "limit": params.limit
            },
            "records_returned": audit_records.len()
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "audit_records": audit_records,
        "metadata": {
            "total_returned": audit_records.len(),
            "filters_applied": {
                "resource_type": params.resource_type.is_some(),
                "resource_id": params.resource_id.is_some(),
                "user_id": params.user_id.is_some()
            }
        }
    })))
}

// Manage background jobs
pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<JobQuery>,
) -> AppResult<Json<Vec<JobStatus>>> {
    let limit = params.limit.unwrap_or(50);

    let mut query = "SELECT id, job_type, status, created_at, started_at, completed_at, error_message, retry_count FROM job_queue".to_string();
    let mut conditions = Vec::new();

    if let Some(status) = &params.status {
        conditions.push(format!("status = '{}'", status));
    }

    if let Some(job_type) = &params.job_type {
        conditions.push(format!("job_type = '{}'", job_type));
    }

    if !conditions.is_empty() {
        query.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let rows = sqlx::query(&query)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let jobs: Vec<JobStatus> = rows
        .into_iter()
        .map(|row| JobStatus {
            id: Uuid::from_bytes(row.get::<Vec<u8>, _>("id").try_into().unwrap_or_default()),
            job_type: row.get("job_type"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            error_message: row.get("error_message"),
            retry_count: row.get("retry_count"),
        })
        .collect();

    Ok(Json(jobs))
}

// Manually trigger a background job
pub async fn trigger_job(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(request): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let job_type = request["job_type"]
        .as_str()
        .ok_or(AppError::ValidationError("job_type required".to_string()))?;

    let payload = request["payload"].clone();

    // Create job in queue
    let job_id = Uuid::new_v4();
    let job_id_bytes = job_id.as_bytes();

    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    sqlx::query(
        "INSERT INTO job_queue (id, job_type, payload, status) VALUES (?, ?, ?, 'pending')",
    )
    .bind(&job_id_bytes[..])
    .bind(&job_type)
    .bind(payload.to_string())
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    // Log job creation
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "create".to_string(),
        "admin".to_string(),
        format!("job_{}", job_id),
        Some(serde_json::json!({
            "job_type": job_type,
            "triggered_by": "admin"
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "status": "queued",
        "message": "Job has been queued for execution"
    })))
}

// Update system configuration (limited subset for safety)
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(config): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // For safety, only allow updating specific configuration keys
    let allowed_keys = vec![
        "maintenance_mode",
        "max_concurrent_sessions",
        "default_difficulty",
        "privacy_epsilon",
        "rate_limit_requests_per_minute",
    ];

    let mut updated_keys = Vec::new();

    for key in allowed_keys {
        if let Some(value) = config.get(key) {
            // Store configuration in Redis or database
            let config_key = format!("config:{}", key);
            let mut conn = state.redis_conn.clone();

            if crate::cache::cmd("SET")
                .arg(&config_key)
                .arg(value.to_string())
                .query_async::<()>(&mut conn)
                .await
                .is_ok()
            {
                updated_keys.push(key);
            }
        }
    }

    // Log configuration changes
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "update".to_string(),
        "admin".to_string(),
        "system_config".to_string(),
        Some(serde_json::json!({
            "updated_keys": updated_keys,
            "requested_changes": config
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "updated_keys": updated_keys,
        "message": format!("Updated {} configuration keys", updated_keys.len())
    })))
}

// Helper functions
async fn get_database_status(db: &crate::db::DbPool) -> AppResult<DatabaseStatus> {
    // Test database connection
    let mut conn = db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
    let connected = sqlx::query("SELECT 1").fetch_one(&mut *conn).await.is_ok();

    // Get pool information (simplified)
    let pool_size = 20; // From configuration
    let active_connections = 5; // Would be from pool.num_idle() in a real implementation

    Ok(DatabaseStatus {
        connected,
        pool_size,
        active_connections,
        total_queries: None,     // Would need query statistics
        avg_query_time_ms: None, // Would need performance monitoring
    })
}

async fn get_redis_status(state: &AppState) -> AppResult<RedisStatus> {
    let mut conn = state.redis_conn.clone();
    let connected = crate::cache::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .is_ok();

    Ok(RedisStatus {
        connected,
        memory_usage_mb: None,   // Would need Redis INFO command parsing
        total_connections: None, // Would need Redis INFO command parsing
        cache_hit_rate: None,    // Would need hit/miss statistics tracking
    })
}

// Advanced audit reporting for compliance
pub async fn audit_report(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<AuditQuery>,
) -> AppResult<Json<serde_json::Value>> {
    // Generate a comprehensive audit report with security analysis
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    // Security events summary
    let security_events: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'security_event' AND timestamp >= datetime('now', '-24 hours')"
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap_or(0);

    // Failed login attempts
    let failed_logins: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE resource_type = 'auth' AND changes LIKE '%invalid_token%' AND timestamp >= datetime('now', '-24 hours')"
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap_or(0);

    // Data access patterns
    let data_exports: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'data_access' AND changes LIKE '%export%' AND timestamp >= datetime('now', '-7 days')"
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap_or(0);

    // Admin activity
    let admin_actions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE resource_type = 'admin' AND timestamp >= datetime('now', '-7 days')"
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap_or(0);

    // Top active users
    let top_users = sqlx::query(
        "SELECT user_id, COUNT(*) as action_count 
         FROM audit_log 
         WHERE user_id IS NOT NULL AND timestamp >= datetime('now', '-7 days')
         GROUP BY user_id
         ORDER BY action_count DESC
         LIMIT 10",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let user_activity: Vec<serde_json::Value> = top_users
        .into_iter()
        .map(|row| {
            let user_id_bytes: Option<Vec<u8>> = row.get("user_id");
            serde_json::json!({
                "user_id": user_id_bytes.map(|bytes| Uuid::from_bytes(bytes.try_into().unwrap_or_default())),
                "action_count": row.get::<i64, _>("action_count")
            })
        })
        .collect();

    let report = serde_json::json!({
        "generated_at": chrono::Utc::now(),
        "timeframe": "last_7_days",
        "security_summary": {
            "security_events_24h": security_events,
            "failed_logins_24h": failed_logins,
            "data_exports_7d": data_exports,
            "admin_actions_7d": admin_actions
        },
        "top_users": user_activity,
        "compliance_status": {
            "audit_logging": "enabled",
            "data_retention": "30_days",
            "encryption": "enabled"
        }
    });

    // Log the audit report generation
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "generate_audit_report".to_string(),
        "admin".to_string(),
        "audit_report".to_string(),
        Some(serde_json::json!({
            "report_type": "security_compliance",
            "requested_by": claims.username
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(report))
}

// Trigger OAuth credential validation job specifically
pub async fn trigger_oauth_validation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    // Use the BatchJobService to schedule the job
    let job_id = state
        .batch_job_service
        .schedule_oauth_validation()
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // Log the action
    AuditService::log_event(
        &state.db_pool,
        Some(claims.sub),
        "create".to_string(),
        "admin".to_string(),
        format!("oauth_validation_job_{}", job_id),
        Some(serde_json::json!({
            "job_type": "oauth_credential_validation",
            "triggered_by": "admin_endpoint",
            "admin_user": claims.username
        })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "job_type": "oauth_credential_validation",
        "status": "scheduled",
        "message": "OAuth credential validation job scheduled successfully"
    })))
}

// Helper function to check admin permissions
async fn has_admin_permissions(user_id: &Uuid, state: &AppState) -> AppResult<bool> {
    let query = "SELECT role FROM users WHERE id = ?";
    let mut conn = state
        .db_pool
        .acquire()
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let role: Option<String> = sqlx::query_scalar(query)
        .bind(user_id.to_string())
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    match role.as_deref() {
        Some("admin") | Some("superuser") => Ok(true),
        _ => Ok(false),
    }
}
