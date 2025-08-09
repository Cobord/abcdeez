use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============= Cloud Providers =============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CloudProvider {
    ICloud,
    GoogleDrive,
    OneDrive,
    Dropbox,
    Local,
}

impl CloudProvider {
    pub fn from_platform(platform: &str) -> Self {
        match platform {
            "ios" | "macos" => CloudProvider::ICloud,
            "android" => CloudProvider::GoogleDrive,
            "windows" => CloudProvider::OneDrive,
            _ => CloudProvider::Local,
        }
    }
}

// ============= Sync Types =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    LocalWins,
    RemoteWins,
    Merge,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Conflicted,
}

// ============= Database Models =============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncMetadata {
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub id: Uuid,
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub user_id: Uuid,
    pub device_id: String,
    pub device_name: Option<String>,
    pub platform: Option<String>,
    pub last_sync_at: DateTime<Utc>,
    pub sync_version: i32,
    pub sync_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncQueueItem {
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub id: Uuid,
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub user_id: Uuid,
    pub device_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: String,
    pub data: String, // JSON
    pub sync_version: i32,
    pub created_at: DateTime<Utc>,
    pub synced: bool,
    pub synced_at: Option<DateTime<Utc>>,
    pub conflict: bool,
    pub conflict_data: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncConflict {
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub id: Uuid,
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub user_id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub local_data: String, // JSON
    pub remote_data: String, // JSON
    pub resolution_strategy: Option<String>,
    pub resolved_data: Option<String>, // JSON
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegisteredDevice {
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub id: Uuid,
    #[serde(serialize_with = "crate::utils::serialize_uuid_as_string")]
    pub user_id: Uuid,
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub platform: String,
    pub platform_version: Option<String>,
    pub app_version: Option<String>,
    pub push_token: Option<String>,
    pub last_seen_at: DateTime<Utc>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

// ============= API Request/Response Types =============

#[derive(Debug, Deserialize)]
pub struct SyncPushRequest {
    pub device_id: String,
    pub sync_version: i32,
    pub changes: Vec<SyncChange>,
    pub checkpoint: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChange {
    pub entity_type: String,
    pub entity_id: String,
    pub operation: SyncOperation,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub version: i32,
}

#[derive(Debug, Serialize)]
pub struct SyncPushResponse {
    pub sync_token: String,
    pub new_version: i32,
    pub conflicts: Vec<SyncConflictInfo>,
    pub accepted_changes: i32,
    pub rejected_changes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflictInfo {
    pub entity_type: String,
    pub entity_id: String,
    pub conflict_type: String,
    pub local_version: i32,
    pub remote_version: i32,
    pub suggested_resolution: ConflictResolution,
}

#[derive(Debug, Deserialize)]
pub struct SyncPullRequest {
    pub device_id: String,
    pub last_sync_token: Option<String>,
    pub sync_version: Option<i32>,
    pub entity_types: Option<Vec<String>>, // Filter specific entity types
}

#[derive(Debug, Serialize)]
pub struct SyncPullResponse {
    pub sync_token: String,
    pub changes: Vec<SyncChange>,
    pub deleted_entities: Vec<DeletedEntity>,
    pub sync_version: i32,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedEntity {
    pub entity_type: String,
    pub entity_id: String,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveConflictRequest {
    pub conflict_id: Uuid,
    pub resolution: ConflictResolution,
    pub merged_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ResolveConflictResponse {
    pub resolved: bool,
    pub final_data: serde_json::Value,
    pub sync_version: i32,
}

#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub last_sync_at: Option<DateTime<Utc>>,
    pub pending_changes: i32,
    pub conflicts: i32,
    pub sync_in_progress: bool,
    pub connected_devices: Vec<DeviceInfo>,
    pub cloud_provider: CloudProvider,
    pub storage_used_bytes: Option<i64>,
    pub storage_quota_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
    pub last_sync_at: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub platform: String,
    pub platform_version: Option<String>,
    pub app_version: Option<String>,
    pub push_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterDeviceResponse {
    pub device_id: String,
    pub sync_token: String,
    pub initial_sync_required: bool,
}

// ============= Sync Data Structures =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPacket {
    pub version: i32,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
    pub changes: Vec<SyncChange>,
    pub checksum: String,
}

impl SyncPacket {
    pub fn calculate_checksum(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        
        hasher.update(self.version.to_string());
        hasher.update(&self.device_id);
        hasher.update(&self.user_id);
        
        for change in &self.changes {
            hasher.update(&change.entity_type);
            hasher.update(&change.entity_id);
            hasher.update(format!("{:?}", change.operation));
        }
        
        format!("{:x}", hasher.finalize())
    }
    
    pub fn verify_checksum(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }
}

// ============= Helper Functions =============

pub fn merge_sync_data(
    local: &serde_json::Value,
    remote: &serde_json::Value,
    entity_type: &str,
) -> serde_json::Value {
    // Simple merge strategy - can be enhanced based on entity type
    match entity_type {
        "session" | "response" => {
            // For immutable data, prefer the one with later timestamp
            if let (Some(local_time), Some(remote_time)) = (
                local.get("created_at").and_then(|v| v.as_str()),
                remote.get("created_at").and_then(|v| v.as_str()),
            ) {
                if local_time > remote_time {
                    local.clone()
                } else {
                    remote.clone()
                }
            } else {
                remote.clone()
            }
        }
        "learner_model" | "user_gamification" => {
            // For models, merge fields with conflict resolution
            let mut merged = local.clone();
            if let (Some(local_obj), Some(remote_obj)) = (
                merged.as_object_mut(),
                remote.as_object(),
            ) {
                for (key, remote_value) in remote_obj {
                    // Use remote value if it's newer or local doesn't have it
                    if !local_obj.contains_key(key) {
                        local_obj.insert(key.clone(), remote_value.clone());
                    } else if let (Some(local_updated), Some(remote_updated)) = (
                        local_obj.get("updated_at").and_then(|v| v.as_str()),
                        remote_value.get("updated_at").and_then(|v| v.as_str()),
                    ) {
                        if remote_updated > local_updated {
                            local_obj.insert(key.clone(), remote_value.clone());
                        }
                    }
                }
            }
            merged
        }
        _ => {
            // Default: prefer remote
            remote.clone()
        }
    }
}