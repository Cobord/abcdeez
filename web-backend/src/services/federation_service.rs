use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    config::Config,
    db::{DbPool, uuid_to_db, json_to_db},
    error::{AppError, AppResult},
    models::federation::*,
};

/// Federation service for managing inter-institutional communication
pub struct FederationService {
    db_pool: Arc<DbPool>,
    config: Arc<Config>,
    http_client: Client,
}

impl FederationService {
    pub fn new(db_pool: Arc<DbPool>, config: Arc<Config>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(false) // Require valid certs in production
            .build()
            .expect("Failed to create HTTP client");

        Self {
            db_pool,
            config,
            http_client,
        }
    }

    /// Register a new federation node
    pub async fn register_node(
        &self,
        request: RegisterNodeRequest,
    ) -> AppResult<RegisterNodeResponse> {
        let node_id = Uuid::new_v4();
        let api_key = self.generate_api_key();

        // Verify the institution's public key
        if !self.verify_public_key(&request.public_key).await {
            return Err(AppError::ValidationError("Invalid public key".to_string()));
        }

        // Check if institution already exists
        let mut conn = self.db_pool.acquire().await?;
        let existing = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM federation_nodes WHERE institution_id = ?)",
        )
        .bind(&request.institution_id)
        .fetch_one(&mut *conn)
        .await?;

        if existing {
            return Err(AppError::Conflict(
                "Institution already registered".to_string(),
            ));
        }

        // Insert new node
        let now = Utc::now();
        let capabilities_json = json_to_db(&request.capabilities)?;
        let metadata_json = json_to_db(&json!({
            "admin_contact": request.admin_contact,
            "compliance_certifications": request.compliance_certifications
        }))?;
        
        sqlx::query(
            "INSERT INTO federation_nodes
             (id, institution_id, institution_name, base_url, api_version, public_key,
              status, capabilities, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid_to_db(node_id))
        .bind(&request.institution_id)
        .bind(&request.institution_name)
        .bind(&request.base_url)
        .bind(&request.api_version)
        .bind(&request.public_key)
        .bind("Pending")
        .bind(capabilities_json)
        .bind(metadata_json)
        .bind(now)
        .bind(now)
        .execute(&mut *conn)
        .await?;

        // Store API key securely
        sqlx::query(
            "INSERT INTO federation_api_keys (node_id, api_key, created_at)
             VALUES (?, ?, ?)",
        )
        .bind(uuid_to_db(node_id))
        .bind(&api_key)
        .bind(now)
        .execute(&mut *conn)
        .await?;

        info!(
            "Registered new federation node: {} ({})",
            request.institution_name, node_id
        );

        Ok(RegisterNodeResponse {
            node_id,
            api_key,
            our_public_key: self.get_our_public_key(),
            status: NodeStatus::Pending,
        })
    }

    /// Process heartbeat from federation node
    pub async fn process_heartbeat(&self, heartbeat: NodeHeartbeat) -> AppResult<()> {
        let mut conn = self.db_pool.acquire().await?;

        // Update node status and last heartbeat
        let result = sqlx::query(
            "UPDATE federation_nodes
             SET last_heartbeat = ?, status = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(heartbeat.timestamp)
        .bind(format!("{:?}", heartbeat.status))
        .bind(Utc::now())
        .bind(uuid_to_db(heartbeat.node_id))
        .execute(&mut *conn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Federation node not found".to_string()));
        }

        // Store heartbeat metrics
        sqlx::query(
            "INSERT INTO federation_heartbeats
             (node_id, timestamp, active_experiments, active_learners, api_version)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid_to_db(heartbeat.node_id))
        .bind(heartbeat.timestamp)
        .bind(heartbeat.active_experiments)
        .bind(heartbeat.active_learners)
        .bind(&heartbeat.api_version)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Share data with federation network
    pub async fn share_data(&self, request: FederationShareRequest) -> AppResult<()> {
        // Verify signature
        if !self
            .verify_signature(&request.source_node_id, &request.data, &request.signature)
            .await?
        {
            return Err(AppError::Forbidden);
        }

        // Check data sharing agreements
        for target_node_id in &request.target_node_ids {
            if !self
                .has_valid_agreement(&request.source_node_id, target_node_id)
                .await?
            {
                warn!(
                    "No valid agreement between {} and {}",
                    request.source_node_id, target_node_id
                );
                continue;
            }

            // Queue data for sharing
            self.queue_data_transfer(
                &request.source_node_id,
                target_node_id,
                &request.data_type,
                &request.data,
            )
            .await?;
        }

        Ok(())
    }

    /// Sync protocol with federation network
    pub async fn sync_protocol(
        &self,
        request: ProtocolSyncRequest,
    ) -> AppResult<ProtocolSyncResponse> {
        let mut conn = self.db_pool.acquire().await?;

        // Get protocol changes since last sync
        let changes = if let Some(last_sync) = request.last_sync_timestamp {
            sqlx::query_as::<_, (String, String, String, String, DateTime<Utc>, String)>(
                "SELECT change_type, version, description, data, changed_at, changed_by
                 FROM protocol_changes
                 WHERE protocol_id = ? AND changed_at > ?
                 ORDER BY changed_at",
            )
            .bind(uuid_to_db(request.protocol_id))
            .bind(last_sync)
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(
                |(change_type, version, description, data, changed_at, changed_by)| {
                    ProtocolChange {
                        change_type: match change_type.as_str() {
                            "Created" => ChangeType::Created,
                            "Updated" => ChangeType::Updated,
                            "Published" => ChangeType::Published,
                            "Deprecated" => ChangeType::Deprecated,
                            _ => ChangeType::Updated,
                        },
                        version,
                        description,
                        data: serde_json::from_str(&data).unwrap_or(json!({})),
                        changed_at,
                        changed_by,
                    }
                },
            )
            .collect()
        } else {
            // Get all changes for initial sync
            vec![]
        };

        // Get current version
        let current_version = sqlx::query_scalar::<_, String>(
            "SELECT version FROM protocol_versions
             WHERE protocol_id = ? AND status = 'Published'
             ORDER BY published_at DESC LIMIT 1",
        )
        .bind(uuid_to_db(request.protocol_id))
        .fetch_optional(&mut *conn)
        .await?
        .unwrap_or_else(|| "0.0.0".to_string());

        Ok(ProtocolSyncResponse {
            protocol_id: request.protocol_id,
            current_version,
            changes,
            sync_timestamp: Utc::now(),
        })
    }

    /// Verify compliance status of a node
    pub async fn verify_compliance(
        &self,
        request: ComplianceVerificationRequest,
    ) -> AppResult<ComplianceVerificationResponse> {
        let mut conn = self.db_pool.acquire().await?;

        // Get node's compliance records
        let compliance_records = sqlx::query(
            "SELECT compliance_type, is_compliant, expiry_date, notes
             FROM federation_compliance
             WHERE node_id = ? AND compliance_type IN (SELECT value FROM json_each(?))",
        )
        .bind(uuid_to_db(request.node_id))
        .bind(json_to_db(&request.compliance_types)?)
        .fetch_all(&mut *conn)
        .await?;

        let compliance_status: Vec<ComplianceStatus> = compliance_records
            .into_iter()
            .map(|row| ComplianceStatus {
                compliance_type: serde_json::from_str(row.get("compliance_type")).unwrap(),
                is_compliant: row.get("is_compliant"),
                expiry_date: row.get("expiry_date"),
                notes: row.get("notes"),
            })
            .collect();

        // Get audit dates
        let audit_info = sqlx::query(
            "SELECT last_audit_date, next_audit_date
             FROM federation_nodes
             WHERE id = ?",
        )
        .bind(uuid_to_db(request.node_id))
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(AppError::NotFound("Node not found".to_string()))?;

        let evidence_urls = if request.include_evidence {
            Some(self.get_compliance_evidence_urls(&request.node_id).await?)
        } else {
            None
        };

        Ok(ComplianceVerificationResponse {
            node_id: request.node_id,
            compliance_status,
            last_audit_date: audit_info.get("last_audit_date"),
            next_audit_date: audit_info.get("next_audit_date"),
            evidence_urls,
        })
    }

    /// Get federation network statistics
    pub async fn get_network_stats(&self) -> AppResult<FederationNetworkStats> {
        let mut conn = self.db_pool.acquire().await?;

        let stats = sqlx::query(
            "SELECT
                COUNT(*) as total_nodes,
                SUM(CASE WHEN status = 'Active' THEN 1 ELSE 0 END) as active_nodes,
                (SELECT COUNT(*) FROM federation_agreements) as total_agreements,
                (SELECT COUNT(*) FROM federation_agreements WHERE status = 'Active') as active_agreements,
                (SELECT COALESCE(SUM(data_size), 0) FROM federation_transfers WHERE DATE(created_at) = DATE('now')) as data_shared_today,
                (SELECT COUNT(DISTINCT experiment_id) FROM experiments) as total_experiments,
                (SELECT COUNT(*) FROM learners) as total_learners
             FROM federation_nodes"
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(FederationNetworkStats {
            total_nodes: stats.get("total_nodes"),
            active_nodes: stats.get("active_nodes"),
            total_agreements: stats.get("total_agreements"),
            active_agreements: stats.get("active_agreements"),
            data_shared_today: stats.get("data_shared_today"),
            total_experiments: stats.get("total_experiments"),
            total_learners: stats.get("total_learners"),
            last_updated: Utc::now(),
        })
    }

    /// Create or update a federation agreement
    pub async fn create_agreement(
        &self,
        initiator_id: Uuid,
        partner_id: Uuid,
        agreement_type: AgreementType,
        data_sharing_rules: serde_json::Value,
        compliance_requirements: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> AppResult<Uuid> {
        let agreement_id = Uuid::new_v4();
        let now = Utc::now();

        let mut conn = self.db_pool.acquire().await?;

        sqlx::query(
            "INSERT INTO federation_agreements
             (id, initiator_node_id, partner_node_id, agreement_type, data_sharing_rules,
              compliance_requirements, status, expires_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid_to_db(agreement_id))
        .bind(uuid_to_db(initiator_id))
        .bind(uuid_to_db(partner_id))
        .bind(format!("{:?}", agreement_type))
        .bind(json_to_db(&data_sharing_rules)?)
        .bind(json_to_db(&compliance_requirements)?)
        .bind("Proposed")
        .bind(expires_at)
        .bind(now)
        .bind(now)
        .execute(&mut *conn)
        .await?;

        info!(
            "Created federation agreement {} between {} and {}",
            agreement_id, initiator_id, partner_id
        );

        Ok(agreement_id)
    }

    // Helper methods

    async fn verify_public_key(&self, public_key: &str) -> bool {
        // TODO: Implement actual public key verification
        // For now, just check it's not empty and has reasonable format
        !public_key.is_empty() && public_key.len() > 100
    }

    fn generate_api_key(&self) -> String {
        use rand::Rng;
        use base64::{engine::general_purpose::STANDARD, Engine};
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        STANDARD.encode(&bytes)
    }

    fn get_our_public_key(&self) -> String {
        // TODO: Load from secure storage
        "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----".to_string()
    }

    async fn verify_signature(
        &self,
        node_id: &Uuid,
        _data: &serde_json::Value,
        _signature: &str,
    ) -> AppResult<bool> {
        // CRITICAL: Signature verification must be implemented before production use
        // This is a security vulnerability that allows any node to impersonate others
        tracing::error!(
            "SECURITY WARNING: Signature verification not implemented for node {}",
            node_id
        );
        
        // For now, reject all requests to prevent exploitation
        // TODO: Implement actual signature verification using node's public key
        // 1. Fetch node's public key from database
        // 2. Serialize data in canonical form
        // 3. Verify signature using appropriate crypto library
        Err(AppError::Forbidden)
    }

    async fn has_valid_agreement(&self, source_id: &Uuid, target_id: &Uuid) -> AppResult<bool> {
        let mut conn = self.db_pool.acquire().await?;

        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM federation_agreements
                WHERE status = 'Active'
                AND ((initiator_node_id = ? AND partner_node_id = ?)
                     OR (initiator_node_id = ? AND partner_node_id = ?))
                AND (expires_at IS NULL OR expires_at > ?)
            )",
        )
        .bind(uuid_to_db(*source_id))
        .bind(uuid_to_db(*target_id))
        .bind(uuid_to_db(*target_id))
        .bind(uuid_to_db(*source_id))
        .bind(Utc::now())
        .fetch_one(&mut *conn)
        .await?;

        Ok(exists)
    }

    async fn queue_data_transfer(
        &self,
        source_id: &Uuid,
        target_id: &Uuid,
        data_type: &DataType,
        data: &serde_json::Value,
    ) -> AppResult<()> {
        let transfer_id = Uuid::new_v4();
        let mut conn = self.db_pool.acquire().await?;

        sqlx::query(
            "INSERT INTO federation_transfers
             (id, source_node_id, target_node_id, data_type, data, status, created_at)
             VALUES (?, ?, ?, ?, ?, 'Queued', ?)",
        )
        .bind(uuid_to_db(transfer_id))
        .bind(uuid_to_db(*source_id))
        .bind(uuid_to_db(*target_id))
        .bind(format!("{:?}", data_type))
        .bind(data.to_string())
        .bind(Utc::now())
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    async fn get_compliance_evidence_urls(&self, node_id: &Uuid) -> AppResult<Vec<String>> {
        // TODO: Retrieve actual evidence URLs from storage
        Ok(vec![
            format!(
                "/api/federation/nodes/{}/compliance/evidence/hipaa.pdf",
                node_id
            ),
            format!(
                "/api/federation/nodes/{}/compliance/evidence/irb.pdf",
                node_id
            ),
        ])
    }
}
