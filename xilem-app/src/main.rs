// Copyright 2024 ABCDEEZ
// Main entry point for the ABCDEEZ Xilem learning application

use anyhow::Result;
use winit::error::EventLoopError;
use winit::dpi::LogicalSize;
use xilem::{EventLoop, WindowOptions, Xilem};

mod app;
mod components;
mod models;
mod services;
mod state;
mod utils;
mod views;

use crate::app::app_logic;
use crate::state::AppState;

fn main() -> Result<(), EventLoopError> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting ABCDEEZ Learning App");

    // Create initial application state
    let initial_state = AppState::default();

    // Create the Xilem application
    let app = Xilem::new_simple(
        initial_state,
        app_logic,
        WindowOptions::new("ABCDEEZ Learning")
            .with_initial_inner_size(LogicalSize::new(1024.0, 768.0))
            .with_min_inner_size(LogicalSize::new(320.0, 568.0))
            .with_resizable(true),
    );

    // Run the application
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}