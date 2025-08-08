#![allow(unused)]

/*!
Module orchestrator for screen componentization.

This `screens` directory module is the future home of per-screen files:
- `welcome.rs`       -> `pub fn welcome_screen(data: &mut AppData) -> impl WidgetView<AppData>`
- `domain.rs`        -> `pub fn domain_selection_screen(data: &mut AppData) -> impl WidgetView<AppData>`
- `training.rs`      -> `pub fn training_screen(data: &mut AppData) -> impl WidgetView<AppData>`
- `dashboard.rs`     -> `pub fn dashboard_screen(data: &mut AppData) -> impl WidgetView<AppData>`
- `settings.rs`      -> `pub fn settings_screen(data: &mut AppData) -> impl WidgetView<AppData>`

During the transition from a single `screens.rs` to a componentized layout, this module
acts as the canonical re-export surface so callers (e.g., `lib.rs`) don’t need to change.
When you move the functions into their respective files, make sure each file defines
the exported function exactly as re-exported below.

Example layout once migration is complete:

screens/
  ├─ mod.rs               (this file)
  ├─ welcome.rs           (welcome_screen)
  ├─ domain.rs            (domain_selection_screen)
  ├─ training.rs          (training_screen)
  ├─ dashboard.rs         (dashboard_screen)
  └─ settings.rs          (settings_screen)

Note: While both `app/src/screens.rs` (file-based module) and `app/src/screens/mod.rs`
(directory-based module) cannot coexist for a single `mod screens;` declaration in Rust,
this file is being added now to facilitate the componentization effort. As you move
functions into the new per-screen files, remove/rename `app/src/screens.rs` and wire
`mod screens;` to this directory module instead.
*/

// Per-screen submodules (create each file as you migrate functions).
pub mod dashboard;
pub mod domain;
pub mod settings;
pub mod training;
pub mod welcome;

// Public re-exports so the rest of the codebase can keep using the same names.
pub use dashboard::dashboard_screen;
pub use domain::domain_selection_screen;
pub use settings::settings_screen;
pub use training::training_screen;
pub use welcome::welcome_screen;
