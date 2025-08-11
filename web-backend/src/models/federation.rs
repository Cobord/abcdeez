use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Federation node represents a partner institution
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FederationNode {
    pub id: Uuid,
    pub institution_id: String,
    pub institution_name: String,
    pub base_url: String,
    pub api_version: String,
    pub public_key: String,  // For mutual TLS and signature verification
    pub status: NodeStatus,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub capabilities: Vec<String>,  // JSON array of supported features
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Status of a federation node
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum NodeStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
}

/// Federation agreement between institutions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FederationAgreement {
    pub id: Uuid,
    pub initiator_node_id: Uuid,
    pub partner_node_id: Uuid,
    pub agreement_type: AgreementType,
    pub data_sharing_rules: serde_json::Value,  // JSON rules for what can be shared
    pub compliance_requirements: Vec<String>,
    pub status: AgreementStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum AgreementType {
    DataSharing,
    ResearchCollaboration,
    ProtocolExchange,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum AgreementStatus {
    Proposed,
    Negotiating,
    Active,
    Expired,
    Terminated,
}

/// Request to register a new federation node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeRequest {
    pub institution_id: String,
    pub institution_name: String,
    pub base_url: String,
    pub api_version: String,
    pub public_key: String,
    pub capabilities: Vec<String>,
    pub admin_contact: String,
    pub compliance_certifications: Vec<String>,
}

/// Response from node registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeResponse {
    pub node_id: Uuid,
    pub api_key: String,  // For authenticating requests from this node
    pub our_public_key: String,  // Our public key for them to verify
    pub status: NodeStatus,
}

/// Heartbeat from a federation node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHeartbeat {
    pub node_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub status: NodeStatus,
    pub active_experiments: i32,
    pub active_learners: i32,
    pub api_version: String,
}

/// Request to share data with federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationShareRequest {
    pub source_node_id: Uuid,
    pub target_node_ids: Vec<Uuid>,
    pub data_type: DataType,
    pub data: serde_json::Value,
    pub encryption_key_id: Option<String>,
    pub signature: String,  // Digital signature for verification
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    ExperimentResults,
    ProtocolDefinition,
    LearnerProgress,
    AggregatedAnalytics,
    ComplianceReport,
}

/// Protocol sync request between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSyncRequest {
    pub node_id: Uuid,
    pub protocol_id: Uuid,
    pub version: String,
    pub last_sync_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSyncResponse {
    pub protocol_id: Uuid,
    pub current_version: String,
    pub changes: Vec<ProtocolChange>,
    pub sync_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolChange {
    pub change_type: ChangeType,
    pub version: String,
    pub description: String,
    pub data: serde_json::Value,
    pub changed_at: DateTime<Utc>,
    pub changed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Updated,
    Published,
    Deprecated,
}

/// Compliance verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceVerificationRequest {
    pub node_id: Uuid,
    pub compliance_types: Vec<ComplianceType>,
    pub include_evidence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceType {
    HIPAA,
    FERPA,
    GDPR,
    IRB,
    DataRetention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceVerificationResponse {
    pub node_id: Uuid,
    pub compliance_status: Vec<ComplianceStatus>,
    pub last_audit_date: DateTime<Utc>,
    pub next_audit_date: DateTime<Utc>,
    pub evidence_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub compliance_type: ComplianceType,
    pub is_compliant: bool,
    pub expiry_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Federation network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationNetworkStats {
    pub total_nodes: i32,
    pub active_nodes: i32,
    pub total_agreements: i32,
    pub active_agreements: i32,
    pub data_shared_today: i64,
    pub total_experiments: i32,
    pub total_learners: i32,
    pub last_updated: DateTime<Utc>,
}