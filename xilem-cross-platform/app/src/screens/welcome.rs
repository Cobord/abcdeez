use xilem::{
    view::{button, flex, label, prose, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::{components::*, models::*, AppData, Screen};

// Welcome/Login Screen
pub fn welcome_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Adaptive Learning System")
            .brush(Color::from_rgb8(0, 128, 255)),

        prose("An intelligent learning system that adapts to your knowledge and optimizes your learning path using graph-based cognitive models."),

        card("Login", flex((
            labeled_input(
                "Username:",
                data.username_input.clone(),
                std::sync::Arc::new(|data: &mut AppData, value: String| {
                    data.username_input = value;
                }),
            ),
            labeled_input(
                "Password:",
                data.password_input.clone(),
                std::sync::Arc::new(|data: &mut AppData, value: String| {
                    data.password_input = value;
                }),
            ),
            button(
                if data.login_request_in_flight {
                    "Logging in..."
                } else {
                    "Login"
                }
                .to_string(),
                |data: &mut AppData| {
                    if !data.login_request_in_flight {
                        data.login();
                    }
                },
            ),
        )).direction(Axis::Vertical)),

        card("Quick Start", flex((
            prose("Start learning immediately without creating an account")
                .alignment(TextAlignment::Middle),
            button("Start as Guest", |data: &mut AppData| {
                // Create a guest user and learner
                data.current_user = Some(User {
                    id: uuid::Uuid::new_v4().to_string(),
                    username: "Guest".to_string(),
                    email: "guest@example.com".to_string(),
                    password_hash: String::new(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                });
                data.create_learner();
                data.current_screen = Screen::DomainSelection;
            }),
            button("Quick Tour 📚", |data: &mut AppData| {
                // Start the interactive guided tour
                data.demo_start();
            }),
            button("Training Demo 🎯", |data: &mut AppData| {
                // Start the full training demo
                data.demo_start_training();
            }),
            button("Demo Showcase", |data: &mut AppData| {
                // Run a short automated demo training sequence and navigate to Dashboard
                data.demo_showcase();
                data.current_screen = Screen::Dashboard;
            }),
            // Always-available navigation to avoid dead ends
            button("Go to Dashboard", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
            }),
        )).direction(Axis::Vertical)),
    ))
    .direction(Axis::Vertical)
}
