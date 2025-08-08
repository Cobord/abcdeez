// Copyright 2024 Graph Learning System Authors
// SPDX-License-Identifier: Apache-2.0

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

mod models;
mod api;
mod components;
mod screens;

use xilem::{
    view::{flex, Axis},
    EventLoopBuilder, WidgetView, Xilem,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use models::*;
use api::{ApiClient, MockApiClient};
use components::*;
use screens::*;

// Application screens
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Welcome,
    DomainSelection,
    Training,
    Dashboard,
    Export,
}

// Main application state
pub struct AppData {
    // UI State
    pub current_screen: Screen,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    
    // Authentication
    pub current_user: Option<User>,
    pub username_input: String,
    pub password_input: String,
    pub email_input: String,
    
    // Learner & Session
    pub current_learner: Option<Learner>,
    pub current_session: Option<Session>,
    pub selected_domain: Domain,
    
    // Training state
    pub current_task: Option<Task>,
    pub task_start_time: Option<DateTime<Utc>>,
    pub selected_answer: Option<String>,
    pub session_responses: Vec<TaskResponse>,
    
    // Performance metrics
    pub current_metrics: PerformanceMetrics,
    
    // Export
    pub export_data: Option<ExportData>,
    
    // API client (using mock for now)
    pub api_client: Arc<MockApiClient>,
    
    // Runtime for async operations
    pub runtime: Arc<tokio::runtime::Runtime>,
}

impl Default for AppData {
    fn default() -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        Self {
            current_screen: Screen::Welcome,
            error_message: None,
            success_message: None,
            current_user: None,
            username_input: String::new(),
            password_input: String::new(),
            email_input: String::new(),
            current_learner: None,
            current_session: None,
            selected_domain: Domain::Alphabet,
            current_task: None,
            task_start_time: None,
            selected_answer: None,
            session_responses: Vec::new(),
            current_metrics: PerformanceMetrics::default(),
            export_data: None,
            api_client: Arc::new(MockApiClient::new()),
            runtime: Arc::new(runtime),
        }
    }
}

// Main app logic
fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> {
    // Clear old messages
    if data.error_message.is_some() || data.success_message.is_some() {
        // Messages will auto-clear after being displayed
        // In a real app, you'd use a timer
    }
    
    let screen_content = match data.current_screen {
        Screen::Welcome => welcome_screen(data),
        Screen::DomainSelection => domain_selection_screen(data),
        Screen::Training => training_screen(data),
        Screen::Dashboard => dashboard_screen(data),
        Screen::Export => export_screen(data),
    };
    
    flex((
        nav_bar(&format!("{:?}", data.current_screen)),
        error_message(data.error_message.clone()),
        success_message(data.success_message.clone()),
        screen_content,
    ))
    .direction(Axis::Vertical)
}

// Helper functions for app logic
impl AppData {
    pub fn login(&mut self) {
        let username = self.username_input.clone();
        let password = self.password_input.clone();
        let api = self.api_client.clone();
        
        let result = self.runtime.block_on(async {
            api.login(username, password).await
        });
        
        match result {
            Ok(user) => {
                self.current_user = Some(user);
                self.current_screen = Screen::DomainSelection;
                self.success_message = Some("Login successful!".to_string());
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Login failed: {}", e));
                self.success_message = None;
            }
        }
    }
    
    pub fn create_learner(&mut self) {
        let api = self.api_client.clone();
        let display_name = self.current_user.as_ref().map(|u| u.username.clone());
        
        let result = self.runtime.block_on(async {
            api.create_learner(display_name).await
        });
        
        match result {
            Ok(learner) => {
                self.current_learner = Some(learner);
                self.success_message = Some("Learner profile created!".to_string());
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to create learner: {}", e));
                self.success_message = None;
            }
        }
    }
    
    pub fn start_session(&mut self) {
        if let Some(learner) = &self.current_learner {
            let api = self.api_client.clone();
            let learner_id = learner.id.clone();
            let topology_type = self.selected_domain.as_str().to_string();
            
            let result = self.runtime.block_on(async {
                api.create_session(learner_id, topology_type, None).await
            });
            
            match result {
                Ok(session) => {
                    self.current_session = Some(session);
                    self.current_screen = Screen::Training;
                    self.session_responses.clear();
                    self.generate_next_task();
                    self.success_message = Some("Training session started!".to_string());
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to start session: {}", e));
                    self.success_message = None;
                }
            }
        } else {
            self.error_message = Some("Please create a learner profile first".to_string());
        }
    }
    
    pub fn generate_next_task(&mut self) {
        match self.selected_domain {
            Domain::Alphabet => {
                // Generate a random alphabet task
                let letter = ('A'..='Z')
                    .nth((chrono::Utc::now().timestamp() as usize) % 26)
                    .unwrap_or('A');
                let position = (letter as usize) - ('A' as usize) + 1;
                
                // Generate options (including correct answer)
                let mut options = vec![letter];
                for _ in 0..3 {
                    let random_letter = ('A'..='Z')
                        .nth((chrono::Utc::now().timestamp() as usize * (options.len() + 1)) % 26)
                        .unwrap_or('B');
                    if !options.contains(&random_letter) {
                        options.push(random_letter);
                    }
                }
                
                self.current_task = Some(Task::Alphabet(AlphabetTask {
                    letter,
                    position,
                    options,
                }));
                self.task_start_time = Some(chrono::Utc::now());
            }
            Domain::Music => {
                // Generate a simple music theory task
                let task = MusicTask {
                    task_type: "interval".to_string(),
                    prompt: "What interval is C to E?".to_string(),
                    correct_answer: "Major Third".to_string(),
                    options: vec![
                        "Major Third".to_string(),
                        "Minor Third".to_string(),
                        "Perfect Fourth".to_string(),
                        "Perfect Fifth".to_string(),
                    ],
                    difficulty: 0.5,
                    musical_context: serde_json::json!({
                        "key": "C Major",
                        "notes": ["C", "E"]
                    }),
                };
                self.current_task = Some(Task::Music(task));
                self.task_start_time = Some(chrono::Utc::now());
            }
            _ => {
                self.current_task = None;
            }
        }
        self.selected_answer = None;
    }
    
    pub fn submit_answer(&mut self, answer: String) {
        if let (Some(task), Some(start_time)) = (&self.current_task, self.task_start_time) {
            let response_time_ms = (chrono::Utc::now() - start_time).num_milliseconds() as i32;
            
            let correct = match task {
                Task::Alphabet(alphabet_task) => {
                    answer == alphabet_task.position.to_string()
                }
                Task::Music(music_task) => {
                    answer == music_task.correct_answer
                }
                Task::Custom(_) => false,
            };
            
            // Update metrics
            self.current_metrics.update(correct, response_time_ms);
            
            // Store response
            let response = TaskResponse {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: self.current_session.as_ref().map(|s| s.id.clone()).unwrap_or_default(),
                task_type: match task {
                    Task::Alphabet(_) => "alphabet".to_string(),
                    Task::Music(_) => "music".to_string(),
                    Task::Custom(_) => "custom".to_string(),
                },
                task_data: serde_json::json!({}), // Simplified for now
                user_answer: Some(answer.clone()),
                correct,
                response_time_ms,
                timestamp: chrono::Utc::now(),
            };
            
            self.session_responses.push(response);
            
            // Generate next task
            self.generate_next_task();
        }
    }
    
    pub fn end_session(&mut self) {
        if let Some(session) = &mut self.current_session {
            session.end_time = Some(chrono::Utc::now());
            session.status = "completed".to_string();
            
            // Calculate session summary
            let total_responses = self.session_responses.len();
            let correct_responses = self.session_responses.iter().filter(|r| r.correct).count();
            let accuracy = if total_responses > 0 {
                correct_responses as f64 / total_responses as f64
            } else {
                0.0
            };
            
            session.summary = Some(serde_json::json!({
                "total_responses": total_responses,
                "correct_responses": correct_responses,
                "accuracy": accuracy,
                "metrics": self.current_metrics
            }));
            
            self.current_screen = Screen::Dashboard;
            self.success_message = Some("Session completed!".to_string());
        }
    }
    
    pub fn export_current_data(&mut self) {
        if let (Some(learner), Some(session)) = (&self.current_learner, &self.current_session) {
            self.export_data = Some(ExportData {
                learner: learner.clone(),
                sessions: vec![session.clone()],
                responses: self.session_responses.clone(),
                metrics: self.current_metrics.clone(),
                export_time: chrono::Utc::now(),
            });
            
            self.success_message = Some("Data exported successfully!".to_string());
        } else {
            self.error_message = Some("No data to export".to_string());
        }
    }
}

/// Entry point for the app
pub fn run(event_loop: EventLoopBuilder) {
    let data = AppData::default();
    
    let app = Xilem::new(data, app_logic);
    app.run_windowed(event_loop, "Graph-Coded Learning System".into())
        .unwrap();
}