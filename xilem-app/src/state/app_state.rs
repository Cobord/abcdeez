// Root application state

use abcdeez_core::learning::learner::LearnerModel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::models::{PendingResponse, Task};
use crate::services::AdaptiveLearningService;
use crate::state::{SessionState, Theme, ThemeMode, UserState};
use crate::utils::easter_egg::EasterEggManager;
use crate::demo::DemoController;
use crate::gamification::GamificationManager;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingKey {
    Login,
    Session,
    Task,
    Analytics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Screen {
    // Auth Flow
    Login,
    Signup,
    OAuthCallback,

    // Main App
    Dashboard,
    Learning,
    Welcome,
    DomainSelection,
    Training,

    // Analytics
    Progress,
    Analytics,
    Visualizations,

    // Social
    Leaderboard,
    Challenges,
    Friends,

    // Settings
    Settings,
    Profile,

    // Special
    Loading,
    Error(String),
    Logout,
    WidgetGallery,  // Hidden screen
}

impl Screen {
    pub fn title(&self) -> &str {
        match self {
            Screen::Login => "Login",
            Screen::Signup => "Sign Up",
            Screen::Dashboard => "Dashboard",
            Screen::Learning => "Learning Session",
            Screen::Welcome => "Welcome",
            Screen::DomainSelection => "Select Domain",
            Screen::Training => "Training",
            Screen::Progress => "Progress",
            Screen::Analytics => "Analytics",
            Screen::Visualizations => "Visualizations",
            Screen::Leaderboard => "Leaderboard",
            Screen::Challenges => "Challenges",
            Screen::Friends => "Friends",
            Screen::Settings => "Settings",
            Screen::Profile => "Profile",
            Screen::Loading => "Loading",
            Screen::Error(_) => "Error",
            Screen::Logout => "Logout",
            Screen::WidgetGallery => "Widget Gallery",
            _ => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub recoverable: bool,
}

pub struct AppState {
    // User & Authentication
    pub user: Option<UserState>,
    pub current_user: Option<UserState>,  // Compatibility alias
    pub auth_token: Option<String>,
    pub refresh_token: Option<String>,
    
    // Auth form fields
    pub auth_username: String,
    pub auth_password: String,
    pub auth_email: String,
    pub auth_confirm_password: String,
    pub username_input: String,  // Compatibility field for welcome screen
    pub password_input: String,  // Compatibility field for welcome screen
    
    // OAuth Authentication
    pub oauth_login_in_flight: bool,
    pub login_request_in_flight: bool,
    pub is_guest_mode: bool,
    
    // iOS Auth Bridge
    #[cfg(target_os = "ios")]
    pub ios_auth_bridge: Option<crate::auth::ios::IOSAuthBridge>,
    
    // Easter egg tracking
    pub triple_click_count: u32,
    pub last_triple_click: Option<DateTime<Utc>>,

    // Current Session
    pub session: Option<SessionState>,
    pub learner_model: Option<LearnerModel>,
    pub adaptive_service: Option<AdaptiveLearningService>,

    // Navigation
    pub current_screen: Screen,
    pub navigation_stack: Vec<Screen>,

    // UI State
    pub loading: HashSet<LoadingKey>,
    pub errors: Vec<AppError>,
    pub theme: Theme,
    pub theme_mode: ThemeMode,

    // Sync & Offline
    pub sync_queue: Vec<PendingResponse>,
    pub last_sync: Option<DateTime<Utc>>,
    pub connection_status: ConnectionStatus,

    // WebSocket
    pub ws_connected: bool,
    pub ws_reconnect_attempts: u32,

    // Performance Metrics
    pub frame_times: Vec<f64>,
    pub response_times: Vec<u128>,

    // Cache
    pub task_cache: HashMap<String, Task>,
    
    // Fun features
    pub easter_egg_manager: EasterEggManager,
    pub demo_controller: Option<DemoController>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user: None,
            current_user: None,
            auth_token: None,
            refresh_token: None,
            auth_username: String::new(),
            auth_password: String::new(),
            auth_email: String::new(),
            auth_confirm_password: String::new(),
            username_input: String::new(),
            password_input: String::new(),
            oauth_login_in_flight: false,
            login_request_in_flight: false,
            is_guest_mode: false,
            #[cfg(target_os = "ios")]
            ios_auth_bridge: Some(crate::auth::ios::IOSAuthBridge::new()),
            triple_click_count: 0,
            last_triple_click: None,
            session: None,
            learner_model: None,
            adaptive_service: None,
            current_screen: Screen::Welcome,
            navigation_stack: Vec::new(),
            loading: HashSet::new(),
            errors: Vec::new(),
            theme: Theme::default(),
            theme_mode: ThemeMode::Light,
            sync_queue: Vec::new(),
            last_sync: None,
            connection_status: ConnectionStatus::Disconnected,
            ws_connected: false,
            ws_reconnect_attempts: 0,
            frame_times: Vec::with_capacity(60),
            response_times: Vec::with_capacity(100),
            task_cache: HashMap::new(),
            easter_egg_manager: EasterEggManager::new(),
            demo_controller: Some(DemoController::new()),
        }
    }
}

#[cfg(feature = "xilem-native")]
impl xilem::AppState for AppState {
    fn keep_running(&self) -> bool {
        !matches!(self.current_screen, Screen::Logout)
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn navigate(&mut self, screen: Screen) {
        self.navigation_stack.push(self.current_screen.clone());
        self.current_screen = screen;
    }

    pub fn navigate_back(&mut self) {
        if let Some(previous) = self.navigation_stack.pop() {
            self.current_screen = previous;
        }
    }

    pub fn add_error(&mut self, message: String, recoverable: bool) {
        self.errors.push(AppError {
            message,
            timestamp: Utc::now(),
            recoverable,
        });
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn is_loading(&self, key: LoadingKey) -> bool {
        self.loading.contains(&key)
    }

    pub fn set_loading(&mut self, key: LoadingKey, loading: bool) {
        if loading {
            self.loading.insert(key);
        } else {
            self.loading.remove(&key);
        }
    }
    
    // Authentication methods
    pub fn login(&mut self) {
        self.login_request_in_flight = true;
        // TODO: Implement actual login logic
        // For now, just simulate successful login
        self.login_request_in_flight = false;
        self.current_screen = Screen::Dashboard;
    }
    
    pub fn apple_sign_in(&mut self) {
        self.oauth_login_in_flight = true;
        
        #[cfg(target_os = "ios")]
        {
            if let Some(ref bridge) = self.ios_auth_bridge {
                match bridge.apple_sign_in() {
                    Ok(result) => {
                        // Handle successful sign in
                        println!("Apple Sign In successful: {:?}", result.user_id);
                        self.current_screen = Screen::Dashboard;
                    }
                    Err(e) => {
                        self.add_error(format!("Apple Sign In failed: {}", e), true);
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "ios"))]
        {
            // Simulate Apple sign in for non-iOS platforms
            println!("Simulating Apple Sign In (not on iOS)");
            self.current_screen = Screen::Dashboard;
        }
        
        self.oauth_login_in_flight = false;
    }
    
    pub fn github_sign_in(&mut self) {
        self.oauth_login_in_flight = true;
        // TODO: Implement GitHub OAuth flow
        println!("GitHub Sign In initiated");
        // For now, simulate successful sign in
        self.current_screen = Screen::Dashboard;
        self.oauth_login_in_flight = false;
    }
    
    pub fn create_learner(&mut self) {
        // Create a new learner model for guest or authenticated user
        // Note: In a real implementation, we'd need topology and learner ID
        // For now, we'll skip this as it requires proper initialization
        println!("Creating new learner model (placeholder)");
    }
    
    // Easter egg methods
    pub fn try_crab_triple_click(&mut self) {
        let now = Utc::now();
        
        // Check if this is within 500ms of last click
        if let Some(last_click) = self.last_triple_click {
            let diff = now.signed_duration_since(last_click);
            if diff.num_milliseconds() < 500 {
                self.triple_click_count += 1;
            } else {
                self.triple_click_count = 1;
            }
        } else {
            self.triple_click_count = 1;
        }
        
        self.last_triple_click = Some(now);
        
        // Activate easter egg on triple click
        if self.triple_click_count >= 3 {
            self.easter_egg_manager.crab.activate(crate::utils::easter_egg::CrabTrigger::TripleClick);
            self.triple_click_count = 0;
            println!("🦀 Little Crab activated!");
        }
    }
    
    // Demo methods
    pub fn demo_start(&mut self) {
        println!("Starting Quick Tour demo");
        if let Some(ref mut demo) = self.demo_controller {
            demo.start_scenario("quick_tour");
        }
    }
    
    pub fn demo_start_training(&mut self) {
        println!("Starting Training Demo");
        if let Some(ref mut demo) = self.demo_controller {
            demo.start_scenario("training_demo");
        }
    }
    
    pub fn demo_showcase(&mut self) {
        println!("Running Demo Showcase");
        if let Some(ref mut demo) = self.demo_controller {
            demo.start_scenario("showcase");
            // Populate dashboard with demo data
            self.populate_demo_data();
        }
    }
    
    fn populate_demo_data(&mut self) {
        // Create demo session data for dashboard
        use crate::models::Response;
        
        // Create some demo responses for visualization
        let demo_responses = vec![
            Response {
                correct: true,
                response_time_ms: 1250,
                timestamp: Utc::now() - chrono::Duration::minutes(10),
                id: uuid::Uuid::new_v4(),
                session_id: uuid::Uuid::new_v4(),
                sequence_number: 1,
                task_data: serde_json::json!({
                    "prompt": "Demo task 1",
                    "task_type": "sequence",
                    "options": ["A", "B", "C", "D"],
                    "correct_answer": "B",
                    "difficulty": 0.5
                }),
                hint_level: Some(0),
                user_answer: Some("B".to_string()),
                task_type: "sequence".to_string(),
            },
            Response {
                correct: true,
                response_time_ms: 980,
                timestamp: Utc::now() - chrono::Duration::minutes(8),
                id: uuid::Uuid::new_v4(),
                session_id: uuid::Uuid::new_v4(),
                sequence_number: 2,
                task_data: serde_json::json!({
                    "prompt": "Demo task 2",
                    "task_type": "ordering",
                    "options": ["A", "B", "C", "D"],
                    "correct_answer": "C",
                    "difficulty": 0.6
                }),
                hint_level: Some(0),
                user_answer: Some("C".to_string()),
                task_type: "ordering".to_string(),
            },
            Response {
                correct: false,
                response_time_ms: 2100,
                timestamp: Utc::now() - chrono::Duration::minutes(6),
                id: uuid::Uuid::new_v4(),
                session_id: uuid::Uuid::new_v4(),
                sequence_number: 3,
                task_data: serde_json::json!({
                    "prompt": "Demo task 3",
                    "task_type": "comparison",
                    "options": ["A", "B", "C", "D"],
                    "correct_answer": "B",
                    "difficulty": 0.7
                }),
                hint_level: Some(0),
                user_answer: Some("A".to_string()),
                task_type: "comparison".to_string(),
            },
            Response {
                correct: true,
                response_time_ms: 750,
                timestamp: Utc::now() - chrono::Duration::minutes(4),
                id: uuid::Uuid::new_v4(),
                session_id: uuid::Uuid::new_v4(),
                sequence_number: 4,
                task_data: serde_json::json!({
                    "prompt": "Demo task 4",
                    "task_type": "path_finding",
                    "options": ["A", "B", "C", "D"],
                    "correct_answer": "D",
                    "difficulty": 0.8
                }),
                hint_level: Some(0),
                user_answer: Some("D".to_string()),
                task_type: "path".to_string(),
            },
            Response {
                correct: true,
                response_time_ms: 1100,
                timestamp: Utc::now() - chrono::Duration::minutes(2),
                id: uuid::Uuid::new_v4(),
                session_id: uuid::Uuid::new_v4(),
                sequence_number: 5,
                task_data: serde_json::json!({
                    "prompt": "Demo task 5",
                    "task_type": "sequence",
                    "options": ["A", "B", "C", "D"],
                    "correct_answer": "B",
                    "difficulty": 0.9
                }),
                hint_level: Some(0),
                user_answer: Some("B".to_string()),
                task_type: "sequence".to_string(),
            },
        ];
        
        // Update response times for performance metrics
        self.response_times = demo_responses.iter()
            .map(|r| r.response_time_ms)
            .collect();
        
        println!("Demo data populated for dashboard");
    }
}