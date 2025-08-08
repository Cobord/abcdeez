use xilem::{
    view::{button, flex, label, prose, text_input, Axis},
    Color, WidgetView,
};

use crate::{AppData, Screen, components::*, models::*};
use graph_learning_core::LearnerMetrics;

// Welcome/Login Screen
pub fn welcome_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Adaptive Learning System")
            .color(Color::rgb8(0, 128, 255)),
        
        prose("An intelligent learning system that adapts to your knowledge and optimizes your learning path using graph-based cognitive models.")
            .alignment(TextAlignment::Middle),
        
        card("Login", flex((
            labeled_input(
                "Username:", 
                data.username_input.clone(),
                |data: &mut AppData, value| {
                    data.username_input = value;
                }
            ),
            labeled_input(
                "Password:", 
                data.password_input.clone(),
                |data: &mut AppData, value| {
                    data.password_input = value;
                }
            ),
            button("Login", |data: &mut AppData| {
                data.login();
            }),
        )).direction(Axis::Vertical)),
        
        card("Quick Start", flex((
            prose("Start learning immediately without creating an account")
                .alignment(TextAlignment::Middle),
            button("Start as Guest", |data: &mut AppData| {
                // Create a guest user and learner
                data.current_user = Some(User {
                    id: uuid::Uuid::new_v4().to_string(),
                    username: "Guest".to_string(),
                    email: "guest@example.com".to_string(),
                    token: None,
                });
                data.create_learner();
                data.current_screen = Screen::DomainSelection;
            }),
        )).direction(Axis::Vertical)),
    ))
    .direction(Axis::Vertical)
}

// Domain Selection Screen
pub fn domain_selection_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let domains = vec![
        Domain::Alphabet,
        Domain::DaysOfWeek,
        Domain::Music,
        Domain::Mathematics,
    ];
    
    let domain_cards = domains
        .into_iter()
        .map(|domain| {
            let is_selected = data.selected_domain == domain;
            let domain_clone = domain.clone();
            
            card(
                domain.display_name(),
                flex((
                    prose(domain.description())
                        .alignment(TextAlignment::Start),
                    if is_selected {
                        label("Selected")
                            .brush(Color::from_rgb8(0, 200, 0))
                            .alignment(TextAlignment::Middle)
                    } else {
                        button("Select", move |data: &mut AppData| {
                            data.selected_domain = domain_clone.clone();
                            // Recreate learner with new topology
                            data.create_learner();
                        })
                    },
                ))
                .direction(Axis::Vertical)
            )
        })
        .collect::<Vec<_>>();
    
    flex((
        label("Select Learning Domain")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        prose("Choose a domain to begin your adaptive training session")
            .alignment(TextAlignment::Middle),
        
        flex(domain_cards)
            .direction(Axis::Vertical),
        
        card("Session Settings", flex((
            checkbox(
                data.use_adaptive_scheduling,
                "Use Adaptive Scheduling (AI-powered task selection)",
                |data: &mut AppData, checked| {
                    data.use_adaptive_scheduling = checked;
                }
            ),
            checkbox(
                data.enable_hints,
                "Enable Hints (Show help when struggling)",
                |data: &mut AppData, checked| {
                    data.enable_hints = checked;
                }
            ),
        )).direction(Axis::Vertical)),
        
        button("Start Training Session", |data: &mut AppData| {
            data.start_session();
        }),
    ))
    .direction(Axis::Vertical)
}

// Training Session Screen
pub fn training_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    // Session progress bar
    let progress = if data.session_responses.len() > 0 {
        let correct = data.session_responses.iter().filter(|r| r.correct).count();
        correct as f64 / data.session_responses.len() as f64
    } else {
        0.0
    };
    
    let progress_display = card("Session Progress", flex((
        flex((
            metric_display(
                "Tasks Completed:",
                data.session_responses.len().to_string(),
                Color::from_rgb8(0, 128, 255)
            ),
            metric_display(
                "Accuracy:",
                format!("{:.1}%", data.current_metrics.accuracy_rate * 100.0),
                if data.current_metrics.accuracy_rate >= 0.7 {
                    Color::from_rgb8(0, 200, 0)
                } else {
                    Color::from_rgb8(255, 128, 0)
                }
            ),
            metric_display(
                "Streak:",
                format!("{} (Best: {})", data.current_metrics.streak_count, data.current_metrics.best_streak),
                Color::from_rgb8(255, 128, 0)
            ),
        ))
        .direction(Axis::Horizontal),
        progress_bar(progress, "Overall Progress".to_string()),
    ))
    .direction(Axis::Vertical));
    
    // Task display with feedback
    let task_display = if let Some(ui_task) = &data.current_task {
        let options_display = if data.show_feedback {
            // Show feedback after answer submission
            let feedback_color = if data.last_response_correct {
                Color::from_rgb8(0, 200, 0)
            } else {
                Color::from_rgb8(255, 0, 0)
            };
            
            let feedback_message = if data.last_response_correct {
                "Correct! Well done!"
            } else {
                &format!("Incorrect. The correct answer was: {}", ui_task.core_task.correct_answer)
            };
            
            flex((
                label(feedback_message)
                    .brush(feedback_color)
                    .alignment(TextAlignment::Middle),
                button("Continue", |data: &mut AppData| {
                    data.continue_to_next_task();
                }),
            ))
            .direction(Axis::Vertical)
        } else {
            // Show answer options
            let option_buttons = ui_task.display_options
                .iter()
                .enumerate()
                .map(|(index, option)| {
                    let option_text = option.clone();
                    button(option_text, move |data: &mut AppData| {
                        data.submit_answer(index);
                    })
                })
                .collect::<Vec<_>>();
            
            flex(option_buttons)
                .direction(Axis::Vertical)
        };
        
        card("Current Task", flex((
            label(&ui_task.display_prompt)
                .alignment(TextAlignment::Middle),
            label(format!("Difficulty: {:.1}", ui_task.core_task.difficulty))
                .brush(Color::from_rgb8(128, 128, 128))
                .alignment(TextAlignment::Middle),
            options_display,
        ))
        .direction(Axis::Vertical))
    } else {
        card("Loading", label("Generating next task...")
            .alignment(TextAlignment::Middle))
    };
    
    // Learner metrics display (if available)
    let metrics_display = if let Some(learner) = &data.current_learner {
        if let Some(core_metrics) = &data.current_metrics.core_metrics {
            card("Learning Metrics", flex((
                metric_display(
                    "Bidirectionality:",
                    format!("{:.3}", core_metrics.bidirectionality_index),
                    Color::from_rgb8(0, 128, 255)
                ),
                metric_display(
                    "Symbolic Distance:",
                    format!("{:.3}", core_metrics.symbolic_distance_slope),
                    Color::from_rgb8(128, 0, 255)
                ),
                metric_display(
                    "Chunk Boundary:",
                    format!("{:.3}", core_metrics.chunk_boundary_penalty),
                    Color::from_rgb8(255, 128, 0)
                ),
            ))
            .direction(Axis::Vertical))
        } else {
            label("Metrics will appear after a few responses")
                .alignment(TextAlignment::Middle)
        }
    } else {
        label("")
            .alignment(TextAlignment::Middle)
    };
    
    flex((
        label("Training Session")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        progress_display,
        task_display,
        metrics_display,
        
        flex((
            button("Pause Session", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
                data.success_message = Some("Session paused. You can resume anytime.".to_string());
            }),
            button("End Session", |data: &mut AppData| {
                data.end_session();
            }),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}

// Performance Dashboard Screen
pub fn dashboard_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    // Overall performance metrics
    let overall_metrics = card("Overall Performance", flex((
        metric_display(
            "Total Tasks:",
            data.current_metrics.total_responses.to_string(),
            Color::from_rgb8(0, 128, 255)
        ),
        metric_display(
            "Accuracy:",
            format!("{:.1}%", data.current_metrics.accuracy_rate * 100.0),
            Color::from_rgb8(0, 200, 0)
        ),
        metric_display(
            "Avg Response Time:",
            format!("{:.0}ms", data.current_metrics.average_response_time_ms),
            Color::from_rgb8(255, 128, 0)
        ),
        metric_display(
            "Best Streak:",
            data.current_metrics.best_streak.to_string(),
            Color::from_rgb8(128, 0, 255)
        ),
    ))
    .direction(Axis::Vertical));
    
    // Core learning metrics
    let learning_metrics = if let Some(core_metrics) = &data.current_metrics.core_metrics {
        card("Learning Analytics", flex((
            label("Cognitive Metrics")
                .brush(Color::from_rgb8(64, 64, 64))
                .alignment(TextAlignment::Start),
            metric_display(
                "Bidirectionality Index:",
                format!("{:.3}", core_metrics.bidirectionality_index),
                Color::from_rgb8(0, 128, 255)
            ),
            prose("Measures how well you can navigate forward and backward in the sequence")
                .alignment(TextAlignment::Start),
            
            metric_display(
                "Symbolic Distance Slope:",
                format!("{:.3}", core_metrics.symbolic_distance_slope),
                Color::from_rgb8(128, 0, 255)
            ),
            prose("Shows how distance affects your performance on comparison tasks")
                .alignment(TextAlignment::Start),
            
            metric_display(
                "Chunk Boundary Penalty:",
                format!("{:.3}", core_metrics.chunk_boundary_penalty),
                Color::from_rgb8(255, 128, 0)
            ),
            prose("Indicates difficulty crossing conceptual boundaries in the domain")
                .alignment(TextAlignment::Start),
        ))
        .direction(Axis::Vertical))
    } else {
        card("Learning Analytics", 
            label("Complete more tasks to see detailed analytics")
                .alignment(TextAlignment::Middle))
    };
    
    // Operation proficiencies
    let proficiencies = if let Some(learner) = &data.current_learner {
        let profs = &learner.core_model.operation_proficiencies;
        let prof_displays = profs
            .values()
            .take(5)
            .map(|prof| {
                let proficiency = (prof.theta + 2.0) / 4.0; // Normalize from [-2, 2] to [0, 1]
                progress_bar(
                    proficiency.min(1.0).max(0.0),
                    format!("{:?} ({}x)", prof.operation, prof.practice_count)
                )
            })
            .collect::<Vec<_>>();
        
        card("Operation Proficiencies", flex(prof_displays)
            .direction(Axis::Vertical))
    } else {
        card("Operation Proficiencies",
            label("No proficiency data yet")
                .alignment(TextAlignment::Middle))
    };
    
    // Recent responses
    let recent_responses = data.session_responses
        .iter()
        .rev()
        .take(5)
        .map(|response| {
            let status = if response.correct { "✓" } else { "✗" };
            let color = if response.correct {
                Color::from_rgb8(0, 200, 0)
            } else {
                Color::from_rgb8(255, 0, 0)
            };
            
            label(format!("{} {} - {}ms",
                status,
                response.task.prompt.chars().take(30).collect::<String>(),
                response.response_time_ms
            ))
            .brush(color)
            .alignment(TextAlignment::Start)
        })
        .collect::<Vec<_>>();
    
    let recent_display = if recent_responses.is_empty() {
        card("Recent Activity",
            label("No responses yet")
                .alignment(TextAlignment::Middle))
    } else {
        card("Recent Activity", flex(recent_responses)
            .direction(Axis::Vertical))
    };
    
    flex((
        label("Performance Dashboard")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        overall_metrics,
        learning_metrics,
        proficiencies,
        recent_display,
        
        flex((
            if data.current_session.is_some() && data.current_session.as_ref().unwrap().status == "active" {
                button("Resume Session", |data: &mut AppData| {
                    data.current_screen = Screen::Training;
                })
            } else {
                button("New Session", |data: &mut AppData| {
                    data.current_screen = Screen::DomainSelection;
                })
            },
            button("Export Data", |data: &mut AppData| {
                data.export_current_data();
            }),
            button("Settings", |data: &mut AppData| {
                data.current_screen = Screen::Settings;
            }),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}

// Settings Screen
pub fn settings_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Settings")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        card("Learning Preferences", flex((
            checkbox(
                data.use_adaptive_scheduling,
                "Adaptive Scheduling",
                |data: &mut AppData, checked| {
                    data.use_adaptive_scheduling = checked;
                }
            ),
            prose("Uses AI to select optimal tasks based on your learning state")
                .alignment(TextAlignment::Start),
            
            checkbox(
                data.enable_hints,
                "Enable Hints",
                |data: &mut AppData, checked| {
                    data.enable_hints = checked;
                }
            ),
            prose("Shows helpful hints when you're struggling with a task")
                .alignment(TextAlignment::Start),
            
            label(format!("Difficulty Level: {:.0}%", data.difficulty_level * 100.0))
                .alignment(TextAlignment::Start),
            prose("Adjusts the baseline difficulty of tasks")
                .alignment(TextAlignment::Start),
        ))
        .direction(Axis::Vertical)),
        
        card("Account", 
            if let Some(user) = &data.current_user {
                flex((
                    metric_display("Username:", user.username.clone(), Color::from_rgb8(64, 64, 64)),
                    metric_display("Email:", user.email.clone(), Color::from_rgb8(64, 64, 64)),
                ))
                .direction(Axis::Vertical)
            } else {
                flex((
                    label("Guest User")
                        .alignment(TextAlignment::Start),
                ))
                .direction(Axis::Vertical)
            }
        ),
        
        card("Data Export", flex((
            prose("Export your learning data for analysis or backup")
                .alignment(TextAlignment::Start),
            button("Export as JSON", |data: &mut AppData| {
                data.export_current_data();
                if let Some(export) = &data.export_data {
                    let json = serde_json::to_string_pretty(export).unwrap_or_default();
                    // In a real app, save to file
                    data.success_message = Some("Data exported as JSON".to_string());
                }
            }),
        ))
        .direction(Axis::Vertical)),
        
        button("Back to Dashboard", |data: &mut AppData| {
            data.current_screen = Screen::Dashboard;
        }),
    ))
    .direction(Axis::Vertical)
}