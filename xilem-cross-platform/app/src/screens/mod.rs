// Screen modules for the Xilem UI application

mod dashboard;
mod domain_selection;
mod settings;
mod training;
mod welcome;

// Re-export all screen functions
pub use dashboard::dashboard_screen;
pub use domain_selection::domain_selection_screen;
pub use settings::settings_screen;
pub use training::training_screen;
pub use welcome::welcome_screen;
