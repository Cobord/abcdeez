// Learning view using high-level cross-platform components

use crate::state::{AppState, LoadingKey, Screen, SessionState};
use crate::models::{Task, ResponseMetrics};
use crate::components::{Components, AppComponents, AppColor, AppScreen, ComponentOutput};

pub fn learning_view(state: &mut AppState) -> ComponentOutput {
    let header = learning_header(state);
    let footer = learning_footer(state);
    
    let main_content = if state.session.is_some() {
        let task = state.session.as_ref().unwrap().current_task.clone();
        if let Some(task) = task {
            task_presenter_view(state, &task)
        } else {
            Components::loading_spinner(Some("Loading next task..."))
        }
    } else {
        no_session_view(state)
    };
    
    Components::app_scaffold(header, main_content, Some(footer))
}

fn learning_header(state: &AppState) -> ComponentOutput {
    let progress = if let Some(session) = &state.session {
        Components::learning_progress(
            session.tasks_completed + 1,
            20, // Total tasks target
            calculate_accuracy(&session.performance_buffer)
        )
    } else {
        Components::empty_state("", "No active session", "", None)
    };
    
    let hint_button = Components::hint_button(
        state.session.as_ref().map(|s| s.hint_level as usize).unwrap_or(0),
        |state| {
            if let Some(session) = &mut state.session {
                session.hint_level += 1;
                // TODO: Request hint from backend
            }
        }
    );
    
    Components::settings_section(
        "",
        vec![
            Components::nav_button("← Exit", AppScreen::Dashboard, false, |state| {
                state.navigate(Screen::Dashboard);
            }),
            progress,
            hint_button,
        ]
    )
}

fn task_presenter_view(_state: &mut AppState, task: &Task) -> ComponentOutput {
    let task_clone = task.clone();
    let options = task.options.clone();
    
    let task_display = Components::task_presenter(
        task,
        move |state, answer| {
            submit_response(state, answer, task_clone.clone());
        }
    );
    
    let task_options = Components::task_options(
        options,
        move |state, index| {
            let answer = state.session.as_ref()
                .and_then(|s| s.current_task.as_ref())
                .map(|t| t.options[index].clone())
                .unwrap_or_default();
            
            if let Some(task) = state.session.as_ref().and_then(|s| s.current_task.as_ref()) {
                submit_response(state, answer, task.clone());
            }
        }
    );
    
    Components::settings_section(
        "",
        vec![task_display, task_options]
    )
}

fn no_session_view(_state: &AppState) -> ComponentOutput {
    let start_button = Components::welcome_card(
        "",
        "No active learning session",
        |state| {
            state.set_loading(LoadingKey::Session, true);
            create_mock_session(state);
        }
    );
    
    Components::empty_state(
        "📚",
        "Ready to Learn?",
        "Start a new session to practice",
        Some(start_button)
    )
}

fn learning_footer(state: &AppState) -> ComponentOutput {
    if let Some(session) = &state.session {
        let accuracy = calculate_accuracy(&session.performance_buffer);
        let avg_time = calculate_avg_response_time(&session.performance_buffer);
        
        Components::stats_grid(vec![
            Components::stat_card("Accuracy", &format!("{:.0}%", accuracy * 100.0), AppColor::Success),
            Components::stat_card("Avg Time", &format!("{:.0}ms", avg_time), AppColor::Primary),
            Components::stat_card("Streak", &format!("{}", session.struggle_indicators.consecutive_errors), AppColor::Warning),
        ])
    } else {
        Components::empty_state("", "", "", None)
    }
}

fn submit_response(state: &mut AppState, answer: String, task: Task) {
    if let Some(session) = &mut state.session {
        let is_correct = answer == task.correct_answer;
        
        if is_correct {
            session.struggle_indicators.consecutive_errors = 0;
        } else {
            session.struggle_indicators.consecutive_errors += 1;
        }
        
        session.performance_buffer.push(ResponseMetrics {
            correct: is_correct,
            response_time_ms: 1000, // TODO: Get actual response time
            timestamp: chrono::Utc::now(),
            difficulty: task.difficulty,
        });
        
        session.tasks_completed += 1;
        load_next_task(state);
    }
}

fn load_next_task(state: &mut AppState) {
    // TODO: Request next task from backend
    if let Some(session) = &mut state.session {
        session.current_task = Some(create_mock_alphabet_task());
    }
}

fn create_mock_session(state: &mut AppState) {
    let session = SessionState {
        session_id: uuid::Uuid::new_v4(),
        learner_id: uuid::Uuid::new_v4(),
        topology: abcdeez_core::core::Topology::new_linear(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
            "F".to_string(),
            "G".to_string(),
        ]),
        start_time: chrono::Utc::now(),
        tasks_completed: 0,
        current_task: Some(create_mock_alphabet_task()),
        pending_response: None,
        hint_level: 0,
        struggle_indicators: Default::default(),
        performance_buffer: Vec::new(),
    };
    
    state.session = Some(session);
    state.set_loading(LoadingKey::Session, false);
}

fn create_mock_alphabet_task() -> Task {
    use abcdeez_core::tasks::core::TaskType as CoreTaskType;
    use crate::models::TaskType;
    
    Task {
        task_type: TaskType::Core(CoreTaskType::Successor {
            item: "B".to_string(),
        }),
        prompt: "What letter comes after B?".to_string(),
        correct_answer: "C".to_string(),
        options: vec!["A".to_string(), "C".to_string(), "D".to_string(), "E".to_string()],
        difficulty: 0.3,
        operation: "successor".to_string(),
    }
}

fn calculate_accuracy(metrics: &[ResponseMetrics]) -> f64 {
    if metrics.is_empty() {
        return 0.0;
    }
    let correct = metrics.iter().filter(|m| m.correct).count() as f64;
    correct / metrics.len() as f64
}

fn calculate_avg_response_time(metrics: &[ResponseMetrics]) -> f64 {
    if metrics.is_empty() {
        return 0.0;
    }
    let sum: u128 = metrics.iter().map(|m| m.response_time_ms).sum();
    sum as f64 / metrics.len() as f64
}