// Authentication views (login/signup)

use xilem::core::one_of::Either;
use xilem::view::*;
use xilem::WidgetView;
use xilem::style::{Style, Background};
use xilem::FontWeight;
use xilem::Color;
use xilem::TextAlign;

use crate::state::{AppState, LoadingKey, Screen};

pub fn auth_view(state: &mut AppState) -> impl WidgetView<AppState> {
    match state.current_screen {
        Screen::Login => Either::A(login_view(state)),
        Screen::Signup => Either::B(signup_view(state)),
        _ => Either::A(login_view(state)),
    }
}

fn login_view(state: &mut AppState) -> impl WidgetView<AppState> {
    let is_loading = state.is_loading(LoadingKey::Login);

    flex((
        // Logo/Title
        FlexSpacer::Flex(1.0),
        label("ABCDEEZ Learning")
            .text_size(32.0)
            .weight(FontWeight::BOLD)
            ,
        FlexSpacer::Fixed(40.0),

        // Login form
        sized_box(
            flex((
                // Username field
                label("Username")
                    .text_size(14.0)
                    ,
                FlexSpacer::Fixed(8.0),
                text_input(
                    state.auth_username.clone(),
                    |state: &mut AppState, text| {
                        state.auth_username = text;
                    },
                )
                .disabled(is_loading)
                .padding(12.0)
                .background(Background::Color(state.theme.surface_color()))
                .corner_radius(8.0),

                FlexSpacer::Fixed(20.0),

                // Password field
                label("Password")
                    .text_size(14.0)
                    ,
                FlexSpacer::Fixed(8.0),
                text_input(
                    state.auth_password.clone(),
                    |state: &mut AppState, text| {
                        state.auth_password = text;
                    },
                )
                .disabled(is_loading)
                .padding(12.0)
                .background(Background::Color(state.theme.surface_color()))
                .corner_radius(8.0),

                FlexSpacer::Fixed(30.0),

                // Login button
                if is_loading {
                    Either::A(
                        sized_box(spinner())
                            .width(30.0)
                            .height(30.0)
                    )
                } else {
                    Either::B(
                        button(label("Login"), |state: &mut AppState| {
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
                        })
                        .background(Background::Color(state.theme.primary_color()))
                        .padding(15.0)
                        .corner_radius(8.0)
                    )
                },

                FlexSpacer::Fixed(20.0),

                // Divider
                label("OR")
                    .text_size(12.0)
                    ,

                FlexSpacer::Fixed(20.0),

                // OAuth buttons
                flex_row((
                    button(label("Sign in with Apple"), |_state: &mut AppState| {
                        // TODO: Implement Apple Sign In
                    })
                    .padding(12.0)
                    .background(Background::Color(Color::BLACK))
                    .corner_radius(8.0)
                    .flex(1.0),

                    FlexSpacer::Fixed(10.0),

                    button(label("GitHub"), |_state: &mut AppState| {
                        // TODO: Implement GitHub OAuth
                    })
                    .padding(12.0)
                    .background(Background::Color(Color::from_rgb8(36, 41, 47)))
                    .corner_radius(8.0)
                    .flex(1.0),
                ))
                .gap(10.0),

                FlexSpacer::Fixed(30.0),

                // Sign up link
                flex_row((
                    label("Don't have an account?")
                        .text_size(14.0)
                        ,
                    FlexSpacer::Fixed(5.0),
                    button(label("Sign Up"), |state: &mut AppState| {
                        state.navigate(Screen::Signup);
                    })
                ))
                .main_axis_alignment(MainAxisAlignment::Center),
            ))
            .direction(Axis::Vertical)
        )
        .width(350.0)
        .padding(20.0),

        FlexSpacer::Flex(1.0),
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}

fn signup_view(state: &mut AppState) -> impl WidgetView<AppState> {
    let is_loading = state.is_loading(LoadingKey::Login);

    flex((
        // Back button
        flex_row((
            button("← Back", |state: &mut AppState| {
                state.navigate(Screen::Login);
            })
            .padding(10.0),
            FlexSpacer::Flex(1.0),
        ))
        .padding(10.0),

        FlexSpacer::Flex(1.0),

        label("Create Account")
            .text_size(28.0)
            .weight(FontWeight::BOLD)
            ,
        FlexSpacer::Fixed(30.0),

        // Signup form
        sized_box(
            flex((
                // Email field
                label("Email")
                    .text_size(14.0)
                    ,
                FlexSpacer::Fixed(8.0),
                text_input(
                    state.auth_email.clone(),
                    |state: &mut AppState, text| {
                        state.auth_email = text;
                    },
                )
                .disabled(is_loading)
                .padding(12.0)
                .background(Background::Color(state.theme.surface_color()))
                .corner_radius(8.0),

                FlexSpacer::Fixed(20.0),

                // Username field
                label("Username")
                    .text_size(14.0)
                    ,
                FlexSpacer::Fixed(8.0),
                text_input(
                    state.auth_username.clone(),
                    |state: &mut AppState, text| {
                        state.auth_username = text;
                    },
                )
                .disabled(is_loading)
                .padding(12.0)
                .background(Background::Color(state.theme.surface_color()))
                .corner_radius(8.0),

                FlexSpacer::Fixed(20.0),

                // Password field
                label("Password")
                    .text_size(14.0)
                    ,
                FlexSpacer::Fixed(8.0),
                text_input(
                    state.auth_password.clone(),
                    |state: &mut AppState, text| {
                        state.auth_password = text;
                    },
                )
                .disabled(is_loading)
                .padding(12.0)
                .background(Background::Color(state.theme.surface_color()))
                .corner_radius(8.0),

                FlexSpacer::Fixed(30.0),

                // Sign up button
                button(label("Create Account"), |state: &mut AppState| {
                    state.set_loading(LoadingKey::Login, true);
                    // TODO: Call API client for actual signup
                    // For now, simulate successful signup
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
                    state.auth_confirm_password.clear();
                    state.set_loading(LoadingKey::Login, false);
                    state.navigate(Screen::Dashboard);
                })
                .disabled(is_loading)
                .background_color(state.theme.primary_color())
                .padding(15.0)
                .corner_radius(8.0),

                FlexSpacer::Fixed(20.0),

                // Terms text
                prose("By creating an account, you agree to our Terms of Service and Privacy Policy")
                    .text_size(12.0)
                                        .text_alignment(TextAlign::Center),
            ))
            .direction(Axis::Vertical)
        )
        .width(350.0)
        .padding(20.0),

        FlexSpacer::Flex(1.0),
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}