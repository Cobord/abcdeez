# main.rs - Application Entry Point

## Conceptual Overview
Main entry point for the ABCDEEZ Xilem-based learning application. Responsible for bootstrapping the entire UI framework and initializing the application lifecycle.

## Key Data Flows
- Creates initial AppState with default values
- Initializes tracing for debugging/logging
- Sets up Xilem application with window configuration
- Delegates main application logic to app::app_logic
- Handles the main event loop

## Main Responsibilities
- Application initialization and startup
- Window configuration (size, title, minimum dimensions)
- Tracing/logging setup
- Event loop management
- Application termination handling

## Dependencies on Other Components
- `app::app_logic` - Main UI logic routing
- `state::AppState` - Global application state management
- Xilem framework for UI runtime

## User-Facing Functionality
- Creates the main application window
- Sets up the learning application interface
- Handles application lifecycle (start/stop)