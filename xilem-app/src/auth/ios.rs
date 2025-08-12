// iOS-specific authentication bridge for Apple Sign In
// This module handles the native iOS authentication flow

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use once_cell::sync::Lazy;

/// Result from iOS authentication (matches the C struct)
#[repr(C)]
pub struct IOSAuthResult {
    pub success: bool,
    pub user_id: *const c_char,
    pub email: *const c_char,
    pub given_name: *const c_char,
    pub family_name: *const c_char,
    pub is_private_email: bool,
    pub error_message: *const c_char,
}

// External C functions implemented in iOS native code
#[cfg(target_os = "ios")]
extern "C" {
    pub fn ios_apple_sign_in(callback: *const c_void) -> *mut IOSAuthResult;
    pub fn ios_apple_sign_in_async(callback: extern "C" fn(*mut IOSAuthResult));
    pub fn ios_free_auth_result(result: *mut IOSAuthResult);
    pub fn ios_is_apple_signin_available() -> bool;
    pub fn ios_get_credential_state(user_id: *const c_char) -> i32;
    pub fn ios_get_credential_state_async(user_id: *const c_char, callback: extern "C" fn(i32));
}

// Global callback registry for auth callbacks
#[cfg(target_os = "ios")]
static AUTH_CALLBACK_REGISTRY: Lazy<Mutex<Option<oneshot::Sender<Result<AppleSignInResult, String>>>>> = 
    Lazy::new(|| Mutex::new(None));

#[cfg(target_os = "ios")]
static CREDENTIAL_CALLBACK_REGISTRY: Lazy<Mutex<Option<oneshot::Sender<CredentialState>>>> = 
    Lazy::new(|| Mutex::new(None));

// Callback function that will be called from iOS
#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn auth_completion_callback(result_ptr: *mut IOSAuthResult) {
    tracing::info!("auth_completion_callback called from iOS, ptr={:?}", result_ptr);
    if result_ptr.is_null() {
        if let Some(tx) = AUTH_CALLBACK_REGISTRY.lock().unwrap().take() {
            let _ = tx.send(Err("Null result pointer from iOS".to_string()));
        }
        return;
    }
    
    tracing::info!("About to dereference result pointer");
    unsafe {
        let result = &*result_ptr;
        tracing::info!("Result: success={}, error_msg_ptr={:?}", result.success, result.error_message);
        
        if !result.success {
            let error_msg = if result.error_message.is_null() {
                "Unknown error".to_string()
            } else {
                CStr::from_ptr(result.error_message)
                    .to_string_lossy()
                    .to_string()
            };
            
            tracing::info!("Auth callback received error: {}", error_msg);
            
            if let Some(tx) = AUTH_CALLBACK_REGISTRY.lock().unwrap().take() {
                tracing::info!("Sending error through channel: {}", error_msg);
                let send_result = tx.send(Err(error_msg.clone()));
                tracing::info!("Channel send result: {:?}", send_result.is_ok());
            } else {
                tracing::warn!("No receiver waiting for auth callback!");
            }
            ios_free_auth_result(result_ptr);
            return;
        }
        
        // Extract user data from successful authentication
        let user_id = if result.user_id.is_null() {
            String::new()
        } else {
            CStr::from_ptr(result.user_id)
                .to_string_lossy()
                .to_string()
        };
        
        let email = if result.email.is_null() {
            None
        } else {
            Some(CStr::from_ptr(result.email)
                .to_string_lossy()
                .to_string())
        };
        
        let given_name = if result.given_name.is_null() {
            None
        } else {
            Some(CStr::from_ptr(result.given_name)
                .to_string_lossy()
                .to_string())
        };
        
        let family_name = if result.family_name.is_null() {
            None
        } else {
            Some(CStr::from_ptr(result.family_name)
                .to_string_lossy()
                .to_string())
        };
        
        let auth_result = AppleSignInResult {
            user_id,
            email,
            given_name,
            family_name,
            is_private_email: result.is_private_email,
        };
        
        if let Some(tx) = AUTH_CALLBACK_REGISTRY.lock().unwrap().take() {
            let _ = tx.send(Ok(auth_result));
        }
        
        ios_free_auth_result(result_ptr);
    }
}

// Callback for credential state check
#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn credential_state_callback(state: i32) {
    let credential_state = match state {
        0 => CredentialState::Revoked,
        1 => CredentialState::Authorized,
        2 => CredentialState::NotFound,
        3 => CredentialState::Transferred,
        _ => CredentialState::Unknown,
    };
    
    if let Some(tx) = CREDENTIAL_CALLBACK_REGISTRY.lock().unwrap().take() {
        let _ = tx.send(credential_state);
    }
}

/// Bridge between Rust and iOS authentication
#[derive(Debug, Clone, Copy)]
pub struct IOSAuthBridge;

// Implementation for iOS platform
#[cfg(target_os = "ios")]
impl IOSAuthBridge {
    pub fn new() -> Self {
        IOSAuthBridge
    }
    
    /// Initiate Apple Sign In using native iOS implementation (async version)
    pub async fn apple_sign_in_async(&self) -> Result<AppleSignInResult, String> {
        tracing::info!("apple_sign_in_async: Starting FROM IOSAUTH BRIDGE");
        // Create a channel for the callback result
        let (tx, rx) = oneshot::channel();
        
        // Store the sender in the global registry
        {
            let mut registry = AUTH_CALLBACK_REGISTRY.lock().unwrap();
            *registry = Some(tx);
        }
        
        // Call the async iOS function with our callback
        unsafe {
            tracing::info!("apple_sign_in_async: Calling iOS function");
            ios_apple_sign_in_async(auth_completion_callback);
        }
        
        // Wait for the callback to send the result
        tracing::info!("apple_sign_in_async: Waiting for callback result");
        match rx.await {
            Ok(result) => {
                tracing::info!("apple_sign_in_async: Received result from callback: {:?}", result.is_ok());
                result
            }
            Err(e) => {
                tracing::error!("apple_sign_in_async: Channel error: {:?}", e);
                Err("Authentication callback was dropped".to_string())
            }
        }
    }
    
    /// Initiate Apple Sign In using native iOS implementation (blocking version for compatibility)
    pub fn apple_sign_in(&self) -> Result<AppleSignInResult, String> {
        tracing::info!("IOSAuthBridge::apple_sign_in BLOCKING VERSION CALLED - THIS SHOULD NOT BE CALLED!");
        unsafe {
            let result_ptr = ios_apple_sign_in(std::ptr::null());
            if result_ptr.is_null() {
                return Err("Failed to initiate Apple Sign In".to_string());
            }
            
            let result = &*result_ptr;
            
            if !result.success {
                let error_msg = if result.error_message.is_null() {
                    "Unknown error".to_string()
                } else {
                    CStr::from_ptr(result.error_message)
                        .to_string_lossy()
                        .to_string()
                };
                
                ios_free_auth_result(result_ptr);
                return Err(error_msg);
            }
            
            // Extract user data from successful authentication
            let user_id = if result.user_id.is_null() {
                String::new()
            } else {
                CStr::from_ptr(result.user_id)
                    .to_string_lossy()
                    .to_string()
            };
            
            let email = if result.email.is_null() {
                None
            } else {
                Some(CStr::from_ptr(result.email)
                    .to_string_lossy()
                    .to_string())
            };
            
            let given_name = if result.given_name.is_null() {
                None
            } else {
                Some(CStr::from_ptr(result.given_name)
                    .to_string_lossy()
                    .to_string())
            };
            
            let family_name = if result.family_name.is_null() {
                None
            } else {
                Some(CStr::from_ptr(result.family_name)
                    .to_string_lossy()
                    .to_string())
            };
            
            let auth_result = AppleSignInResult {
                user_id,
                email,
                given_name,
                family_name,
                is_private_email: result.is_private_email,
            };
            
            ios_free_auth_result(result_ptr);
            Ok(auth_result)
        }
    }
    
    /// Check if Apple Sign In is available
    pub fn is_available(&self) -> bool {
        unsafe { ios_is_apple_signin_available() }
    }
    
    /// Get credential state for a user ID (async version)
    pub async fn get_credential_state_async(&self, user_id: &str) -> CredentialState {
        // Create a channel for the callback result
        let (tx, rx) = oneshot::channel();
        
        // Store the sender in the global registry
        {
            let mut registry = CREDENTIAL_CALLBACK_REGISTRY.lock().unwrap();
            *registry = Some(tx);
        }
        
        // Call the async iOS function with our callback
        let c_user_id = CString::new(user_id).unwrap_or_else(|_| CString::new("").unwrap());
        unsafe {
            ios_get_credential_state_async(c_user_id.as_ptr(), credential_state_callback);
        }
        
        // Wait for the callback to send the result
        rx.await.unwrap_or(CredentialState::Unknown)
    }
    
    /// Get credential state for a user ID (blocking version for compatibility)
    pub fn get_credential_state(&self, user_id: &str) -> CredentialState {
        let c_user_id = CString::new(user_id).unwrap_or_else(|_| CString::new("").unwrap());
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

/// Result from Apple Sign In
#[derive(Debug, Clone)]
pub struct AppleSignInResult {
    pub user_id: String,
    pub email: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub is_private_email: bool,
}

/// Apple ID credential states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialState {
    Revoked,
    Authorized,
    NotFound,
    Transferred,
    Unknown,
}

// Stub implementation for non-iOS platforms
#[cfg(not(target_os = "ios"))]
impl IOSAuthBridge {
    pub fn new() -> Self {
        IOSAuthBridge
    }
    
    pub async fn apple_sign_in_async(&self) -> Result<AppleSignInResult, String> {
        Err("Apple Sign In is only available on iOS".to_string())
    }
    
    pub fn apple_sign_in(&self) -> Result<AppleSignInResult, String> {
        Err("Apple Sign In is only available on iOS".to_string())
    }
    
    pub fn is_available(&self) -> bool {
        false
    }
    
    pub async fn get_credential_state_async(&self, _user_id: &str) -> CredentialState {
        CredentialState::Unknown
    }
    
    pub fn get_credential_state(&self, _user_id: &str) -> CredentialState {
        CredentialState::Unknown
    }
}