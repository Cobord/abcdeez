// Main application logic and root view

use xilem::core::one_of::{OneOf5};
use xilem::view::*;
use xilem::WidgetView;
use xilem::style::{Style, Background};

use crate::state::{AppState, Screen};
use crate::views::{auth, dashboard, learning, settings};

pub fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> {
    // Apply theme to root container
    sized_box(match state.current_screen {
        Screen::Login | Screen::Signup => OneOf5::A(auth::auth_view(state)),
        Screen::Dashboard => OneOf5::B(dashboard::dashboard_view(state)),
        Screen::Learning => OneOf5::C(learning::learning_view(state)),
        Screen::Settings | Screen::Profile => OneOf5::D(settings::settings_view(state)),
        _ => OneOf5::E(loading_view()),
    })
    .expand()
    .background(Background::Color(state.theme.background_color()))
}

fn loading_view() -> impl WidgetView<AppState> {
    flex((
        FlexSpacer::Flex(1.0),
        sized_box(spinner()).width(50.0).height(50.0),
        FlexSpacer::Fixed(20.0),
        label("Loading...").text_size(18.0),
        FlexSpacer::Flex(1.0),
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}
