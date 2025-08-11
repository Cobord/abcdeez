// Component builders - simplified API for common component patterns
// These wrap the trait methods for easier use across the app

use crate::components::{AppComponents, Component, Components, ComponentOutput, AppColor};
use crate::state::AppState;
use crate::models::Response;

// Re-export common builders for easy access
pub fn card(title: &str, content: impl Component<Output = ComponentOutput>) -> ComponentOutput {
    Components::card(title, content)
}

pub fn progress_bar(progress: f64, label: &str) -> ComponentOutput {
    Components::progress_bar(progress, label)
}

pub fn metric_display(label: &str, value: &str, color: AppColor) -> ComponentOutput {
    Components::metric_display(label, value, color)
}

pub fn checkbox<F>(checked: bool, label: &str, on_toggle: F) -> ComponentOutput 
where
    F: Fn(&mut AppState, bool) + Send + Sync + 'static
{
    Components::checkbox(checked, label, on_toggle)
}

pub fn labeled_input<F>(label: &str, value: String, on_change: F) -> ComponentOutput
where
    F: Fn(&mut AppState, String) + Send + Sync + 'static
{
    Components::labeled_input(label, value, on_change)
}

pub fn button<F>(text: &str, on_click: F) -> ComponentOutput
where
    F: Fn(&mut AppState) + Send + Sync + 'static
{
    Components::simple_button(text, on_click)
}

pub fn text(content: &str) -> ComponentOutput {
    Components::simple_label(content.to_string())
}

pub fn label(content: &str) -> ComponentOutput {
    Components::simple_label(content.to_string())
}

pub fn column(items: Vec<ComponentOutput>) -> ComponentOutput {
    Components::simple_flex_column(items)
}

pub fn row(items: Vec<ComponentOutput>) -> ComponentOutput {
    Components::simple_flex_row(items)
}

pub fn toast(message: &str, is_success: bool) -> ComponentOutput {
    Components::toast_notification(message, is_success)
}

pub fn loading(message: &str) -> ComponentOutput {
    Components::loading_overlay(message)
}

pub fn error_msg(message: Option<String>) -> ComponentOutput {
    Components::error_message(message)
}

pub fn success_msg(message: Option<String>) -> ComponentOutput {
    Components::success_message(message)
}

pub fn nav_bar(current_screen: &str) -> ComponentOutput {
    Components::nav_bar(current_screen)
}

// Demo-aware builders
pub fn highlighted(
    element_id: &str, 
    content: impl Component<Output = ComponentOutput>,
    has_highlight: bool,
    tooltip: Option<String>
) -> ComponentOutput {
    Components::highlighted(element_id, content, has_highlight, tooltip)
}

pub fn demo_button(
    element_id: &str,
    text: &str,
    has_highlight: bool,
    tooltip: Option<String>
) -> ComponentOutput {
    Components::demo_button(element_id, text, has_highlight, tooltip)
}

pub fn demo_card(
    element_id: &str,
    title: &str,
    content: impl Component<Output = ComponentOutput>,
    has_highlight: bool,
    tooltip: Option<String>
) -> ComponentOutput {
    Components::demo_card(element_id, title, content, has_highlight, tooltip)
}

// Modal builder
pub fn confirm_modal<F1, F2>(
    title: &str,
    message: &str,
    on_confirm: F1,
    on_cancel: F2
) -> ComponentOutput
where
    F1: Fn(&mut AppState) + Send + Sync + 'static,
    F2: Fn(&mut AppState) + Send + Sync + 'static,
{
    Components::confirm_modal(title, message, on_confirm, on_cancel)
}

// Visualization builders
pub fn histogram(response_times: &[u128]) -> ComponentOutput {
    Components::response_time_histogram(response_times)
}

pub fn learning_curve(responses: &[Response]) -> ComponentOutput {
    Components::learning_curve_display(responses)
}

pub fn error_analysis(responses: &[Response]) -> ComponentOutput {
    Components::error_analysis_display(responses)
}

pub fn strategy_chart(strategies: Vec<(&str, f64)>) -> ComponentOutput {
    Components::strategy_analysis_display(strategies)
}

// Domain builders
pub fn domain_card(domain: &str, description: &str, selected: bool) -> ComponentOutput {
    Components::domain_card(domain, description, selected)
}

pub fn session_info(
    session_id: &str,
    status: &str,
    duration: &str,
    domain: &str
) -> ComponentOutput {
    Components::session_info(session_id, status, duration, domain)
}

// Task builders
pub fn task_card(prompt: &str, hint: Option<&str>) -> ComponentOutput {
    Components::task_card_display(prompt, hint)
}

pub fn answer_options<F>(options: Vec<String>, on_select: F) -> ComponentOutput
where
    F: Fn(&mut AppState, usize) + Send + Sync + 'static
{
    Components::answer_options_display(options, on_select)
}

pub fn performance_chart(
    accuracy: f64,
    total_responses: usize,
    correct: usize,
    avg_time: f64
) -> ComponentOutput {
    Components::performance_chart(accuracy, total_responses, correct, avg_time)
}

// Layout helpers
pub fn app_scaffold(
    header: ComponentOutput,
    content: ComponentOutput,
    footer: Option<ComponentOutput>
) -> ComponentOutput {
    Components::app_scaffold(header, content, footer)
}

pub fn centered(max_width: f64, child: ComponentOutput) -> ComponentOutput {
    Components::centered_container(max_width, child)
}

pub fn stats_grid(stats: Vec<ComponentOutput>) -> ComponentOutput {
    Components::stats_grid(stats)
}

// Feedback helpers
pub fn spinner(message: Option<&str>) -> ComponentOutput {
    Components::loading_spinner(message)
}

pub fn empty_state(
    icon: &str,
    title: &str,
    message: &str,
    action: Option<ComponentOutput>
) -> ComponentOutput {
    Components::empty_state(icon, title, message, action)
}

pub fn error_banner<F>(message: &str, on_dismiss: F) -> ComponentOutput
where
    F: Fn(&mut AppState) + Send + Sync + 'static
{
    Components::error_banner(message, on_dismiss)
}