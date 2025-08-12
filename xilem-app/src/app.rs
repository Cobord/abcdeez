// Main application logic and root view

use crate::state::{AppState, Screen};
use crate::views::{auth, dashboard, learning, settings, welcome, widget_gallery, progress, analytics, social};
use crate::components::{Components, AppComponents, ComponentOutput};
use crate::components::layout::adaptive_app_layout;

pub fn app_logic(state: &mut AppState) -> ComponentOutput {
    // Get window dimensions (with defaults for now)
    let window_width = state.window_width.unwrap_or(1200.0);
    let window_height = state.window_height.unwrap_or(800.0);
    // Process any pending authentication events
    state.process_auth_events();
    
    // Update easter egg manager
    state.easter_egg_manager.update(0.016); // ~60fps frame time
    
    // Check for demo controller updates
    // Note: Cannot execute demo actions here due to borrow checker
    // Demo actions should be triggered through UI callbacks instead
    
    // Route to the appropriate view based on current screen
    let content = match state.current_screen {
        Screen::Login | Screen::Signup => auth::auth_view(state, window_width),
        Screen::Welcome => welcome::welcome_screen(state, window_width),
        Screen::Dashboard => dashboard::dashboard_view(state, window_width),
        Screen::Learning | Screen::Training => learning::learning_view(state, window_width),
        Screen::Settings | Screen::Profile => settings::settings_view(state, window_width),
        Screen::WidgetGallery => widget_gallery::widget_gallery(window_width),
        Screen::Visualizations => {
            use crate::viz::visualization_dashboard;
            visualization_dashboard(state, window_width)
        },
        Screen::DomainSelection => dashboard::dashboard_view(state, window_width),
        Screen::Progress => progress::progress_view(state, window_width),
        Screen::Analytics => analytics::analytics_view(state, window_width),
        Screen::Leaderboard => social::leaderboard_view(state, window_width),
        Screen::Challenges => social::challenges_view(state, window_width),
        Screen::Friends => social::friends_view(state, window_width),
        _ => Components::loading_spinner(Some("Loading...")),
    };
    
    // Wrap content in responsive layout for authenticated users
    if state.user.is_some() && !matches!(state.current_screen, Screen::Login | Screen::Signup | Screen::Welcome) {
        adaptive_app_layout(state, content, window_width, window_height)
    } else {
        content
    }
}