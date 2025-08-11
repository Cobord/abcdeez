// Main application logic and root view

use crate::state::{AppState, Screen};
use crate::views::{auth, dashboard, learning, settings, welcome, widget_gallery};
use crate::components::{Components, AppComponents, ComponentOutput};

pub fn app_logic(state: &mut AppState) -> ComponentOutput {
    // Update easter egg manager
    state.easter_egg_manager.update(0.016); // ~60fps frame time
    
    // Check for demo controller updates
    // Note: Cannot execute demo actions here due to borrow checker
    // Demo actions should be triggered through UI callbacks instead
    
    // Route to the appropriate view based on current screen
    match state.current_screen {
        Screen::Login | Screen::Signup => auth::auth_view(state),
        Screen::Welcome => welcome::welcome_screen(state),
        Screen::Dashboard => dashboard::dashboard_view(state),
        Screen::Learning | Screen::Training => learning::learning_view(state),
        Screen::Settings | Screen::Profile => settings::settings_view(state),
        Screen::WidgetGallery => widget_gallery::widget_gallery(),
        Screen::Visualizations => {
            use crate::viz::visualization_dashboard;
            visualization_dashboard(state)
        },
        Screen::DomainSelection => dashboard::dashboard_view(state),
        _ => Components::loading_spinner(Some("Loading...")),
    }
}