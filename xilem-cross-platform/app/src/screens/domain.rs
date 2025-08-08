use xilem::{
    view::{button, flex, label, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::AppData;

/// Stub implementation of `domain_selection_screen` to enable gradual migration
/// from the single-file `app/src/screens.rs` to per-screen modules under
/// `app/src/screens/`.
///
/// NOTE:
/// - This file is not yet wired into the build until `mod screens;` in `lib.rs`
///   points to `app/src/screens/mod.rs` (the directory-based module) and the
///   functions are moved over from the legacy `screens.rs`.
/// - Keep this function signature exactly the same as the legacy one so callers
///   can be switched over without churn.
///
/// When you are ready to migrate:
/// 1) Move the existing domain selection view code from `app/src/screens.rs`
///    into this function body.
/// 2) Ensure `app/src/screens/mod.rs` re-exports `domain::domain_selection_screen`.
/// 3) Remove/rename the legacy `app/src/screens.rs` to avoid module conflicts.
pub fn domain_selection_screen(_data: &mut AppData) -> impl WidgetView<AppData> {
    // Minimal placeholder content; replace with full content during migration.
    flex((
        label("Domain Selection")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        label("This is a stub screen for domain selection.").alignment(TextAlignment::Middle),
        button("This is a stub — wired during migration", |_data: &mut AppData| {
            // no-op
        }),
    ))
    .direction(Axis::Vertical)
}
