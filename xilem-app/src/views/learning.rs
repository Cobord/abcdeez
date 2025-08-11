// Learning interface view

use xilem::core::one_of::{Either, OneOf3};
use xilem::view::*;
use xilem::core::one_of::OneOf5;
use xilem::WidgetView;
use xilem::style::{Style, Background};
use xilem::FontWeight;
use xilem::Color;
use xilem::TextAlign;

use crate::models::{Task, TaskType};
use abcdeez_core::tasks::core::TaskType as CoreTaskType;
use abcdeez_core::tasks::extended::ExtendedTaskType;

// Task visual modules
use super::core_task_visual::core_task_visual;
use super::extended_task_visual::extended_task_visual;
use crate::state::{AppState, LoadingKey, Screen};

pub fn learning_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let main_view = if let Some(session) = &state.session {
        if let Some(task) = &session.current_task {
            OneOf3::A(task_presenter(state, task))
        } else {
            OneOf3::B(loading_task_view(state))
        }
    } else {
        OneOf3::C(no_session_view(state))
    };

    let response_view = if state.session.is_some() {
        Either::A(response_controls(state))
    } else {
        Either::B(sized_box(label("")).height(0.0))
    };

    flex((
        learning_header(state),
        sized_box(main_view).expand().padding(20.0),
        response_view,
        learning_footer(state),
    ))
    .direction(Axis::Vertical)
    .must_fill_major_axis(true)
}

fn learning_header(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        button("← Exit", |state: &mut AppState| {
            state.navigate(Screen::Dashboard);
        })
        .padding(10.0),

        FlexSpacer::Flex(1.0),

        // Progress indicator
        if let Some(session) = &state.session {
            Either::A(
                flex_row((
                    label(format!("Task {}", session.tasks_completed + 1))
                        .text_size(16.0),
                    FlexSpacer::Fixed(20.0),
                    // Simple progress bar representation
                    sized_box(label(""))
                        .width(100.0 * (session.tasks_completed as f64 / 20.0))
                        .height(4.0)
                        .background(Background::Color(state.theme.success_color())),
                ))
            )
        } else {
            Either::B(label("No active session").text_size(14.0))
        },

        FlexSpacer::Flex(1.0),

        button(label("Hint").color(Color::WHITE), |state: &mut AppState| {
            if let Some(session) = &mut state.session {
                session.hint_level += 1;
                // TODO: Request hint from backend
            }
        })
        .padding(10.0)
        .background(Background::Color(state.theme.warning_color()))
        .corner_radius(5.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .padding(15.0)
    .background(Background::Color(state.theme.surface_color()))
}

fn task_presenter(state: &AppState, task: &Task) -> impl WidgetView<AppState> + use<> {
    match &task.task_type {
        TaskType::Core(core_task) => OneOf5::A(core_task_view(state, task, core_task)),
        TaskType::Extended(extended_task) => OneOf5::B(extended_task_view(state, task, extended_task)),
        TaskType::Music(_music_task) => OneOf5::C(label("Music task - Coming soon!")),
        TaskType::Navigation(_nav_task) => OneOf5::D(label("Navigation task - Coming soon!")),
        TaskType::Boundary(_boundary_task) => OneOf5::E(label("Boundary task - Coming soon!")),
    }
}

fn core_task_view(
    state: &AppState,
    task: &Task,
    core_task: &CoreTaskType,
) -> impl WidgetView<AppState> {
    // Clone required data to avoid returning a view that borrows from args
    let prompt_text = task.prompt.clone();
    let options = task.options.clone();
    let surface = state.theme.surface_color();
    let primary = state.theme.primary_color();

    flex((
        // Task prompt
        label(&*prompt_text)
            .text_size(24.0)
            .text_alignment(TextAlign::Center)
            .weight(FontWeight::MEDIUM),

        FlexSpacer::Fixed(40.0),

        // Visual representation based on task type
        core_task_visual(core_task, primary, surface),

        FlexSpacer::Fixed(60.0),

        // Multiple choice options
        {
            let buttons: Vec<_> = options
                .into_iter()
                .enumerate()
                .map(|(i, option)| {
                    let task_val = task.clone();
                    button(
                        option.clone(),
                        move |state: &mut AppState| {
                            submit_response(state, option.clone(), task_val.clone());
                        },
                    )
                    .background(Background::Color(surface))
                    .padding(25.0)
                    .corner_radius(10.0)
                    .grid_pos((i % 2) as i32, (i / 2) as i32)
                })
                .collect();
            let rows = ((buttons.len() + 1) / 2) as i32;
            grid(buttons, 2, rows).spacing(15.0)
        }
        ,
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}

fn loading_task_view(_state: &AppState) -> impl WidgetView<AppState> {
    flex((
        FlexSpacer::Flex(1.0),
        sized_box(spinner()).width(50.0).height(50.0),
        FlexSpacer::Fixed(20.0),
        label("Loading next task...")
            .text_size(16.0)
            .color(Color::from_rgb8(128, 128, 128)),
        FlexSpacer::Flex(1.0),
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}

fn no_session_view(state: &AppState) -> impl WidgetView<AppState> {
    flex((
        FlexSpacer::Flex(1.0),
        label("No active learning session")
            .text_size(20.0)
            .weight(FontWeight::MEDIUM),
        FlexSpacer::Fixed(20.0),
        button(label("Start New Session").color(Color::WHITE), |state: &mut AppState| {
            // TODO: Initialize session with core module
            state.set_loading(LoadingKey::Session, true);
            // For now, create a mock session
            create_mock_session(state);
        })
        .background(Background::Color(state.theme.primary_color()))
        .padding(15.0)
        .corner_radius(10.0),
        FlexSpacer::Flex(1.0),
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}

fn response_controls(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        // Timer display
        label("Time: 0:00")
            .text_size(14.0)
            .color(Color::from_rgb8(128, 128, 128)),
        
        FlexSpacer::Flex(1.0),

        // Skip button
        button("Skip", |state: &mut AppState| {
            // TODO: Skip current task
            load_next_task(state);
        })
        .padding(10.0),
    ))
    .padding(15.0)
    .background(Background::Color(state.theme.surface_color()))
}

fn learning_footer(state: &AppState) -> impl WidgetView<AppState> + use<> {
    if let Some(session) = &state.session {
        let accuracy = calculate_accuracy(&session.performance_buffer);
        let avg_time = calculate_avg_response_time(&session.performance_buffer);

        Either::A(
            flex_row((
                metric_display("Accuracy", &format!("{:.0}%", accuracy * 100.0), state.theme.success_color()),
                metric_display("Avg Time", &format!("{:.0}ms", avg_time), state.theme.primary_color()),
                metric_display("Streak", &format!("{}", session.struggle_indicators.consecutive_errors), state.theme.warning_color()),
            ))
            .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
            .padding(15.0)
            .background(Background::Color(state.theme.surface_color())),
        )
    } else {
        Either::B(sized_box(label("")).height(0.0))
    }
}

fn metric_display(label_text: &str, value: &str, color: Color) -> impl WidgetView<AppState> {
    flex((
        label(label_text)
            .text_size(10.0)
            .color(Color::from_rgb8(128, 128, 128)),
        label(value)
            .text_size(16.0)
            .weight(FontWeight::BOLD)
            .color(color),
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
    
}

// Helper functions

fn submit_response(state: &mut AppState, answer: String, task: Task) {
    if let Some(session) = &mut state.session {
        // Check if answer is correct
        let is_correct = answer == task.correct_answer;

        // Update struggle indicators
        if is_correct {
            session.struggle_indicators.consecutive_errors = 0;
        } else {
            session.struggle_indicators.consecutive_errors += 1;
        }

        // Add to performance buffer
        session.performance_buffer.push(crate::models::ResponseMetrics {
            correct: is_correct,
            response_time_ms: 1000, // TODO: Get actual response time
            timestamp: chrono::Utc::now(),
            difficulty: task.difficulty,
        });

        session.tasks_completed += 1;

        // Load next task
        load_next_task(state);
    }
}

fn load_next_task(state: &mut AppState) {
    // TODO: Request next task from backend
    // For now, create a mock task
    if let Some(session) = &mut state.session {
        session.current_task = Some(create_mock_alphabet_task());
    }
}

fn create_mock_session(state: &mut AppState) {
    use crate::state::SessionState;

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

// Core task visual is now in a separate module to avoid type complexity

fn extended_task_view(
    state: &AppState,
    task: &Task,
    extended_task: &ExtendedTaskType,
) -> impl WidgetView<AppState> {
    // Clone required data
    let prompt_text = task.prompt.clone();
    let options = task.options.clone();
    let surface = state.theme.surface_color();
    let primary = state.theme.primary_color();

    flex((
        // Task prompt
        label(&*prompt_text)
            .text_size(24.0)
            .text_alignment(TextAlign::Center)
            .weight(FontWeight::MEDIUM),

        FlexSpacer::Fixed(40.0),

        // Visual representation based on extended task type
        extended_task_visual(extended_task, primary, surface),

        FlexSpacer::Fixed(60.0),

        // Multiple choice options or input area
        if options.is_empty() {
            Either::A(
                // Free text input for tasks without options
                sized_box(label("Type your answer below"))
                    .padding(20.0)
                    .background(Background::Color(surface))
                    .corner_radius(10.0)
            )
        } else {
            Either::B({
                let buttons: Vec<_> = options
                    .into_iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let task_val = task.clone();
                        button(
                            option.clone(),
                            move |state: &mut AppState| {
                                submit_response(state, option.clone(), task_val.clone());
                            },
                        )
                        .background(Background::Color(surface))
                        .padding(25.0)
                        .corner_radius(10.0)
                        .grid_pos((i % 2) as i32, (i / 2) as i32)
                    })
                    .collect();
                let rows = ((buttons.len() + 1) / 2) as i32;
                grid(buttons, 2, rows).spacing(15.0)
            })
        },
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}

// Extended task visual is now in a separate module to avoid type complexity

fn calculate_accuracy(metrics: &[crate::models::ResponseMetrics]) -> f64 {
    if metrics.is_empty() {
        return 0.0;
    }
    let correct = metrics.iter().filter(|m| m.correct).count() as f64;
    correct / metrics.len() as f64
}

fn calculate_avg_response_time(metrics: &[crate::models::ResponseMetrics]) -> f64 {
    if metrics.is_empty() {
        return 0.0;
    }
    let sum: u128 = metrics.iter().map(|m| m.response_time_ms).sum();
    sum as f64 / metrics.len() as f64
}