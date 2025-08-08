use xilem::{
    view::{button, flex, label, prose, textbox, checkbox, Axis},
    Color, TextAlignment, WidgetView,
};
use uuid::Uuid;

use crate::{AppData, Screen, components::*, models::*};

// Welcome/Login Screen
pub fn welcome_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Graph-Coded Learning System")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        prose("Welcome to the Graph-Coded Learning System. This application helps track and analyze learning patterns using graph-based cognitive models.")
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
        
        card("New User?", flex((
            prose("Create an account to start tracking your learning progress")
                .alignment(TextAlignment::Middle),
            labeled_input(
                "Email:", 
                data.email_input.clone(),
                |data: &mut AppData, value| {
                    data.email_input = value;
                }
            ),
            button("Register", |data: &mut AppData| {
                // In a real app, this would call the register API
                data.login(); // For now, just login
            }),
        )).direction(Axis::Vertical)),
        
        button("Continue as Guest", |data: &mut AppData| {
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
    ))
    .direction(Axis::Vertical)
}

// Domain Selection Screen
pub fn domain_selection_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let domains = vec![
        Domain::Alphabet,
        Domain::Music,
        Domain::Mathematics,
    ];
    
    let domain_buttons = domains
        .into_iter()
        .map(|domain| {
            let is_selected = data.selected_domain == domain;
            let domain_clone = domain.clone();
            flex((
                domain_card(&domain, is_selected),
                button("Select", move |data: &mut AppData| {
                    data.selected_domain = domain_clone.clone();
                }),
            ))
            .direction(Axis::Vertical)
        })
        .collect::<Vec<_>>();
    
    flex((
        label("Select Learning Domain")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        prose("Choose a domain to begin your training session")
            .alignment(TextAlignment::Middle),
        
        flex(domain_buttons)
            .direction(Axis::Vertical),
        
        label(format!("Selected: {}", data.selected_domain.display_name()))
            .brush(Color::from_rgb8(0, 200, 0))
            .alignment(TextAlignment::Middle),
        
        button("Start Training Session", |data: &mut AppData| {
            data.start_session();
        }),
    ))
    .direction(Axis::Vertical)
}

// Training Session Screen
pub fn training_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let task_display = if let Some(task) = &data.current_task {
        match task {
            Task::Alphabet(alphabet_task) => {
                let options = (1..=26)
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>();
                
                flex((
                    card("Task", flex((
                        label(format!("What position is '{}' in the alphabet?", alphabet_task.letter))
                            .alignment(TextAlignment::Middle),
                        label("(A=1, B=2, C=3, ...)")
                            .brush(Color::from_rgb8(128, 128, 128))
                            .alignment(TextAlignment::Middle),
                    )).direction(Axis::Vertical)),
                    
                    card("Your Answer", flex((
                        textbox(
                            data.selected_answer.clone().unwrap_or_default(),
                            |data: &mut AppData, value| {
                                data.selected_answer = Some(value);
                            }
                        ),
                        button("Submit", |data: &mut AppData| {
                            if let Some(answer) = data.selected_answer.clone() {
                                data.submit_answer(answer);
                            }
                        }),
                    )).direction(Axis::Vertical)),
                ))
                .direction(Axis::Vertical)
            }
            Task::Music(music_task) => {
                let options = music_task.options.clone();
                let option_buttons = options
                    .into_iter()
                    .map(|option| {
                        let option_clone = option.clone();
                        button(option, move |data: &mut AppData| {
                            data.submit_answer(option_clone.clone());
                        })
                    })
                    .collect::<Vec<_>>();
                
                flex((
                    card("Music Theory Task", flex((
                        label(&music_task.prompt)
                            .alignment(TextAlignment::Middle),
                        label(format!("Difficulty: {:.1}", music_task.difficulty))
                            .brush(Color::from_rgb8(128, 128, 128))
                            .alignment(TextAlignment::Middle),
                    )).direction(Axis::Vertical)),
                    
                    card("Select Your Answer", flex(option_buttons)
                        .direction(Axis::Vertical)),
                ))
                .direction(Axis::Vertical)
            }
            Task::Custom(_) => {
                label("Custom task type")
                    .alignment(TextAlignment::Middle)
            }
        }
    } else {
        label("Loading next task...")
            .alignment(TextAlignment::Middle)
    };
    
    // Session info
    let session_info_display = if let Some(session) = &data.current_session {
        session_info(session)
    } else {
        label("No active session")
            .alignment(TextAlignment::Middle)
    };
    
    // Progress display
    let progress_display = flex((
        metric_display(
            "Responses:", 
            data.session_responses.len().to_string(),
            Color::from_rgb8(0, 128, 255)
        ),
        metric_display(
            "Correct:", 
            data.session_responses.iter().filter(|r| r.correct).count().to_string(),
            Color::from_rgb8(0, 200, 0)
        ),
        metric_display(
            "Current Accuracy:", 
            format!("{:.1}%", data.current_metrics.accuracy_rate * 100.0),
            Color::from_rgb8(255, 128, 0)
        ),
    ))
    .direction(Axis::Horizontal);
    
    flex((
        label("Training Session")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        session_info_display,
        progress_display,
        task_display,
        
        button("End Session", |data: &mut AppData| {
            data.end_session();
        }),
    ))
    .direction(Axis::Vertical)
}

// Performance Dashboard Screen
pub fn dashboard_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let metrics_display = performance_chart(&data.current_metrics);
    
    // Recent responses list
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
                response.task_type,
                response.response_time_ms
            ))
            .brush(color)
            .alignment(TextAlignment::Start)
        })
        .collect::<Vec<_>>();
    
    let recent_responses_display = if recent_responses.is_empty() {
        label("No responses yet")
            .alignment(TextAlignment::Middle)
    } else {
        flex(recent_responses)
            .direction(Axis::Vertical)
    };
    
    // Session history
    let session_display = if let Some(session) = &data.current_session {
        flex((
            label("Last Session")
                .brush(Color::from_rgb8(64, 64, 64))
                .alignment(TextAlignment::Start),
            session_info(session),
        ))
        .direction(Axis::Vertical)
    } else {
        label("No sessions completed")
            .alignment(TextAlignment::Middle)
    };
    
    flex((
        label("Performance Dashboard")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        card("Overall Performance", metrics_display),
        
        card("Recent Responses", recent_responses_display),
        
        card("Session Summary", session_display),
        
        flex((
            button("New Session", |data: &mut AppData| {
                data.current_screen = Screen::DomainSelection;
            }),
            button("Export Data", |data: &mut AppData| {
                data.current_screen = Screen::Export;
            }),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}

// Data Export Screen
pub fn export_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let export_display = if let Some(export_data) = &data.export_data {
        flex((
            label("Export Ready")
                .brush(Color::from_rgb8(0, 200, 0))
                .alignment(TextAlignment::Middle),
            
            card("Export Summary", flex((
                metric_display(
                    "Learner:", 
                    export_data.learner.display_name.clone().unwrap_or("Anonymous".to_string()),
                    Color::from_rgb8(0, 128, 255)
                ),
                metric_display(
                    "Sessions:", 
                    export_data.sessions.len().to_string(),
                    Color::from_rgb8(0, 128, 255)
                ),
                metric_display(
                    "Total Responses:", 
                    export_data.responses.len().to_string(),
                    Color::from_rgb8(0, 128, 255)
                ),
                metric_display(
                    "Export Time:", 
                    export_data.export_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Color::from_rgb8(128, 128, 128)
                ),
            )).direction(Axis::Vertical)),
            
            card("Export Format", flex((
                button("JSON", |data: &mut AppData| {
                    if let Some(export) = &data.export_data {
                        let json = serde_json::to_string_pretty(export).unwrap_or_default();
                        // In a real app, save to file or clipboard
                        data.success_message = Some("Data exported as JSON".to_string());
                    }
                }),
                button("CSV", |data: &mut AppData| {
                    // In a real app, convert to CSV format
                    data.success_message = Some("Data exported as CSV".to_string());
                }),
            )).direction(Axis::Horizontal)),
        ))
        .direction(Axis::Vertical)
    } else {
        flex((
            label("No Data to Export")
                .alignment(TextAlignment::Middle),
            
            prose("Complete a training session to generate exportable data")
                .alignment(TextAlignment::Middle),
        ))
        .direction(Axis::Vertical)
    };
    
    flex((
        label("Export Data")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        
        prose("Export your learning data for analysis or backup")
            .alignment(TextAlignment::Middle),
        
        export_display,
        
        button("Generate Export", |data: &mut AppData| {
            data.export_current_data();
        }),
        
        button("Back to Dashboard", |data: &mut AppData| {
            data.current_screen = Screen::Dashboard;
        }),
    ))
    .direction(Axis::Vertical)
}