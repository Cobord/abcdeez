// Dashboard view using high-level cross-platform components

use crate::state::{AppState, Screen};
use crate::components::{Components, AppComponents, AppColor, AppScreen, ComponentOutput};

pub fn dashboard_view(state: &mut AppState) -> ComponentOutput {
    // Build the dashboard using high-level components
    let username = state.user.as_ref()
        .map(|u| u.username.as_str())
        .unwrap_or("Guest");
    
    let header = Components::header_bar(
        "ABCDEEZ",
        |state| {
            state.navigate(Screen::Settings);
        }
    );
    
    let welcome = Components::welcome_card(
        username,
        "Ready to continue learning?",
        |state| {
            state.navigate(Screen::Learning);
        }
    );
    
    let stats = Components::stats_grid(vec![
        Components::stat_card("Sessions", "12", AppColor::Primary),
        Components::stat_card("Accuracy", "85%", AppColor::Success),
        Components::stat_card("Streak", "5 days", AppColor::Warning),
    ]);
    
    let quick_actions = Components::settings_section(
        "Quick Actions",
        vec![
            // In a real implementation, we'd have action buttons here
        ]
    );
    
    let recent_activity = Components::settings_section(
        "Recent Activity",
        vec![
            Components::activity_card("Completed Alphabet Session", "2 hours ago", true),
            Components::activity_card("New Achievement: Quick Learner", "5 hours ago", false),
            Components::activity_card("Joined Weekly Challenge", "1 day ago", false),
        ]
    );
    
    let bottom_nav = Components::bottom_nav_bar(
        AppScreen::Dashboard,
        |state, screen| {
            state.navigate(map_app_screen_to_screen(screen));
        }
    );
    
    // Combine everything into the app scaffold
    let content = Components::settings_section(
        "",
        vec![welcome, stats, quick_actions, recent_activity]
    );
    
    Components::app_scaffold(
        header,
        content,
        Some(bottom_nav)
    )
}

fn map_app_screen_to_screen(app_screen: AppScreen) -> Screen {
    match app_screen {
        AppScreen::Login => Screen::Login,
        AppScreen::Signup => Screen::Signup,
        AppScreen::Dashboard => Screen::Dashboard,
        AppScreen::Learning => Screen::Learning,
        AppScreen::Progress => Screen::Progress,
        AppScreen::Settings => Screen::Settings,
        AppScreen::Profile => Screen::Profile,
        AppScreen::Leaderboard => Screen::Leaderboard,
    }
}