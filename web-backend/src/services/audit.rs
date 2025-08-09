use crate::{db::DbPool, middleware::Claims};
use anyhow::Result;
use axum::{extract::Request, http::HeaderMap};
use chrono::Utc;
use sqlx::Row;
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
}
