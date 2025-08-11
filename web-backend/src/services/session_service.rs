use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use crate::{
    cache,
    error::{AppError, AppResult},
    state::AppState,
};
use rand::{Rng, thread_rng};
use sha2::{Sha256, Digest};

/// Session management service to prevent session fixation attacks
pub struct SessionService {
    state: Arc<AppState>,
}

impl SessionService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
    
    /// Generate a new secure session ID
    pub fn generate_session_id() -> String {
        // Use UUID v4 with additional entropy
        let uuid = Uuid::new_v4();
        let random_bytes: [u8; 16] = thread_rng().gen();
        
        // Combine UUID with random bytes and hash
        let mut hasher = Sha256::new();
        hasher.update(uuid.as_bytes());
        hasher.update(&random_bytes);
        hasher.update(Utc::now().timestamp().to_le_bytes());
        
        // Return hex-encoded hash
        format!("{:x}", hasher.finalize())
    }
    
    /// Invalidate an existing session
    pub async fn invalidate_session(&self, session_id: &str) -> AppResult<()> {
        let mut conn = self.state.cache_conn.clone();
        
        // Delete session data
        let session_key = format!("session:{}", session_id);
        cache::cmd("DEL")
            .arg(&session_key)
            .query_async::<()>(&mut conn)
            .await
            .ok();
        
        // Add to blacklist with TTL matching token expiration
        let blacklist_key = format!("blacklist:session:{}", session_id);
        cache::cmd("SETEX")
            .arg(&blacklist_key)
            .arg(self.state.config.jwt_expiration_hours * 3600)
            .arg("1")
            .query_async::<()>(&mut conn)
            .await
            .ok();
        
        Ok(())
    }
    
    /// Create a new session with regenerated ID (prevents session fixation)
    pub async fn create_session(
        &self,
        user_id: Uuid,
        old_session_id: Option<&str>,
    ) -> AppResult<String> {
        // Invalidate old session if exists
        if let Some(old_id) = old_session_id {
            self.invalidate_session(old_id).await?;
        }
        
        // Generate new session ID
        let new_session_id = Self::generate_session_id();
        
        // Store session data
        let mut conn = self.state.cache_conn.clone();
        let session_key = format!("session:{}", new_session_id);
        let session_data = serde_json::json!({
            "user_id": user_id.to_string(),
            "created_at": Utc::now().timestamp(),
            "last_active": Utc::now().timestamp(),
        });
        
        cache::cmd("SETEX")
            .arg(&session_key)
            .arg(self.state.config.jwt_expiration_hours * 3600)
            .arg(session_data.to_string())
            .query_async::<()>(&mut conn)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        
        Ok(new_session_id)
    }
    
    /// Verify session is valid and not blacklisted
    pub async fn verify_session(&self, session_id: &str) -> AppResult<bool> {
        let mut conn = self.state.cache_conn.clone();
        
        // Check if blacklisted
        let blacklist_key = format!("blacklist:session:{}", session_id);
        let is_blacklisted: bool = cache::cmd("EXISTS")
            .arg(&blacklist_key)
            .query_async::<String>(&mut conn)
            .await
            .ok()
            .map(|s| s == "1")
            .unwrap_or(false);
        
        if is_blacklisted {
            return Ok(false);
        }
        
        // Check if session exists and is valid
        let session_key = format!("session:{}", session_id);
        let session_exists: bool = cache::cmd("EXISTS")
            .arg(&session_key)
            .query_async::<String>(&mut conn)
            .await
            .ok()
            .map(|s| s == "1")
            .unwrap_or(false);
        
        Ok(session_exists)
    }
    
    /// Update session last active time
    pub async fn touch_session(&self, session_id: &str) -> AppResult<()> {
        let mut conn = self.state.cache_conn.clone();
        let session_key = format!("session:{}", session_id);
        
        // Get existing session
        if let Ok(session_str) = cache::cmd("GET")
            .arg(&session_key)
            .query_async::<String>(&mut conn)
            .await
        {
            if let Ok(mut session_data) = serde_json::from_str::<serde_json::Value>(&session_str) {
                // Update last active
                session_data["last_active"] = serde_json::json!(Utc::now().timestamp());
                
                // Reset TTL and update data
                cache::cmd("SETEX")
                    .arg(&session_key)
                    .arg(self.state.config.jwt_expiration_hours * 3600)
                    .arg(session_data.to_string())
                    .query_async::<()>(&mut conn)
                    .await
                    .ok();
            }
        }
        
        Ok(())
    }
}

/// OAuth CSRF protection service
pub struct OAuthCsrfService {
    state: Arc<AppState>,
}

impl OAuthCsrfService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
    
    /// Generate a secure CSRF token for OAuth flows
    pub fn generate_csrf_token() -> String {
        // Generate random bytes
        let random_bytes: [u8; 32] = thread_rng().gen();
        
        // Hash with timestamp for additional entropy
        let mut hasher = Sha256::new();
        hasher.update(&random_bytes);
        hasher.update(Utc::now().timestamp().to_le_bytes());
        
        // Return base64-encoded token
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(hasher.finalize())
    }
    
    /// Store CSRF token for verification
    pub async fn store_csrf_token(
        &self,
        token: &str,
        session_id: Option<&str>,
    ) -> AppResult<()> {
        let mut conn = self.state.cache_conn.clone();
        
        // Store with 10 minute TTL (OAuth flow should complete quickly)
        let csrf_key = format!("oauth_csrf:{}", token);
        let csrf_data = serde_json::json!({
            "session_id": session_id,
            "created_at": Utc::now().timestamp(),
        });
        
        cache::cmd("SETEX")
            .arg(&csrf_key)
            .arg(600) // 10 minutes
            .arg(csrf_data.to_string())
            .query_async::<()>(&mut conn)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        
        Ok(())
    }
    
    /// Verify CSRF token and consume it (one-time use)
    pub async fn verify_and_consume_csrf_token(
        &self,
        token: &str,
        session_id: Option<&str>,
    ) -> AppResult<bool> {
        let mut conn = self.state.cache_conn.clone();
        let csrf_key = format!("oauth_csrf:{}", token);
        
        // Get token data
        let token_data = match cache::cmd("GET")
            .arg(&csrf_key)
            .query_async::<String>(&mut conn)
            .await
        {
            Ok(data) => data,
            Err(_) => return Ok(false),
        };
        
        // Parse and verify
        let data: serde_json::Value = serde_json::from_str(&token_data)
            .map_err(|_| AppError::InternalServerError)?;
        
        // Verify session ID matches if provided
        if let Some(sid) = session_id {
            if data["session_id"].as_str() != Some(sid) {
                return Ok(false);
            }
        }
        
        // Verify not expired (additional check)
        let created_at = data["created_at"].as_i64().unwrap_or(0);
        let now = Utc::now().timestamp();
        if now - created_at > 600 {
            // Expired
            return Ok(false);
        }
        
        // Delete token (one-time use)
        cache::cmd("DEL")
            .arg(&csrf_key)
            .query_async::<()>(&mut conn)
            .await
            .ok();
        
        Ok(true)
    }
}

/// Helper for constant-time string comparison
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (byte_a, byte_b) in a.iter().zip(b.iter()) {
        result |= byte_a ^ byte_b;
    }
    
    result == 0
}

use base64::Engine;