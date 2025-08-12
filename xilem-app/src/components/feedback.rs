// Feedback components for user notifications and status

use crate::state::AppState;
use crate::components::{AppColor, AppComponents, ComponentOutput, Components, SpacerSize};

/// Create a toast notification
pub fn toast<F>(
    message: &str,
    toast_type: ToastType,
    duration_ms: Option<u32>,
    on_dismiss: Option<F>,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let (icon, color) = match toast_type {
        ToastType::Success => ("✓", AppColor::Success),
        ToastType::Error => ("✕", AppColor::Error),
        ToastType::Warning => ("⚠", AppColor::Warning),
        ToastType::Info => ("ℹ", AppColor::Info),
    };
    
    let mut items = vec![
        Components::label(icon),
        Components::label(message),
    ];
    
    if let Some(on_dismiss) = on_dismiss {
        items.push(Components::simple_button("×", on_dismiss));
    }
    
    // Note: duration would be handled by the platform layer
    Components::simple_flex_row(items)
}

#[derive(Debug, Clone, Copy)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// Create a progress indicator with message
pub fn progress_indicator(
    message: &str,
    progress: Option<f32>,
    show_percentage: bool,
) -> ComponentOutput {
    let mut items = vec![
        Components::loading_spinner(Some(message)),
    ];
    
    if let Some(progress) = progress {
        items.push(Components::progress_bar(progress as f64, ""));
        if show_percentage {
            items.push(Components::label(&format!("{}%", (progress * 100.0) as u32)));
        }
    }
    
    Components::simple_flex_column(items)
}

/// Create a skeleton loader for content
pub fn skeleton_loader(
    lines: usize,
    show_avatar: bool,
) -> ComponentOutput {
    let mut items = vec![];
    
    if show_avatar {
        items.push(Components::label("⭕"));  // Placeholder for avatar
    }
    
    for _ in 0..lines {
        items.push(Components::label("████████████"));  // Placeholder for text
    }
    
    Components::simple_flex_column(items)
}

/// Create an alert banner
pub fn alert(
    title: &str,
    message: &str,
    alert_type: AlertType,
    actions: Vec<AlertAction>,
) -> ComponentOutput {
    let icon = match alert_type {
        AlertType::Success => "✓",
        AlertType::Error => "✕",
        AlertType::Warning => "⚠",
        AlertType::Info => "ℹ",
    };
    
    let mut items = vec![
        Components::simple_flex_row(vec![
            Components::label(icon),
            Components::label(title),
        ]),
        Components::label(message),
    ];
    
    if !actions.is_empty() {
        let action_buttons: Vec<ComponentOutput> = actions.into_iter()
            .map(|action| {
                Components::action_button(
                    &action.label,
                    action.color,
                    action.on_click
                )
            })
            .collect();
        
        items.push(Components::spacer(SpacerSize::Small));
        items.push(Components::simple_flex_row(action_buttons));
    }
    
    Components::card(title, Components::simple_flex_column(items))
}

#[derive(Debug, Clone, Copy)]
pub enum AlertType {
    Success,
    Error,
    Warning,
    Info,
}

pub struct AlertAction {
    pub label: String,
    pub color: AppColor,
    pub on_click: Box<dyn Fn(&mut AppState) + Send + Sync>,
}

/// Create a confirmation dialog
pub fn confirmation_dialog<F1, F2>(
    title: &str,
    message: &str,
    confirm_label: &str,
    cancel_label: &str,
    is_destructive: bool,
    on_confirm: F1,
    on_cancel: F2,
) -> ComponentOutput 
where
    F1: Fn(&mut AppState) + Send + Sync + 'static,
    F2: Fn(&mut AppState) + Send + Sync + 'static,
{
    let confirm_color = if is_destructive {
        AppColor::Error
    } else {
        AppColor::Primary
    };
    
    let items = vec![
        Components::label(title),
        Components::label(message),
        Components::spacer(SpacerSize::Medium),
        Components::simple_flex_row(vec![
            Components::action_button(cancel_label, AppColor::Secondary, on_cancel),
            Components::action_button(confirm_label, confirm_color, on_confirm),
        ]),
    ];
    
    Components::card("", Components::simple_flex_column(items))
}

/// Create a snackbar notification
pub fn snackbar<F>(
    message: &str,
    action_label: Option<&str>,
    on_action: Option<F>,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let mut items = vec![
        Components::label(message),
    ];
    
    if let (Some(label), Some(on_action)) = (action_label, on_action) {
        items.push(Components::simple_button(label, on_action));
    }
    
    Components::simple_flex_row(items)
}

/// Create a status badge
pub fn status_badge(
    text: &str,
    status: StatusType,
) -> ComponentOutput {
    let (prefix, _color) = match status {
        StatusType::Active => ("🟢", AppColor::Success),
        StatusType::Inactive => ("🔴", AppColor::Error),
        StatusType::Pending => ("🟡", AppColor::Warning),
        StatusType::Completed => ("✓", AppColor::Success),
    };
    
    Components::label(&format!("{} {}", prefix, text))
}

#[derive(Debug, Clone, Copy)]
pub enum StatusType {
    Active,
    Inactive,
    Pending,
    Completed,
}

/// Create a loading overlay
pub fn loading_overlay_with_progress<F>(
    title: &str,
    message: &str,
    progress: Option<f32>,
    can_cancel: bool,
    on_cancel: Option<F>,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let mut items = vec![
        Components::label(title),
        Components::loading_spinner(Some(message)),
    ];
    
    if let Some(progress) = progress {
        items.push(Components::progress_bar(progress as f64, ""));
    }
    
    if can_cancel {
        if let Some(on_cancel) = on_cancel {
            items.push(Components::spacer(SpacerSize::Medium));
            items.push(Components::action_button("Cancel", AppColor::Secondary, on_cancel));
        }
    }
    
    Components::card("", Components::simple_flex_column(items))
}

/// Create a help tooltip
pub fn help_tooltip(
    content: &str,
    help_text: &str,
) -> ComponentOutput {
    // Would show help_text on hover/tap
    Components::simple_flex_row(vec![
        Components::label(content),
        Components::label("?"),  // Help icon
    ])
}

/// Create an inline error message
pub fn inline_error(
    message: &str,
) -> ComponentOutput {
    Components::simple_flex_row(vec![
        Components::label("⚠"),
        Components::label(message),
    ])
}

/// Create a success indicator
pub fn success_indicator(
    message: &str,
    show_confetti: bool,
) -> ComponentOutput {
    let mut items = vec![
        Components::label("✓"),
        Components::label(message),
    ];
    
    if show_confetti {
        items.push(Components::label("🎉"));
    }
    
    Components::simple_flex_row(items)
}

/// Create a step indicator for multi-step processes
pub fn step_indicator(
    current_step: usize,
    total_steps: usize,
    step_labels: Option<Vec<&str>>,
) -> ComponentOutput {
    let mut items = vec![];
    
    for i in 0..total_steps {
        let is_current = i == current_step;
        let is_completed = i < current_step;
        
        let indicator = if is_completed {
            "✓"
        } else if is_current {
            &(i + 1).to_string()
        } else {
            "○"
        };
        
        items.push(Components::label(indicator));
        
        if let Some(ref labels) = step_labels {
            if let Some(label) = labels.get(i) {
                items.push(Components::label(label));
            }
        }
        
        if i < total_steps - 1 {
            items.push(Components::label("→"));
        }
    }
    
    Components::simple_flex_row(items)
}