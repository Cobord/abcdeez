// Library module for xilem-app

pub mod app;
pub mod components;
pub mod models;
pub mod platform;
pub mod services;
pub mod state;
pub mod utils;
pub mod views;

// Keep the old run function for backward compatibility when using xilem-native
#[cfg(feature = "xilem-native")]
pub fn run(ev: xilem::EventLoopBuilder) -> Result<(), winit::error::EventLoopError> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting ABCDEEZ Learning App");

    // Create initial application state
    let initial_state = crate::state::AppState::default();

    // Create the Xilem application using the native platform wrapper
    let app = xilem::Xilem::new_simple(
        initial_state,
        |state| {
            use crate::components::Component;
            crate::app::app_logic(state).build()
        },
        xilem::WindowOptions::new("ABCDEEZ Learning")
            .with_initial_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .with_min_inner_size(winit::dpi::LogicalSize::new(320.0, 568.0))
            .with_resizable(true),
    );

    // Run the application
    app.run_in(ev)?;
    Ok(())
}

// Web entry point for WASM
#[cfg(all(feature = "xilem-web", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run_web() {
    use platform::{PlatformRunner, CurrentPlatformRunner};
    
    // Set panic hook for better error messages in browser
    console_error_panic_hook::set_once();
    
    // Initialize web logging
    tracing_wasm::set_as_global_default();
    
    let app_state = state::AppState::new();
    CurrentPlatformRunner::run(app_state).expect("Failed to start web app");
}

// Hot Module Replacement hooks for live editing
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct AppHandle {
    state_json: String,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[no_mangle]
pub fn app_init() -> AppHandle {
    web_sys::console::log_1(&"🚀 App initialized for hot reload".into());
    
    // Store initial state
    let app_state = state::AppState::new();
    let state_json = serde_json::to_string(&app_state).unwrap_or_default();
    
    AppHandle { state_json }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[no_mangle]
pub fn app_on_before_swap(handle: &AppHandle) -> JsValue {
    web_sys::console::log_1(&"📦 Preserving app state before hot reload".into());
    JsValue::from_str(&handle.state_json)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[no_mangle]
pub fn app_on_after_swap(state: JsValue) {
    if let Some(state_str) = state.as_string() {
        web_sys::console::log_1(&format!("📥 Restoring app state: {} bytes", state_str.len()).into());
        // In production: deserialize and restore the AppState to the running app
        // if let Ok(restored_state) = serde_json::from_str::<state::AppState>(&state_str) {
        //     // Apply restored state to the app
        // }
    }
}
