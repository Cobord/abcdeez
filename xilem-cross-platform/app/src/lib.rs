// Copyright 2024 Graph Learning System Authors
// SPDX-License-Identifier: Apache-2.0

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

mod api;
mod components;
mod demo;
pub mod models;
mod screens;

use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use xilem::{
    view::{button, flex, label, Axis},
    EventLoopBuilder, TextAlignment, WidgetView, Xilem,
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
use demo::DemoController;
use models::*;
use screens::{
    dashboard_screen, domain_selection_screen, settings_screen, training_screen, welcome_screen,
};

// Application screens
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    pub show_end_session_confirmation: bool,

    // Authentication
    pub current_user: Option<User>,
    pub username_input: String,
    pub password_input: String,
    pub email_input: String,

    // Async request states
    pub login_request_in_flight: bool,
    pub create_session_in_flight: bool,
    pub submit_response_in_flight: bool,
    pub end_session_in_flight: bool,
    pub export_data_in_flight: bool,

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
    pub export_format: ExportFormat,

    // API client (using mock for now)
    pub api_client: Arc<MockApiClient>,

    // Enhanced demo controller
    pub demo_controller: DemoController,
    pub demo_seed: Option<u64>,
    pub demo_rng: Option<StdRng>,

    // Demo preferences
    pub demo_auto_advance: bool,
    pub demo_show_tooltips: bool,
    pub demo_highlight_elements: bool,
    pub demo_speed: f64,
    pub preferred_demo: String,

    // Display preferences
    pub show_advanced_metrics: bool,
    pub show_response_times: bool,
    pub enable_animations: bool,
    pub anonymous_export: bool,

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
            show_end_session_confirmation: false,
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
            export_format: ExportFormat::Json,
            api_client: Arc::new(MockApiClient::new()),

            // Async request states
            login_request_in_flight: false,
            create_session_in_flight: false,
            submit_response_in_flight: false,
            end_session_in_flight: false,
            export_data_in_flight: false,

            // Enhanced demo controller
            demo_controller: DemoController::new(),
            demo_seed: None,
            demo_rng: None,

            // Demo preferences
            demo_auto_advance: true,
            demo_show_tooltips: true,
            demo_highlight_elements: true,
            demo_speed: 1.0,
            preferred_demo: "quick_tour".to_string(),

            // Display preferences
            show_advanced_metrics: false,
            show_response_times: true,
            enable_animations: true,
            anonymous_export: false,

            // Runtime for async operations
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

    // Handle async operations synchronously for now (Xilem task view not available)
    // In production, these would be proper async tasks
    if data.login_request_in_flight {
        // Simulate login completion
        let username = data.username_input.clone();
        data.login_request_in_flight = false;
        data.current_user = Some(User {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            email: format!("{}@example.com", username),
            password_hash: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        data.current_screen = Screen::DomainSelection;
        data.success_message = Some("Login successful!".to_string());
        data.error_message = None;
        data.create_learner();
    }

    if data.create_session_in_flight {
        data.create_session_in_flight = false;
        if let Some(learner) = &data.current_learner {
            let session = Session {
                id: Uuid::new_v4().to_string(),
                learner_id: learner.id.clone(),
                topology_type: data.selected_domain.as_str().to_string(),
                topology: data.topology.clone(),
                start_time: Utc::now(),
                end_time: None,
                status: "active".to_string(),
                summary: None,
                responses: Vec::new(),
            };
            data.current_session = Some(session);
            data.current_screen = Screen::Training;
            data.session_responses.clear();
            data.current_metrics = PerformanceMetrics::default();
            data.generate_next_task();
            data.success_message = Some("Training session started!".to_string());
            data.error_message = None;
        }
    }

    if data.submit_response_in_flight {
        data.submit_response_in_flight = false;
        data.show_feedback = true;
    }

    if data.end_session_in_flight {
        data.end_session_in_flight = false;
        if let Some(session) = &mut data.current_session {
            session.end_time = Some(Utc::now());
            session.status = "completed".to_string();

            let total_responses = data.session_responses.len();
            let correct_responses = data.session_responses.iter().filter(|r| r.correct).count();
            let accuracy = if total_responses > 0 {
                correct_responses as f64 / total_responses as f64
            } else {
                0.0
            };

            session.summary = Some(serde_json::json!({
                "total_responses": total_responses,
                "correct_responses": correct_responses,
                "accuracy": accuracy,
                "metrics": data.current_metrics,
            }));
        }
        data.current_screen = Screen::Dashboard;
        data.success_message = Some("Session completed!".to_string());
    }

    if data.export_data_in_flight {
        data.export_data_in_flight = false;
        data.success_message = Some("Data exported successfully!".to_string());
    }

    // Main content - use conditional rendering to avoid impl trait issues
    let content = flex((
        nav_bar(&format!("{:?}", data.current_screen)),
        error_message(data.error_message.clone()),
        success_message(data.success_message.clone()),
        flex((
            (data.current_screen == Screen::Welcome).then(|| welcome_screen(data)),
            (data.current_screen == Screen::DomainSelection).then(|| domain_selection_screen(data)),
            (data.current_screen == Screen::Training).then(|| training_screen(data)),
            (data.current_screen == Screen::Dashboard).then(|| dashboard_screen(data)),
            (data.current_screen == Screen::Settings).then(|| settings_screen(data)),
        ))
        .direction(Axis::Vertical),
    ))
    .direction(Axis::Vertical);

    // Enhanced guided demo overlay
    let overlay = if data.demo_controller.is_active {
        let (current, total) = data.demo_controller.get_progress();
        let step = data.demo_controller.get_current_step();

        let step_content = if let Some(step) = step {
            let title = format!("📚 {} ({}/{})", step.title, current, total);
            let description = step.description.clone();

            // Progress bar
            let progress_text = format!(
                "[{}{}] {}/{}",
                "=".repeat(current),
                "-".repeat(total.saturating_sub(current)),
                current,
                total
            );

            flex((
                label(title).alignment(TextAlignment::Middle),
                label(progress_text).alignment(TextAlignment::Middle),
                label(description).alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical)
        } else {
            flex((
                label("Loading demo...").alignment(TextAlignment::Middle),
                label("").alignment(TextAlignment::Middle),
                label("").alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical)
        };

        let controls = flex((
            button("◀ Back", |d: &mut AppData| {
                d.demo_controller.previous_step();
            }),
            button(
                if data.demo_controller.is_paused {
                    "▶ Resume"
                } else {
                    "⏸ Pause"
                },
                |d: &mut AppData| {
                    if d.demo_controller.is_paused {
                        d.demo_controller.resume();
                    } else {
                        d.demo_controller.pause();
                    }
                },
            ),
            button("Next ▶", |d: &mut AppData| {
                d.demo_next_step();
            }),
            button("✖ End", |d: &mut AppData| {
                d.demo_end();
            }),
        ))
        .direction(Axis::Horizontal);

        Some(card::<AppData, _>(
            "Guided Demo",
            flex((step_content, controls)).direction(Axis::Vertical),
        ))
    } else {
        None
    };

    // Compose content + overlay (overlay last so it appears after main content)
    flex((content, overlay)).direction(Axis::Vertical)
}

// Helper functions for app logic
impl AppData {
    pub fn login(&mut self) {
        // Set in-flight flag; the view::task in app_logic() will pick this up and perform async login.
        if !self.login_request_in_flight {
            self.login_request_in_flight = true;
            self.error_message = None;
            self.success_message = None;
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
    /// Can optionally take a seed for deterministic behavior (useful for testing).
    pub fn demo_showcase(&mut self) {
        self.demo_showcase_with_seed(None);
    }

    /// Demo showcase with optional seed for deterministic behavior
    pub fn demo_showcase_with_seed(&mut self, seed: Option<u64>) {
        // Initialize RNG with seed if provided (for deterministic demos)
        let actual_seed = seed.unwrap_or_else(|| {
            // Use current timestamp as seed if not provided
            chrono::Utc::now().timestamp() as u64
        });
        self.demo_seed = Some(actual_seed);
        self.demo_rng = Some(StdRng::seed_from_u64(actual_seed));
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
        self.start_session_sync();

        // Perform a short sequence of interactions to demonstrate functionality.
        // We'll generate a small number of tasks and submit answers programmatically.
        // Use deterministic choices based on RNG for reproducible demos.
        for _i in 0..5 {
            // Ensure there is an active task (generate_next_task resets current_task)
            self.generate_next_task();

            // Simulate some elapsed time for realistic response times (500-2000ms)
            if let Some(rng) = &mut self.demo_rng {
                let simulated_ms = rng.gen_range(500..2000);
                // Adjust task_start_time to simulate elapsed time
                if let Some(start_time) = &mut self.task_start_time {
                    *start_time = start_time
                        .checked_sub(std::time::Duration::from_millis(simulated_ms))
                        .unwrap_or(*start_time);
                }
            }

            // If a task exists, submit an answer based on RNG for variety but determinism
            if let Some(task) = &self.current_task {
                // Choose answer deterministically based on RNG
                let answer_index = if let Some(rng) = &mut self.demo_rng {
                    // 70% chance of correct answer, 30% incorrect for realistic demo
                    if rng.gen_bool(0.7) {
                        // Find the correct answer index
                        task.display_options
                            .iter()
                            .position(|opt| opt == &task.core_task.correct_answer)
                            .unwrap_or(0)
                    } else {
                        // Pick a random incorrect answer
                        let incorrect_indices: Vec<usize> = task
                            .display_options
                            .iter()
                            .enumerate()
                            .filter(|(_, opt)| *opt != &task.core_task.correct_answer)
                            .map(|(idx, _)| idx)
                            .collect();
                        if !incorrect_indices.is_empty() {
                            incorrect_indices[rng.gen_range(0..incorrect_indices.len())]
                        } else {
                            0
                        }
                    }
                } else {
                    // Fallback to first option if no RNG
                    0
                };

                // Update metrics via submit_answer (will update model and session responses)
                // Use internal method to process synchronously for demo
                self.submit_answer_internal(answer_index, true);

                // Request hint and struggle checks to trigger intervention paths (no-op if disabled)
                self.request_hint();
                self.check_struggle_and_provide_help();
            }
        }

        // End the session synchronously for demo
        self.end_session_sync();

        // Start guided demo to explain results
        self.demo_start();

        // Friendly message for demo viewers
        self.success_message =
            Some("Demo showcase complete — check the Dashboard for results.".to_string());
        self.error_message = None;
    }

    // Guided demo controls
    pub fn demo_start(&mut self) {
        // Start the quick tour scenario
        if let Err(e) = self.demo_controller.start_scenario("quick_tour") {
            self.error_message = Some(format!("Failed to start demo: {}", e));
        }
    }

    pub fn demo_start_training(&mut self) {
        // Start the full training demo scenario
        if let Err(e) = self.demo_controller.start_scenario("training_demo") {
            self.error_message = Some(format!("Failed to start training demo: {}", e));
        }
    }

    pub fn demo_next_step(&mut self) {
        if !self.demo_controller.is_active {
            return;
        }

        // Execute any action for the current step
        // Note: We need to work around the borrow checker here
        let should_execute = self.demo_controller.get_current_step().is_some();
        if should_execute {
            // Clone the controller to avoid borrow issues
            let action = self
                .demo_controller
                .get_current_step()
                .and_then(|s| s.action.clone());
            if let Some(action) = action {
                match action {
                    demo::DemoAction::SubmitAnswer(answer) => {
                        self.selected_answer = Some(answer);
                        self.submit_response_in_flight = true;
                    }
                    demo::DemoAction::RequestHint => {
                        self.request_hint();
                    }
                    demo::DemoAction::NavigateTo(screen) => {
                        self.current_screen = screen;
                    }
                    demo::DemoAction::SelectDomain(domain) => {
                        // Handle domain selection
                        println!("Demo: Selecting domain {}", domain);
                    }
                    demo::DemoAction::RunMiniDemo => {
                        for i in 0..3 {
                            self.generate_next_task();
                            self.selected_answer = Some(format!("Answer{}", i));
                            self.submit_response_in_flight = true;
                        }
                    }
                    _ => {} // Handle other actions as needed
                }
            }
        }

        // Advance to next step
        if !self.demo_controller.next_step() {
            // Demo complete
            self.success_message = Some("Demo completed successfully!".to_string());
        }

        // Apply navigation if specified in the step
        if let Some(step) = self.demo_controller.get_current_step() {
            if let Some(screen) = &step.navigation {
                self.current_screen = screen.clone();
            }
        }
    }

    pub fn demo_end(&mut self) {
        self.demo_controller.end_demo();
        self.success_message = Some("Exited guided demo.".to_string());
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

    pub fn start_session_sync(&mut self) {
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

                // Store these locally
                self.task_generator = Some(task_generator);
                self.task_session = Some(task_session);
                self.intervention_system = Some(intervention_system);

                // Create session directly for demo
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

                // Store these locally for now
                self.task_generator = Some(task_generator);
                self.task_session = Some(task_session);
                self.intervention_system = Some(intervention_system);

                // Trigger async session creation
                self.create_session_in_flight = true;
                self.error_message = None;
                self.success_message = None;
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
                // For demos, just use random task generation
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
        self.submit_answer_internal(answer_index, false);
    }

    fn submit_answer_internal(&mut self, answer_index: usize, is_demo: bool) {
        // Prevent double submission
        if !is_demo && (self.submit_response_in_flight || self.show_feedback) {
            return;
        }

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

                // Prepare for async submission
                self.last_response_correct = correct;
                self.selected_answer = Some(answer);
                self.selected_answer_index = Some(answer_index);
                self.current_hint = None; // Clear any existing hint

                // For demos, process synchronously
                if is_demo {
                    self.show_feedback = true;
                } else {
                    // Trigger async submission for normal flow
                    self.submit_response_in_flight = true;
                }
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
        if self.current_session.is_some() && !self.end_session_in_flight {
            // Trigger async session end
            self.end_session_in_flight = true;
            self.error_message = None;
        }
    }

    fn end_session_sync(&mut self) {
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

            self.success_message = Some("Session completed!".to_string());
        }
    }

    pub fn export_current_data(&mut self) {
        if self.current_learner.is_some() && !self.export_data_in_flight {
            // Create UI-specific export synchronously for local state
            if let Some(learner) = &self.current_learner {
                let mut ui_export = ExportData {
                    learner: learner.clone(),
                    sessions: self
                        .current_session
                        .as_ref()
                        .map_or(vec![], |s| vec![s.clone()]),
                    metrics: self.current_metrics.clone(),
                    export_time: Utc::now(),
                    demo_seed: None,
                };

                // Include demo seed in export for reproducibility
                if let Some(seed) = self.demo_seed {
                    ui_export.demo_seed = Some(seed);
                }

                self.export_data = Some(ui_export);
            }

            // Trigger async export
            self.export_data_in_flight = true;
            self.error_message = None;
        } else if self.current_learner.is_none() {
            self.error_message = Some("No data to export".to_string());
        }
    }
}

// Re-export for tests
pub use models::User;

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
