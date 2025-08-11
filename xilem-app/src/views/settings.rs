// Settings view using high-level cross-platform components

use crate::state::{AppState, Screen, ThemeMode};
use crate::components::{Components, AppComponents, AppTheme, ComponentOutput};

pub fn settings_view(state: &mut AppState) -> ComponentOutput {
    let header = Components::header_bar("Settings", |state| {
        state.navigate_back();
    });
    
    let profile = profile_section(state);
    let preferences = preferences_section(state);
    let account = account_section(state);
    
    let content = Components::settings_section(
        "",
        vec![profile, preferences, account]
    );
    
    Components::app_scaffold(header, content, None)
}

fn profile_section(state: &AppState) -> ComponentOutput {
    let username = state.user.as_ref()
        .map(|u| u.username.as_str())
        .unwrap_or("Guest User");
    
    let avatar = Components::empty_state("👤", username, "", None);
    
    let edit_button = Components::nav_button(
        "Edit Profile",
        crate::components::AppScreen::Profile,
        false,
        |state| {
            state.navigate(Screen::Profile);
        }
    );
    
    Components::settings_section(
        "Profile",
        vec![avatar, edit_button]
    )
}

fn preferences_section(state: &AppState) -> ComponentOutput {
    let current_theme = map_theme_mode_to_app_theme(state.theme_mode);
    
    let theme_row = Components::setting_row(
        "Theme",
        Components::theme_selector(current_theme, |state, theme| {
            state.theme_mode = map_app_theme_to_theme_mode(theme);
            state.theme.mode = state.theme_mode;
        })
    );
    
    // For notifications and sound, we'd need checkbox components
    // For now, use placeholder buttons
    let notifications_row = Components::setting_row(
        "Notifications",
        Components::nav_button("Enable", crate::components::AppScreen::Settings, false, |_state| {
            // TODO: Update notification preference
        })
    );
    
    let sound_row = Components::setting_row(
        "Sound Effects",
        Components::nav_button("Enable", crate::components::AppScreen::Settings, true, |_state| {
            // TODO: Update sound preference
        })
    );
    
    Components::settings_section(
        "Preferences",
        vec![theme_row, notifications_row, sound_row]
    )
}

fn account_section(_state: &AppState) -> ComponentOutput {
    let export_button = Components::nav_button(
        "Export Data",
        crate::components::AppScreen::Settings,
        false,
        |_state| {
            // TODO: Export user data
        }
    );
    
    let privacy_button = Components::nav_button(
        "Privacy Settings",
        crate::components::AppScreen::Settings,
        false,
        |_state| {
            // TODO: Navigate to privacy settings
        }
    );
    
    let signout_button = Components::nav_button(
        "Sign Out",
        crate::components::AppScreen::Login,
        false,
        |state| {
            state.user = None;
            state.auth_token = None;
            state.navigate(Screen::Login);
        }
    );
    
    let delete_button = Components::error_banner(
        "Delete Account",
        |_state| {
            // TODO: Confirm and delete account
        }
    );
    
    Components::settings_section(
        "Account",
        vec![export_button, privacy_button, signout_button, delete_button]
    )
}

fn map_theme_mode_to_app_theme(mode: ThemeMode) -> AppTheme {
    match mode {
        ThemeMode::Light => AppTheme::Light,
        ThemeMode::Dark => AppTheme::Dark,
        ThemeMode::Auto => AppTheme::Auto,
    }
}

fn map_app_theme_to_theme_mode(theme: AppTheme) -> ThemeMode {
    match theme {
        AppTheme::Light => ThemeMode::Light,
        AppTheme::Dark => ThemeMode::Dark,
        AppTheme::Auto => ThemeMode::Auto,
    }
}