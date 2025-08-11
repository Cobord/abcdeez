// Copyright 2024 ABCDEEZ
// Main entry point for the ABCDEEZ Xilem learning application

#[cfg(feature = "xilem-native")]
fn main() {
    // If both native and web are set, prefer native.
    use abcdeez_app::platform::{CurrentPlatformRunner, PlatformRunner};
    use abcdeez_app::state::AppState;

    let app_state = AppState::new();
    CurrentPlatformRunner::run(app_state).expect("App exited with error");
}

#[cfg(all(not(feature = "xilem-native"), feature = "xilem-web"))]
fn main() {
    // Web entry point - this would typically be called from WASM
    use abcdeez_app::platform::{CurrentPlatformRunner, PlatformRunner};
    use abcdeez_app::state::AppState;

    let app_state = AppState::new();
    CurrentPlatformRunner::run(app_state).expect("App exited with error");
}

#[cfg(not(any(feature = "xilem-native", feature = "xilem-web")))]
compile_error!("Either 'xilem-native' or 'xilem-web' feature must be enabled");