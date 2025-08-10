use crate::{db::DbPool, middleware::Claims};
use anyhow::Result;
use axum::{extract::Request, http::HeaderMap};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

pub struct AuditService;

#[derive(Debug, Clone)]
pub struct AuditContext {
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub role: Option<String>,
}

impl AuditContext {
    pub fn from_request(request: &Request, claims: Option<&Claims>) -> Self {
        let headers = request.headers();

        Self {
            user_id: claims.map(|c| c.sub),
            username: claims.map(|c| c.username.clone()),
            ip_address: Self::extract_ip_address(headers),
            user_agent: Self::extract_user_agent(headers),
            session_id: claims.and_then(|c| c.session_id.clone()),
            role: claims.map(|c| c.role.clone()),
        }
    }

    pub fn anonymous(headers: &HeaderMap) -> Self {
        Self {
            user_id: None,
            username: None,
            ip_address: Self::extract_ip_address(headers),
            user_agent: Self::extract_user_agent(headers),
            session_id: None,
            role: None,
        }
    }

    fn extract_ip_address(headers: &HeaderMap) -> Option<String> {
        // Check for forwarded headers first (proxy/load balancer)
        headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next()) // Get first IP in chain
            .map(|s| s.trim().to_string())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string())
            })
            // Note: In a real deployment, you'd also check the connection info
            // For now, we'll use a placeholder since we can't access connection info
            .or_else(|| Some("127.0.0.1".to_string()))
    }

    fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
        headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    }
}

/// Audit trail retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRetentionPolicy {
    /// Policy name/identifier
    pub name: String,
    /// Resource types this policy applies to (empty means all)
    pub resource_types: Vec<String>,
    /// Action types this policy applies to (empty means all)
    pub action_types: Vec<String>,
    /// How long to retain audit records (in days)
    pub retention_days: u32,
    /// Whether this is a legal hold (cannot be deleted)
    pub legal_hold: bool,
    /// Minimum retention period that cannot be overridden (in days)
    pub minimum_retention_days: u32,
    /// Policy priority (higher number = higher priority)
    pub priority: u32,
    /// Policy description
    pub description: String,
}

impl Default for AuditRetentionPolicy {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            resource_types: vec![],
            action_types: vec![],
            retention_days: 2555, // 7 years for compliance
            legal_hold: false,
            minimum_retention_days: 365, // 1 year minimum
            priority: 0,
            description: "Default retention policy".to_string(),
        }
    }
}

/// Audit trail retention manager
#[derive(Debug, Clone)]
pub struct AuditRetentionManager {
    policies: HashMap<String, AuditRetentionPolicy>,
    default_policy: AuditRetentionPolicy,
}

impl AuditRetentionManager {
    pub fn new() -> Self {
        let mut policies = HashMap::new();

        // Security events - longer retention
        policies.insert(
            "security".to_string(),
            AuditRetentionPolicy {
                name: "security".to_string(),
                resource_types: vec!["system".to_string()],
                action_types: vec!["security_event".to_string(), "auth_failure".to_string()],
                retention_days: 2555, // 7 years
                legal_hold: false,
                minimum_retention_days: 1095, // 3 years minimum
                priority: 100,
                description: "Security events require extended retention for compliance"
                    .to_string(),
            },
        );

        // Authentication events
        policies.insert(
            "auth".to_string(),
            AuditRetentionPolicy {
                name: "auth".to_string(),
                resource_types: vec!["auth".to_string()],
                action_types: vec![
                    "login".to_string(),
                    "logout".to_string(),
                    "token_refresh".to_string(),
                ],
                retention_days: 1095, // 3 years
                legal_hold: false,
                minimum_retention_days: 365, // 1 year minimum
                priority: 80,
                description: "Authentication events for security analysis".to_string(),
            },
        );

        // Data access events
        policies.insert(
            "data_access".to_string(),
            AuditRetentionPolicy {
                name: "data_access".to_string(),
                resource_types: vec!["learner".to_string(), "session".to_string()],
                action_types: vec![
                    "read".to_string(),
                    "export".to_string(),
                    "data_access".to_string(),
                ],
                retention_days: 1825, // 5 years
                legal_hold: false,
                minimum_retention_days: 730, // 2 years minimum
                priority: 60,
                description: "Data access events for privacy compliance".to_string(),
            },
        );

        // Administrative actions
        policies.insert(
            "admin".to_string(),
            AuditRetentionPolicy {
                name: "admin".to_string(),
                resource_types: vec!["admin".to_string()],
                action_types: vec![
                    "create".to_string(),
                    "update".to_string(),
                    "delete".to_string(),
                ],
                retention_days: 2555, // 7 years
                legal_hold: false,
                minimum_retention_days: 1095, // 3 years minimum
                priority: 90,
                description: "Administrative actions require extended retention".to_string(),
            },
        );

        Self {
            policies,
            default_policy: AuditRetentionPolicy::default(),
        }
    }

    /// Get the applicable retention policy for an audit record
    pub fn get_applicable_policy(
        &self,
        resource_type: &str,
        action: &str,
    ) -> &AuditRetentionPolicy {
        let mut best_match: Option<&AuditRetentionPolicy> = None;
        let mut best_priority = 0;

        for policy in self.policies.values() {
            let resource_match = policy.resource_types.is_empty()
                || policy.resource_types.contains(&resource_type.to_string());
            let action_match =
                policy.action_types.is_empty() || policy.action_types.contains(&action.to_string());

            if resource_match && action_match && policy.priority >= best_priority {
                best_match = Some(policy);
                best_priority = policy.priority;
            }
        }

        best_match.unwrap_or(&self.default_policy)
    }

    /// Add or update a retention policy
    pub fn add_policy(&mut self, policy: AuditRetentionPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Remove a retention policy
    pub fn remove_policy(&mut self, name: &str) -> Option<AuditRetentionPolicy> {
        self.policies.remove(name)
    }

    /// List all retention policies
    pub fn list_policies(&self) -> Vec<&AuditRetentionPolicy> {
        self.policies.values().collect()
    }
}

/// Audit cleanup statistics
#[derive(Debug, Serialize)]
pub struct AuditCleanupStats {
    pub total_records_examined: u64,
    pub records_eligible_for_deletion: u64,
    pub records_deleted: u64,
    pub records_on_legal_hold: u64,
    pub oldest_record_date: Option<DateTime<Utc>>,
    pub cleanup_duration_ms: u64,
    pub policies_applied: Vec<String>,
}

impl AuditService {
    pub async fn log_event(
        db: &DbPool,
        user_id: Option<Uuid>,
        action: String,
        resource_type: String,
        resource_id: String,
        changes: Option<serde_json::Value>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let audit_id = Uuid::new_v4();
        let audit_id_bytes = audit_id.as_bytes();
        let user_id_bytes = user_id.map(|id| id.as_bytes().to_vec());
        let changes_str = changes.map(|c| c.to_string());

        let mut conn = db.acquire().await?;
        sqlx::query(
            "INSERT INTO audit_log (id, user_id, action, resource_type, resource_id, changes, ip_address, user_agent) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&audit_id_bytes[..])
        .bind(user_id_bytes.as_deref())
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(changes_str)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn log_event_with_context(
        db: &DbPool,
        context: &AuditContext,
        action: String,
        resource_type: String,
        resource_id: String,
        changes: Option<serde_json::Value>,
    ) -> Result<()> {
        Self::log_event(
            db,
            context.user_id,
            action,
            resource_type,
            resource_id,
            changes,
            context.ip_address.clone(),
            context.user_agent.clone(),
        )
        .await
    }

    pub async fn log_security_event(
        db: &DbPool,
        context: &AuditContext,
        event_type: String,
        details: serde_json::Value,
    ) -> Result<()> {
        let enhanced_details = serde_json::json!({
            "event_type": event_type,
            "details": details,
            "security_event": true,
            "session_id": context.session_id,
            "role": context.role
        });

        Self::log_event_with_context(
            db,
            context,
            "security_event".to_string(),
            "system".to_string(),
            event_type,
            Some(enhanced_details),
        )
        .await
    }

    pub async fn log_data_access(
        db: &DbPool,
        context: &AuditContext,
        resource_type: String,
        resource_id: String,
        access_type: String, // "read", "export", "search"
        query_parameters: Option<serde_json::Value>,
    ) -> Result<()> {
        let access_details = serde_json::json!({
            "access_type": access_type,
            "query_parameters": query_parameters,
            "data_access": true
        });

        Self::log_event_with_context(
            db,
            context,
            "data_access".to_string(),
            resource_type,
            resource_id,
            Some(access_details),
        )
        .await
    }

    pub async fn get_audit_trail(
        db: &DbPool,
        resource_type: Option<String>,
        resource_id: Option<String>,
        user_id: Option<Uuid>,
        limit: Option<i64>,
    ) -> Result<Vec<serde_json::Value>> {
        let limit = limit.unwrap_or(100);
        let user_id_bytes = user_id.map(|id| id.as_bytes().to_vec());

        let mut conn = db.acquire().await?;
        let rows = if let (Some(rt), Some(rid)) = (resource_type.as_ref(), resource_id.as_ref()) {
            sqlx::query(
                "SELECT timestamp, action, resource_type, resource_id, changes, ip_address 
                 FROM audit_log 
                 WHERE resource_type = ? AND resource_id = ? 
                 ORDER BY timestamp DESC LIMIT ?",
            )
            .bind(rt)
            .bind(rid)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await?
        } else if let Some(uid_bytes) = user_id_bytes {
            sqlx::query(
                "SELECT timestamp, action, resource_type, resource_id, changes, ip_address 
                 FROM audit_log 
                 WHERE user_id = ? 
                 ORDER BY timestamp DESC LIMIT ?",
            )
            .bind(&uid_bytes[..])
            .bind(limit)
            .fetch_all(&mut *conn)
            .await?
        } else {
            sqlx::query(
                "SELECT timestamp, action, resource_type, resource_id, changes, ip_address 
                 FROM audit_log 
                 ORDER BY timestamp DESC LIMIT ?",
            )
            .bind(limit)
            .fetch_all(&mut *conn)
            .await?
        };

        let audit_records = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "timestamp": row.get::<chrono::DateTime<Utc>, _>("timestamp"),
                    "action": row.get::<String, _>("action"),
                    "resource_type": row.get::<String, _>("resource_type"),
                    "resource_id": row.get::<String, _>("resource_id"),
                    "changes": row.get::<Option<String>, _>("changes")
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
                    "ip_address": row.get::<Option<String>, _>("ip_address").unwrap_or_default()
                })
            })
            .collect();

        Ok(audit_records)
    }

    /// Apply audit trail retention policies and clean up old records
    pub async fn apply_retention_policies(
        db: &DbPool,
        retention_manager: &AuditRetentionManager,
        dry_run: bool,
    ) -> Result<AuditCleanupStats> {
        let start_time = std::time::Instant::now();
        let mut stats = AuditCleanupStats {
            total_records_examined: 0,
            records_eligible_for_deletion: 0,
            records_deleted: 0,
            records_on_legal_hold: 0,
            oldest_record_date: None,
            cleanup_duration_ms: 0,
            policies_applied: retention_manager
                .list_policies()
                .iter()
                .map(|p| p.name.clone())
                .collect(),
        };

        let mut conn = db.acquire().await?;

        // Get all audit records with their types for policy matching
        let rows = sqlx::query(
            "SELECT id, timestamp, action, resource_type, resource_id 
             FROM audit_log 
             ORDER BY timestamp ASC",
        )
        .fetch_all(&mut *conn)
        .await?;

        stats.total_records_examined = rows.len() as u64;

        let mut records_to_delete = Vec::new();
        let now = Utc::now();

        for row in rows {
            let timestamp: DateTime<Utc> = row.get("timestamp");
            let action: String = row.get("action");
            let resource_type: String = row.get("resource_type");
            let id_bytes: Vec<u8> = row.get("id");

            // Update oldest record date
            if stats.oldest_record_date.is_none() || timestamp < stats.oldest_record_date.unwrap() {
                stats.oldest_record_date = Some(timestamp);
            }

            // Get applicable retention policy
            let policy = retention_manager.get_applicable_policy(&resource_type, &action);

            // Check if record is on legal hold
            if policy.legal_hold {
                stats.records_on_legal_hold += 1;
                continue;
            }

            // Calculate record age in days
            let age_days = (now - timestamp).num_days() as u32;

            // Check if record exceeds retention period
            if age_days > policy.retention_days && age_days > policy.minimum_retention_days {
                stats.records_eligible_for_deletion += 1;
                if !dry_run {
                    records_to_delete.push(id_bytes);
                }
            }
        }

        // Perform actual deletion if not a dry run
        if !dry_run && !records_to_delete.is_empty() {
            for id_bytes in &records_to_delete {
                let result = sqlx::query("DELETE FROM audit_log WHERE id = ?")
                    .bind(id_bytes)
                    .execute(&mut *conn)
                    .await?;

                if result.rows_affected() > 0 {
                    stats.records_deleted += 1;
                }
            }

            // Log the cleanup operation
            Self::log_event(
                db,
                None, // System operation
                "audit_cleanup".to_string(),
                "system".to_string(),
                "audit_retention".to_string(),
                Some(serde_json::json!({
                    "records_deleted": stats.records_deleted,
                    "records_eligible": stats.records_eligible_for_deletion,
                    "dry_run": false,
                    "cleanup_timestamp": now
                })),
                None,
                Some("AuditRetentionService".to_string()),
            )
            .await?;
        } else if dry_run {
            // Log dry run operation
            Self::log_event(
                db,
                None,
                "audit_cleanup_dry_run".to_string(),
                "system".to_string(),
                "audit_retention".to_string(),
                Some(serde_json::json!({
                    "records_would_be_deleted": stats.records_eligible_for_deletion,
                    "total_examined": stats.total_records_examined,
                    "dry_run": true,
                    "cleanup_timestamp": now
                })),
                None,
                Some("AuditRetentionService".to_string()),
            )
            .await?;
        }

        stats.cleanup_duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(stats)
    }

    /// Get retention policy statistics
    pub async fn get_retention_statistics(
        db: &DbPool,
        retention_manager: &AuditRetentionManager,
    ) -> Result<serde_json::Value> {
        let mut conn = db.acquire().await?;

        // Get record counts by age ranges
        let total_records: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&mut *conn)
            .await?;

        let now = Utc::now();
        let one_year_ago = now - Duration::days(365);
        let three_years_ago = now - Duration::days(1095);
        let seven_years_ago = now - Duration::days(2555);

        let records_last_year: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE timestamp > ?")
                .bind(one_year_ago)
                .fetch_one(&mut *conn)
                .await?;

        let records_1_3_years: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE timestamp BETWEEN ? AND ?")
                .bind(three_years_ago)
                .bind(one_year_ago)
                .fetch_one(&mut *conn)
                .await?;

        let records_3_7_years: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE timestamp BETWEEN ? AND ?")
                .bind(seven_years_ago)
                .bind(three_years_ago)
                .fetch_one(&mut *conn)
                .await?;

        let records_over_7_years: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE timestamp < ?")
                .bind(seven_years_ago)
                .fetch_one(&mut *conn)
                .await?;

        // Get oldest and newest record timestamps
        let oldest_record: Option<DateTime<Utc>> =
            sqlx::query_scalar("SELECT MIN(timestamp) FROM audit_log")
                .fetch_one(&mut *conn)
                .await?;

        let newest_record: Option<DateTime<Utc>> =
            sqlx::query_scalar("SELECT MAX(timestamp) FROM audit_log")
                .fetch_one(&mut *conn)
                .await?;

        // Get record counts by resource type and action
        let type_breakdown = sqlx::query(
            "SELECT resource_type, action, COUNT(*) as count 
             FROM audit_log 
             GROUP BY resource_type, action 
             ORDER BY count DESC",
        )
        .fetch_all(&mut *conn)
        .await?;

        let mut type_stats = Vec::new();
        for row in type_breakdown {
            let resource_type: String = row.get("resource_type");
            let action: String = row.get("action");
            let count: i64 = row.get("count");

            let policy = retention_manager.get_applicable_policy(&resource_type, &action);

            type_stats.push(serde_json::json!({
                "resource_type": resource_type,
                "action": action,
                "record_count": count,
                "applicable_policy": policy.name,
                "retention_days": policy.retention_days,
                "legal_hold": policy.legal_hold
            }));
        }

        Ok(serde_json::json!({
            "total_records": total_records,
            "age_distribution": {
                "last_year": records_last_year,
                "1_to_3_years": records_1_3_years,
                "3_to_7_years": records_3_7_years,
                "over_7_years": records_over_7_years
            },
            "oldest_record": oldest_record,
            "newest_record": newest_record,
            "record_type_breakdown": type_stats,
            "active_policies": retention_manager
                .list_policies()
                .iter()
                .map(|p| serde_json::json!({
                    "name": p.name,
                    "retention_days": p.retention_days,
                    "priority": p.priority,
                    "legal_hold": p.legal_hold,
                    "description": p.description
                }))
                .collect::<Vec<_>>()
        }))
    }

    /// Create a compliance report before deletion
    pub async fn generate_compliance_report(
        db: &DbPool,
        retention_manager: &AuditRetentionManager,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<serde_json::Value> {
        let mut conn = db.acquire().await?;

        let from_date = from_date.unwrap_or(Utc::now() - Duration::days(365));
        let to_date = to_date.unwrap_or(Utc::now());

        // Get all records in the date range
        let records = sqlx::query(
            "SELECT action, resource_type, resource_id, timestamp, ip_address, user_agent
             FROM audit_log 
             WHERE timestamp BETWEEN ? AND ?
             ORDER BY timestamp DESC",
        )
        .bind(from_date)
        .bind(to_date)
        .fetch_all(&mut *conn)
        .await?;

        let mut report_sections = Vec::new();
        let mut total_actions = std::collections::HashMap::new();
        let mut ip_summary = std::collections::HashMap::new();

        for row in &records {
            let action: String = row.get("action");
            let resource_type: String = row.get("resource_type");
            let timestamp: DateTime<Utc> = row.get("timestamp");
            let ip_address: Option<String> = row.get("ip_address");

            // Count actions
            *total_actions
                .entry(format!("{}:{}", resource_type, action))
                .or_insert(0) += 1;

            // Count IPs
            if let Some(ip) = ip_address {
                *ip_summary.entry(ip).or_insert(0) += 1;
            }

            // Apply retention policy
            let policy = retention_manager.get_applicable_policy(&resource_type, &action);
            let age_days = (to_date - timestamp).num_days();

            if age_days > policy.retention_days as i64 {
                report_sections.push(serde_json::json!({
                    "timestamp": timestamp,
                    "action": action,
                    "resource_type": resource_type,
                    "age_days": age_days,
                    "policy_retention_days": policy.retention_days,
                    "eligible_for_deletion": !policy.legal_hold,
                    "policy_name": policy.name
                }));
            }
        }

        Ok(serde_json::json!({
            "report_generated_at": Utc::now(),
            "report_period": {
                "from": from_date,
                "to": to_date
            },
            "summary": {
                "total_records_reviewed": records.len(),
                "records_eligible_for_deletion": report_sections.len(),
                "action_breakdown": total_actions,
                "unique_ip_addresses": ip_summary.len(),
                    "top_ip_addresses": (|| {
                        let mut ip_vec: Vec<_> = ip_summary.clone().into_iter().collect();
                        ip_vec.sort_by(|a, b| b.1.cmp(&a.1));
                        ip_vec
                            .into_iter()
                            .take(10)
                            .collect::<std::collections::HashMap<_, _>>()
                    })()
            },
            "records_eligible_for_deletion": report_sections,
            "retention_policies_applied": retention_manager
                .list_policies()
                .iter()
                .map(|p| serde_json::json!({
                    "name": p.name,
                    "retention_days": p.retention_days,
                    "legal_hold": p.legal_hold,
                    "priority": p.priority
                }))
                .collect::<Vec<_>>()
        }))
    }
}
