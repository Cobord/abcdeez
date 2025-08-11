// Dashboard view

use xilem::view::*;
use xilem::WidgetView;
use xilem::style::Style;
use xilem::FontWeight;
use xilem::Color;

use crate::state::{AppState, Screen};

pub fn dashboard_view(state: &mut AppState) -> impl WidgetView<AppState> {
    flex((
        // Header
        header_bar(state),

        // Main content
        sized_box(
            flex((
                // Welcome message
                welcome_card(state),
                FlexSpacer::Fixed(20.0),

                // Quick stats
                stats_overview(state),
                FlexSpacer::Fixed(20.0),

                // Quick actions
                quick_actions(state),
                FlexSpacer::Fixed(20.0),

                // Recent activity
                recent_activity(state),
            ))
            .direction(Axis::Vertical)
            .padding(20.0)
        )
        .expand(),

        // Bottom navigation
        bottom_navigation(state),
    ))
    .direction(Axis::Vertical)
    .must_fill_major_axis(true)
}

fn header_bar(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        label("ABCDEEZ")
            .text_size(24.0)
            .weight(FontWeight::BOLD)
            .color(state.theme.primary_color()),
        FlexSpacer::Flex(1.0),
        button("Settings", |state: &mut AppState| {
            state.navigate(Screen::Settings);
        })
        .padding(10.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .padding(15.0)
    .background_color(state.theme.surface_color())
}

fn welcome_card(state: &AppState) -> impl WidgetView<AppState> {
    sized_box(
        flex((
            label("Welcome back!")
                .text_size(28.0)
                .weight(FontWeight::BOLD),
            FlexSpacer::Fixed(10.0),
            label("Ready to continue learning?")
                .text_size(16.0)
                .color(Color::from_rgb8(128, 128, 128)),
            FlexSpacer::Fixed(20.0),
            button(label("Start Learning Session").color(Color::WHITE), |state: &mut AppState| {
                state.navigate(Screen::Learning);
            })
            .background_color(state.theme.primary_color())
            .padding(15.0)
            .corner_radius(10.0),
        ))
        .direction(Axis::Vertical)
        .padding(20.0)
    )
    .background_color(state.theme.surface_color())
    .corner_radius(12.0)
}

fn stats_overview(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        stat_card("Sessions", "12", state.theme.primary_color()),
        stat_card("Accuracy", "85%", state.theme.success_color()),
        stat_card("Streak", "5 days", state.theme.warning_color()),
    ))
    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
}

fn stat_card(title: &str, value: &str, color: Color) -> impl WidgetView<AppState> {
    sized_box(
        flex((
            label(title)
                .text_size(12.0)
                .color(Color::from_rgb8(128, 128, 128)),
            FlexSpacer::Fixed(5.0),
            label(value)
                .text_size(24.0)
                .weight(FontWeight::BOLD)
                .color(color),
        ))
        .direction(Axis::Vertical)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .padding(15.0)
    )
    .background_color(Color::from_rgb8(245, 245, 250))
    .corner_radius(10.0)
}

fn quick_actions(state: &AppState) -> impl WidgetView<AppState> {
    flex((
        label("Quick Actions")
            .text_size(18.0)
            .weight(FontWeight::BOLD),
        FlexSpacer::Fixed(15.0),
        flex_row((
            action_button("Practice\nAlphabet", Screen::Learning, state.theme.primary_color()),
            FlexSpacer::Fixed(10.0),
            action_button("View\nProgress", Screen::Progress, state.theme.success_color()),
            FlexSpacer::Fixed(10.0),
            action_button("Leaderboard", Screen::Leaderboard, state.theme.warning_color()),
        )),
    ))
    .direction(Axis::Vertical)
}

fn action_button(label_text: &str, target_screen: Screen, color: Color) -> impl WidgetView<AppState> {
    button(
        label(label_text).color(color),
        move |state: &mut AppState| {
            state.navigate(target_screen.clone());
        },
    )
    .background_color(color.with_alpha(0.1))
    .padding(20.0)
    .corner_radius(10.0)
}

fn recent_activity(state: &AppState) -> impl WidgetView<AppState> {
    sized_box(
        flex((
            label("Recent Activity")
                .text_size(18.0)
                .weight(FontWeight::BOLD),
            FlexSpacer::Fixed(15.0),
            activity_item("Completed Alphabet Session", "2 hours ago", true),
            activity_item("New Achievement: Quick Learner", "5 hours ago", false),
            activity_item("Joined Weekly Challenge", "1 day ago", false),
        ))
        .direction(Axis::Vertical)
        .padding(15.0)
    )
    .background_color(state.theme.surface_color())
    .corner_radius(10.0)
}

fn activity_item(title: &str, time: &str, highlight: bool) -> impl WidgetView<AppState> {
    flex_row((
        label(title)
            .text_size(14.0)
            .flex(1.0),
        label(time)
            .text_size(12.0)
            .color(Color::from_rgb8(128, 128, 128)),
    ))
    .padding(10.0)
    .background_color(if highlight {
        Color::from_rgb8(240, 248, 255)
    } else {
        Color::TRANSPARENT
    })
    .corner_radius(5.0)
}

fn bottom_navigation(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        nav_button("Home", Screen::Dashboard, true, state),
        nav_button("Learn", Screen::Learning, false, state),
        nav_button("Progress", Screen::Progress, false, state),
        nav_button("Profile", Screen::Profile, false, state),
    ))
    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
    .padding(10.0)
    .background_color(state.theme.surface_color())
}

fn nav_button(label_text: &str, screen: Screen, is_active: bool, state: &AppState) -> impl WidgetView<AppState> {
    button(
        label(label_text).color(if is_active {
            state.theme.primary_color()
        } else {
            Color::from_rgb8(128, 128, 128)
        }),
        move |app_state: &mut AppState| {
            app_state.navigate(screen.clone());
        },
    )
    .padding(10.0)
}