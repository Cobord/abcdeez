// Enhanced demo system for guided tours and interactive walkthroughs
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use crate::state::{AppState, Screen};

/// Represents a single step in the demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub highlight_elements: Vec<String>,
    pub action: Option<DemoAction>,
    pub navigation: Option<Screen>,
    pub wait_for_user: bool,
    pub auto_advance_ms: Option<u64>,
    pub validation: Option<StepValidation>,
    pub skip_condition: Option<SkipCondition>,
    pub on_enter_script: Option<String>,
    pub on_exit_script: Option<String>,
}

/// Actions that can be performed during a demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DemoAction {
    ClickButton(String),
    EnterText { field: String, text: String },
    SelectDomain(String),
    SubmitAnswer(String),
    RequestHint,
    NavigateTo(Screen),
    ShowTooltip { element: String, text: String },
    RunMiniDemo,
    Wait(u64), // Wait for milliseconds
    SimulateTyping { field: String, text: String, delay_ms: u64 },
    HighlightSequence(Vec<String>), // Highlight elements in sequence
    ShowNotification { message: String, type_: NotificationType },
    PlaySound(String),
    TriggerEasterEgg,
    ShowConfetti,
    ExecuteCallback(String), // Named callback to execute
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

/// Validation to check before advancing to next step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepValidation {
    ElementExists(String),
    ElementHasValue { element: String, value: String },
    StateCondition(String), // Check app state condition
    CustomValidation(String), // Custom validation function name
}

/// Condition to skip this step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkipCondition {
    IfCompleted(String), // Skip if user has completed something
    IfState(String), // Skip based on app state
    IfTime { before_hour: u8, after_hour: u8 },
}

/// Highlight style for UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightStyle {
    pub color: String,
    pub animation: HighlightAnimation,
    pub opacity: f32,
    pub border_width: f32,
    pub z_index: i32,
    pub pointer_events: bool, // Allow interaction with highlighted element
    pub dim_background: bool, // Dim everything except highlighted element
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HighlightAnimation {
    None,
    Pulse,
    Glow,
    Bounce,
    Arrow,
    Ripple,
    Shake,
    Spotlight,
}

impl Default for HighlightStyle {
    fn default() -> Self {
        Self {
            color: "#4CAF50".to_string(),
            animation: HighlightAnimation::Pulse,
            opacity: 0.3,
            border_width: 3.0,
            z_index: 9999,
            pointer_events: true,
            dim_background: false,
        }
    }
}

/// Demo scenario - a collection of steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: DemoCategory,
    pub difficulty: DemoDifficulty,
    pub estimated_duration_minutes: u32,
    pub prerequisites: Vec<String>, // Other scenario IDs that should be completed first
    pub steps: Vec<DemoStep>,
    pub completion_message: String,
    pub completion_rewards: Vec<DemoReward>,
    pub allow_skip: bool,
    pub allow_restart: bool,
    pub track_analytics: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DemoCategory {
    Onboarding,
    Feature,
    Advanced,
    Troubleshooting,
    Tips,
    Research,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DemoDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DemoReward {
    Achievement(String),
    UnlockFeature(String),
    Points(u32),
    Badge(String),
}

/// Demo state for tracking progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoState {
    pub completed_scenarios: Vec<String>,
    pub started_scenarios: HashMap<String, DemoProgress>,
    pub total_time_spent: Duration,
    pub achievements: Vec<String>,
    pub skip_count: u32,
    pub restart_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoProgress {
    pub scenario_id: String,
    pub current_step: usize,
    pub started_at: String, // ISO timestamp
    pub last_active: String, // ISO timestamp
    pub completion_percentage: f32,
}

/// Analytics for demo usage
#[derive(Debug, Clone)]
pub struct DemoAnalytics {
    pub step_durations: HashMap<String, Vec<Duration>>,
    pub step_failures: HashMap<String, u32>,
    pub step_skips: HashMap<String, u32>,
    pub help_requests: HashMap<String, u32>,
    pub drop_off_points: Vec<(String, String)>, // (scenario_id, step_id)
}

/// Main demo controller with enhanced features
pub struct DemoController {
    pub scenarios: HashMap<String, DemoScenario>,
    pub current_scenario: Option<String>,
    pub current_step_index: usize,
    pub highlights: HashMap<String, HighlightStyle>,
    pub is_active: bool,
    pub is_paused: bool,
    pub step_history: Vec<String>,
    pub tooltips: HashMap<String, String>,
    pub state: DemoState,
    pub analytics: DemoAnalytics,
    pub callbacks: HashMap<String, Box<dyn Fn(&mut AppState) + Send + Sync>>,
    pub step_timer: Option<Instant>,
    pub auto_advance_timer: Option<Instant>,
    pub user_interactions: VecDeque<UserInteraction>,
    pub demo_speed: f32, // Speed multiplier for animations and delays
    pub voice_enabled: bool,
    pub subtitle_enabled: bool,
    pub keyboard_shortcuts_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UserInteraction {
    pub timestamp: Instant,
    pub interaction_type: InteractionType,
    pub element_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum InteractionType {
    Click,
    Hover,
    KeyPress(String),
    Scroll,
    Focus,
}

impl Default for DemoController {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoController {
    pub fn new() -> Self {
        let mut controller = Self {
            scenarios: HashMap::new(),
            current_scenario: None,
            current_step_index: 0,
            highlights: HashMap::new(),
            is_active: false,
            is_paused: false,
            step_history: Vec::new(),
            tooltips: HashMap::new(),
            state: DemoState {
                completed_scenarios: Vec::new(),
                started_scenarios: HashMap::new(),
                total_time_spent: Duration::ZERO,
                achievements: Vec::new(),
                skip_count: 0,
                restart_count: 0,
            },
            analytics: DemoAnalytics {
                step_durations: HashMap::new(),
                step_failures: HashMap::new(),
                step_skips: HashMap::new(),
                help_requests: HashMap::new(),
                drop_off_points: Vec::new(),
            },
            callbacks: HashMap::new(),
            step_timer: None,
            auto_advance_timer: None,
            user_interactions: VecDeque::with_capacity(100),
            demo_speed: 1.0,
            voice_enabled: false,
            subtitle_enabled: true,
            keyboard_shortcuts_enabled: true,
        };
        
        controller.init_default_scenarios();
        controller.register_default_callbacks();
        controller
    }
    
    fn register_default_callbacks(&mut self) {
        // Register common callbacks that demos can use
        // Note: In real implementation, these would modify AppState
    }
    
    fn init_default_scenarios(&mut self) {
        // Onboarding - First Time User Experience
        let first_time_tour = DemoScenario {
            id: "first_time_tour".to_string(),
            name: "Welcome Tour".to_string(),
            description: "Perfect for first-time users to understand the basics".to_string(),
            category: DemoCategory::Onboarding,
            difficulty: DemoDifficulty::Beginner,
            estimated_duration_minutes: 5,
            prerequisites: vec![],
            steps: vec![
                DemoStep {
                    id: "welcome".to_string(),
                    title: "Welcome to ABCDEEZ! 🎉".to_string(),
                    description: "Let's take a quick tour to get you started with adaptive learning.".to_string(),
                    highlight_elements: vec![],
                    action: Some(DemoAction::ShowConfetti),
                    navigation: Some(Screen::Welcome),
                    wait_for_user: true,
                    auto_advance_ms: None,
                    validation: None,
                    skip_condition: None,
                    on_enter_script: Some("analytics.track('demo_started')".to_string()),
                    on_exit_script: None,
                },
                DemoStep {
                    id: "explain_adaptive".to_string(),
                    title: "What is Adaptive Learning?".to_string(),
                    description: "Our system adapts to YOUR learning pace and style, making learning more efficient and enjoyable!".to_string(),
                    highlight_elements: vec!["adaptive_info".to_string()],
                    action: Some(DemoAction::ShowNotification {
                        message: "The system learns from your responses!".to_string(),
                        type_: NotificationType::Info,
                    }),
                    navigation: None,
                    wait_for_user: true,
                    auto_advance_ms: Some(5000),
                    validation: None,
                    skip_condition: None,
                    on_enter_script: None,
                    on_exit_script: None,
                },
            ],
            completion_message: "🎊 Congratulations! You're ready to start your learning journey!".to_string(),
            completion_rewards: vec![
                DemoReward::Achievement("First Steps".to_string()),
                DemoReward::Points(100),
            ],
            allow_skip: true,
            allow_restart: true,
            track_analytics: true,
        };
        
        // Feature Deep Dive - Learning Session
        let learning_session_tour = DemoScenario {
            id: "learning_session_tour".to_string(),
            name: "Master the Learning Session".to_string(),
            description: "Deep dive into all learning session features".to_string(),
            category: DemoCategory::Feature,
            difficulty: DemoDifficulty::Intermediate,
            estimated_duration_minutes: 10,
            prerequisites: vec!["first_time_tour".to_string()],
            steps: vec![
                DemoStep {
                    id: "session_start".to_string(),
                    title: "Starting Your Learning Session".to_string(),
                    description: "Let's explore how to start and customize your learning experience.".to_string(),
                    highlight_elements: vec!["start_session_button".to_string()],
                    action: Some(DemoAction::HighlightSequence(vec![
                        "domain_selector".to_string(),
                        "difficulty_slider".to_string(),
                        "session_options".to_string(),
                    ])),
                    navigation: Some(Screen::DomainSelection),
                    wait_for_user: false,
                    auto_advance_ms: Some(3000),
                    validation: Some(StepValidation::ElementExists("domain_cards".to_string())),
                    skip_condition: None,
                    on_enter_script: None,
                    on_exit_script: None,
                },
                DemoStep {
                    id: "answer_techniques".to_string(),
                    title: "Answering Techniques".to_string(),
                    description: "Learn different ways to submit answers: clicking, keyboard shortcuts, or voice input!".to_string(),
                    highlight_elements: vec!["answer_area".to_string()],
                    action: Some(DemoAction::SimulateTyping {
                        field: "answer_input".to_string(),
                        text: "Your answer here".to_string(),
                        delay_ms: 100,
                    }),
                    navigation: Some(Screen::Training),
                    wait_for_user: true,
                    auto_advance_ms: None,
                    validation: None,
                    skip_condition: None,
                    on_enter_script: None,
                    on_exit_script: None,
                },
            ],
            completion_message: "🎯 You've mastered the learning session! Time to practice!".to_string(),
            completion_rewards: vec![
                DemoReward::Achievement("Session Master".to_string()),
                DemoReward::UnlockFeature("advanced_hints".to_string()),
                DemoReward::Points(250),
            ],
            allow_skip: true,
            allow_restart: true,
            track_analytics: true,
        };
        
        // Advanced Features Tour
        let advanced_features = DemoScenario {
            id: "advanced_features".to_string(),
            name: "Advanced Features".to_string(),
            description: "Discover hidden features and power user tips".to_string(),
            category: DemoCategory::Advanced,
            difficulty: DemoDifficulty::Advanced,
            estimated_duration_minutes: 15,
            prerequisites: vec!["learning_session_tour".to_string()],
            steps: vec![
                DemoStep {
                    id: "keyboard_shortcuts".to_string(),
                    title: "Keyboard Shortcuts ⌨️".to_string(),
                    description: "Power users love shortcuts! Press 'H' for hints, 'Space' to submit, 'N' for next task.".to_string(),
                    highlight_elements: vec![],
                    action: Some(DemoAction::ShowTooltip {
                        element: "keyboard_guide".to_string(),
                        text: "Press '?' anytime to see all shortcuts".to_string(),
                    }),
                    navigation: None,
                    wait_for_user: true,
                    auto_advance_ms: None,
                    validation: None,
                    skip_condition: None,
                    on_enter_script: None,
                    on_exit_script: None,
                },
                DemoStep {
                    id: "easter_eggs".to_string(),
                    title: "Hidden Surprises! 🦀".to_string(),
                    description: "There are hidden easter eggs throughout the app. Try triple-clicking or typing 'crab'!".to_string(),
                    highlight_elements: vec![],
                    action: Some(DemoAction::TriggerEasterEgg),
                    navigation: None,
                    wait_for_user: true,
                    auto_advance_ms: None,
                    validation: None,
                    skip_condition: None,
                    on_enter_script: None,
                    on_exit_script: None,
                },
            ],
            completion_message: "🚀 You're now a power user! Enjoy all the advanced features!".to_string(),
            completion_rewards: vec![
                DemoReward::Achievement("Power User".to_string()),
                DemoReward::Badge("Advanced".to_string()),
                DemoReward::Points(500),
            ],
            allow_skip: true,
            allow_restart: false,
            track_analytics: true,
        };
        
        // Research Features Tour
        let research_tour = DemoScenario {
            id: "research_tour".to_string(),
            name: "Research Dashboard".to_string(),
            description: "Learn how to use research and analytics features".to_string(),
            category: DemoCategory::Research,
            difficulty: DemoDifficulty::Expert,
            estimated_duration_minutes: 20,
            prerequisites: vec!["advanced_features".to_string()],
            steps: vec![
                DemoStep {
                    id: "research_intro".to_string(),
                    title: "Research Dashboard Overview".to_string(),
                    description: "The research dashboard provides deep insights into learning patterns and cognitive metrics.".to_string(),
                    highlight_elements: vec!["research_dashboard".to_string()],
                    action: None,
                    navigation: Some(Screen::Analytics),
                    wait_for_user: true,
                    auto_advance_ms: None,
                    validation: None,
                    skip_condition: Some(SkipCondition::IfCompleted("researcher_role".to_string())),
                    on_enter_script: None,
                    on_exit_script: None,
                },
            ],
            completion_message: "🔬 You're ready to conduct learning research!".to_string(),
            completion_rewards: vec![
                DemoReward::Achievement("Researcher".to_string()),
                DemoReward::UnlockFeature("export_data".to_string()),
                DemoReward::Badge("Science".to_string()),
            ],
            allow_skip: false,
            allow_restart: true,
            track_analytics: true,
        };
        
        // Quick Tips
        let quick_tips = DemoScenario {
            id: "quick_tips".to_string(),
            name: "Quick Tips".to_string(),
            description: "Short tips to improve your learning".to_string(),
            category: DemoCategory::Tips,
            difficulty: DemoDifficulty::Beginner,
            estimated_duration_minutes: 2,
            prerequisites: vec![],
            steps: vec![
                DemoStep {
                    id: "tip_streaks".to_string(),
                    title: "Build Learning Streaks! 🔥".to_string(),
                    description: "Practice daily to build streaks and earn bonus points!".to_string(),
                    highlight_elements: vec!["streak_counter".to_string()],
                    action: None,
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(3000),
                    validation: None,
                    skip_condition: None,
                    on_enter_script: None,
                    on_exit_script: None,
                },
            ],
            completion_message: "💡 Now you know the secrets to effective learning!".to_string(),
            completion_rewards: vec![DemoReward::Points(50)],
            allow_skip: true,
            allow_restart: true,
            track_analytics: false,
        };
        
        // Add all scenarios
        self.scenarios.insert(first_time_tour.id.clone(), first_time_tour);
        self.scenarios.insert(learning_session_tour.id.clone(), learning_session_tour);
        self.scenarios.insert(advanced_features.id.clone(), advanced_features);
        self.scenarios.insert(research_tour.id.clone(), research_tour);
        self.scenarios.insert(quick_tips.id.clone(), quick_tips);
    }
    
    /// Start a demo scenario with validation
    pub fn start_scenario(&mut self, scenario_id: &str) -> Result<(), String> {
        // Check if scenario exists
        let scenario = self.scenarios.get(scenario_id)
            .ok_or_else(|| format!("Scenario '{}' not found", scenario_id))?;
        
        // Check prerequisites
        for prereq in &scenario.prerequisites {
            if !self.state.completed_scenarios.contains(prereq) {
                return Err(format!("Complete '{}' first", prereq));
            }
        }
        
        // Initialize demo
        self.current_scenario = Some(scenario_id.to_string());
        self.current_step_index = 0;
        self.is_active = true;
        self.is_paused = false;
        self.step_history.clear();
        self.highlights.clear();
        self.tooltips.clear();
        self.step_timer = Some(Instant::now());
        
        // Track in state
        self.state.started_scenarios.insert(
            scenario_id.to_string(),
            DemoProgress {
                scenario_id: scenario_id.to_string(),
                current_step: 0,
                started_at: chrono::Utc::now().to_rfc3339(),
                last_active: chrono::Utc::now().to_rfc3339(),
                completion_percentage: 0.0,
            },
        );
        
        self.apply_current_step();
        Ok(())
    }
    
    /// Advance to next step with validation
    pub fn next_step(&mut self) -> Result<bool, String> {
        if !self.is_active || self.is_paused {
            return Ok(false);
        }
        
        // Validate current step before advancing
        if let Some(step) = self.get_current_step() {
            if let Some(validation) = &step.validation {
                if !self.validate_step(validation)? {
                    return Err("Step validation failed".to_string());
                }
            }
        }
        
        // Record step duration
        if let Some(timer) = self.step_timer {
            let duration = timer.elapsed();
            if let Some(step) = self.get_current_step() {
                self.analytics.step_durations
                    .entry(step.id.clone())
                    .or_insert_with(Vec::new)
                    .push(duration);
            }
        }
        
        let scenario_id = self.current_scenario.clone().unwrap();
        let scenario = self.scenarios.get(&scenario_id).unwrap();
        
        if self.current_step_index < scenario.steps.len() - 1 {
            // Record history
            if let Some(current_step) = scenario.steps.get(self.current_step_index) {
                self.step_history.push(current_step.id.clone());
                
                // Execute exit script
                if let Some(script) = &current_step.on_exit_script {
                    self.execute_script(script);
                }
            }
            
            self.current_step_index += 1;
            self.step_timer = Some(Instant::now());
            
            // Update progress
            if let Some(progress) = self.state.started_scenarios.get_mut(&scenario_id) {
                progress.current_step = self.current_step_index;
                progress.completion_percentage = 
                    (self.current_step_index as f32 / scenario.steps.len() as f32) * 100.0;
                progress.last_active = chrono::Utc::now().to_rfc3339();
            }
            
            // Check skip condition for new step
            if let Some(step) = scenario.steps.get(self.current_step_index) {
                if let Some(skip_condition) = &step.skip_condition {
                    if self.should_skip(skip_condition) {
                        return self.next_step(); // Recursively skip
                    }
                }
            }
            
            self.apply_current_step();
            Ok(true)
        } else {
            self.complete_demo();
            Ok(false)
        }
    }
    
    /// Go to previous step
    pub fn previous_step(&mut self) -> bool {
        if !self.is_active || self.current_step_index == 0 {
            return false;
        }
        
        self.current_step_index -= 1;
        self.step_timer = Some(Instant::now());
        self.apply_current_step();
        true
    }
    
    /// Skip current step
    pub fn skip_step(&mut self) -> Result<bool, String> {
        let scenario = self.get_current_scenario()
            .ok_or("No active scenario")?;
        
        if !scenario.allow_skip {
            return Err("This demo doesn't allow skipping".to_string());
        }
        
        self.state.skip_count += 1;
        
        if let Some(step) = self.get_current_step() {
            self.analytics.step_skips
                .entry(step.id.clone())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
        
        self.next_step()
    }
    
    /// Restart current scenario
    pub fn restart_scenario(&mut self) -> Result<(), String> {
        let scenario = self.get_current_scenario()
            .ok_or("No active scenario")?;
        
        if !scenario.allow_restart {
            return Err("This demo doesn't allow restarting".to_string());
        }
        
        self.state.restart_count += 1;
        self.current_step_index = 0;
        self.step_history.clear();
        self.apply_current_step();
        Ok(())
    }
    
    /// Apply current step (highlights, tooltips, etc.)
    fn apply_current_step(&mut self) {
        self.highlights.clear();
        self.tooltips.clear();
        
        // Clone the step to avoid borrow issues
        let step = self.get_current_step().cloned();
        
        if let Some(step) = step {
            // Apply highlights with custom styles
            for element_id in &step.highlight_elements {
                let mut style = HighlightStyle::default();
                
                // Customize style based on step
                if step.title.contains("Important") {
                    style.color = "#FF5722".to_string();
                    style.animation = HighlightAnimation::Shake;
                }
                
                self.highlights.insert(element_id.clone(), style);
            }
            
            // Apply tooltips
            if let Some(DemoAction::ShowTooltip { element, text }) = &step.action {
                self.tooltips.insert(element.clone(), text.clone());
            }
            
            // Execute enter script
            if let Some(script) = &step.on_enter_script {
                self.execute_script(script);
            }
            
            // Set up auto-advance timer
            if let Some(ms) = step.auto_advance_ms {
                let adjusted_ms = (ms as f32 / self.demo_speed) as u64;
                self.auto_advance_timer = Some(Instant::now() + Duration::from_millis(adjusted_ms));
            }
        }
    }
    
    /// Validate a step condition
    fn validate_step(&self, validation: &StepValidation) -> Result<bool, String> {
        match validation {
            StepValidation::ElementExists(id) => {
                // In real implementation, check if element exists in DOM
                Ok(true)
            }
            StepValidation::ElementHasValue { element, value } => {
                // In real implementation, check element value
                Ok(true)
            }
            StepValidation::StateCondition(condition) => {
                // In real implementation, evaluate state condition
                Ok(true)
            }
            StepValidation::CustomValidation(name) => {
                // In real implementation, call custom validation
                Ok(true)
            }
        }
    }
    
    /// Check if step should be skipped
    fn should_skip(&self, condition: &SkipCondition) -> bool {
        match condition {
            SkipCondition::IfCompleted(task) => {
                self.state.completed_scenarios.contains(task)
            }
            SkipCondition::IfState(_state) => {
                // In real implementation, check app state
                false
            }
            SkipCondition::IfTime { before_hour, after_hour } => {
                let hour = chrono::Local::now().time().hour() as u8;
                hour < *before_hour || hour > *after_hour
            }
        }
    }
    
    /// Execute a script (simplified)
    fn execute_script(&self, _script: &str) {
        // In real implementation, execute JavaScript or command
    }
    
    /// Complete the current demo
    fn complete_demo(&mut self) {
        if let Some(scenario_id) = &self.current_scenario.clone() {
            // Clone scenario data to avoid borrow issues
            let scenario_data = self.scenarios.get(scenario_id).map(|s| (
                s.completion_rewards.clone(),
                s.track_analytics,
                s.completion_message.clone()
            ));
            
            if let Some((rewards, track_analytics, _completion_msg)) = scenario_data {
                // Mark as completed
                if !self.state.completed_scenarios.contains(scenario_id) {
                    self.state.completed_scenarios.push(scenario_id.clone());
                }
                
                // Remove from started
                self.state.started_scenarios.remove(scenario_id);
                
                // Process rewards
                for reward in &rewards {
                    self.process_reward(reward);
                }
                
                // Track analytics
                if track_analytics {
                    // In real implementation, send analytics
                }
            }
        }
        
        self.is_active = false;
        self.current_scenario = None;
    }
    
    /// Process a demo reward
    fn process_reward(&mut self, reward: &DemoReward) {
        match reward {
            DemoReward::Achievement(name) => {
                self.state.achievements.push(name.clone());
            }
            DemoReward::UnlockFeature(_feature) => {
                // In real implementation, unlock feature
            }
            DemoReward::Points(_points) => {
                // In real implementation, add points
            }
            DemoReward::Badge(_badge) => {
                // In real implementation, award badge
            }
        }
    }
    
    /// Update demo based on elapsed time
    pub fn update(&mut self, _app_state: &mut AppState) {
        if !self.is_active || self.is_paused {
            return;
        }
        
        // Check auto-advance timer
        if let Some(timer) = self.auto_advance_timer {
            if Instant::now() >= timer {
                self.auto_advance_timer = None;
                let _ = self.next_step();
            }
        }
        
        // Update total time spent
        if let Some(timer) = self.step_timer {
            self.state.total_time_spent = timer.elapsed();
        }
    }
    
    /// Execute current step action
    pub fn execute_current_action(&mut self, app_state: &mut AppState) {
        if let Some(step) = self.get_current_step() {
            if let Some(action) = &step.action {
                match action {
                    DemoAction::NavigateTo(screen) => {
                        app_state.current_screen = screen.clone();
                    }
                    DemoAction::ShowNotification { message, type_ } => {
                        // In real implementation, show notification
                        println!("Demo notification: {:?} - {}", type_, message);
                    }
                    DemoAction::TriggerEasterEgg => {
                        app_state.easter_egg_manager.handle_click(0.5, 0.5);
                        app_state.easter_egg_manager.handle_click(0.5, 0.5);
                        app_state.easter_egg_manager.handle_click(0.5, 0.5);
                    }
                    DemoAction::ShowConfetti => {
                        // In real implementation, show confetti animation
                        println!("🎊 Confetti!");
                    }
                    DemoAction::PlaySound(sound) => {
                        // In real implementation, play sound
                        println!("🔊 Playing: {}", sound);
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Handle user interaction
    pub fn handle_interaction(&mut self, interaction: UserInteraction) {
        self.user_interactions.push_back(interaction.clone());
        if self.user_interactions.len() > 100 {
            self.user_interactions.pop_front();
        }
        
        // Check if interaction matches expected action
        if let Some(step) = self.get_current_step() {
            if step.wait_for_user {
                match interaction.interaction_type {
                    InteractionType::Click => {
                        if let Some(DemoAction::ClickButton(button_id)) = &step.action {
                            if interaction.element_id.as_ref() == Some(button_id) {
                                let _ = self.next_step();
                            }
                        }
                    }
                    InteractionType::KeyPress(key) => {
                        // Handle keyboard shortcuts
                        match key.as_str() {
                            "Enter" | " " => { let _ = self.next_step(); }
                            "Escape" => { self.pause(); }
                            "?" => { self.show_help(); }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Show help for current demo
    fn show_help(&self) {
        println!("Demo Help:");
        println!("  Space/Enter - Next step");
        println!("  Escape - Pause demo");
        println!("  S - Skip step");
        println!("  R - Restart demo");
        println!("  Q - Quit demo");
    }
    
    // Getters
    pub fn get_current_step(&self) -> Option<&DemoStep> {
        self.current_scenario.as_ref().and_then(|id| {
            self.scenarios.get(id).and_then(|s| {
                s.steps.get(self.current_step_index)
            })
        })
    }
    
    pub fn get_current_scenario(&self) -> Option<&DemoScenario> {
        self.current_scenario.as_ref().and_then(|id| {
            self.scenarios.get(id)
        })
    }
    
    pub fn get_progress(&self) -> (usize, usize) {
        self.get_current_scenario()
            .map(|s| (self.current_step_index + 1, s.steps.len()))
            .unwrap_or((0, 0))
    }
    
    pub fn get_available_scenarios(&self) -> Vec<&DemoScenario> {
        let mut scenarios: Vec<_> = self.scenarios.values().collect();
        scenarios.sort_by_key(|s| s.category as u8);
        scenarios
    }
    
    pub fn get_recommended_scenario(&self) -> Option<&DemoScenario> {
        // Find the first uncompleted scenario with met prerequisites
        self.scenarios.values()
            .filter(|s| !self.state.completed_scenarios.contains(&s.id))
            .filter(|s| s.prerequisites.iter().all(|p| 
                self.state.completed_scenarios.contains(p)))
            .min_by_key(|s| s.difficulty as u8)
    }
    
    // Control methods
    pub fn pause(&mut self) {
        self.is_paused = true;
    }
    
    pub fn resume(&mut self) {
        self.is_paused = false;
        self.step_timer = Some(Instant::now());
    }
    
    pub fn end_demo(&mut self) {
        // Track drop-off point
        if let Some(scenario_id) = &self.current_scenario {
            if let Some(step) = self.get_current_step() {
                self.analytics.drop_off_points.push((
                    scenario_id.clone(),
                    step.id.clone(),
                ));
            }
        }
        
        self.is_active = false;
        self.is_paused = false;
        self.current_scenario = None;
        self.current_step_index = 0;
        self.highlights.clear();
        self.tooltips.clear();
        self.auto_advance_timer = None;
        self.step_timer = None;
    }
    
    pub fn set_speed(&mut self, speed: f32) {
        self.demo_speed = speed.max(0.1).min(5.0);
    }
    
    pub fn should_highlight(&self, element_id: &str) -> Option<&HighlightStyle> {
        self.highlights.get(element_id)
    }
    
    pub fn get_tooltip(&self, element_id: &str) -> Option<&String> {
        self.tooltips.get(element_id)
    }
}

/// Create a demo overlay component
pub fn demo_overlay_text(controller: &DemoController) -> String {
    if !controller.is_active {
        return String::new();
    }
    
    if let Some(step) = controller.get_current_step() {
        let (current, total) = controller.get_progress();
        let scenario = controller.get_current_scenario().unwrap();
        
        format!(
            "📚 {} - {} ({}/{})\n\n{}\n\n{}\n\nPress Space to continue • Esc to pause • ? for help",
            scenario.name,
            step.title,
            current,
            total,
            step.description,
            if controller.is_paused { "⏸ PAUSED" } else { "" }
        )
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_controller_initialization() {
        let controller = DemoController::new();
        assert!(!controller.is_active);
        assert!(controller.scenarios.len() >= 5);
        assert!(controller.scenarios.contains_key("first_time_tour"));
    }
    
    #[test]
    fn test_prerequisites() {
        let mut controller = DemoController::new();
        
        // Should fail to start advanced tour without prerequisites
        let result = controller.start_scenario("advanced_features");
        assert!(result.is_err());
        
        // Complete prerequisites
        controller.state.completed_scenarios.push("first_time_tour".to_string());
        controller.state.completed_scenarios.push("learning_session_tour".to_string());
        
        // Now should succeed
        let result = controller.start_scenario("advanced_features");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_demo_rewards() {
        let mut controller = DemoController::new();
        controller.state.completed_scenarios.push("first_time_tour".to_string());
        controller.state.completed_scenarios.push("learning_session_tour".to_string());
        
        controller.start_scenario("advanced_features").unwrap();
        
        // Complete all steps
        while controller.next_step().unwrap_or(false) {}
        
        // Check rewards were processed
        assert!(controller.state.achievements.contains(&"Power User".to_string()));
        assert!(controller.state.completed_scenarios.contains(&"advanced_features".to_string()));
    }
    
    #[test]
    fn test_skip_conditions() {
        let mut controller = DemoController::new();
        
        // Mark researcher role as completed
        controller.state.completed_scenarios.push("researcher_role".to_string());
        controller.state.completed_scenarios.push("first_time_tour".to_string());
        controller.state.completed_scenarios.push("learning_session_tour".to_string());
        controller.state.completed_scenarios.push("advanced_features".to_string());
        
        controller.start_scenario("research_tour").unwrap();
        
        // First step should be skipped due to skip condition
        let step = controller.get_current_step().unwrap();
        assert_ne!(step.id, "research_intro");
    }
    
    #[test]
    fn test_demo_analytics() {
        let mut controller = DemoController::new();
        controller.start_scenario("quick_tips").unwrap();
        
        // Simulate some interactions
        controller.next_step().unwrap();
        
        // Skip a step
        controller.skip_step().unwrap();
        
        assert_eq!(controller.state.skip_count, 1);
        assert!(controller.analytics.step_skips.contains_key("tip_streaks"));
    }
}