// Re-export the run_web function from abcdeez-app
// This module just forwards to the actual app implementation
pub use abcdeez_app::run_web;

// Optional: Re-export hot reload functions if needed
#[cfg(target_arch = "wasm32")]
pub use abcdeez_app::{app_init, app_on_before_swap, app_on_after_swap, AppHandle};