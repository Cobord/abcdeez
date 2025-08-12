// Root application state

use abcdeez_core::learning::learner::LearnerModel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;

use crate::models::{PendingResponse, Task};
use crate::services::AdaptiveLearningService;
use crate::state::{SessionState, Theme, ThemeMode, UserState};
use crate::utils::easter_egg::EasterEggManager;
use crate::demo::DemoController;
use crate::gamification::{GamificationManager, AchievementEvent};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoadingKey {
    Login,
    Signup,
    Session,
    Task,
    Analytics,
}

/// Events sent from background authentication tasks
#[derive(Debug, Clone)]
pub enum AuthEvent {
    AppleSignInSuccess {
        user_id: String,
        email: Option<String>,
        given_name: Option<String>,
        family_name: Option<String>,
    },
    AppleSignInError(String),
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
    
    /// Parse a Screen from a URL fragment/hash
    pub fn from_url_fragment(fragment: &str) -> Option<Screen> {
        // Remove # if present and convert to lowercase for case-insensitive matching
        let clean_fragment = fragment.trim_start_matches('#').to_lowercase();
        
        match clean_fragment.as_str() {
            "login" => Some(Screen::Login),
            "signup" | "sign-up" => Some(Screen::Signup),
            "dashboard" => Some(Screen::Dashboard),
            "learning" => Some(Screen::Learning),
            "welcome" => Some(Screen::Welcome),
            "domain-selection" | "domains" => Some(Screen::DomainSelection),
            "training" => Some(Screen::Training),
            "progress" => Some(Screen::Progress),
            "analytics" => Some(Screen::Analytics),
            "visualizations" | "viz" => Some(Screen::Visualizations),
            "leaderboard" => Some(Screen::Leaderboard),
            "challenges" => Some(Screen::Challenges),
            "friends" => Some(Screen::Friends),
            "settings" => Some(Screen::Settings),
            "profile" => Some(Screen::Profile),
            "widget-gallery" | "widgets" => Some(Screen::WidgetGallery),
            _ => None,
        }
    }
    
    /// Get the URL fragment for this screen
    pub fn to_url_fragment(&self) -> &str {
        match self {
            Screen::Login => "login",
            Screen::Signup => "signup",
            Screen::Dashboard => "dashboard",
            Screen::Learning => "learning",
            Screen::Welcome => "welcome",
            Screen::DomainSelection => "domain-selection",
            Screen::Training => "training",
            Screen::Progress => "progress",
            Screen::Analytics => "analytics",
            Screen::Visualizations => "visualizations",
            Screen::Leaderboard => "leaderboard",
            Screen::Challenges => "challenges",
            Screen::Friends => "friends",
            Screen::Settings => "settings",
            Screen::Profile => "profile",
            Screen::WidgetGallery => "widget-gallery",
            _ => "dashboard",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub recoverable: bool,
}

#[derive(Serialize, Deserialize)]
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
    #[serde(skip)]
    pub ios_auth_bridge: Option<crate::auth::ios::IOSAuthBridge>,
    
    // Authentication event channel
    #[serde(skip)]
    pub auth_event_sender: Option<mpsc::Sender<AuthEvent>>,
    #[serde(skip)]
    pub auth_event_receiver: Option<mpsc::Receiver<AuthEvent>>,
    
    // Easter egg tracking
    pub triple_click_count: u32,
    pub last_triple_click: Option<DateTime<Utc>>,

    // Current Session
    pub session: Option<SessionState>,
    pub learner_model: Option<LearnerModel>,
    pub adaptive_service: Option<AdaptiveLearningService>,
    #[serde(skip)]
    pub current_task_start_time: Option<std::time::Instant>,

    // Navigation
    pub current_screen: Screen,
    pub navigation_stack: Vec<Screen>,
    
    // UI State - tab positions for each screen
    pub tab_states: HashMap<String, usize>,  // screen_name -> current_tab_index
    pub form_states: HashMap<String, String>,  // form_field_id -> value
    pub selected_domain: Option<String>,  // Currently selected domain in leaderboard
    pub search_query: String,  // Current search query in sidebar/friends
    pub settings_toggles: HashMap<String, bool>,  // settings_key -> enabled

    // UI State
    pub loading: HashSet<LoadingKey>,
    pub errors: Vec<AppError>,
    pub notifications: Vec<(String, crate::demo::NotificationType)>,
    pub theme: Theme,
    pub theme_mode: ThemeMode,
    // Responsive UI metrics
    pub ui_scale: f64,
    pub safe_area_insets: SafeAreaInsets,
    pub window_width: f64,
    pub window_height: f64,

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
    #[serde(skip)]
    pub demo_controller: Option<DemoController>,
    
    // Gamification
    #[serde(skip, default)]
    pub gamification: GamificationManager,
}

impl Default for AppState {
    fn default() -> Self {
        // Create the auth event channel
        let (sender, receiver) = mpsc::channel();
        
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
            auth_event_sender: Some(sender),
            auth_event_receiver: Some(receiver),
            triple_click_count: 0,
            last_triple_click: None,
            session: None,
            learner_model: None,
            adaptive_service: None,
            current_task_start_time: None,
            current_screen: Screen::Welcome,
            navigation_stack: Vec::new(),
            tab_states: HashMap::new(),
            form_states: HashMap::new(),
            selected_domain: None,
            search_query: String::new(),
            settings_toggles: HashMap::new(),
            loading: HashSet::new(),
            errors: Vec::new(),
            notifications: Vec::new(),
            theme: Theme::default(),
            theme_mode: ThemeMode::Light,
            ui_scale: 1.0,
            safe_area_insets: SafeAreaInsets::zero(),
            window_width: 1024.0,  // Default desktop width
            window_height: 768.0,  // Default desktop height
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
            gamification: GamificationManager::new(),
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
        self.current_screen = screen.clone();
        
        // Update browser URL on web platform
        #[cfg(target_arch = "wasm32")]
        self.update_browser_url(&screen);
    }

    pub fn navigate_back(&mut self) {
        if let Some(previous) = self.navigation_stack.pop() {
            self.current_screen = previous.clone();
            
            // Update browser URL on web platform
            #[cfg(target_arch = "wasm32")]
            self.update_browser_url(&previous);
        }
    }
    
    /// Handle deep link navigation from URL
    pub fn handle_deep_link(&mut self, url: &str) {
        // Parse the URL and extract the fragment
        if let Some(fragment_start) = url.find('#') {
            let fragment = &url[fragment_start + 1..];
            
            // Parse the screen from the fragment
            if let Some(screen) = Screen::from_url_fragment(fragment) {
                // Navigate directly to the screen without pushing to history
                self.current_screen = screen;
            }
        }
    }
    
    /// Initialize app state with deep link if available
    pub fn init_with_deep_link(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(location) = window.location().href() {
                    self.handle_deep_link(&location);
                }
            }
        }
    }
    
    /// Update browser URL when navigating (web platform only)
    #[cfg(target_arch = "wasm32")]
    fn update_browser_url(&self, screen: &Screen) {
        if let Some(window) = web_sys::window() {
            if let Some(history) = window.history().ok() {
                let fragment = screen.to_url_fragment();
                let new_url = format!("#{}", fragment);
                
                // Use replaceState to update URL without triggering navigation
                let _ = history.replace_state_with_url(
                    &wasm_bindgen::JsValue::NULL,
                    "",
                    Some(&new_url)
                );
            }
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
    
    pub fn add_notification(&mut self, message: &str, notification_type: crate::demo::NotificationType) {
        self.notifications.push((message.to_string(), notification_type));
        tracing::info!("Notification: {} ({:?})", message, notification_type);
    }
    
    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
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
    
    // Tab state management
    pub fn get_tab_state(&self, screen_key: &str) -> usize {
        self.tab_states.get(screen_key).copied().unwrap_or(0)
    }
    
    pub fn set_tab_state(&mut self, screen_key: String, index: usize) {
        self.tab_states.insert(screen_key, index);
    }
    
    // Form state management
    pub fn get_form_value(&self, field_id: &str) -> String {
        self.form_states.get(field_id).cloned().unwrap_or_default()
    }
    
    pub fn set_form_value(&mut self, field_id: String, value: String) {
        self.form_states.insert(field_id, value);
    }
    
    // Authentication methods
    pub fn login(&mut self) {
        tracing::info!("Login initiated for user: {}", self.username_input);
        
        if self.username_input.is_empty() || self.password_input.is_empty() {
            tracing::error!("Login failed: empty credentials");
            self.add_error("Username and password are required".to_string(), true);
            return;
        }
        
        self.login_request_in_flight = true;
        
        // TODO: Implement actual login logic with backend
        // For now, simulate successful login
        tracing::info!("Simulating successful login for demo");
        
        // Create mock user
        self.user = Some(UserState {
            id: uuid::Uuid::new_v4(),
            username: self.username_input.clone(),
            email: format!("{}@example.com", self.username_input),
            display_name: Some(self.username_input.clone()),
            created_at: chrono::Utc::now(),
            learner_id: Some(uuid::Uuid::new_v4()),
            level: 1,
            xp: 0,
            streak: 0,
            achievements: Vec::new(),
        });
        
        self.current_user = self.user.clone(); // Compatibility
        self.password_input.clear(); // Clear sensitive data
        self.login_request_in_flight = false;
        self.current_screen = Screen::Dashboard;
    }
    
    pub fn apple_sign_in(&mut self) {
        tracing::info!("Apple Sign In initiated");
        tracing::info!("Starting Apple Sign In flow");
        self.oauth_login_in_flight = true;
        
        #[cfg(target_os = "ios")]
        {
            tracing::info!("iOS platform detected");
            if let Some(bridge) = self.ios_auth_bridge.clone() {
                tracing::info!("iOS auth bridge available, spawning async task");
                
                // Clone the sender for the background thread
                let sender = self.auth_event_sender.clone();
                
                tracing::debug!("Using simpler approach for iOS - calling async directly");
                
                // Try a much simpler approach - just block on the future directly
                // Use futures::executor::block_on instead of tokio
                let handle = std::thread::spawn(move || {
                    println!("Inside spawned thread - using futures::executor!");
                    
                    println!("About to call bridge.apple_sign_in_async() with futures::executor::block_on");
                    match futures::executor::block_on(bridge.apple_sign_in_async()) {
                        Ok(result) => {
                            // Log success without exposing the actual user ID
                            let user_id_prefix = if result.user_id.len() > 6 {
                                &result.user_id[..6]
                            } else {
                                &result.user_id
                            };
                            tracing::info!(
                                user_id_prefix = user_id_prefix,
                                has_email = result.email.is_some(),
                                is_private_email = result.is_private_email,
                                "Apple Sign In successful"
                            );
                            
                            // Send success event back to main thread
                            if let Some(ref sender) = sender {
                                let event = AuthEvent::AppleSignInSuccess {
                                    user_id: result.user_id,
                                    email: result.email,
                                    given_name: result.given_name,
                                    family_name: result.family_name,
                                };
                                let _ = sender.send(event);
                            }
                        }
                        Err(e) => {
                            println!("Spawned thread: Got error from apple_sign_in_async: {}", e);
                            tracing::info!("Spawned thread: Apple Sign In failed: {}", e);
                            
                            // Send error event back to main thread
                            if let Some(ref sender) = sender {
                                println!("Spawned thread: Have sender, sending error");
                                tracing::info!("Spawned thread: Sending error event to main thread: {}", e);
                                let event = AuthEvent::AppleSignInError(e);
                                let send_result = sender.send(event);
                                println!("Spawned thread: Send result: {:?}", send_result.is_ok());
                                tracing::info!("Spawned thread: Main thread send result: {:?}", send_result.is_ok());
                            } else {
                                println!("Spawned thread: NO SENDER AVAILABLE!");
                                tracing::warn!("No sender available to send error event!");
                            }
                        }
                    }
                    println!("Spawned thread: Finished processing result");
                });
                
                tracing::info!("Apple Sign In initiated asynchronously");
                // Note: oauth_login_in_flight remains true until the callback updates it
            } else {
                tracing::warn!("iOS auth bridge not available");
                self.oauth_login_in_flight = false;
            }
        }
        
        #[cfg(not(target_os = "ios"))]
        {
            // Simulate Apple sign in for non-iOS platforms
            tracing::debug!("Simulating Apple Sign In (not on iOS)");
            self.current_screen = Screen::Dashboard;
            self.oauth_login_in_flight = false;
        }
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
        tracing::info!("Creating new learner model for {} mode", if self.is_guest_mode { "guest" } else { "authenticated" });
        // TODO: Actually create learner model with proper topology
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
            tracing::info!("🦀 Little Crab easter egg activated via triple-click!");
            self.add_error("🦀 You found Little Crab! He'll help you learn!".to_string(), true);
        }
    }
    
    // Demo methods
    pub fn demo_start(&mut self) {
        tracing::info!("Starting interactive Quick Tour demo");
        if let Some(ref mut demo) = self.demo_controller {
            let _ = demo.start_scenario("quick_tour");
        }
    }
    
    pub fn demo_start_training(&mut self) {
        tracing::info!("Starting Training Demo scenario");
        if let Some(ref mut demo) = self.demo_controller {
            let _ = demo.start_scenario("training_demo");
        }
    }
    
    pub fn demo_showcase(&mut self) {
        tracing::info!("Running Demo Showcase with sample data");
        if let Some(ref mut demo) = self.demo_controller {
            let _ = demo.start_scenario("showcase");
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
    
    // Gamification methods
    pub fn award_achievement(&mut self, event: AchievementEvent) {
        let user_id = self.user.as_ref()
            .map(|u| u.id.to_string())
            .unwrap_or_else(|| "guest".to_string());
        
        let unlocked = self.gamification.process_event(&user_id, event);
        
        if !unlocked.is_empty() {
            println!("🎉 Achievements unlocked: {:?}", unlocked);
        }
    }
    
    pub fn update_gamification_session(&mut self, correct: u32, total: u32, duration_minutes: u32) {
        let user_id = self.user.as_ref()
            .map(|u| u.id.to_string())
            .unwrap_or_else(|| "guest".to_string());
        
        let accuracy = if total > 0 { correct as f32 / total as f32 } else { 0.0 };
        
        // Update profile statistics
        if let Some(profile) = self.gamification.user_profiles.get_mut(&user_id) {
            use crate::gamification::profile::SessionData;
            
            profile.update_statistics(SessionData {
                tasks_completed: total,
                duration_minutes,
                accuracy,
                fastest_response_ms: self.response_times.iter().min().copied().unwrap_or(0) as u32,
            });
            
            // Award XP based on performance
            let base_xp = total * 10;
            let accuracy_bonus = (accuracy * 50.0) as u32;
            let total_xp = base_xp + accuracy_bonus;
            profile.add_experience(total_xp);
        }
        
        // Update leaderboards
        self.gamification.update_leaderboards();
    }
    
    pub fn get_current_user_profile(&self) -> Option<&crate::gamification::profile::GamificationProfile> {
        let user_id = self.user.as_ref()
            .map(|u| u.id.to_string())
            .unwrap_or_else(|| "guest".to_string());
        
        self.gamification.user_profiles.get(&user_id)
    }
    
    /// Process any pending authentication events
    pub fn process_auth_events(&mut self) {
        if let Some(ref receiver) = self.auth_event_receiver {
            // Try to receive without blocking
            match receiver.try_recv() {
                Ok(event) => {
                    tracing::info!("process_auth_events: Received auth event: {:?}", event);
                    match event {
                        AuthEvent::AppleSignInSuccess { user_id: _, email, given_name, family_name } => {
                        tracing::info!("Processing Apple Sign In success event");
                        
                        // Create user state from Apple Sign In data
                        let username = email.clone()
                            .or_else(|| given_name.clone())
                            .unwrap_or_else(|| "Apple User".to_string());
                        
                        let display_name = match (given_name, family_name) {
                            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                            (Some(first), None) => Some(first),
                            (None, Some(last)) => Some(last),
                            _ => None,
                        };
                        
                        self.user = Some(crate::state::UserState {
                            id: uuid::Uuid::new_v4(),
                            username,
                            email: email.unwrap_or_default(),
                            display_name,
                            created_at: chrono::Utc::now(),
                            learner_id: Some(uuid::Uuid::new_v4()),
                            level: 1,
                            xp: 0,
                            streak: 0,
                            achievements: Vec::new(),
                        });
                        
                        // Clear the in-flight flag
                        self.oauth_login_in_flight = false;
                        
                        // Navigate to dashboard
                        self.current_screen = Screen::Dashboard;
                        
                        tracing::info!("Apple Sign In completed successfully");
                    }
                    
                    AuthEvent::AppleSignInError(error) => {
                        tracing::error!("Processing Apple Sign In error: {}", error);
                        
                        // Clear the in-flight flag
                        self.oauth_login_in_flight = false;
                        
                        // Add error to UI
                        self.add_error(format!("Apple Sign In failed: {}", error), true);
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // No events to process - this is normal
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    tracing::error!("Auth event channel disconnected!");
                }
            }
        } else {
            tracing::warn!("No auth event receiver available!");
        }
    }
}

// Safe-area insets for platforms like iOS
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct SafeAreaInsets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl SafeAreaInsets {
    pub const fn zero() -> Self {
        Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 }
    }
}