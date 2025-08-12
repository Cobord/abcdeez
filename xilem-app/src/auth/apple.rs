use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleAuthResult {
    pub user_id: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub identity_token: String,
    pub authorization_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleUserInfo {
    pub email: Option<String>,
    pub name: Option<AppleUserName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleUserName {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleSignInRequest {
    pub identity_token: String,
    pub authorization_code: Option<String>,
    pub user_info: Option<AppleUserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleSignInResponse {
    pub token: String,
    pub user: UserResponse,
    pub is_new_user: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

// FFI bindings for iOS
#[cfg(target_os = "ios")]
mod ios {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_void};

    #[repr(C)]
    pub struct IOSAuthResult {
        pub user_id: *const c_char,
        pub email: *const c_char,
        pub full_name: *const c_char,
        pub identity_token: *const c_char,
        pub authorization_code: *const c_char,
        pub error: *const c_char,
    }

    extern "C" {
        pub fn ios_apple_sign_in(callback: *const c_void) -> *mut IOSAuthResult;
        pub fn ios_free_auth_result(result: *mut IOSAuthResult);
        pub fn ios_get_credential_state(user_id: *const c_char) -> i32;
        pub fn ios_is_apple_signin_available() -> bool;
    }

    impl IOSAuthResult {
        pub fn to_rust_result(&self) -> Result<AppleAuthResult, String> {
            if !self.error.is_null() {
                let error = unsafe { CStr::from_ptr(self.error) }
                    .to_string_lossy()
                    .to_string();
                return Err(error);
            }

            let user_id = if !self.user_id.is_null() {
                unsafe { CStr::from_ptr(self.user_id) }
                    .to_string_lossy()
                    .to_string()
            } else {
                return Err("No user ID returned".to_string());
            };

            let email = if !self.email.is_null() {
                Some(
                    unsafe { CStr::from_ptr(self.email) }
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            };

            let full_name = if !self.full_name.is_null() {
                Some(
                    unsafe { CStr::from_ptr(self.full_name) }
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            };

            let identity_token = if !self.identity_token.is_null() {
                unsafe { CStr::from_ptr(self.identity_token) }
                    .to_string_lossy()
                    .to_string()
            } else {
                return Err("No identity token returned".to_string());
            };

            let authorization_code = if !self.authorization_code.is_null() {
                unsafe { CStr::from_ptr(self.authorization_code) }
                    .to_string_lossy()
                    .to_string()
            } else {
                return Err("No authorization code returned".to_string());
            };

            Ok(AppleAuthResult {
                user_id,
                email,
                full_name,
                identity_token,
                authorization_code,
            })
        }
    }

    pub async fn sign_in_with_apple() -> Result<AppleAuthResult, String> {
        info!("Starting Apple Sign-In on iOS");

        // Check if Apple Sign-In is available
        if !unsafe { ios_is_apple_signin_available() } {
            warn!("Apple Sign-In is not available on this device");
            return Err("Apple Sign-In is not available".to_string());
        }

        // Call native iOS Apple Sign-In
        let result_ptr = unsafe { ios_apple_sign_in(std::ptr::null()) };
        
        if result_ptr.is_null() {
            error!("Apple Sign-In returned null pointer");
            return Err("Apple Sign-In failed".to_string());
        }

        let result = unsafe { &*result_ptr };
        let rust_result = result.to_rust_result();
        
        // Free the native memory
        unsafe { ios_free_auth_result(result_ptr) };
        
        match &rust_result {
            Ok(auth) => {
                info!(
                    user_id = %auth.user_id,
                    has_email = auth.email.is_some(),
                    "Apple Sign-In successful"
                );
            }
            Err(e) => {
                error!(error = %e, "Apple Sign-In failed");
            }
        }
        
        rust_result
    }

    pub fn check_credential_state(user_id: &str) -> CredentialState {
        let c_user_id = CString::new(user_id).unwrap();
        let state = unsafe { ios_get_credential_state(c_user_id.as_ptr()) };
        
        match state {
            0 => CredentialState::Revoked,
            1 => CredentialState::Authorized,
            2 => CredentialState::NotFound,
            3 => CredentialState::Transferred,
            _ => CredentialState::Unknown,
        }
    }
}

// Mock implementation for non-iOS platforms
#[cfg(not(target_os = "ios"))]
mod mock {
    use super::*;

    pub async fn sign_in_with_apple() -> Result<AppleAuthResult, String> {
        warn!("Using mock Apple Sign-In (not on iOS)");
        
        // Return mock data for testing
        Ok(AppleAuthResult {
            user_id: "mock_user_123".to_string(),
            email: Some("mock@example.com".to_string()),
            full_name: Some("Mock User".to_string()),
            identity_token: "mock_identity_token".to_string(),
            authorization_code: "mock_auth_code".to_string(),
        })
    }

    pub fn check_credential_state(_user_id: &str) -> CredentialState {
        CredentialState::Authorized
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialState {
    Revoked,
    Authorized,
    NotFound,
    Transferred,
    Unknown,
}

// Public API
pub async fn sign_in_with_apple() -> Result<AppleAuthResult, String> {
    debug!("Initiating Apple Sign-In");
    
    #[cfg(target_os = "ios")]
    let result = ios::sign_in_with_apple().await;
    
    #[cfg(not(target_os = "ios"))]
    let result = mock::sign_in_with_apple().await;
    
    result
}

pub fn check_credential_state(user_id: &str) -> CredentialState {
    debug!(user_id = %user_id, "Checking Apple credential state");
    
    #[cfg(target_os = "ios")]
    let state = ios::check_credential_state(user_id);
    
    #[cfg(not(target_os = "ios"))]
    let state = mock::check_credential_state(user_id);
    
    info!(user_id = %user_id, state = ?state, "Credential state checked");
    state
}

pub fn is_apple_signin_available() -> bool {
    #[cfg(target_os = "ios")]
    {
        unsafe { ios::ios_is_apple_signin_available() }
    }
    
    #[cfg(not(target_os = "ios"))]
    {
        true // Always available in mock mode
    }
}

// Send Apple auth result to backend
#[cfg(feature = "tokio-runtime")]
pub async fn authenticate_with_backend(
    auth_result: AppleAuthResult,
    api_base_url: &str,
) -> Result<AppleSignInResponse, String> {
    info!("Sending Apple auth to backend");
    
    // Parse full name into first and last
    let user_info = if auth_result.email.is_some() || auth_result.full_name.is_some() {
        let name = auth_result.full_name.as_ref().and_then(|full_name| {
            let parts: Vec<&str> = full_name.split_whitespace().collect();
            if parts.is_empty() {
                None
            } else if parts.len() == 1 {
                Some(AppleUserName {
                    first_name: Some(parts[0].to_string()),
                    last_name: None,
                })
            } else {
                Some(AppleUserName {
                    first_name: Some(parts[0].to_string()),
                    last_name: Some(parts[1..].join(" ")),
                })
            }
        });
        
        Some(AppleUserInfo {
            email: auth_result.email.clone(),
            name,
        })
    } else {
        None
    };
    
    let request = AppleSignInRequest {
        identity_token: auth_result.identity_token,
        authorization_code: Some(auth_result.authorization_code),
        user_info,
    };
    
    // Send to backend
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/apple-signin", api_base_url))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    let status = response.status();
    if status.is_success() {
        let signin_response = response
            .json::<AppleSignInResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        info!(
            user_id = %signin_response.user.id,
            is_new_user = signin_response.is_new_user,
            "Backend authentication successful"
        );
        
        Ok(signin_response)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!(status = ?status, error = %error_text, "Backend authentication failed");
        Err(format!("Authentication failed: {}", error_text))
    }
}

// Stub for web builds where reqwest is not available
#[cfg(not(feature = "tokio-runtime"))]
pub async fn authenticate_with_backend(
    _auth_result: AppleAuthResult,
    _api_base_url: &str,
) -> Result<AppleSignInResponse, String> {
    Err("Apple Sign-In backend authentication not available in web builds".to_string())
}