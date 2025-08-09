use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::sync::*,
    state::AppState,
};

// Helper structs for database queries
#[derive(FromRow)]
struct SyncQueueRow {
    entity_type: String,
    entity_id: String,
    operation: String,
    data: String,
    sync_version: i32,
    created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct SyncDeletionRow {
    entity_type: String,
    entity_id: String,
    created_at: DateTime<Utc>,
}

// ============= Device Management =============

/// Register a new device for sync
pub async fn register_device(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<RegisterDeviceRequest>,
) -> AppResult<Json<RegisterDeviceResponse>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    // Check if device already registered
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM registered_devices WHERE user_id = ? AND device_id = ?)",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(&req.device_id)
    .fetch_one(&mut *conn)
    .await?;

    let device_record_id = Uuid::new_v4();

    if existing {
        // Update existing device
        sqlx::query(
            "UPDATE registered_devices 
             SET device_name = ?, device_type = ?, platform = ?, 
                 platform_version = ?, app_version = ?, push_token = ?,
                 last_seen_at = ?, active = true
             WHERE user_id = ? AND device_id = ?",
        )
        .bind(&req.device_name)
        .bind(&req.device_type)
        .bind(&req.platform)
        .bind(req.platform_version.as_deref())
        .bind(req.app_version.as_deref())
        .bind(req.push_token.as_deref())
        .bind(Utc::now())
        .bind(user_id.as_bytes().as_slice())
        .bind(&req.device_id)
        .execute(&mut *conn)
        .await?;
    } else {
        // Register new device
        sqlx::query(
            "INSERT INTO registered_devices 
             (id, user_id, device_id, device_name, device_type, platform, 
              platform_version, app_version, push_token, last_seen_at, active, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, true, ?)",
        )
        .bind(device_record_id.as_bytes().as_slice())
        .bind(user_id.as_bytes().as_slice())
        .bind(&req.device_id)
        .bind(&req.device_name)
        .bind(&req.device_type)
        .bind(&req.platform)
        .bind(req.platform_version.as_deref())
        .bind(req.app_version.as_deref())
        .bind(req.push_token.as_deref())
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&mut *conn)
        .await?;
    }

    // Generate sync token
    let sync_token = generate_sync_token(user_id, &req.device_id);

    // Create or update sync metadata
    let sync_metadata_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO sync_metadata 
         (id, user_id, device_id, device_name, platform, last_sync_at, sync_version, sync_token, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?)
         ON CONFLICT(user_id, device_id) DO UPDATE SET
         sync_token = excluded.sync_token,
         updated_at = excluded.updated_at"
    )
    .bind(sync_metadata_id.as_bytes().as_slice())
    .bind(user_id.as_bytes().as_slice())
    .bind(&req.device_id)
    .bind(&req.device_name)
    .bind(&req.platform)
    .bind(Utc::now())
    .bind(&sync_token)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&mut *conn)
    .await?;

    // Check if initial sync is needed
    let has_data = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE learner_id IN 
         (SELECT id FROM learners WHERE user_id = ?))",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_one(&mut *conn)
    .await?;

    Ok(Json(RegisterDeviceResponse {
        device_id: req.device_id,
        sync_token,
        initial_sync_required: !has_data,
    }))
}

// ============= Sync Operations =============

/// Push changes from device to server
pub async fn sync_push(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SyncPushRequest>,
) -> AppResult<Json<SyncPushResponse>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    // Verify device
    verify_device(&mut conn, user_id, &req.device_id).await?;

    // Start transaction
    let mut tx = state.db_pool.begin().await?;

    let mut accepted = 0;
    let mut rejected = 0;
    let mut conflicts = Vec::new();

    for change in &req.changes {
        match process_sync_change(&mut tx, user_id, &req.device_id, change).await {
            Ok(()) => accepted += 1,
            Err(SyncError::Conflict(info)) => {
                conflicts.push(info);
                rejected += 1;
            }
            Err(_) => rejected += 1,
        }
    }

    // Update sync version
    let new_version = req.sync_version + 1;
    sqlx::query(
        "UPDATE sync_metadata 
         SET sync_version = ?, last_sync_at = ?, updated_at = ?
         WHERE user_id = ? AND device_id = ?",
    )
    .bind(new_version)
    .bind(Utc::now())
    .bind(Utc::now())
    .bind(user_id.as_bytes().as_slice())
    .bind(&req.device_id)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    // Generate new sync token
    let sync_token = generate_sync_token(user_id, &req.device_id);

    Ok(Json(SyncPushResponse {
        sync_token,
        new_version,
        conflicts,
        accepted_changes: accepted,
        rejected_changes: rejected,
    }))
}

/// Pull changes from server to device
pub async fn sync_pull(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SyncPullRequest>,
) -> AppResult<Json<SyncPullResponse>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    // Verify device
    verify_device(&mut conn, user_id, &req.device_id).await?;

    // Get current sync version
    let current_version = sqlx::query_scalar::<_, i32>(
        "SELECT sync_version FROM sync_metadata WHERE user_id = ? AND device_id = ?",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(&req.device_id)
    .fetch_optional(&mut *conn)
    .await?
    .unwrap_or(0);

    let from_version = req.sync_version.unwrap_or(0);

    // Get changes since last sync
    let changes = get_changes_since_version(&mut conn, user_id, from_version).await?;

    // Get deleted entities
    let deleted_entities = get_deleted_entities_since(&mut conn, user_id, from_version).await?;

    // Generate sync token
    let sync_token = generate_sync_token(user_id, &req.device_id);

    // Check if there are more changes
    let has_more = changes.len() >= 100; // Paginate large syncs

    Ok(Json(SyncPullResponse {
        sync_token,
        changes,
        deleted_entities,
        sync_version: current_version,
        has_more,
    }))
}

/// Get sync status
pub async fn sync_status(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<SyncStatusResponse>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    // Get last sync info
    let last_sync = sqlx::query_as::<_, SyncMetadata>(
        "SELECT * FROM sync_metadata WHERE user_id = ? ORDER BY last_sync_at DESC LIMIT 1",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_optional(&mut *conn)
    .await?;

    // Count pending changes
    let pending_changes = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sync_queue WHERE user_id = ? AND NOT synced",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_one(&mut *conn)
    .await? as i32;

    // Count conflicts
    let conflicts = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sync_conflicts WHERE user_id = ? AND NOT resolved",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_one(&mut *conn)
    .await? as i32;

    // Check if sync in progress
    let sync_in_progress = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sync_queue WHERE user_id = ? AND synced = false)",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_one(&mut *conn)
    .await?;

    // Get connected devices
    let devices = sqlx::query_as::<_, RegisteredDevice>(
        "SELECT * FROM registered_devices WHERE user_id = ? AND active = true",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_all(&mut *conn)
    .await?;

    let connected_devices: Vec<DeviceInfo> = devices
        .into_iter()
        .map(|d| DeviceInfo {
            device_id: d.device_id.clone(),
            device_name: d.device_name,
            platform: d.platform,
            last_sync_at: d.last_seen_at,
            is_current: false, // TODO: Mark current device
        })
        .collect();

    // Determine cloud provider based on platform
    let cloud_provider = if let Some(ref sync) = last_sync {
        CloudProvider::from_platform(&sync.platform.clone().unwrap_or_default())
    } else {
        CloudProvider::Local
    };

    Ok(Json(SyncStatusResponse {
        last_sync_at: last_sync.map(|s| s.last_sync_at),
        pending_changes,
        conflicts,
        sync_in_progress,
        connected_devices,
        cloud_provider,
        storage_used_bytes: None,
        storage_quota_bytes: None,
    }))
}

/// Resolve a sync conflict
pub async fn resolve_conflict(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(conflict_id): Path<Uuid>,
    Json(req): Json<ResolveConflictRequest>,
) -> AppResult<Json<ResolveConflictResponse>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    // Get conflict
    let conflict = sqlx::query_as::<_, SyncConflict>(
        "SELECT * FROM sync_conflicts WHERE id = ? AND user_id = ?",
    )
    .bind(conflict_id.as_bytes().as_slice())
    .bind(user_id.as_bytes().as_slice())
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AppError::NotFound("Conflict not found".to_string()))?;

    // Determine final data based on resolution strategy
    let final_data = match req.resolution {
        ConflictResolution::LocalWins => serde_json::from_str(&conflict.local_data)?,
        ConflictResolution::RemoteWins => serde_json::from_str(&conflict.remote_data)?,
        ConflictResolution::Merge => {
            if let Some(merged) = req.merged_data {
                merged
            } else {
                // Auto-merge
                let local: serde_json::Value = serde_json::from_str(&conflict.local_data)?;
                let remote: serde_json::Value = serde_json::from_str(&conflict.remote_data)?;
                merge_sync_data(&local, &remote, &conflict.entity_type)
            }
        }
        ConflictResolution::Manual => req.merged_data.ok_or(AppError::BadRequest(
            "Merged data required for manual resolution".to_string(),
        ))?,
    };

    // Update conflict as resolved
    sqlx::query(
        "UPDATE sync_conflicts 
         SET resolved = true, resolved_at = ?, resolved_by = 'user', 
             resolution_strategy = ?, resolved_data = ?
         WHERE id = ?",
    )
    .bind(Utc::now())
    .bind(format!("{:?}", req.resolution))
    .bind(serde_json::to_string(&final_data)?)
    .bind(conflict_id.as_bytes().as_slice())
    .execute(&mut *conn)
    .await?;

    // Apply resolved data
    apply_resolved_data(
        &mut conn,
        user_id,
        &conflict.entity_type,
        &conflict.entity_id,
        &final_data,
    )
    .await?;

    // Get new sync version
    let sync_version = sqlx::query_scalar::<_, i32>(
        "SELECT MAX(sync_version) FROM sync_metadata WHERE user_id = ?",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_one(&mut *conn)
    .await?;

    Ok(Json(ResolveConflictResponse {
        resolved: true,
        final_data,
        sync_version,
    }))
}

// ============= Helper Functions =============

fn generate_sync_token(user_id: Uuid, device_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(user_id.as_bytes());
    hasher.update(device_id);
    hasher.update(Utc::now().timestamp().to_string());
    format!("{:x}", hasher.finalize())
}

async fn verify_device(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
    device_id: &str,
) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM registered_devices WHERE user_id = ? AND device_id = ? AND active = true)"
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(device_id)
    .fetch_one(conn)
    .await?;

    if !exists {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

#[derive(Debug)]
enum SyncError {
    Conflict(SyncConflictInfo),
    Invalid(String),
}

async fn process_sync_change(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user_id: Uuid,
    device_id: &str,
    change: &SyncChange,
) -> Result<(), SyncError> {
    // Check for conflicts
    let existing_version = get_entity_version(tx, &change.entity_type, &change.entity_id).await;

    if let Some(existing) = existing_version {
        if existing >= change.version {
            // Conflict detected
            let conflict_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO sync_conflicts 
                 (id, user_id, entity_type, entity_id, local_data, remote_data, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(conflict_id.as_bytes().as_slice())
            .bind(user_id.as_bytes().as_slice())
            .bind(&change.entity_type)
            .bind(&change.entity_id)
            .bind(serde_json::to_string(&change.data).unwrap())
            .bind("{}") // TODO: Get actual remote data
            .bind(Utc::now())
            .execute(&mut **tx)
            .await
            .ok();

            return Err(SyncError::Conflict(SyncConflictInfo {
                entity_type: change.entity_type.clone(),
                entity_id: change.entity_id.clone(),
                conflict_type: "version_mismatch".to_string(),
                local_version: change.version,
                remote_version: existing,
                suggested_resolution: ConflictResolution::RemoteWins,
            }));
        }
    }

    // Apply change
    match change.operation {
        SyncOperation::Create | SyncOperation::Update => {
            apply_entity_change(
                tx,
                user_id,
                &change.entity_type,
                &change.entity_id,
                &change.data,
            )
            .await
            .map_err(|e| SyncError::Invalid(e.to_string()))?;
        }
        SyncOperation::Delete => {
            delete_entity(tx, user_id, &change.entity_type, &change.entity_id)
                .await
                .map_err(|e| SyncError::Invalid(e.to_string()))?;
        }
    }

    // Record in sync queue
    let queue_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO sync_queue 
         (id, user_id, device_id, entity_type, entity_id, operation, data, sync_version, created_at, synced, synced_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, true, ?)"
    )
    .bind(queue_id.as_bytes().as_slice())
    .bind(user_id.as_bytes().as_slice())
    .bind(device_id)
    .bind(&change.entity_type)
    .bind(&change.entity_id)
    .bind(format!("{:?}", change.operation))
    .bind(serde_json::to_string(&change.data).unwrap())
    .bind(change.version)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&mut **tx)
    .await
    .ok();

    Ok(())
}

async fn get_entity_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entity_type: &str,
    entity_id: &str,
) -> Option<i32> {
    // Simplified - in production, track versions per entity type
    sqlx::query_scalar::<_, i32>(
        "SELECT sync_version FROM sync_queue 
         WHERE entity_type = ? AND entity_id = ?
         ORDER BY sync_version DESC LIMIT 1",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_optional(&mut **tx)
    .await
    .ok()
    .flatten()
}

async fn apply_entity_change(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user_id: Uuid,
    entity_type: &str,
    entity_id: &str,
    data: &serde_json::Value,
) -> AppResult<()> {
    // Route to appropriate handler based on entity type
    match entity_type {
        "session" => {
            // Update session data
            // Implementation depends on your session structure
        }
        "learner_model" => {
            // Update learner model
        }
        "achievement" => {
            // Update achievement progress
        }
        _ => {}
    }

    Ok(())
}

async fn delete_entity(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user_id: Uuid,
    entity_type: &str,
    entity_id: &str,
) -> AppResult<()> {
    // Route to appropriate deletion based on entity type
    match entity_type {
        "session" => {
            sqlx::query("UPDATE sessions SET status = 'deleted' WHERE id = ?")
                .bind(entity_id)
                .execute(&mut **tx)
                .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn get_changes_since_version(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
    from_version: i32,
) -> AppResult<Vec<SyncChange>> {
    // Get changes from sync queue
    let rows = sqlx::query_as::<_, SyncQueueRow>(
        "SELECT entity_type, entity_id, operation, data, sync_version, created_at
         FROM sync_queue 
         WHERE user_id = ? AND sync_version > ?
         ORDER BY sync_version
         LIMIT 100",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(from_version)
    .fetch_all(conn)
    .await?;

    let changes = rows
        .into_iter()
        .map(|row| SyncChange {
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            operation: match row.operation.as_str() {
                "create" => SyncOperation::Create,
                "update" => SyncOperation::Update,
                "delete" => SyncOperation::Delete,
                _ => SyncOperation::Update,
            },
            data: serde_json::from_str(&row.data).unwrap_or(serde_json::Value::Null),
            timestamp: row.created_at,
            version: row.sync_version,
        })
        .collect();

    Ok(changes)
}

async fn get_deleted_entities_since(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
    from_version: i32,
) -> AppResult<Vec<DeletedEntity>> {
    // Get deleted entities
    let rows = sqlx::query_as::<_, SyncDeletionRow>(
        "SELECT entity_type, entity_id, created_at
         FROM sync_queue 
         WHERE user_id = ? AND operation = 'delete' AND sync_version > ?
         ORDER BY sync_version",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(from_version)
    .fetch_all(conn)
    .await?;

    let deleted = rows
        .into_iter()
        .map(|row| DeletedEntity {
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            deleted_at: row.created_at,
        })
        .collect();

    Ok(deleted)
}

async fn apply_resolved_data(
    conn: &mut sqlx::SqliteConnection,
    user_id: Uuid,
    entity_type: &str,
    entity_id: &str,
    data: &serde_json::Value,
) -> AppResult<()> {
    // Apply the resolved data to the appropriate entity
    // This is simplified - implement based on your entity types

    Ok(())
}
