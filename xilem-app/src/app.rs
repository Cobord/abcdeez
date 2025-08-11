// Main application logic and root view

use crate::state::{AppState, Screen};
use crate::views::{auth, dashboard, learning, settings};
use crate::components::{Components, AppComponents, ComponentOutput};

pub fn app_logic(state: &mut AppState) -> ComponentOutput {
    // Route to the appropriate view based on current screen
    match state.current_screen {
        Screen::Login | Screen::Signup => auth::auth_view(state),
        Screen::Dashboard => dashboard::dashboard_view(state),
        Screen::Learning => learning::learning_view(state),
        Screen::Settings | Screen::Profile => settings::settings_view(state),
        _ => Components::loading_spinner(Some("Loading...")),
    }
}