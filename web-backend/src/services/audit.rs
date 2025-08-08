use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;

pub struct AuditService;

impl AuditService {
    pub async fn log_event(
        db: &PgPool,
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
        let user_id_bytes = user_id.map(|id| id.as_bytes());
        let changes_str = changes.map(|c| c.to_string());

        sqlx::query!(
            "INSERT INTO audit_log (id, user_id, action, resource_type, resource_id, changes, ip_address, user_agent) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            audit_id_bytes,
            user_id_bytes,
            action,
            resource_type,
            resource_id,
            changes_str,
            ip_address,
            user_agent
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_audit_trail(
        db: &PgPool,
        resource_type: Option<String>,
        resource_id: Option<String>,
        user_id: Option<Uuid>,
        limit: Option<i64>,
    ) -> Result<Vec<serde_json::Value>> {
        let limit = limit.unwrap_or(100);
        let user_id_bytes = user_id.map(|id| id.as_bytes());

        let rows = if let (Some(rt), Some(rid)) = (resource_type.as_ref(), resource_id.as_ref()) {
            sqlx::query!(
                "SELECT timestamp, action, resource_type, resource_id, changes, ip_address 
                 FROM audit_log 
                 WHERE resource_type = $1 AND resource_id = $2 
                 ORDER BY timestamp DESC LIMIT $3",
                rt,
                rid,
                limit
            )
            .fetch_all(db)
            .await?
        } else if let Some(uid_bytes) = user_id_bytes {
            sqlx::query!(
                "SELECT timestamp, action, resource_type, resource_id, changes, ip_address 
                 FROM audit_log 
                 WHERE user_id = $1 
                 ORDER BY timestamp DESC LIMIT $2",
                uid_bytes,
                limit
            )
            .fetch_all(db)
            .await?
        } else {
            sqlx::query!(
                "SELECT timestamp, action, resource_type, resource_id, changes, ip_address 
                 FROM audit_log 
                 ORDER BY timestamp DESC LIMIT $1",
                limit
            )
            .fetch_all(db)
            .await?
        };

        let audit_records = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "timestamp": row.timestamp,
                    "action": row.action,
                    "resource_type": row.resource_type,
                    "resource_id": row.resource_id,
                    "changes": row.changes.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
                    "ip_address": row.ip_address
                })
            })
            .collect();

        Ok(audit_records)
    }
}