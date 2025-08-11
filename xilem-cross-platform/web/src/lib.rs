use wasm_bindgen::prelude::*;
use xilem_web::{
    document_body,
    elements::html::{button, div, h1, h2, h3, p},
    interfaces::{Element as _, HtmlButtonElement, HtmlDivElement},
    App, DomView,
};
use xilem_core::one_of::OneOf4;

// Import the shared app state and views
use abcdeez_core::{
    learning::adaptive::AdaptiveScheduler,
    learning::learner::LearnerModel,
    tasks::core::{TaskGenerator},
    core::topology::Topology,
};

#[derive(Debug)]
pub struct AppState {
    current_screen: Screen,
    selected_domain: Option<String>,
    total_trials: usize,
    correct_trials: usize,
    current_task: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Welcome,
    DomainSelection,
    Training,
    Dashboard,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Welcome,
            selected_domain: None,
            total_trials: 0,
            correct_trials: 0,
            current_task: Some("A _ C".to_string()),
        }
    }
}

fn app_view(state: &mut AppState) -> impl HtmlDivElement<AppState> {
    div((
        match state.current_screen.clone() {
            Screen::Welcome => OneOf4::A(welcome_view(state)),
            Screen::DomainSelection => OneOf4::B(domain_selection_view(state)),
            Screen::Training => OneOf4::C(training_view(state)),
            Screen::Dashboard => OneOf4::D(dashboard_view(state)),
        },
    ))
    .class("container")
}

fn welcome_view(_state: &mut AppState) -> impl HtmlDivElement<AppState> {
    div((
        div((
            h1("Welcome to ABCDEEZ"),
            p("Adaptive learning system for cognitive skill acquisition"),
            div((
                button("Start as Guest")
                    .on_click(|state: &mut AppState, _| {
                        state.current_screen = Screen::DomainSelection;
                    })
                    .class("button"),
                button("View Dashboard")
                    .on_click(|state: &mut AppState, _| {
                        state.current_screen = Screen::Dashboard;
                    })
                    .class("button secondary"),
            ))
            .class("button-group"),
        ))
        .class("card"),
    ))
}

fn domain_selection_view(_state: &mut AppState) -> impl HtmlDivElement<AppState> {
    div((
        div((
            h2("Select Learning Domain"),
            div((
                domain_card("alphabet", "🔤 Alphabet", "Learn letter sequences"),
                domain_card("numbers", "🔢 Numbers", "Practice numerical patterns"),
                domain_card("music", "🎵 Music", "Musical note training"),
                domain_card("days", "📅 Days", "Days of the week"),
            ))
            .class("domain-grid"),
        ))
        .class("card"),
    ))
}

fn domain_card(domain: &'static str, title: &'static str, description: &'static str) -> impl HtmlDivElement<AppState> {
    div((
        h3(title),
        p(description),
    ))
    .class("domain-card")
    .on_click(move |state: &mut AppState, _| {
        state.selected_domain = Some(domain.to_string());
        state.current_screen = Screen::Training;
    })
}

fn training_view(state: &mut AppState) -> impl HtmlDivElement<AppState> {
    let task_display = state.current_task.as_deref().unwrap_or("Press Start").to_string();
    let accuracy = if state.total_trials > 0 {
        (state.correct_trials as f64 / state.total_trials as f64) * 100.0
    } else {
        0.0
    };
    let domain_name = state.selected_domain.as_deref().unwrap_or("None").to_string();
    
    div((
        div((
            h2(format!("Training: {}", domain_name)),
            div(task_display)
                .class("task-display"),
            div(
                generate_response_buttons()
            )
            .class("response-grid"),
            div((
                div(format!("Accuracy: {:.1}%", accuracy)),
                div(
                    div("")
                        .class("progress-fill")
                )
                .class("progress-bar"),
            ))
            .class("progress"),
        ))
        .class("card"),
    ))
}

fn generate_response_buttons() -> impl DomView<AppState> {
    div(
        ('A'..='F').map(|c| {
            button(c.to_string())
                .class("response-btn")
                .on_click(move |state: &mut AppState, _| {
                    submit_response(state, c);
                })
        }).collect::<Vec<_>>()
    )
}

fn dashboard_view(state: &mut AppState) -> impl HtmlDivElement<AppState> {
    let accuracy = if state.total_trials > 0 {
        (state.correct_trials as f64 / state.total_trials as f64) * 100.0
    } else {
        0.0
    };
    
    div((
        div((
            h2("Performance Dashboard"),
            div((
                metric_card(&state.total_trials.to_string(), "Total Trials"),
                metric_card(&state.correct_trials.to_string(), "Correct"),
                metric_card(&format!("{:.1}%", accuracy), "Accuracy"),
            ))
            .class("metrics"),
            button("Back to Welcome")
                .on_click(|state: &mut AppState, _| {
                    state.current_screen = Screen::Welcome;
                })
                .class("button"),
        ))
        .class("card"),
    ))
}

fn metric_card(value: &str, label: &str) -> impl HtmlDivElement<AppState> {
    div((
        div(value.to_string()).class("metric-value"),
        div(label.to_string()).class("metric-label"),
    ))
    .class("metric-card")
}

fn submit_response(state: &mut AppState, response: char) {
    state.total_trials += 1;
    
    // Simple check - in real app would check against actual task
    if response == 'B' {
        state.correct_trials += 1;
    }
    
    // Generate new task
    state.current_task = Some(format!("{} _ {}", 
        (b'A' + (state.total_trials % 26) as u8) as char,
        (b'C' + (state.total_trials % 24) as u8) as char
    ));
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    App::new(
        document_body(),
        AppState::default(),
        app_view,
    ).run();
}