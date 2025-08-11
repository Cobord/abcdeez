// Authentication module with Apple Sign In and guest mode support

pub mod apple_signin;
#[cfg(target_os = "ios")]
pub mod ios;

pub use apple_signin::*;
#[cfg(target_os = "ios")]
pub use ios::*;

use crate::state::AppState;

// OAuth authentication result
#[derive(Debug, Clone)]
pub struct OAuthResult {
    pub user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub is_private_email: bool,
}

// Authentication provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthProvider {
    Apple,
    GitHub,
    Guest,
}

// Authentication trait for different providers
pub trait AuthenticationProvider {
    fn authenticate(&self, state: &mut AppState);
    fn logout(&self, state: &mut AppState);
    fn is_authenticated(&self, state: &AppState) -> bool;
}