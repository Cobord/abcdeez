// Logging infrastructure for ABCDEEZ

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod apple;

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub mod standard;

use tracing_subscriber::prelude::*;

/// Initialize the logging system for the current platform
pub fn init_logging() {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    {
        apple::init_apple_logging();
    }
    
    #[cfg(not(any(target_os = "ios", target_os = "macos")))]
    {
        standard::init_standard_logging();
    }
    
    tracing::info!("🦀 ABCDEEZ logging system initialized");
}