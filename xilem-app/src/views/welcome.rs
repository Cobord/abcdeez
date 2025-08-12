// Welcome Screen with authentication options and quick start features

use crate::auth::{standard_apple_signin_button, github_signin_button};
use crate::components::{AppComponents, Component, ComponentOutput, Components, AppColor, SpacerSize};
use crate::components::layout::{page_layout, two_column_layout, content_container};
use crate::components::cards::{feature_card};
use crate::components::feedback::{toast, ToastType};
use crate::state::{AppState, Screen};

pub fn welcome_screen(state: &mut AppState, window_width: f64) -> ComponentOutput {
    // Main title with easter egg
    let title = Components::simple_button(
        "🎓 ABCDEEZ Learning System",
        |state: &mut AppState| {
            state.try_crab_triple_click();
        }
    );
    
    let subtitle = Components::label(
        "An intelligent adaptive learning system that optimizes your learning path using graph-based cognitive models"
    );
    
    // Create auth section
    let auth_section = create_auth_section(state);
    
    // Create features section
    let features_section = create_features_section(state);
    
    // Layout based on screen size
    let main_content = if window_width > 900.0 {
        two_column_layout(
            auth_section,
            features_section,
            window_width
        )
    } else {
        Components::simple_flex_column(vec![
            auth_section,
            Components::spacer(SpacerSize::Large),
            features_section,
        ])
    };
    
    // Build complete layout
    let content = Components::simple_flex_column(vec![
        title,
        Components::spacer(SpacerSize::Small),
        subtitle,
        Components::spacer(SpacerSize::XLarge),
        main_content,
    ]);
    
    // Wrap in content container for proper margins
    content_container(content, window_width)
}

fn create_auth_section(state: &mut AppState) -> ComponentOutput {
    let mut items = vec![];
    
    // Login form
    items.push(Components::label("Sign In"));
    items.push(Components::spacer(SpacerSize::Small));
    
    items.push(Components::labeled_input(
        "Username or Email",
        state.username_input.clone(),
        |state: &mut AppState, value: String| {
            state.username_input = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Small));
    
    items.push(Components::labeled_input(
        "Password",
        state.password_input.clone(),
        |state: &mut AppState, value: String| {
            state.password_input = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Login button with loading state
    if state.login_request_in_flight {
        items.push(Components::loading_spinner(Some("Signing in...")));
    } else {
        items.push(Components::action_button(
            "Sign In",
            AppColor::Primary,
            |state: &mut AppState| {
                if !state.login_request_in_flight {
                    state.login();
                }
            }
        ));
    }
    
    items.push(Components::spacer(SpacerSize::Medium));
    items.push(Components::divider(crate::components::Orientation::Horizontal));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Social login options
    items.push(Components::label("Or continue with"));
    items.push(Components::spacer(SpacerSize::Small));
    
    // OAuth buttons
    items.push(standard_apple_signin_button::<Components>(state));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(github_signin_button::<Components>(state));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Sign up link
    items.push(Components::simple_flex_row(vec![
        Components::label("New to ABCDEEZ?"),
        Components::simple_button("Create Account", |state| {
            state.navigate(Screen::Signup);
        }),
    ]));
    
    Components::card(
        "Welcome Back",
        Components::simple_flex_column(items)
    )
}

fn create_features_section(state: &mut AppState) -> ComponentOutput {
    let mut items = vec![];
    
    // Quick actions
    items.push(Components::label("Quick Start"));
    items.push(Components::spacer(SpacerSize::Small));
    
    // Guest mode card
    let guest_card = feature_card(
        "🚀",
        "Try as Guest",
        "Start learning immediately without an account",
        "Start Now",
        |state: &mut AppState| {
            state.current_user = None;
            state.is_guest_mode = true;
            state.create_learner();
            state.navigate(Screen::DomainSelection);
        }
    );
    
    items.push(guest_card);
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Interactive tour card
    let tour_card = feature_card(
        "🎯",
        "Interactive Tour",
        "Take a guided tour of all features",
        "Start Tour",
        |state: &mut AppState| {
            state.demo_start();
        }
    );
    
    items.push(tour_card);
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Demo mode card
    let demo_card = feature_card(
        "📚",
        "Watch Demo",
        "See the system in action with sample data",
        "View Demo",
        |state: &mut AppState| {
            state.demo_start_training();
            state.navigate(Screen::Learning);
        }
    );
    
    items.push(demo_card);
    items.push(Components::spacer(SpacerSize::Large));
    
    // Key features list
    items.push(Components::label("Key Features"));
    items.push(Components::spacer(SpacerSize::Small));
    
    let features = vec![
        "✨ Adaptive learning powered by AI",
        "📊 Real-time progress tracking",
        "🎮 Gamification with achievements",
        "🏆 Global leaderboards",
        "📈 Advanced analytics",
        "🎯 Personalized learning paths",
    ];
    
    for feature in features {
        items.push(Components::label(feature));
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    Components::simple_flex_column(items)
}