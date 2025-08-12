// Authentication views using the new component library

use crate::state::{AppState, LoadingKey, Screen};
use crate::components::{Components, AppComponents, AppColor, ComponentOutput, SpacerSize};
use crate::components::layout::{page_layout, two_column_layout, content_container};
use crate::components::forms::{form_field, FieldType, form_with_validation, FormField};
use crate::components::cards::{feature_card};
use crate::components::feedback::{toast, ToastType};
use crate::auth::{sign_in_with_apple, AppleSignInResponse, standard_apple_signin_button, github_signin_button};
use tracing::{debug, error, info};

pub fn auth_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    let is_signup = matches!(state.current_screen, Screen::Signup);
    
    let auth_form = if is_signup {
        signup_form(state)
    } else {
        login_form(state)
    };
    
    let info_panel = auth_info_panel(is_signup);
    
    // Use two-column layout on larger screens
    let main_content = if window_width > 900.0 {
        two_column_layout(auth_form, info_panel, window_width)
    } else {
        Components::simple_flex_column(vec![
            auth_form,
            Components::spacer(SpacerSize::Large),
            info_panel,
        ])
    };
    
    // Wrap in content container
    content_container(main_content, window_width)
}

fn login_form(state: &mut AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Welcome Back"));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(Components::label("Sign in to continue your learning journey"));
    items.push(Components::spacer(SpacerSize::Large));
    
    // Username/email field
    items.push(form_field(
        "Username or Email",
        state.auth_username.clone(),
        Some("Enter your username or email"),
        FieldType::Text,
        |state: &mut AppState, value| {
            state.auth_username = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Password field
    items.push(form_field(
        "Password",
        state.auth_password.clone(),
        Some("Enter your password"),
        FieldType::Password,
        |state: &mut AppState, value| {
            state.auth_password = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Small));
    
    // Remember me checkbox
    items.push(Components::checkbox(
        false,
        "Remember me",
        |_state, _checked| {}
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Login button with loading state
    if state.is_loading(LoadingKey::Login) {
        items.push(Components::loading_spinner(Some("Signing in...")));
    } else {
        items.push(Components::action_button(
            "Sign In",
            AppColor::Primary,
            |state: &mut AppState| {
                if state.auth_username.is_empty() || state.auth_password.is_empty() {
                    tracing::error!("Login attempted with empty credentials");
                    state.add_error("Please enter username and password".to_string(), true);
                    return;
                }
                
                state.set_loading(LoadingKey::Login, true);
                
                // Simulate successful login
                tracing::info!("User {} logging in", state.auth_username);
                state.user = Some(crate::state::UserState {
                    id: uuid::Uuid::new_v4(),
                    username: state.auth_username.clone(),
                    email: format!("{}@example.com", state.auth_username),
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
            }
        ));
    }
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Forgot password link
    items.push(Components::simple_button(
        "Forgot password?",
        |state| {
            tracing::error!("Password reset not implemented");
            state.add_error("Password reset coming soon! Contact support for help.".to_string(), true);
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::divider(crate::components::Orientation::Horizontal));
    items.push(Components::spacer(SpacerSize::Large));
    
    // Social login options
    items.push(Components::label("Or sign in with"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // OAuth buttons
    items.push(standard_apple_signin_button::<Components>(state));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(github_signin_button::<Components>(state));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Sign up link
    items.push(Components::simple_flex_row(vec![
        Components::label("Don't have an account?"),
        Components::simple_button("Sign Up", |state| {
            state.navigate(Screen::Signup);
        }),
    ]));
    
    Components::card(
        "Sign In",
        Components::simple_flex_column(items)
    )
}

fn signup_form(state: &mut AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Create Account"));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(Components::label("Start your learning journey today"));
    items.push(Components::spacer(SpacerSize::Large));
    
    // Username field
    items.push(form_field(
        "Username",
        state.auth_username.clone(),
        Some("Choose a username"),
        FieldType::Text,
        |state: &mut AppState, value| {
            state.auth_username = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Email field
    items.push(form_field(
        "Email",
        state.auth_email.clone(),
        Some("your@email.com"),
        FieldType::Email,
        |state: &mut AppState, value| {
            state.auth_email = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Password field
    items.push(form_field(
        "Password",
        state.auth_password.clone(),
        Some("Min 8 characters"),
        FieldType::Password,
        |state: &mut AppState, value| {
            state.auth_password = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Confirm password field
    items.push(form_field(
        "Confirm Password",
        state.auth_password_confirm.clone(),
        Some("Re-enter password"),
        FieldType::Password,
        |state: &mut AppState, value| {
            state.auth_password_confirm = value;
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Small));
    
    // Terms checkbox
    items.push(Components::checkbox(
        false,
        "I agree to the Terms of Service and Privacy Policy",
        |_state, _checked| {}
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Sign up button with loading state
    if state.is_loading(LoadingKey::Signup) {
        items.push(Components::loading_spinner(Some("Creating account...")));
    } else {
        items.push(Components::action_button(
            "Create Account",
            AppColor::Primary,
            |state: &mut AppState| {
                // Validate fields
                if state.auth_username.is_empty() || 
                   state.auth_email.is_empty() || 
                   state.auth_password.is_empty() {
                    tracing::error!("Signup attempted with missing fields");
                    state.add_error("Please fill in all required fields".to_string(), true);
                    return;
                }
                
                if state.auth_password != state.auth_password_confirm {
                    tracing::error!("Password confirmation mismatch");
                    state.add_error("Passwords do not match".to_string(), true);
                    return;
                }
                
                state.set_loading(LoadingKey::Signup, true);
                
                // Simulate successful signup
                tracing::info!("Creating account for user {}", state.auth_username);
                state.user = Some(crate::state::UserState {
                    id: uuid::Uuid::new_v4(),
                    username: state.auth_username.clone(),
                    email: state.auth_email.clone(),
                    display_name: Some(state.auth_username.clone()),
                    created_at: chrono::Utc::now(),
                    learner_id: Some(uuid::Uuid::new_v4()),
                    level: 1,
                    xp: 0,
                    streak: 0,
                    achievements: Vec::new(),
                });
                
                // Clear sensitive data
                state.auth_password.clear();
                state.auth_password_confirm.clear();
                state.set_loading(LoadingKey::Signup, false);
                state.navigate(Screen::Welcome);
            }
        ));
    }
    
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::divider(crate::components::Orientation::Horizontal));
    items.push(Components::spacer(SpacerSize::Large));
    
    // Social signup options
    items.push(Components::label("Or sign up with"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    items.push(standard_apple_signin_button::<Components>(state));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(github_signin_button::<Components>(state));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Sign in link
    items.push(Components::simple_flex_row(vec![
        Components::label("Already have an account?"),
        Components::simple_button("Sign In", |state| {
            state.navigate(Screen::Login);
        }),
    ]));
    
    Components::card(
        "Sign Up",
        Components::simple_flex_column(items)
    )
}

fn auth_info_panel(is_signup: bool) -> ComponentOutput {
    let mut items = vec![];
    
    if is_signup {
        items.push(Components::label("Why Join ABCDEEZ?"));
        items.push(Components::spacer(SpacerSize::Medium));
        
        let benefits = vec![
            ("🎯", "Personalized Learning", "AI adapts to your learning style"),
            ("📊", "Track Progress", "Detailed analytics and insights"),
            ("🏆", "Compete & Win", "Global leaderboards and challenges"),
            ("🎮", "Gamified Experience", "Earn XP, badges, and rewards"),
            ("👥", "Learn Together", "Connect with friends and compete"),
            ("📈", "Proven Results", "Improve faster with our system"),
        ];
        
        for (icon, title, desc) in benefits {
            items.push(Components::simple_flex_column(vec![
                Components::simple_flex_row(vec![
                    Components::label(icon),
                    Components::label(title),
                ]),
                Components::label(desc),
                Components::spacer(SpacerSize::Small),
            ]));
        }
    } else {
        items.push(Components::label("Welcome Back!"));
        items.push(Components::spacer(SpacerSize::Medium));
        
        // Show some motivational stats
        items.push(feature_card(
            "🌍",
            "Join 100K+ Learners",
            "Active community worldwide",
            "",
            |_| {}
        ));
        
        items.push(Components::spacer(SpacerSize::Medium));
        
        items.push(feature_card(
            "📚",
            "1M+ Tasks Completed",
            "By our community this month",
            "",
            |_| {}
        ));
        
        items.push(Components::spacer(SpacerSize::Medium));
        
        items.push(feature_card(
            "⭐",
            "4.8/5 Rating",
            "From thousands of reviews",
            "",
            |_| {}
        ));
    }
    
    Components::simple_flex_column(items)
}