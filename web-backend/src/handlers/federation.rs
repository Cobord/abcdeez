use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::federation::*,
    services::federation_service::FederationService,
    state::AppState,
};

#[derive(Deserialize)]
pub struct FederationQuery {
    pub include_inactive: Option<bool>,
    pub institution_id: Option<String>,
}

/// Register a new federation node (admin only)
pub async fn register_node(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<RegisterNodeRequest>,
) -> AppResult<(StatusCode, Json<RegisterNodeResponse>)> {
    // Verify admin permissions
    if !claims.permissions.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden);
    }

    let federation_service =
        FederationService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let response = federation_service.register_node(request).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// List all federation nodes
pub async fn list_nodes(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<FederationQuery>,
) -> AppResult<Json<Vec<FederationNode>>> {
    let mut conn = state.db_pool.acquire().await?;

    let mut sql = String::from(
        "SELECT id, institution_id, institution_name, base_url, api_version, public_key,
                status, last_heartbeat, capabilities, metadata, created_at, updated_at
         FROM federation_nodes",
    );

    let mut conditions = vec![];

    if !query.include_inactive.unwrap_or(false) {
        conditions.push("status = 'Active'");
    }

    if let Some(ref inst_id) = query.institution_id {
        conditions.push("institution_id = ?");
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY institution_name");

    let mut query_builder = sqlx::query(&sql);
    if let Some(ref inst_id) = query.institution_id {
        query_builder = query_builder.bind(inst_id);
    }

    let rows = query_builder.fetch_all(&mut *conn).await?;

    let nodes: Vec<FederationNode> = rows
        .into_iter()
        .map(|row| {
            let id_bytes: Vec<u8> = row.get("id");
            let capabilities_str: String = row.get("capabilities");
            let metadata_str: Option<String> = row.get("metadata");

            FederationNode {
                id: Uuid::from_slice(&id_bytes).unwrap(),
                institution_id: row.get("institution_id"),
                institution_name: row.get("institution_name"),
                base_url: row.get("base_url"),
                api_version: row.get("api_version"),
                public_key: row.get("public_key"),
                status: match row.get::<String, _>("status").as_str() {
                    "Active" => NodeStatus::Active,
                    "Inactive" => NodeStatus::Inactive,
                    "Suspended" => NodeStatus::Suspended,
                    "Pending" => NodeStatus::Pending,
                    _ => NodeStatus::Inactive,
                },
                last_heartbeat: row.get("last_heartbeat"),
                capabilities: serde_json::from_str(&capabilities_str).unwrap_or_default(),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(nodes))
}

/// Get a specific federation node
pub async fn get_node(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(node_id): Path<Uuid>,
) -> AppResult<Json<FederationNode>> {
    let mut conn = state.db_pool.acquire().await?;

    let row = sqlx::query(
        "SELECT id, institution_id, institution_name, base_url, api_version, public_key,
                status, last_heartbeat, capabilities, metadata, created_at, updated_at
         FROM federation_nodes WHERE id = ?",
    )
    .bind(node_id.as_bytes().as_slice())
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AppError::NotFound("Federation node not found".to_string()))?;

    let capabilities_str: String = row.get("capabilities");
    let metadata_str: Option<String> = row.get("metadata");

    let node = FederationNode {
        id: node_id,
        institution_id: row.get("institution_id"),
        institution_name: row.get("institution_name"),
        base_url: row.get("base_url"),
        api_version: row.get("api_version"),
        public_key: row.get("public_key"),
        status: match row.get::<String, _>("status").as_str() {
            "Active" => NodeStatus::Active,
            "Inactive" => NodeStatus::Inactive,
            "Suspended" => NodeStatus::Suspended,
            "Pending" => NodeStatus::Pending,
            _ => NodeStatus::Inactive,
        },
        last_heartbeat: row.get("last_heartbeat"),
        capabilities: serde_json::from_str(&capabilities_str).unwrap_or_default(),
        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(node))
}

/// Process heartbeat from a federation node
pub async fn heartbeat(
    State(state): State<Arc<AppState>>,
    Json(heartbeat): Json<NodeHeartbeat>,
) -> AppResult<StatusCode> {
    let federation_service =
        FederationService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    federation_service.process_heartbeat(heartbeat).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Share data with the federation network
pub async fn share_data(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<FederationShareRequest>,
) -> AppResult<StatusCode> {
    // Verify the user has permission to share data
    if !claims.permissions.contains(&"federation_share".to_string())
        && !claims.permissions.contains(&"admin".to_string())
    {
        return Err(AppError::Forbidden);
    }

    let federation_service =
        FederationService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    federation_service.share_data(request).await?;

    Ok(StatusCode::ACCEPTED)
}

/// Sync protocol with federation
pub async fn sync_protocol(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<ProtocolSyncRequest>,
) -> AppResult<Json<ProtocolSyncResponse>> {
    let federation_service =
        FederationService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let response = federation_service.sync_protocol(request).await?;

    Ok(Json(response))
}

/// Verify compliance status of a node
pub async fn verify_compliance(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<ComplianceVerificationRequest>,
) -> AppResult<Json<ComplianceVerificationResponse>> {
    // Only admins can verify compliance
    if !claims.permissions.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden);
    }

    let federation_service =
        FederationService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let response = federation_service.verify_compliance(request).await?;

    Ok(Json(response))
}

/// Get federation network statistics
pub async fn network_stats(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<FederationNetworkStats>> {
    let federation_service =
        FederationService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let stats = federation_service.get_network_stats().await?;

    Ok(Json(stats))
}

/// List federation agreements
pub async fn list_agreements(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<FederationAgreement>>> {
    let mut conn = state.db_pool.acquire().await?;

    let rows = sqlx::query(
        "SELECT id, initiator_node_id, partner_node_id, agreement_type, data_sharing_rules,
                compliance_requirements, status, expires_at, created_at, updated_at
         FROM federation_agreements
         ORDER BY created_at DESC",
    )
    .fetch_all(&mut *conn)
    .await?;

    let agreements: Vec<FederationAgreement> = rows
        .into_iter()
        .map(|row| {
            let id_bytes: Vec<u8> = row.get("id");
            let initiator_bytes: Vec<u8> = row.get("initiator_node_id");
            let partner_bytes: Vec<u8> = row.get("partner_node_id");
            let rules_str: String = row.get("data_sharing_rules");
            let compliance_str: String = row.get("compliance_requirements");

            FederationAgreement {
                id: Uuid::from_slice(&id_bytes).unwrap(),
                initiator_node_id: Uuid::from_slice(&initiator_bytes).unwrap(),
                partner_node_id: Uuid::from_slice(&partner_bytes).unwrap(),
                agreement_type: match row.get::<String, _>("agreement_type").as_str() {
                    "DataSharing" => AgreementType::DataSharing,
                    "ResearchCollaboration" => AgreementType::ResearchCollaboration,
                    "ProtocolExchange" => AgreementType::ProtocolExchange,
                    "Full" => AgreementType::Full,
                    _ => AgreementType::DataSharing,
                },
                data_sharing_rules: serde_json::from_str(&rules_str).unwrap_or_default(),
                compliance_requirements: serde_json::from_str(&compliance_str).unwrap_or_default(),
                status: match row.get::<String, _>("status").as_str() {
                    "Proposed" => AgreementStatus::Proposed,
                    "Negotiating" => AgreementStatus::Negotiating,
                    "Active" => AgreementStatus::Active,
                    "Expired" => AgreementStatus::Expired,
                    "Terminated" => AgreementStatus::Terminated,
                    _ => AgreementStatus::Proposed,
                },
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(agreements))
}

/// Create a new federation agreement
pub async fn create_agreement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<serde_json::Value>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    // Only admins can create agreements
    if !claims.permissions.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden);
    }

    let initiator_id: Uuid = request["initiator_node_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(AppError::ValidationError(
            "Invalid initiator_node_id".to_string(),
        ))?;

    let partner_id: Uuid = request["partner_node_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(AppError::ValidationError(
            "Invalid partner_node_id".to_string(),
        ))?;

    let agreement_type = match request["agreement_type"].as_str() {
        Some("DataSharing") => AgreementType::DataSharing,
        Some("ResearchCollaboration") => AgreementType::ResearchCollaboration,
        Some("ProtocolExchange") => AgreementType::ProtocolExchange,
        Some("Full") => AgreementType::Full,
        _ => AgreementType::DataSharing,
    };

    let data_sharing_rules = request["data_sharing_rules"].clone();
    let compliance_requirements: Vec<String> = request["compliance_requirements"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let expires_at = request["expires_at"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let federation_service =
        FederationService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let agreement_id = federation_service
        .create_agreement(
            initiator_id,
            partner_id,
            agreement_type,
            data_sharing_rules,
            compliance_requirements,
            expires_at,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "agreement_id": agreement_id,
            "status": "Proposed"
        })),
    ))
}
