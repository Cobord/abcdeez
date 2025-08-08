use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json, Extension,
};
use std::{sync::Arc, collections::HashMap};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    state::AppState,
    services::{AnalyticsService, LearnerService, audit::AuditService},
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
    // TODO: Add admin permission check
    // For now, any authenticated user can access admin endpoints
    
    // Get database statistics
    let db_stats = get_database_status(&state.db_pool).await?;
    
    // Get Redis status
    let redis_stats = get_redis_status(&state).await?;
    
    // Get active sessions count
    let active_sessions = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM sessions WHERE status = 'active'"
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0) as usize;

    let system_status = SystemStatus {
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Would be calculated from startup time
        database: db_stats,
        redis: redis_stats,
        active_sessions,
        active_websockets: 0, // Would be tracked in application state
        memory_usage_mb: None, // Would need system monitoring
        request_rate: 120.0, // Would be calculated from metrics
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
    ).await.ok();

    Ok(Json(system_status))
}

// List all users with pagination and filtering
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);
    let offset = params.get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let users = sqlx::query!(
        "SELECT id, username, email, created_at, updated_at, metadata
         FROM users 
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    let user_data: Vec<serde_json::Value> = users
        .into_iter()
        .map(|user| {
            serde_json::json!({
                "id": Uuid::from_bytes(user.id.try_into().unwrap_or_default()),
                "username": user.username,
                "email": user.email,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
                "metadata": user.metadata.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
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
    ).await.ok();

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
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);
    let offset = params.get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let learners = sqlx::query!(
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
         LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let learner_data: Vec<serde_json::Value> = learners
        .into_iter()
        .map(|learner| {
            serde_json::json!({
                "id": Uuid::from_bytes(learner.id.try_into().unwrap_or_default()),
                "user_id": learner.user_id.map(|bytes| Uuid::from_bytes(bytes.try_into().unwrap_or_default())),
                "username": learner.username,
                "display_name": learner.display_name,
                "created_at": learner.created_at,
                "last_active": learner.last_active,
                "total_practice_time_seconds": learner.total_practice_time_seconds,
                "session_count": learner.session_count.unwrap_or(0),
                "response_count": learner.response_count.unwrap_or(0),
                "metadata": learner.metadata.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            })
        })
        .collect();

    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM learners")
        .fetch_one(&state.db_pool)
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
        params.resource_type,
        params.resource_id,
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
    ).await.ok();

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

    let rows = sqlx::query(&query)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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

    sqlx::query!(
        "INSERT INTO job_queue (id, job_type, payload, status) VALUES ($1, $2, $3, 'pending')",
        job_id_bytes,
        job_type,
        payload.to_string()
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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
    ).await.ok();

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
            
            if redis::cmd("SET")
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
    ).await.ok();

    Ok(Json(serde_json::json!({
        "updated_keys": updated_keys,
        "message": format!("Updated {} configuration keys", updated_keys.len())
    })))
}

// Helper functions
async fn get_database_status(db: &sqlx::PgPool) -> AppResult<DatabaseStatus> {
    // Test database connection
    let connected = sqlx::query("SELECT 1")
        .fetch_one(db)
        .await
        .is_ok();

    // Get pool information (simplified)
    let pool_size = 20; // From configuration
    let active_connections = 5; // Would be from pool.num_idle() in a real implementation

    Ok(DatabaseStatus {
        connected,
        pool_size,
        active_connections,
        total_queries: None, // Would need query statistics
        avg_query_time_ms: None, // Would need performance monitoring
    })
}

async fn get_redis_status(state: &AppState) -> AppResult<RedisStatus> {
    let mut conn = state.redis_conn.clone();
    let connected = redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .is_ok();

    Ok(RedisStatus {
        connected,
        memory_usage_mb: None, // Would need Redis INFO command parsing
        total_connections: None, // Would need Redis INFO command parsing
        cache_hit_rate: None, // Would need hit/miss statistics tracking
    })
}