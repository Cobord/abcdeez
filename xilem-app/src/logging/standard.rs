// Standard logging for non-Apple platforms
use tracing_subscriber::prelude::*;

/// Initialize standard logging (for Linux, Windows, etc.)
pub fn init_standard_logging() {
    // Use different configurations for debug and release builds
    #[cfg(debug_assertions)]
    {
        // Development: detailed logging with pretty formatting
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .pretty()
            .init();
    }
    
    #[cfg(not(debug_assertions))]
    {
        // Production: JSON logging for better parsing
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .with_file(false)
            .with_line_number(false)
            .json()
            .init();
    }
}