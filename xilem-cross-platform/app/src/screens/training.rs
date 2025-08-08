use xilem::{
    view::{button, flex, label, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::{AppData, Screen};

/// Training screen (stub)
///
/// This is a placeholder to enable gradual componentization.
/// Replace the body with the real training UI when migrating from the
/// legacy single-file `app/src/screens.rs`.
pub fn training_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Training (stub)")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        label("This is a stub for the Training screen.").alignment(TextAlignment::Middle),
        button("Back to Dashboard", |d: &mut AppData| {
            d.current_screen = Screen::Dashboard;
        }),
    ))
    .direction(Axis::Vertical)
}

/// Dashboard screen (stub)
///
/// This is a placeholder to enable gradual componentization.
/// Replace the body with the real dashboard UI when migrating from the
/// legacy single-file `app/src/screens.rs`.
pub fn dashboard_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Dashboard (stub)")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        label("This is a stub for the Dashboard screen.").alignment(TextAlignment::Middle),
        button("Go to Training", |d: &mut AppData| {
            d.current_screen = Screen::Training;
        }),
        button("Settings", |d: &mut AppData| {
            d.current_screen = Screen::Settings;
        }),
    ))
    .direction(Axis::Vertical)
}

/// Settings screen (stub)
///
/// This is a placeholder to enable gradual componentization.
/// Replace the body with the real settings UI when migrating from the
/// legacy single-file `app/src/screens.rs`.
pub fn settings_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Settings (stub)")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        label("This is a stub for the Settings screen.").alignment(TextAlignment::Middle),
        button("Back to Dashboard", |d: &mut AppData| {
            d.current_screen = Screen::Dashboard;
        }),
    ))
    .direction(Axis::Vertical)
}
