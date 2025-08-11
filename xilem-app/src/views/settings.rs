// Settings and profile views

use xilem::view::*;
use xilem::WidgetView;
use xilem::style::Style;
use xilem::FontWeight;
use xilem::Color;

use crate::state::{AppState, Screen, ThemeMode};

pub fn settings_view(state: &mut AppState) -> impl WidgetView<AppState> {
    flex((
        // Header
        settings_header(state),

        // Settings content
        sized_box(
            flex((
                // Profile section
                profile_section(state),
                FlexSpacer::Fixed(30.0),

                // Preferences section
                preferences_section(state),
                FlexSpacer::Fixed(30.0),

                // Account section
                account_section(state),
            ))
            .direction(Axis::Vertical)
            .padding(20.0)
        )
        .expand(),
    ))
    .direction(Axis::Vertical)
    .must_fill_major_axis(true)
}

fn settings_header(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        button("← Back", |state: &mut AppState| {
            state.navigate_back();
        })
        .padding(10.0),

        FlexSpacer::Flex(1.0),

        label("Settings")
            .text_size(20.0)
            .weight(FontWeight::BOLD),

        FlexSpacer::Flex(1.0),

        // Placeholder for right side
        sized_box(label("")).width(60.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .padding(15.0)
    .background_color(state.theme.surface_color())
}

fn profile_section(state: &AppState) -> impl WidgetView<AppState> {
    sized_box(
        flex((
            label("Profile")
                .text_size(18.0)
                .weight(FontWeight::BOLD),
            FlexSpacer::Fixed(20.0),

            // Avatar placeholder
            sized_box(
                label("👤")
                    .text_size(48.0)
            )
            .width(100.0)
            .height(100.0)
            .background_color(state.theme.surface_color())
            .corner_radius(50.0),

            FlexSpacer::Fixed(15.0),

            // User info
            if let Some(user) = &state.user {
                label(&*user.username)
                    .text_size(20.0)
                    .weight(FontWeight::MEDIUM)
            } else {
                label("Guest User")
                    .text_size(20.0)
                    .weight(FontWeight::MEDIUM)
            },

            FlexSpacer::Fixed(10.0),

            button(label("Edit Profile").color(Color::WHITE), |state: &mut AppState| {
                state.navigate(Screen::Profile);
            })
            .background_color(state.theme.primary_color())
            .padding(12.0)
            .corner_radius(8.0),
        ))
        .direction(Axis::Vertical)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .padding(20.0)
    )
    .background_color(state.theme.surface_color())
    .corner_radius(12.0)
}

fn preferences_section(state: &AppState) -> impl WidgetView<AppState> {
    sized_box(
        flex((
            label("Preferences")
                .text_size(18.0)
                .weight(FontWeight::BOLD),
            FlexSpacer::Fixed(15.0),

            // Theme setting
            setting_row(
                "Theme",
                flex_row((
                    theme_option("Light", ThemeMode::Light, state),
                    theme_option("Dark", ThemeMode::Dark, state),
                    theme_option("Auto", ThemeMode::Auto, state),
                ))
            ),

            FlexSpacer::Fixed(15.0),

            // Notifications setting
            setting_row(
                "Notifications",
                checkbox("Enable notifications", false, |_state: &mut AppState, _checked| {
                    // TODO: Update notification preference
                })
            ),

            FlexSpacer::Fixed(15.0),

            // Sound setting
            setting_row(
                "Sound Effects",
                checkbox("Enable sound", true, |_state: &mut AppState, _checked| {
                    // TODO: Update sound preference
                })
            ),
        ))
        .direction(Axis::Vertical)
        .padding(20.0)
    )
    .background_color(state.theme.surface_color())
    .corner_radius(12.0)
}

fn account_section(state: &AppState) -> impl WidgetView<AppState> {
    sized_box(
        flex((
            label("Account")
                .text_size(18.0)
                .weight(FontWeight::BOLD),
            FlexSpacer::Fixed(15.0),

            button("Export Data", |_state: &mut AppState| {
                // TODO: Export user data
            })
            .padding(12.0)
            .background_color(state.theme.surface_color())
            .corner_radius(8.0),

            FlexSpacer::Fixed(10.0),

            button("Privacy Settings", |_state: &mut AppState| {
                // TODO: Navigate to privacy settings
            })
            .padding(12.0)
            .background_color(state.theme.surface_color())
            .corner_radius(8.0),

            FlexSpacer::Fixed(10.0),

            button(label("Sign Out").color(Color::WHITE), |state: &mut AppState| {
                state.user = None;
                state.auth_token = None;
                state.navigate(Screen::Login);
            })
            .padding(12.0)
            .background_color(state.theme.error_color())
            .corner_radius(8.0),

            FlexSpacer::Fixed(20.0),

            button("Delete Account", |_state: &mut AppState| {
                // TODO: Confirm and delete account
            })
            .padding(12.0)
            .border(state.theme.error_color(), 1.0)
            .background_color(Color::TRANSPARENT),
        ))
        .direction(Axis::Vertical)
        .padding(20.0)
    )
    .background_color(state.theme.surface_color())
    .corner_radius(12.0)
}

fn setting_row(
    label_text: &str,
    control: impl WidgetView<AppState> + 'static,
) -> impl WidgetView<AppState> {
    flex_row((
        label(label_text)
            .text_size(14.0)
            .flex(1.0),
        control,
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
}

fn theme_option(label_text: &str, mode: ThemeMode, state: &AppState) -> impl WidgetView<AppState> {
    let is_selected = state.theme_mode == mode;
    button(
        label(label_text).color(if is_selected { Color::WHITE } else { state.theme.text_color() }),
        move |app_state: &mut AppState| {
            app_state.theme_mode = mode;
            app_state.theme.mode = mode;
        },
    )
    .background_color(if is_selected {
        state.theme.primary_color()
    } else {
        state.theme.surface_color()
    })
    .padding(8.0)
    .corner_radius(5.0)
    
}