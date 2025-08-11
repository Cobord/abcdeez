// Library module for xilem-app

pub mod app;
pub mod components;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;
pub mod views;

pub fn run() -> Result<(), winit::error::EventLoopError> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting ABCDEEZ Learning App");

    // Create initial application state
    let initial_state = crate::state::AppState::default();

    // Create the Xilem application
    let app = xilem::Xilem::new_simple(
        initial_state,
        crate::app::app_logic,
        xilem::WindowOptions::new("ABCDEEZ Learning")
            .with_initial_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .with_min_inner_size(winit::dpi::LogicalSize::new(320.0, 568.0))
            .with_resizable(true),
    );

    // Run the application
    app.run_in(winit::event_loop::EventLoop::with_user_event())?;
    Ok(())
}