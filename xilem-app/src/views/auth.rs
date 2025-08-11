// Authentication views using component system

use crate::state::{AppState, LoadingKey, Screen};
use crate::components::{Components, AppComponents, ComponentOutput};

pub fn auth_view(state: &mut AppState) -> ComponentOutput {
    let username = state.auth_username.clone();
    let password = state.auth_password.clone();
    
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
    
    Components::centered_container(
        350.0,
        form,
    )
}