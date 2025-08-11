use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Protocol version represents a specific version of a research protocol
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtocolVersion {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub version: String,  // Semantic versioning: "1.0.0", "1.1.0-beta"
    pub parent_version_id: Option<Uuid>,  // For branching
    pub status: ProtocolStatus,
    pub definition: serde_json::Value,  // Complete protocol definition
    pub changelog: String,
    pub validation_rules: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub deprecated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ProtocolStatus {
    Draft,
    Review,
    Published,
    Deprecated,
    Archived,
}

/// Main protocol entity that groups versions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Protocol {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub experiment_id: Uuid,
    pub current_version_id: Option<Uuid>,
    pub created_by: Uuid,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Protocol branch for experimental variations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtocolBranch {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub name: String,
    pub base_version_id: Uuid,
    pub description: String,
    pub created_by: Uuid,
    pub merged_into_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
}

/// Request to create a new protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProtocolRequest {
    pub name: String,
    pub description: String,
    pub experiment_id: Uuid,
    pub initial_definition: serde_json::Value,
    pub tags: Vec<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProtocolResponse {
    pub protocol_id: Uuid,
    pub initial_version_id: Uuid,
    pub version: String,
}

/// Request to create a new version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionRequest {
    pub protocol_id: Uuid,
    pub base_version_id: Option<Uuid>,
    pub version: String,
    pub definition: serde_json::Value,
    pub changelog: String,
    pub validation_rules: Option<serde_json::Value>,
}

/// Request to publish a version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishVersionRequest {
    pub version_id: Uuid,
    pub set_as_current: bool,
    pub notify_federation: bool,
}

/// Version comparison request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareVersionsRequest {
    pub version_a_id: Uuid,
    pub version_b_id: Uuid,
    pub include_definition_diff: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareVersionsResponse {
    pub version_a: VersionSummary,
    pub version_b: VersionSummary,
    pub differences: Vec<VersionDifference>,
    pub definition_diff: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSummary {
    pub id: Uuid,
    pub version: String,
    pub status: ProtocolStatus,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDifference {
    pub field: String,
    pub change_type: DiffChangeType,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffChangeType {
    Added,
    Modified,
    Removed,
}

/// Version history request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVersionHistoryRequest {
    pub protocol_id: Uuid,
    pub include_branches: bool,
    pub include_deprecated: bool,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistoryResponse {
    pub protocol_id: Uuid,
    pub versions: Vec<VersionHistoryItem>,
    pub branches: Option<Vec<BranchInfo>>,
    pub total_versions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistoryItem {
    pub id: Uuid,
    pub version: String,
    pub status: ProtocolStatus,
    pub parent_version_id: Option<Uuid>,
    pub changelog: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub id: Uuid,
    pub name: String,
    pub base_version: String,
    pub created_at: DateTime<Utc>,
    pub is_merged: bool,
}

/// Protocol validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub error_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub warning_type: String,
    pub message: String,
}

/// Protocol metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetrics {
    pub protocol_id: Uuid,
    pub version_id: Uuid,
    pub total_sessions: i32,
    pub active_learners: i32,
    pub avg_completion_rate: f64,
    pub avg_success_rate: f64,
    pub last_used: Option<DateTime<Utc>>,
    pub calculated_at: DateTime<Utc>,
}