// Welcome Screen with authentication options and quick start features

use crate::auth::{standard_apple_signin_button, github_signin_button};
use crate::components::{AppComponents, Component, ComponentOutput, Components};
use crate::state::{AppState, Screen};

pub fn welcome_screen(state: &mut AppState) -> ComponentOutput {
    // Check for triple click on title
    let mut title_views = vec![];
    
    // Main title button (with easter egg triple-click)
    title_views.push(Components::simple_button(
        "🎓 Adaptive Learning System",
        |state: &mut AppState| {
            state.try_crab_triple_click();
        }
    ));
    
    // Subtitle
    title_views.push(Components::simple_label(
        "An intelligent learning system that adapts to your knowledge and optimizes your learning path using graph-based cognitive models.".to_string()
    ));
    
    // Login Card
    let mut login_views = vec![];
    
    // Username input
    login_views.push(Components::labeled_input(
        "Username:",
        state.username_input.clone(),
        |state: &mut AppState, value: String| {
            state.username_input = value;
        }
    ));
    
    // Password input  
    login_views.push(Components::labeled_input(
        "Password:",
        state.password_input.clone(),
        |state: &mut AppState, value: String| {
            state.password_input = value;
        }
    ));
    
    // Login button
    if state.login_request_in_flight {
        login_views.push(Components::simple_label("Logging in...".to_string()));
    } else {
        login_views.push(Components::simple_button(
            "Login",
            |state: &mut AppState| {
                if !state.login_request_in_flight {
                    state.login();
                }
            }
        ));
    }
    
    // OAuth section separator
    login_views.push(Components::simple_label("─── Or ───".to_string()));
    
    // Apple Sign In button (App Store compliant)
    login_views.push(standard_apple_signin_button::<Components>(state));
    
    // GitHub Sign In button
    login_views.push(github_signin_button::<Components>(state));
    
    let login_card = Components::card("Login", Components::simple_flex_column(login_views));
    
    // Quick Start Card
    let mut quick_start_views = vec![];
    
    quick_start_views.push(Components::simple_label(
        "Start learning immediately without creating an account".to_string()
    ));
    
    // Guest mode button
    quick_start_views.push(Components::simple_button(
        "🚀 Start as Guest",
        |state: &mut AppState| {
            // Proper guest/anonymous user pattern
            state.current_user = None; // No fake user object
            state.is_guest_mode = true;
            state.create_learner();
            state.current_screen = Screen::DomainSelection;
        }
    ));
    
    // Quick tour button
    quick_start_views.push(Components::simple_button(
        "🎯 Quick Tour",
        |state: &mut AppState| {
            // Start the interactive guided tour
            state.demo_start();
        }
    ));
    
    // Training demo button
    quick_start_views.push(Components::simple_button(
        "📚 Training Demo",
        |state: &mut AppState| {
            // Start the full training demo
            state.demo_start_training();
        }
    ));
    
    // Demo showcase button
    quick_start_views.push(Components::simple_button(
        "✨ Demo Showcase",
        |state: &mut AppState| {
            // Run a short automated demo training sequence
            state.demo_showcase();
            state.current_screen = Screen::Dashboard;
        }
    ));
    
    // Always-available navigation to avoid dead ends
    quick_start_views.push(Components::simple_button(
        "📊 Go to Dashboard",
        |state: &mut AppState| {
            state.current_screen = Screen::Dashboard;
        }
    ));
    
    let quick_start_card = Components::card(
        "Quick Start",
        Components::simple_flex_column(quick_start_views)
    );
    
    // Combine all elements
    let mut main_views = vec![];
    main_views.extend(title_views);
    main_views.push(login_card);
    main_views.push(quick_start_card);
    
    // Add footer with version info
    main_views.push(Components::simple_label(
        "v1.0.0 - Cross-Platform Edition".to_string()
    ));
    
    Components::centered_container(
        600.0,
        Components::simple_flex_column(main_views)
    )
}