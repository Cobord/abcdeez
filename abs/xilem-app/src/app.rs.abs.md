# app.rs - Main Application Logic Router

## Conceptual Overview
Central routing logic that determines which UI screen to display based on application state. Acts as the main dispatcher for different application views and handles loading states.

## Key Data Flows
- Receives AppState mutations from user interactions
- Routes to appropriate view modules based on current_screen
- Applies global theming to the root container
- Manages loading screen display

## Main Responsibilities
- Screen navigation and routing logic
- Global theme application
- Loading state management
- Root container styling and layout

## Dependencies on Other Components
- `state::AppState` - Global state for screen routing
- `views::*` - All view modules for different screens
- Theme system for consistent styling

## User-Facing Functionality
- Seamless navigation between different app screens
- Consistent theming across all views
- Loading indicators during state transitions