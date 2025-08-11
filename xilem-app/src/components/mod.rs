// High-level component library for ABCDEEZ app
// These components abstract over platform differences

mod builders;

use crate::state::{AppState, Screen};
use crate::models::Task;

// Re-export builders for convenience
pub use builders::*;

// Component trait that all platforms must implement
pub trait Component: Sized {
    type Output;
    fn build(self) -> Self::Output;
}

// High-level app-specific components
pub trait AppComponents {
    type Output: Component;
    
    // Card components - used throughout the app
    fn stat_card(title: &str, value: &str, accent_color: AppColor) -> Self::Output;
    fn welcome_card(username: &str, message: &str, on_start: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output;
    fn activity_card(title: &str, time: &str, highlighted: bool) -> Self::Output;
    
    // Navigation components
    fn header_bar(title: &str, on_settings: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output;
    fn bottom_nav_bar(current_screen: Screen, on_navigate: impl Fn(&mut AppState, Screen) + Send + Sync + 'static) -> Self::Output;
    fn nav_button(label: &str, screen: Screen, is_active: bool, on_click: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output;
    
    // Learning components
    fn task_presenter(task: &Task, on_answer: impl Fn(&mut AppState, String) + Send + Sync + 'static) -> Self::Output;
    fn task_options(options: Vec<String>, on_select: impl Fn(&mut AppState, usize) + Send + Sync + 'static) -> Self::Output;
    fn learning_progress(current: usize, total: usize, accuracy: f64) -> Self::Output;
    fn hint_button(hint_level: usize, on_hint: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output;
    
    // Task visual components
    fn alphabet_sequence(items: Vec<String>, highlight_index: Option<usize>) -> Self::Output;
    fn comparison_visual(left: &str, right: &str, show_order: bool) -> Self::Output;
    fn missing_item_visual(before: &str, after: &str) -> Self::Output;
    fn path_visual(start: &str, end: &str, path: Vec<String>) -> Self::Output;
    
    // Basic building blocks (for task visuals that need custom layouts)
    fn simple_label(text: String) -> Self::Output;
    fn simple_flex_column(items: Vec<Self::Output>) -> Self::Output;
    fn simple_flex_row(items: Vec<Self::Output>) -> Self::Output;
    fn simple_button(text: &str, on_click: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output;
    
    // Form components
    fn auth_form(
        username: String,
        password: String,
        on_username: impl Fn(&mut AppState, String) + Send + Sync + 'static,
        on_password: impl Fn(&mut AppState, String) + Send + Sync + 'static,
        on_submit: impl Fn(&mut AppState) + Send + Sync + 'static,
    ) -> Self::Output;
    
    fn settings_section(title: &str, children: Vec<Self::Output>) -> Self::Output;
    fn setting_row(label: &str, control: Self::Output) -> Self::Output;
    fn theme_selector(current: AppTheme, on_change: impl Fn(&mut AppState, AppTheme) + Send + Sync + 'static) -> Self::Output;
    
    // Layout components
    fn app_scaffold(header: Self::Output, content: Self::Output, footer: Option<Self::Output>) -> Self::Output;
    fn centered_container(max_width: f64, child: Self::Output) -> Self::Output;
    fn stats_grid(stats: Vec<Self::Output>) -> Self::Output;
    
    // Feedback components
    fn loading_spinner(message: Option<&str>) -> Self::Output;
    fn empty_state(icon: &str, title: &str, message: &str, action: Option<Self::Output>) -> Self::Output;
    fn error_banner(message: &str, on_dismiss: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output;
    
    // Additional ported components from old app
    
    // Basic UI components
    fn card<V>(title: &str, content: V) -> Self::Output where V: Component<Output = Self::Output>;
    fn progress_bar(progress: f64, label: &str) -> Self::Output;
    fn metric_display(label: &str, value: &str, color: AppColor) -> Self::Output;
    
    // Interactive components
    fn checkbox(checked: bool, label: &str, on_toggle: impl Fn(&mut AppState, bool) + Send + Sync + 'static) -> Self::Output;
    fn labeled_input(label: &str, value: String, on_change: impl Fn(&mut AppState, String) + Send + Sync + 'static) -> Self::Output;
    fn toast_notification(message: &str, is_success: bool) -> Self::Output;
    fn loading_overlay(message: &str) -> Self::Output;
    
    // Navigation
    fn nav_bar(current_screen: &str) -> Self::Output;
    
    // Messages
    fn error_message(message: Option<String>) -> Self::Output;
    fn success_message(message: Option<String>) -> Self::Output;
    
    // Demo-aware components
    fn highlighted<V>(element_id: &str, content: V, has_highlight: bool, tooltip: Option<String>) -> Self::Output 
        where V: Component<Output = Self::Output>;
    fn demo_button(element_id: &str, text: &str, has_highlight: bool, tooltip: Option<String>) -> Self::Output;
    fn demo_card<V>(element_id: &str, title: &str, content: V, has_highlight: bool, tooltip: Option<String>) -> Self::Output
        where V: Component<Output = Self::Output>;
    
    // Modals
    fn confirm_modal(
        title: &str,
        message: &str,
        on_confirm: impl Fn(&mut AppState) + Send + Sync + 'static,
        on_cancel: impl Fn(&mut AppState) + Send + Sync + 'static,
    ) -> Self::Output;
    
    // Visualization components
    fn response_time_histogram(response_times: &[u128]) -> Self::Output;
    fn learning_curve_display(responses: &[crate::models::Response]) -> Self::Output;
    fn error_analysis_display(responses: &[crate::models::Response]) -> Self::Output;
    fn strategy_analysis_display(strategies: Vec<(&str, f64)>) -> Self::Output;
    
    // Domain-specific
    fn domain_card(domain: &str, description: &str, selected: bool) -> Self::Output;
    fn session_info(session_id: &str, status: &str, duration: &str, domain: &str) -> Self::Output;
    
    // Task-specific
    fn task_card_display(prompt: &str, hint: Option<&str>) -> Self::Output;
    fn answer_options_display(options: Vec<String>, on_select: impl Fn(&mut AppState, usize) + Send + Sync + 'static) -> Self::Output;
    fn performance_chart(accuracy: f64, total_responses: usize, correct: usize, avg_time: f64) -> Self::Output;
}

// App-specific enums that abstract platform differences
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppColor {
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
    Info,
    Surface,
    Background,
    Text,
    TextMuted,
}

// Removed AppScreen in favor of using the app-wide `state::Screen`

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppTheme {
    Light,
    Dark,
    Auto,
}

// Platform-specific implementations
#[cfg(feature = "xilem-native")]
pub mod native_components;

#[cfg(feature = "xilem-web")]
pub mod web_components;

// Export the current platform's implementation
// Prefer native if both features are set

#[cfg(feature = "xilem-native")]
pub use native_components::{NativeComponents as Components, NativeComponent as ComponentOutput};

#[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
pub use web_components::{WebComponents as Components, WebComponent as ComponentOutput};