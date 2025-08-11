// Root application state

use abcdeez_core::learning::learner::LearnerModel;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

use crate::models::{PendingResponse, Task};
use crate::services::AdaptiveLearningService;
use crate::state::{SessionState, Theme, ThemeMode, UserState};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingKey {
    Login,
    Session,
    Task,
    Analytics,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    // Auth Flow
    Login,
    Signup,
    OAuthCallback,

    // Main App
    Dashboard,
    Learning,

    // Analytics
    Progress,
    Analytics,

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
}

impl Screen {
    pub fn title(&self) -> &str {
        match self {
            Screen::Login => "Login",
            Screen::Signup => "Sign Up",
            Screen::Dashboard => "Dashboard",
            Screen::Learning => "Learning Session",
            Screen::Progress => "Progress",
            Screen::Analytics => "Analytics",
            Screen::Leaderboard => "Leaderboard",
            Screen::Challenges => "Challenges",
            Screen::Friends => "Friends",
            Screen::Settings => "Settings",
            Screen::Profile => "Profile",
            Screen::Loading => "Loading",
            Screen::Error(_) => "Error",
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
    pub auth_token: Option<String>,
    pub refresh_token: Option<String>,
    
    // Auth form fields
    pub auth_username: String,
    pub auth_password: String,
    pub auth_email: String,
    pub auth_confirm_password: String,

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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user: None,
            auth_token: None,
            refresh_token: None,
            auth_username: String::new(),
            auth_password: String::new(),
            auth_email: String::new(),
            auth_confirm_password: String::new(),
            session: None,
            learner_model: None,
            adaptive_service: None,
            current_screen: Screen::Login,
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
}