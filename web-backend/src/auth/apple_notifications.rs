// Apple Sign in with Apple Server-to-Server Notifications Handler
// 
// This module handles important notifications from Apple about user account changes:
// - Email/mail forwarding preference changes
// - App account deletion
// - Apple Account permanent deletion
//
// Documentation: https://developer.apple.com/documentation/sign_in_with_apple/processing_changes_for_sign_in_with_apple_accounts

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Apple Server-to-Server notification payload
#[derive(Debug, Deserialize, Serialize)]
pub struct AppleNotification {
    /// JWT containing the actual notification
    #[serde(rename = "signedPayload")]
    signed_payload: String,
}

/// Decoded JWT payload from Apple
#[derive(Debug, Deserialize, Serialize)]
pub struct AppleNotificationPayload {
    /// Issuer (Apple)
    iss: String,
    /// Audience (your app's bundle ID)
    aud: String,
    /// Issued at timestamp
    iat: i64,
    /// JWT ID
    jti: String,
    /// Event data
    events: serde_json::Value,
}

/// Event types from Apple
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleEvent {
    #[serde(rename = "type")]
    event_type: AppleEventType,
    sub: String,  // User identifier
    event_time: i64,
    email: Option<String>,
    is_private_email: Option<String>,
}

/// Types of events Apple sends
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AppleEventType {
    /// User disabled email forwarding
    EmailDisabled,
    /// User enabled email forwarding  
    EmailEnabled,
    /// User stopped using Apple Sign In with your app
    ConsentRevoked,
    /// User deleted their Apple account
    AccountDelete,
}

/// Configuration for Apple notifications
pub struct AppleNotificationConfig {
    /// Apple's public keys for JWT verification
    /// These should be fetched from: https://appleid.apple.com/auth/keys
    pub apple_public_keys: Vec<DecodingKey>,
    /// Your app's bundle identifier
    pub bundle_id: String,
}

/// Handler for Apple server-to-server notifications
/// 
/// This endpoint should be configured in your Apple Developer account
/// URL must be HTTPS with TLS 1.2 or higher
pub async fn handle_apple_notification(
    State(config): State<Arc<AppleNotificationConfig>>,
    Json(notification): Json<AppleNotification>,
) -> impl IntoResponse {
    // Decode the JWT header to get the key ID
    let header = match decode_header(&notification.signed_payload) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to decode JWT header: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // Get the key ID from the header
    let kid = match header.kid {
        Some(k) => k,
        None => {
            eprintln!("No key ID in JWT header");
            return StatusCode::BAD_REQUEST;
        }
    };

    // Verify and decode the JWT
    // In production, you'd match the kid with the appropriate public key
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_audience(&[&config.bundle_id]);
    validation.set_issuer(&["https://appleid.apple.com"]);

    // Try each public key until one works
    let mut decoded_token = None;
    for key in &config.apple_public_keys {
        match decode::<AppleNotificationPayload>(
            &notification.signed_payload,
            key,
            &validation,
        ) {
            Ok(token) => {
                decoded_token = Some(token);
                break;
            }
            Err(_) => continue,
        }
    }

    let token = match decoded_token {
        Some(t) => t,
        None => {
            eprintln!("Failed to verify JWT signature");
            return StatusCode::UNAUTHORIZED;
        }
    };

    // Parse the events
    let events: Vec<AppleEvent> = match serde_json::from_value(token.claims.events.clone()) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to parse events: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // Process each event
    for event in events {
        match event.event_type {
            AppleEventType::EmailDisabled => {
                // User disabled email forwarding
                // Update your database to mark this user's email as unreachable
                handle_email_disabled(&event.sub, &event.email).await;
            }
            AppleEventType::EmailEnabled => {
                // User re-enabled email forwarding
                // Update your database to mark this user's email as reachable
                handle_email_enabled(&event.sub, &event.email).await;
            }
            AppleEventType::ConsentRevoked => {
                // User revoked consent for your app
                // You should stop using their Apple Sign In data
                handle_consent_revoked(&event.sub).await;
            }
            AppleEventType::AccountDelete => {
                // User deleted their Apple account
                // You must delete all their data within 30 days
                handle_account_deletion(&event.sub).await;
            }
        }
    }

    StatusCode::OK
}

/// Handle email disabled event
async fn handle_email_disabled(user_id: &str, email: &Option<String>) {
    println!("User {} disabled email forwarding for {:?}", user_id, email);
    // TODO: Update your database
    // - Mark email as unreachable
    // - Stop sending emails to this address
}

/// Handle email enabled event
async fn handle_email_enabled(user_id: &str, email: &Option<String>) {
    println!("User {} enabled email forwarding for {:?}", user_id, email);
    // TODO: Update your database
    // - Mark email as reachable again
    // - Can resume sending emails
}

/// Handle consent revoked event
async fn handle_consent_revoked(user_id: &str) {
    println!("User {} revoked consent for Sign in with Apple", user_id);
    // TODO: Update your database
    // - Mark user as having revoked Apple Sign In
    // - Stop using their Apple-provided data
    // - Offer alternative sign-in methods
}

/// Handle account deletion event
async fn handle_account_deletion(user_id: &str) {
    println!("User {} deleted their Apple account", user_id);
    // TODO: CRITICAL - Must comply with privacy regulations
    // - Schedule user data deletion (must complete within 30 days)
    // - Remove all personal information
    // - Log the deletion for compliance
    // - Send confirmation if you have alternative contact method
}

/// Fetch Apple's public keys for JWT verification
/// These should be cached and refreshed periodically
pub async fn fetch_apple_public_keys() -> Result<Vec<DecodingKey>, Box<dyn std::error::Error>> {
    // In production, fetch from: https://appleid.apple.com/auth/keys
    // These keys should be cached and refreshed periodically (e.g., daily)
    
    // This is a placeholder - implement actual key fetching
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_deserialization() {
        let json = r#"{"type": "email-disabled"}"#;
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(parsed["type"], "email-disabled");
    }

    #[test]
    fn test_notification_structure() {
        let json = r#"{"signedPayload": "eyJ..."}"#;
        let notification: AppleNotification = serde_json::from_str(json).unwrap();
        assert!(notification.signed_payload.starts_with("eyJ"));
    }
}