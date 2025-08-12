// Form components for user input and settings

use crate::state::AppState;
use crate::components::{AppColor, AppComponents, ComponentOutput, Components, SpacerSize};

/// Create a form field with label and input
pub fn form_field<F>(
    label: &str,
    value: String,
    placeholder: Option<&str>,
    field_type: FieldType,
    on_change: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState, String) + Send + Sync + 'static,
{
    let input = match field_type {
        FieldType::Text | FieldType::Email | FieldType::Password => {
            Components::labeled_input(
                placeholder.unwrap_or(label),
                value,
                on_change
            )
        }
        FieldType::Number => {
            Components::labeled_input(
                placeholder.unwrap_or("0"),
                value,
                on_change
            )
        }
        FieldType::TextArea => {
            // For now, use regular input (would need textarea component)
            Components::labeled_input(
                placeholder.unwrap_or(label),
                value,
                on_change
            )
        }
    };
    
    Components::setting_row(label, input)
}

#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
    TextArea,
}

/// Create a toggle switch component
pub fn toggle_switch<F>(
    label: &str,
    is_on: bool,
    on_toggle: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState, bool) + Send + Sync + 'static,
{
    Components::checkbox(is_on, label, on_toggle)
}

/// Create a slider component
pub fn slider<F>(
    label: &str,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    on_change: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState, f32) + Send + Sync + 'static,
{
    // For now, show value as label (would need proper slider component)
    let display = format!("{}: {:.1}", label, value);
    Components::labeled_input(
        &display,
        value.to_string(),
        move |state, val_str| {
            if let Ok(val) = val_str.parse::<f32>() {
                let clamped = val.clamp(min, max);
                let stepped = ((clamped / step).round() * step);
                on_change(state, stepped);
            }
        }
    )
}

/// Create a radio button group
pub fn radio_group<T, F>(
    label: &str,
    options: Vec<(T, String)>,
    selected: &T,
    on_select: F,
) -> ComponentOutput 
where
    T: Clone + PartialEq + ToString + Send + Sync + 'static,
    F: Fn(&mut AppState, T) + Send + Sync + 'static,
{
    let on_select = std::sync::Arc::new(on_select);
    
    let mut items = vec![Components::label(label)];
    
    for (value, label) in options {
        let is_selected = &value == selected;
        let marker = if is_selected { "◉" } else { "◯" };
        let value_clone = value.clone();
        let on_select_clone = on_select.clone();
        
        let radio = Components::simple_button(
            &format!("{} {}", marker, label),
            move |state| {
                on_select_clone(state, value_clone.clone());
            }
        );
        items.push(radio);
    }
    
    Components::simple_flex_column(items)
}

/// Create a dropdown/select component
pub fn dropdown<T, F>(
    label: &str,
    options: Vec<(T, String)>,
    selected: Option<&T>,
    placeholder: Option<&str>,
    on_select: F,
) -> ComponentOutput 
where
    T: Clone + ToString + Send + Sync + 'static,
    F: Fn(&mut AppState, T) + Send + Sync + 'static,
{
    let selected_label = selected
        .and_then(|s| options.iter().find(|(v, _)| v.to_string() == s.to_string()))
        .map(|(_, l)| l.as_str())
        .unwrap_or(placeholder.unwrap_or("Select..."));
    
    // For now, show as button that would open dropdown
    Components::setting_row(
        label,
        Components::simple_button(selected_label, move |_state| {
            // Would open dropdown menu
        })
    )
}

/// Create a date picker component
pub fn date_picker<F>(
    label: &str,
    value: Option<String>,  // ISO date string
    on_change: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState, String) + Send + Sync + 'static,
{
    let display_value = value.unwrap_or_else(|| "Select date...".to_string());
    
    Components::setting_row(
        label,
        Components::labeled_input(&display_value, display_value.clone(), on_change)
    )
}

/// Create a time picker component
pub fn time_picker<F>(
    label: &str,
    value: Option<String>,  // HH:MM format
    on_change: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState, String) + Send + Sync + 'static,
{
    let display_value = value.unwrap_or_else(|| "00:00".to_string());
    
    Components::setting_row(
        label,
        Components::labeled_input(&display_value, display_value.clone(), on_change)
    )
}

/// Create a multi-select component
pub fn multi_select<T, F>(
    label: &str,
    options: Vec<(T, String)>,
    selected: Vec<T>,
    on_change: F,
) -> ComponentOutput 
where
    T: Clone + ToString + Send + Sync + 'static,
    F: Fn(&mut AppState, Vec<T>) + Send + Sync + 'static,
{
    let on_change = std::sync::Arc::new(on_change);
    
    let mut items = vec![Components::label(label)];
    
    for (value, label) in options {
        let is_selected = selected.iter().any(|s| s.to_string() == value.to_string());
        let marker = if is_selected { "☑" } else { "☐" };
        let value_clone = value.clone();
        let selected_clone = selected.clone();
        let on_change_clone = on_change.clone();
        
        let checkbox = Components::simple_button(
            &format!("{} {}", marker, label),
            move |state| {
                let mut new_selected = selected_clone.clone();
                if is_selected {
                    new_selected.retain(|s| s.to_string() != value_clone.to_string());
                } else {
                    new_selected.push(value_clone.clone());
                }
                on_change_clone(state, new_selected);
            }
        );
        items.push(checkbox);
    }
    
    Components::simple_flex_column(items)
}

/// Create a file upload component
pub fn file_upload<F>(
    label: &str,
    accepted_types: Vec<&str>,
    on_upload: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState, String) + Send + Sync + 'static,
{
    let accept_string = accepted_types.join(", ");
    
    Components::setting_row(
        label,
        Components::action_button(
            &format!("Choose File ({})", accept_string),
            AppColor::Secondary,
            move |state| {
                // Would trigger file picker
                on_upload(state, "file_path".to_string());
            }
        )
    )
}

/// Create a color picker component
pub fn color_picker<F>(
    label: &str,
    value: String,  // Hex color
    on_change: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState, String) + Send + Sync + 'static,
{
    Components::setting_row(
        label,
        Components::labeled_input(&value, value.clone(), on_change)
    )
}

/// Create a form validation message
pub fn validation_message(
    field: &str,
    error: Option<&str>,
) -> ComponentOutput {
    if let Some(error) = error {
        Components::error_message(Some(format!("{}: {}", field, error)))
    } else {
        Components::empty()
    }
}

/// Create a complete form with validation
pub fn form_with_validation<F>(
    fields: Vec<FormField>,
    submit_label: &str,
    on_submit: F,
    errors: Vec<(&str, Option<&str>)>,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let mut items = vec![];
    
    // Add form fields
    for field in fields {
        items.push(field.render());
    }
    
    // Add validation messages
    for (field, error) in errors {
        if error.is_some() {
            items.push(validation_message(field, error));
        }
    }
    
    // Add submit button
    items.push(Components::spacer(SpacerSize::Medium));
    items.push(Components::action_button(submit_label, AppColor::Primary, on_submit));
    
    Components::simple_flex_column(items)
}

pub struct FormField {
    pub label: String,
    pub value: String,
    pub field_type: FieldType,
    pub required: bool,
    pub placeholder: Option<String>,
}

impl FormField {
    fn render(self) -> ComponentOutput {
        let label = if self.required {
            format!("{} *", self.label)
        } else {
            self.label.clone()
        };
        
        form_field(
            &label,
            self.value,
            self.placeholder.as_deref(),
            self.field_type,
            |_state, _value| {
                // Handle change
            }
        )
    }
}