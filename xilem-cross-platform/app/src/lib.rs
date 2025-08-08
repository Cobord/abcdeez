// Copyright 2024 Graph Learning System Authors
// SPDX-License-Identifier: Apache-2.0

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

mod api;
mod components;
mod models;
mod screens;

use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use xilem::{
    view::{flex, Axis},
    EventLoopBuilder, WidgetView, Xilem,
};

use graph_learning_core::{
    export::LearnerDataExport,
    hints::{HintLevel, InterventionAction, InterventionSystem, StruggleLevel},
    prelude::*,
    statistics::{ExGaussianParameters, SessionAnalyzer, StrategyType},
    tasks::TaskResponse as CoreTaskResponse,
    AdaptiveScheduler, LearnerMetrics, TaskGenerator, TaskSession,
};

use api::{ApiClient, MockApiClient};
use components::*;
use models::*;
use screens::*;

// Application screens
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Welcome,
    DomainSelection,
    Training,
    Dashboard,
    Settings,
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

    // Core library integration
    pub current_learner: Option<Learner>,
    pub current_session: Option<Session>,
    pub selected_domain: Domain,
    pub topology: Option<Topology>,
    pub adaptive_scheduler: Option<AdaptiveScheduler>,
    pub task_session: Option<TaskSession>,
    pub task_generator: Option<TaskGenerator>,

    // Training state
    pub current_task: Option<UITask>,
    pub task_start_time: Option<Instant>,
    pub selected_answer: Option<String>,
    pub selected_answer_index: Option<usize>,
    pub show_feedback: bool,
    pub last_response_correct: bool,
    pub current_hint: Option<String>,
    pub hint_level: HintLevel,

    // Intervention system
    pub intervention_system: Option<InterventionSystem>,
    pub struggle_level: StruggleLevel,

    // Performance metrics
    pub current_metrics: PerformanceMetrics,
    pub session_responses: Vec<CoreTaskResponse>,

    // Settings
    pub use_adaptive_scheduling: bool,
    pub enable_hints: bool,
    pub difficulty_level: f64,

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
            topology: None,
            adaptive_scheduler: None,
            task_session: None,
            task_generator: None,
            current_task: None,
            task_start_time: None,
            selected_answer: None,
            selected_answer_index: None,
            show_feedback: false,
            last_response_correct: false,
            current_hint: None,
            hint_level: HintLevel::Confirmation,
            intervention_system: None,
            struggle_level: StruggleLevel::None,
            current_metrics: PerformanceMetrics::default(),
            session_responses: Vec::new(),
            use_adaptive_scheduling: true,
            enable_hints: true,
            difficulty_level: 0.5,
            export_data: None,
            api_client: Arc::new(MockApiClient::new()),
            runtime: Arc::new(runtime),
        }
    }
}

// Main app logic
fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> {
    // Clear old messages after some time (in a real app, use a timer)
    if data.error_message.is_some() || data.success_message.is_some() {
        // Messages will auto-clear after being displayed
    }

    // Render all possible screens as Option views; only the active one will be Some(...)
    flex((
        nav_bar(&format!("{:?}", data.current_screen)),
        error_message(data.error_message.clone()),
        success_message(data.success_message.clone()),
        (data.current_screen == Screen::Welcome).then(|| welcome_screen(data)),
        (data.current_screen == Screen::DomainSelection).then(|| domain_selection_screen(data)),
        (data.current_screen == Screen::Training).then(|| training_screen(data)),
        (data.current_screen == Screen::Dashboard).then(|| dashboard_screen(data)),
        (data.current_screen == Screen::Settings).then(|| settings_screen(data)),
    ))
    .direction(Axis::Vertical)
}

// Helper functions for app logic
impl AppData {
    pub fn login(&mut self) {
        let username = self.username_input.clone();
        let password = self.password_input.clone();
        let api = self.api_client.clone();

        let result = self
            .runtime
            .block_on(async { api.login(username, password).await });

        match result {
            Ok(user) => {
                self.current_user = Some(user);
                self.current_screen = Screen::DomainSelection;
                self.success_message = Some("Login successful!".to_string());
                self.error_message = None;
                self.create_learner();
            }
            Err(e) => {
                self.error_message = Some(format!("Login failed: {}", e));
                self.success_message = None;
            }
        }
    }

    pub fn create_learner(&mut self) {
        // Create topology based on selected domain
        let topology = self.create_topology_for_domain();

        // Create learner model using the core library
        let learner_id = Uuid::new_v4().to_string();
        let core_model = LearnerModel::new(learner_id.clone(), &topology);

        self.current_learner = Some(Learner {
            id: learner_id,
            user_id: self.current_user.as_ref().map(|u| u.id.clone()),
            display_name: self.current_user.as_ref().map(|u| u.username.clone()),
            created_at: Utc::now(),
            core_model,
            metadata: None,
        });

        self.topology = Some(topology);
        self.success_message = Some("Learner profile created!".to_string());
        self.error_message = None;
    }

    /// Demo showcase: create a demo user/learner, start a session, run a short mock training
    /// flow to populate the UI for a portfolio/demo. Uses existing AppData methods so behavior
    /// follows the same code paths used in normal operation.
    pub fn demo_showcase(&mut self) {
        // Ensure a demo user exists
        if self.current_user.is_none() {
            self.current_user = Some(User {
                id: Uuid::new_v4().to_string(),
                username: "DemoUser".to_string(),
                email: "demo@example.com".to_string(),
                password_hash: String::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Create the learner model and topology
        self.create_learner();

        // Start a training session (sets up task generator/scheduler/intervention system)
        self.start_session();

        // Perform a short sequence of interactions to demonstrate functionality.
        // We'll generate a small number of tasks and submit answers programmatically.
        // Use conservative choices (choose first option) so the flow proceeds.
        for _ in 0..5 {
            // Ensure there is an active task (generate_next_task resets current_task)
            self.generate_next_task();

            // If a task exists, submit a safe default answer (index 0).
            if self.current_task.is_some() {
                // Simulate some time having passed
                // Update metrics via submit_answer (will update model and session responses)
                self.submit_answer(0);

                // Request hint and struggle checks to trigger intervention paths (no-op if disabled)
                self.request_hint();
                self.check_struggle_and_provide_help();
            }
        }

        // End the session and produce summary + data visible on Dashboard
        self.end_session();

        // Friendly message for demo viewers
        self.success_message =
            Some("Demo showcase complete — check the Dashboard for results.".to_string());
        self.error_message = None;
    }

    fn create_topology_for_domain(&self) -> Topology {
        match self.selected_domain {
            Domain::Alphabet => {
                // Create alphabet topology (A-Z linear)
                let letters: Vec<String> = ('A'..='Z').map(|c| c.to_string()).collect();
                Topology::new_linear(letters)
            }
            Domain::DaysOfWeek => {
                // Create days of week topology (cyclic)
                let days = vec![
                    "Monday".to_string(),
                    "Tuesday".to_string(),
                    "Wednesday".to_string(),
                    "Thursday".to_string(),
                    "Friday".to_string(),
                    "Saturday".to_string(),
                    "Sunday".to_string(),
                ];
                Topology::new_cyclic(days)
            }
            Domain::Music => {
                // Create music theory topology (partial order for scales/chords)
                let notes = vec![
                    "C".to_string(),
                    "D".to_string(),
                    "E".to_string(),
                    "F".to_string(),
                    "G".to_string(),
                    "A".to_string(),
                    "B".to_string(),
                ];
                Topology::new_linear(notes)
            }
            Domain::Mathematics => {
                // Create number sequence topology
                let numbers: Vec<String> = (0..20).map(|n| n.to_string()).collect();
                Topology::new_linear(numbers)
            }
            Domain::Custom(ref name) => {
                // For custom domains, create a simple linear topology
                let items: Vec<String> = (1..10).map(|i| format!("{}-{}", name, i)).collect();
                Topology::new_linear(items)
            }
        }
    }

    pub fn start_session(&mut self) {
        if let Some(learner) = &mut self.current_learner {
            if let Some(topology) = &self.topology {
                // Create task generator
                let task_generator = TaskGenerator::new(topology.clone());

                // Create adaptive scheduler if enabled
                if self.use_adaptive_scheduling {
                    let scheduler = AdaptiveScheduler::new_with_eig(
                        learner.core_model.clone(),
                        topology.clone(),
                        true, // Use Expected Information Gain
                    );
                    self.adaptive_scheduler = Some(scheduler);
                }

                // Create task session
                let task_session = TaskSession::new(topology.clone());

                // Create intervention system for hints and adaptive difficulty
                let intervention_system = InterventionSystem::new(topology.clone());

                // Create UI session
                let session = Session {
                    id: Uuid::new_v4().to_string(),
                    learner_id: learner.id.clone(),
                    topology_type: self.selected_domain.as_str().to_string(),
                    topology: Some(topology.clone()),
                    start_time: Utc::now(),
                    end_time: None,
                    status: "active".to_string(),
                    summary: None,
                    responses: Vec::new(),
                };

                self.current_session = Some(session);
                self.task_generator = Some(task_generator);
                self.task_session = Some(task_session);
                self.intervention_system = Some(intervention_system);
                self.current_screen = Screen::Training;
                self.session_responses.clear();
                self.current_metrics = PerformanceMetrics::default();
                self.generate_next_task();
                self.success_message = Some("Training session started!".to_string());
                self.error_message = None;
            }
        } else {
            self.error_message = Some("Please create a learner profile first".to_string());
        }
    }

    pub fn generate_next_task(&mut self) {
        // Generate task using adaptive scheduler or random generator
        let task = if self.use_adaptive_scheduling {
            if let Some(scheduler) = &mut self.adaptive_scheduler {
                scheduler.select_next_task()
            } else if let Some(generator) = &mut self.task_generator {
                generator.generate_task(None)
            } else {
                return;
            }
        } else if let Some(generator) = &mut self.task_generator {
            generator.generate_task(None)
        } else {
            return;
        };

        // Convert to UI task
        let ui_task = UITask::from_core_task(task);
        self.current_task = Some(ui_task);
        self.task_start_time = Some(Instant::now());
        self.selected_answer = None;
        self.selected_answer_index = None;
        self.show_feedback = false;
    }

    pub fn submit_answer(&mut self, answer_index: usize) {
        if let Some(ui_task) = &self.current_task {
            if let Some(start_time) = self.task_start_time {
                let response_time_ms = start_time.elapsed().as_millis() as u128;

                // Get the selected answer
                let answer = ui_task
                    .display_options
                    .get(answer_index)
                    .cloned()
                    .unwrap_or_default();

                // Check if correct
                let correct = answer == ui_task.core_task.correct_answer;

                // Process response with intervention system
                if let Some(intervention_system) = &mut self.intervention_system {
                    intervention_system.process_response(
                        &ui_task.core_task,
                        correct,
                        response_time_ms as u64,
                    );
                }

                // Create task response
                let response = CoreTaskResponse {
                    task: ui_task.core_task.clone(),
                    user_answer: answer.clone(),
                    correct,
                    response_time_ms,
                    timestamp: Utc::now(),
                };

                // Update learner model and adaptive scheduler
                if let Some(scheduler) = &mut self.adaptive_scheduler {
                    scheduler.update_model(&ui_task.core_task, correct, response_time_ms);
                    // Update the learner model reference
                    if let Some(learner) = &mut self.current_learner {
                        learner.core_model = scheduler.get_learner_model().clone();
                    }
                } else if let Some(learner) = &mut self.current_learner {
                    // Update learner model directly if no scheduler
                    learner
                        .core_model
                        .update_operation_proficiency(&ui_task.core_task.operation, correct);
                }

                // Update metrics
                self.current_metrics
                    .update(correct, response_time_ms as i32);

                // Update learner metrics from core model
                if let Some(learner) = &self.current_learner {
                    let core_metrics = LearnerMetrics::from_model(&learner.core_model);
                    self.current_metrics.update_from_core_metrics(&core_metrics);
                }

                // Store response
                self.session_responses.push(response);

                // Update session
                if let Some(session) = &mut self.current_session {
                    session.responses = self.session_responses.clone();
                }

                // Show feedback
                self.last_response_correct = correct;
                self.show_feedback = true;
                self.selected_answer = Some(answer);
                self.selected_answer_index = Some(answer_index);
                self.current_hint = None; // Clear any existing hint
            }
        }
    }

    pub fn continue_to_next_task(&mut self) {
        self.generate_next_task();
    }

    pub fn request_hint(&mut self) {
        if let (Some(ui_task), Some(intervention_system)) =
            (&self.current_task, &mut self.intervention_system)
        {
            if self.enable_hints {
                let elapsed_ms = self
                    .task_start_time
                    .map(|start| start.elapsed().as_millis() as u64)
                    .unwrap_or(0);

                if let Some(action) =
                    intervention_system.check_intervention_needed(&ui_task.core_task, elapsed_ms)
                {
                    match action {
                        InterventionAction::ProvideHint(hint)
                        | InterventionAction::ProvideWorkedExample(hint) => {
                            self.current_hint = Some(hint);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn check_struggle_and_provide_help(&mut self) {
        if let (Some(ui_task), Some(intervention_system)) =
            (&self.current_task, &mut self.intervention_system)
        {
            let elapsed_ms = self
                .task_start_time
                .map(|start| start.elapsed().as_millis() as u64)
                .unwrap_or(0);

            if let Some(action) =
                intervention_system.check_intervention_needed(&ui_task.core_task, elapsed_ms)
            {
                match action {
                    InterventionAction::ProvideHint(hint)
                    | InterventionAction::ProvideWorkedExample(hint) => {
                        self.current_hint = Some(hint);
                    }
                    InterventionAction::SuggestBreak => {
                        self.success_message =
                            Some("Consider taking a short break to refresh your mind.".to_string());
                    }
                    InterventionAction::IncreaseDifficulty => {
                        self.difficulty_level = (self.difficulty_level + 0.1).min(1.0);
                        self.success_message =
                            Some("Great progress! Increasing difficulty.".to_string());
                    }
                    InterventionAction::DecreaseDifficulty => {
                        self.difficulty_level = (self.difficulty_level - 0.1).max(0.1);
                        self.success_message =
                            Some("Adjusting difficulty to help you learn better.".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn end_session(&mut self) {
        if let Some(session) = &mut self.current_session {
            session.end_time = Some(Utc::now());
            session.status = "completed".to_string();

            // Calculate session summary
            let total_responses = self.session_responses.len();
            let correct_responses = self.session_responses.iter().filter(|r| r.correct).count();
            let accuracy = if total_responses > 0 {
                correct_responses as f64 / total_responses as f64
            } else {
                0.0
            };

            // Get final learner metrics
            let learner_metrics = self
                .current_learner
                .as_ref()
                .map(|l| LearnerMetrics::from_model(&l.core_model));

            session.summary = Some(serde_json::json!({
                "total_responses": total_responses,
                "correct_responses": correct_responses,
                "accuracy": accuracy,
                "metrics": self.current_metrics,
                "learner_metrics": learner_metrics,
            }));

            self.current_screen = Screen::Dashboard;
            self.success_message = Some("Session completed!".to_string());
        }
    }

    pub fn export_current_data(&mut self) {
        if let (Some(learner), Some(task_session)) = (&self.current_learner, &self.task_session) {
            // Create comprehensive export using the core library
            let comprehensive_export = LearnerDataExport::from_learner_model(
                &learner.core_model,
                vec![], // TODO: include TaskSession when Clone or export API allows borrowing
                Some("xilem-ui-session".to_string()),
            );

            // Also create UI-specific export
            let ui_export = ExportData {
                learner: learner.clone(),
                sessions: self
                    .current_session
                    .as_ref()
                    .map_or(vec![], |s| vec![s.clone()]),
                metrics: self.current_metrics.clone(),
                export_time: Utc::now(),
            };

            self.export_data = Some(ui_export);

            // You could save the comprehensive export to a file here
            // let json = comprehensive_export.to_json().unwrap_or_default();

            self.success_message = Some("Comprehensive data export ready! Includes performance trajectories, error patterns, and model parameters.".to_string());
        } else {
            self.error_message = Some("No data to export".to_string());
        }
    }
}

/// Entry point for the app (no-argument). The event loop is created internally so callers
/// can simply call `graph_learning_ui::run()` from main without constructing an EventLoop.
pub fn run() {
    // Create the platform event loop here
    let event_loop = xilem::EventLoop::with_user_event();

    let data = AppData::default();

    let app = Xilem::new(data, app_logic);
    // Run the windowed application with a title using the Xilem runtime.
    // We use the windowed runner to create a visible window for the demo / showcase.
    let _ = app.run_windowed(
        event_loop,
        "Adaptive Learning System - Alphabet Terminal".into(),
    );
}
