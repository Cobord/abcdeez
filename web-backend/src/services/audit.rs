use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;
use anyhow::Result;
use crate::db::DbPool;

pub struct AuditService;

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
        .bind(audit_id_bytes)
        .bind(user_id_bytes)
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
                 ORDER BY timestamp DESC LIMIT ?"
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
                 ORDER BY timestamp DESC LIMIT ?"
            )
            .bind(uid_bytes)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await?
        } else {
            sqlx::query(
                "SELECT timestamp, action, resource_type, resource_id, changes, ip_address 
                 FROM audit_log 
                 ORDER BY timestamp DESC LIMIT ?"
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
}