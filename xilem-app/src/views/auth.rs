// Authentication views using component system

use crate::state::{AppState, LoadingKey, Screen};
use crate::components::{Components, AppComponents, AppColor, ComponentOutput};
use crate::auth::{sign_in_with_apple, AppleSignInResponse};
use tracing::{debug, error, info};

pub fn auth_view(state: &mut AppState) -> ComponentOutput {
    let username = state.auth_username.clone();
    let password = state.auth_password.clone();
    
    // Traditional login form
    let form = Components::auth_form(
        username,
        password,
        |state: &mut AppState, text| {
            state.auth_username = text;
        },
        |state: &mut AppState, text| {
            state.auth_password = text;
        },
        |state: &mut AppState| {
            state.set_loading(LoadingKey::Login, true);
            // TODO: Call API client for actual login
            // For now, simulate successful login
            state.user = Some(crate::state::UserState {
                id: uuid::Uuid::new_v4(),
                username: state.auth_username.clone(),
                email: "".to_string(),
                display_name: Some(state.auth_username.clone()),
                created_at: chrono::Utc::now(),
                learner_id: Some(uuid::Uuid::new_v4()),
                level: 1,
                xp: 0,
                streak: 0,
                achievements: Vec::new(),
            });
            state.auth_password.clear(); // Clear sensitive data
            state.set_loading(LoadingKey::Login, false);
            state.navigate(Screen::Dashboard);
        },
    );
    
    // Apple Sign-In button
    let apple_signin_button = Components::action_button(
        "🍎 Sign in with Apple",
        AppColor::Primary,
        |state: &mut AppState| {
            debug!("Apple Sign-In button clicked");
            // Call the real Apple Sign In implementation
            state.apple_sign_in();
        }
    );
    
    // Guest mode button
    let guest_button = Components::action_button(
        "Continue as Guest",
        AppColor::Secondary,
        |state: &mut AppState| {
            info!("Guest mode selected");
            state.user = Some(crate::state::UserState {
                id: uuid::Uuid::new_v4(),
                username: "Guest".to_string(),
                email: "".to_string(),
                display_name: Some("Guest User".to_string()),
                created_at: chrono::Utc::now(),
                learner_id: Some(uuid::Uuid::new_v4()),
                level: 1,
                xp: 0,
                streak: 0,
                achievements: Vec::new(),
            });
            state.navigate(Screen::Dashboard);
        }
    );
    
    // Combine all elements
    let auth_content = Components::settings_section(
        "Sign In",
        vec![
            apple_signin_button,
            form,
            guest_button,
        ]
    );
    
    // Center within a responsive max-width; allow full-width on small screens
    Components::centered_container(700.0, auth_content)
}