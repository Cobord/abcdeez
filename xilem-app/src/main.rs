// Copyright 2024 ABCDEEZ
// Main entry point for the ABCDEEZ Xilem learning application

#[cfg(feature = "xilem-native")]
fn main() {
    use abcdeez_app::platform::{CurrentPlatformRunner, PlatformRunner};
    use abcdeez_app::state::AppState;
    
    let app_state = AppState::new();
    CurrentPlatformRunner::run(app_state).expect("App exited with error");
}

#[cfg(feature = "xilem-web")]
fn main() {
    // Web entry point - this would typically be called from WASM
    use abcdeez_app::platform::{CurrentPlatformRunner, PlatformRunner};
    use abcdeez_app::state::AppState;
    
    let app_state = AppState::new();
    CurrentPlatformRunner::run(app_state).expect("App exited with error");
}

#[cfg(not(any(feature = "xilem-native", feature = "xilem-web")))]
fn main() {
    compile_error!("Either 'xilem-native' or 'xilem-web' feature must be enabled");
}