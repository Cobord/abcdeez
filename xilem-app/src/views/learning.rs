// Learning view using high-level cross-platform components

use crate::state::{AppState, LoadingKey, Screen, SessionState};
use crate::models::{Task, ResponseMetrics};
use crate::components::{Components, AppComponents, AppColor, ComponentOutput, SpacerSize};
use crate::components::layout::{page_layout, two_column_layout, with_fab};
use crate::components::cards::{stat_card_with_trend, Trend, session_summary_card};
use crate::components::feedback::{progress_indicator, toast, ToastType};
use crate::utils::easter_egg::CrabTrigger;

pub fn learning_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    // Build the main learning interface
    let main_content = if state.session.is_some() {
        let task = state.session.as_ref().unwrap().current_task.clone();
        if let Some(task) = task {
            task_presenter_view(state, &task, window_width)
        } else {
            progress_indicator("Loading next task...", None, false)
        }
    } else {
        no_session_view(state)
    };

    // Add Little Crab overlay if active
    let content = if state.easter_egg_manager.crab.active {
        Components::stack(vec![
            main_content,
            crate::views::dashboard::render_little_crab(state),
        ], 16.0)
    } else {
        main_content
    };

    // Create the header with progress
    let header = learning_header(state);
    
    // Use page layout with FAB for hint button
    let page_content = if state.session.is_some() {
        with_fab(
            content,
            "💡 Hint",
            |state| {
                if let Some(session) = &mut state.session {
                    session.hint_level += 1;
                    session.total_hints_used += 1;
                }
            }
        )
    } else {
        content
    };
    
    // Create the final layout
    page_layout(
        "Learning Session",
        state.session.as_ref().and_then(|s| s.domain.as_deref()),
        Components::simple_flex_column(vec![
            header,
            Components::spacer(SpacerSize::Medium),
            page_content,
            Components::spacer(SpacerSize::Large),
            learning_footer(state),
        ])
    )
}

fn learning_header(state: &AppState) -> ComponentOutput {
    if let Some(session) = &state.session {
        let accuracy = calculate_accuracy(&session.performance_buffer);
        let streak = session.struggle_indicators.consecutive_correct;
        
        // Calculate trend for accuracy
        let trend = if session.performance_buffer.len() > 5 {
            let recent_accuracy = calculate_accuracy(&session.performance_buffer[session.performance_buffer.len()-5..]);
            let diff = ((recent_accuracy - accuracy) * 100.0) as i32;
            if diff > 5 {
                Trend::Up(diff.abs() as u32)
            } else if diff < -5 {
                Trend::Down(diff.abs() as u32)
            } else {
                Trend::Neutral
            }
        } else {
            Trend::Neutral
        };
        
        Components::simple_flex_column(vec![
            // Navigation bar
            Components::simple_flex_row(vec![
                Components::nav_button("← Exit", Screen::Dashboard, false, |state| {
                    state.navigate(Screen::Dashboard);
                }),
                Components::spacer(SpacerSize::Large),
                Components::label(&format!("Task {}/20", session.tasks_completed + 1)),
            ]),
            Components::spacer(SpacerSize::Small),
            // Progress bar
            Components::learning_progress(
                session.tasks_completed + 1,
                20,
                accuracy
            ),
            Components::spacer(SpacerSize::Small),
            // Live stats
            Components::simple_flex_row(vec![
                stat_card_with_trend(
                    &format!("{:.0}%", accuracy * 100.0),
                    "Accuracy",
                    trend,
                    AppColor::Success
                ),
                Components::stat_card(
                    "Streak",
                    &format!("🔥 {}", streak),
                    if streak > 5 { AppColor::Warning } else { AppColor::Secondary }
                ),
                Components::stat_card(
                    "Hints",
                    &format!("{}", session.total_hints_used),
                    AppColor::Info
                ),
            ]),
        ])
    } else {
        Components::empty()
    }
}

fn task_presenter_view(_state: &mut AppState, task: &Task, window_width: f64) -> ComponentOutput {
    let task_clone = task.clone();
    let options = task.options.clone();
    
    // Create task visual based on task type
    let task_visual = create_task_visual(task);
    
    // Task prompt card
    let task_card = Components::card(
        "Current Task",
        Components::simple_flex_column(vec![
            Components::label(&task.prompt),
            Components::spacer(SpacerSize::Medium),
            task_visual,
        ])
    );
    
    // Answer options in a responsive grid
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
    
    // Use two-column layout on larger screens
    if window_width > 768.0 {
        two_column_layout(
            task_card,
            Components::card("Select Answer", task_options),
            window_width
        )
    } else {
        Components::simple_flex_column(vec![
            task_card,
            Components::spacer(SpacerSize::Medium),
            Components::card("Select Answer", task_options),
        ])
    }
}

fn create_task_visual(task: &Task) -> ComponentOutput {
    use crate::models::TaskType;
    use abcdeez_core::tasks::core::TaskType as CoreTaskType;
    
    match &task.task_type {
        TaskType::Core(core_type) => match core_type {
            CoreTaskType::Successor { item } => {
                Components::comparison_visual(item, "?", true)
            },
            CoreTaskType::Predecessor { item } => {
                Components::comparison_visual("?", item, true)
            },
            CoreTaskType::Distance { from, to } => {
                Components::path_visual(from, to, vec!["?".to_string()])
            },
            CoreTaskType::Compare { left, right } => {
                Components::comparison_visual(left, right, false)
            },
            _ => Components::label(&task.prompt),
        },
        _ => Components::label(&task.prompt),
    }
}

fn no_session_view(_state: &AppState) -> ComponentOutput {
    Components::empty_state(
        "📚",
        "Ready to Learn?",
        "Start a new session to practice",
        Some(Components::action_button(
            "Start Learning",
            AppColor::Primary,
            |state| {
                state.set_loading(LoadingKey::Session, true);
                create_mock_session(state);
            }
        ))
    )
}

fn learning_footer(state: &AppState) -> ComponentOutput {
    if let Some(session) = &state.session {
        // Show session summary when enough data
        if session.tasks_completed > 0 {
            let accuracy = calculate_accuracy(&session.performance_buffer);
            let avg_time = calculate_avg_response_time(&session.performance_buffer);
            let duration = chrono::Utc::now().signed_duration_since(session.start_time).num_minutes() as u32;
            
            session_summary_card(
                duration,
                session.tasks_completed as u32,
                accuracy as f32,
                session.tasks_completed as u32 * 10 // Mock XP calculation
            )
        } else {
            Components::empty()
        }
    } else {
        Components::empty()
    }
}

fn submit_response(state: &mut AppState, answer: String, task: Task) {
    if let Some(session) = &mut state.session {
        let is_correct = answer == task.correct_answer;
        
        tracing::debug!(
            task_type = ?task.task_type,
            difficulty = task.difficulty,
            correct = is_correct,
            user_answer = %answer,
            "Response submitted"
        );
        
        // Update streak tracking
        if is_correct {
            session.struggle_indicators.consecutive_correct += 1;
            session.struggle_indicators.consecutive_errors = 0;
            
            // Check for perfect streak achievement
            let streak = session.struggle_indicators.consecutive_correct;
            if streak >= 10 && !state.easter_egg_manager.crab.active {
                tracing::info!(streak = streak, "Perfect streak achieved! Activating Little Crab");
                state.easter_egg_manager.crab.activate(CrabTrigger::PerfectStreak);
            }
            
            // Little Crab celebrates success
            if state.easter_egg_manager.crab.active {
                let accuracy = calculate_accuracy(&session.performance_buffer);
                state.easter_egg_manager.crab.celebrate_success(streak, accuracy as f32);
            }
        } else {
            session.struggle_indicators.consecutive_errors += 1;
            session.struggle_indicators.consecutive_correct = 0;
            
            // Little Crab offers encouragement during struggles
            if state.easter_egg_manager.crab.active && session.struggle_indicators.consecutive_errors >= 3 {
                state.easter_egg_manager.crab.offer_encouragement(session.struggle_indicators.consecutive_errors);
            }
        }
        
        // Calculate actual response time
        let response_time_ms = if let Some(start_time) = state.current_task_start_time {
            start_time.elapsed().as_millis() as u128
        } else {
            1000 // Default fallback
        };
        
        session.performance_buffer.push(ResponseMetrics {
            correct: is_correct,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            difficulty: task.difficulty,
        });
        
        session.tasks_completed += 1;
        
        // Little Crab reacts to domain changes
        if state.easter_egg_manager.crab.active {
            if let Some(domain) = &session.domain {
                state.easter_egg_manager.crab.react_to_domain(&domain.to_string());
            }
        }
        
        load_next_task(state);
    }
}

fn load_next_task(state: &mut AppState) {
    // Set the task start time for response time tracking
    state.current_task_start_time = Some(std::time::Instant::now());
    
    // Request next task from backend
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
        domain: Some("alphabet".to_string()),
        start_time: chrono::Utc::now(),
        tasks_completed: 0,
        current_task: Some(create_mock_alphabet_task()),
        pending_response: None,
        hint_level: 0,
        struggle_indicators: Default::default(),
        performance_buffer: Vec::new(),
        responses_total: 0,
        responses_correct: 0,
        total_hints_used: 0,
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