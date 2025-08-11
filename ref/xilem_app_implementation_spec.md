# Xilem App Implementation Specification

## Executive Summary

This specification defines the concrete implementation architecture for the ABCDEEZ Xilem app, a cross-platform learning application for cognitive science research. The app serves as the user interface layer for an adaptive learning system, integrating with the core learning algorithms and web backend API to deliver a research-grade experience across iOS, Android, and desktop platforms.

## 1. Project Structure

```
xilem-app/
├── src/
│   ├── lib.rs                  # Library entry point for shared code
│   ├── main.rs                 # Desktop application entry
│   ├── app.rs                  # Core application logic
│   ├── state/
│   │   ├── mod.rs             # State management module
│   │   ├── app_state.rs       # Root application state
│   │   ├── session_state.rs   # Learning session state
│   │   ├── user_state.rs      # User/auth state
│   │   └── sync_state.rs      # Offline/sync state
│   ├── views/
│   │   ├── mod.rs             # View module organization
│   │   ├── auth/              # Authentication views
│   │   │   ├── login.rs       # Login/signup screens
│   │   │   └── oauth.rs       # OAuth flow handlers
│   │   ├── learning/          # Learning interface views
│   │   │   ├── task_view.rs   # Task presentation
│   │   │   ├── alphabet.rs    # Alphabet-specific tasks
│   │   │   ├── music.rs       # Music domain tasks
│   │   │   ├── math.rs        # Mathematics tasks
│   │   │   └── feedback.rs    # Feedback/hint display
│   │   ├── dashboard/         # Dashboard views
│   │   │   ├── home.rs        # Main dashboard
│   │   │   ├── progress.rs    # Progress visualization
│   │   │   └── analytics.rs   # Analytics display
│   │   ├── social/            # Social features
│   │   │   ├── leaderboard.rs # Leaderboard view
│   │   │   └── challenges.rs  # Challenge management
│   │   └── settings/          # Settings screens
│   │       ├── profile.rs     # User profile
│   │       └── preferences.rs # App preferences
│   ├── components/            # Reusable UI components
│   │   ├── mod.rs
│   │   ├── chart.rs           # Data visualization
│   │   ├── timer.rs           # Precision timer display
│   │   ├── progress_ring.rs   # Circular progress
│   │   ├── card.rs            # Card container
│   │   └── navigation.rs      # Navigation components
│   ├── models/                # Data models matching backend
│   │   ├── mod.rs
│   │   ├── user.rs            # User/auth models
│   │   ├── learner.rs         # Learner profile
│   │   ├── session.rs         # Session models
│   │   ├── task.rs            # Task representations
│   │   └── response.rs        # Response models
│   ├── services/              # Service layer
│   │   ├── mod.rs
│   │   ├── api.rs             # Backend API client
│   │   ├── websocket.rs       # WebSocket management
│   │   ├── storage.rs         # Local storage/SQLite
│   │   ├── sync.rs            # Offline sync queue
│   │   ├── timing.rs          # High-precision timing
│   │   └── audio.rs           # Audio playback (music)
│   ├── platform/              # Platform-specific code
│   │   ├── mod.rs
│   │   ├── ios.rs             # iOS-specific features
│   │   ├── android.rs         # Android-specific
│   │   └── desktop.rs         # Desktop-specific
│   └── utils/                 # Utilities
│       ├── mod.rs
│       ├── constants.rs       # App constants
│       ├── theme.rs           # Theme/styling
│       └── validation.rs      # Input validation
├── assets/                    # Static assets
│   ├── fonts/                # Custom fonts
│   ├── icons/                # App icons
│   └── sounds/               # Audio files
├── Cargo.toml                # Dependencies
└── build.rs                  # Build script
```

## 2. State Management

### 2.1 Root Application State

```rust
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    // User & Authentication
    pub user: Option<UserState>,
    pub auth_token: Option<String>,
    pub refresh_token: Option<String>,
    
    // Current Session
    pub session: Option<SessionState>,
    pub learner_model: Option<abcdeez_core::LearnerModel>,
    
    // Navigation
    pub current_screen: Screen,
    pub navigation_stack: Vec<Screen>,
    
    // UI State
    pub loading: HashSet<LoadingKey>,
    pub errors: Vec<AppError>,
    pub theme: ThemeMode,
    
    // Sync & Offline
    pub sync_queue: Vec<PendingResponse>,
    pub last_sync: Option<DateTime<Utc>>,
    pub connection_status: ConnectionStatus,
    
    // WebSocket
    pub ws_connected: bool,
    pub ws_reconnect_attempts: u32,
    
    // Performance Metrics
    pub frame_times: RingBuffer<f64>,
    pub response_times: RingBuffer<u128>,
    
    // Cache
    pub task_cache: LruCache<TaskKey, Task>,
    pub image_cache: HashMap<String, vello::peniko::Image>,
}

impl xilem::AppState for AppState {
    fn keep_running(&self) -> bool {
        // Keep running unless explicitly logging out
        !matches!(self.current_screen, Screen::Logout)
    }
}
```

### 2.2 Session State Management

```rust
#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: Uuid,
    pub learner_id: Uuid,
    pub topology: abcdeez_core::Topology,
    pub start_time: DateTime<Utc>,
    pub tasks_completed: usize,
    pub current_task: Option<Task>,
    pub pending_response: Option<PendingResponse>,
    pub hint_level: u32,
    pub struggle_indicators: StruggleState,
    pub performance_buffer: Vec<ResponseMetrics>,
}

#[derive(Debug, Clone)]
pub struct StruggleState {
    pub consecutive_errors: u32,
    pub response_time_ms: u128,
    pub hint_requested: bool,
    pub difficulty_adjustment_needed: bool,
}
```

### 2.3 State Update Pattern

```rust
// All state mutations happen through actions
pub enum AppAction {
    // Authentication
    Login { username: String, password: String },
    LoginComplete(Result<AuthResponse, ApiError>),
    Logout,
    
    // Session Management
    StartSession { topology_type: TopologyType },
    SessionStarted(SessionResponse),
    EndSession,
    
    // Task Flow
    RequestNextTask,
    TaskReceived(Task),
    SubmitResponse { answer: String, time_ms: u128 },
    ResponseProcessed(FeedbackMessage),
    RequestHint,
    HintReceived(HintMessage),
    
    // WebSocket Events
    WsConnected,
    WsDisconnected,
    WsMessage(ServerMessage),
    
    // Sync
    SyncOfflineQueue,
    SyncComplete(SyncResult),
    
    // Navigation
    Navigate(Screen),
    NavigateBack,
}

// State reducer pattern
impl AppState {
    pub fn reduce(&mut self, action: AppAction) {
        match action {
            AppAction::Login { username, password } => {
                self.loading.insert(LoadingKey::Login);
                // Trigger async login...
            }
            AppAction::LoginComplete(result) => {
                self.loading.remove(&LoadingKey::Login);
                match result {
                    Ok(auth) => {
                        self.user = Some(auth.user);
                        self.auth_token = Some(auth.access_token);
                        self.navigate(Screen::Dashboard);
                    }
                    Err(e) => self.errors.push(e.into()),
                }
            }
            // ... handle other actions
        }
    }
}
```

## 3. Component Architecture

### 3.1 Core App Component

```rust
use xilem::{view::*, WidgetView, Xilem, WindowOptions};
use xilem::core::one_of::Either;

pub fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> {
    // Root view with navigation
    sized_box(
        match state.current_screen {
            Screen::Login => Either::A(auth_view(state)),
            Screen::Dashboard => Either::B(dashboard_view(state)),
            Screen::Learning => Either::C(learning_view(state)),
            Screen::Settings => Either::D(settings_view(state)),
            _ => Either::E(loading_view()),
        }
    )
    .expand()
    .background_color(state.theme.background_color())
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let app = Xilem::new_simple(
        AppState::default(),
        app_logic,
        WindowOptions::new("ABCDEEZ Learning")
            .with_initial_inner_size((1024.0, 768.0))
            .with_min_inner_size((320.0, 568.0))
    );
    
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

### 3.2 Learning Interface Component

```rust
use xilem::view::*;
use xilem::WidgetView;

pub fn learning_view(state: &mut AppState) -> impl WidgetView<AppState> {
    flex((
        // Top bar with progress
        learning_header(state),
        
        // Main task area
        sized_box(
            if let Some(task) = &state.session.as_ref().and_then(|s| s.current_task.clone()) {
                task_presenter(state, task)
            } else {
                loading_spinner()
            }
        )
        .expand()
        .padding(20.0),
        
        // Response input area
        response_input_area(state),
        
        // Bottom navigation
        learning_footer(state),
    ))
    .direction(Axis::Vertical)
    .must_fill_major_axis(true)
}

fn task_presenter(state: &mut AppState, task: &Task) -> impl WidgetView<AppState> {
    match &task.task_type {
        TaskType::Alphabet(alphabet_task) => {
            alphabet_task_view(state, alphabet_task)
        }
        TaskType::Music(music_task) => {
            music_task_view(state, music_task)
        }
        TaskType::Math(math_task) => {
            math_task_view(state, math_task)
        }
    }
}

fn alphabet_task_view(state: &mut AppState, task: &AlphabetTask) -> impl WidgetView<AppState> {
    flex((
        // Task prompt
        label(&task.prompt)
            .text_size(24.0)
            .text_alignment(TextAlign::Center),
        
        FlexSpacer::Fixed(40.0),
        
        // Visual representation
        flex_row(
            task.sequence.iter().enumerate().map(|(i, item)| {
                if i == task.missing_index {
                    sized_box(label("?"))
                        .width(60.0)
                        .height(60.0)
                        .border(Color::from_rgb8(100, 100, 255), 2.0)
                        .corner_radius(8.0)
                } else {
                    sized_box(label(item))
                        .width(60.0)
                        .height(60.0)
                        .background_color(Color::from_rgb8(240, 240, 240))
                        .corner_radius(8.0)
                }
            })
        )
        .gap(10.0)
        .main_axis_alignment(MainAxisAlignment::Center),
        
        FlexSpacer::Fixed(40.0),
        
        // Multiple choice options
        grid(
            task.options.iter().enumerate().map(|(i, option)| {
                button(option, move |state: &mut AppState| {
                    state.submit_response(option.clone(), get_response_time());
                })
                .grid_pos(i % 2, i / 2)
            }),
            2, 
            (task.options.len() + 1) / 2
        )
        .spacing(10.0),
    ))
    .direction(Axis::Vertical)
    .cross_axis_alignment(CrossAxisAlignment::Center)
}
```

### 3.3 Progress Visualization Component

```rust
use xilem::view::*;
use plotters::prelude::*;

pub fn progress_chart(state: &AppState) -> impl WidgetView<AppState> {
    // Generate chart image using plotters
    let chart_image = generate_learning_curve_chart(&state.performance_buffer);
    
    flex((
        label("Learning Progress")
            .text_size(18.0)
            .weight(FontWeight::BOLD),
        
        FlexSpacer::Fixed(10.0),
        
        // Display the generated chart
        image(&chart_image)
            .fit(ObjectFit::Contain),
        
        FlexSpacer::Fixed(10.0),
        
        // Stats summary
        flex_row((
            stat_card("Accuracy", format!("{:.1}%", state.get_accuracy() * 100.0)),
            stat_card("Speed", format!("{:.0}ms", state.get_avg_response_time())),
            stat_card("Streak", format!("{}", state.get_current_streak())),
        ))
        .gap(15.0)
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly),
    ))
    .direction(Axis::Vertical)
}

fn stat_card(label: &str, value: String) -> impl WidgetView<AppState> {
    sized_box(
        flex((
            label(label)
                .text_size(12.0)
                .color(Color::from_rgb8(128, 128, 128)),
            label(value)
                .text_size(20.0)
                .weight(FontWeight::BOLD),
        ))
        .direction(Axis::Vertical)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .padding(10.0)
    )
    .background_color(Color::from_rgb8(245, 245, 250))
    .corner_radius(8.0)
}
```

## 4. Navigation Flow

### 4.1 Screen Hierarchy

```rust
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

impl AppState {
    pub fn navigate(&mut self, screen: Screen) {
        self.navigation_stack.push(self.current_screen.clone());
        self.current_screen = screen;
    }
    
    pub fn navigate_back(&mut self) {
        if let Some(previous) = self.navigation_stack.pop() {
            self.current_screen = previous;
        }
    }
}
```

### 4.2 Navigation Component

```rust
pub fn navigation_bar(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        // Back button (if applicable)
        if !state.navigation_stack.is_empty() {
            Either::A(
                button("←", |state: &mut AppState| {
                    state.navigate_back();
                })
                .padding(10.0)
            )
        } else {
            Either::B(FlexSpacer::Fixed(50.0))
        },
        
        // Title
        label(state.current_screen.title())
            .text_size(18.0)
            .weight(FontWeight::BOLD)
            .flex(1.0),
        
        // Action buttons
        match state.current_screen {
            Screen::Dashboard => {
                Either::A(button("Settings", |state: &mut AppState| {
                    state.navigate(Screen::Settings);
                }))
            }
            _ => Either::B(FlexSpacer::Fixed(50.0))
        },
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .padding(10.0)
    .background_color(Color::from_rgb8(250, 250, 250))
}
```

## 5. Data Models

### 5.1 Core Models (matching backend API)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learner {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub total_practice_time_seconds: i64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub learner_id: Uuid,
    pub topology_type: String,
    pub topology_data: serde_json::Value,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_type: TaskType,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub difficulty: f64,
    pub operation: OperationType,
    pub metadata: TaskMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: i32,
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: Option<String>,
    pub correct: bool,
    pub response_time_ms: u128,
    pub hint_level: Option<u32>,
    pub timestamp: DateTime<Utc>,
}
```

### 5.2 UI-Specific Models

```rust
#[derive(Debug, Clone)]
pub struct TaskPresentation {
    pub visual_elements: Vec<VisualElement>,
    pub input_method: InputMethod,
    pub timing_critical: bool,
    pub audio_components: Vec<AudioClip>,
}

#[derive(Debug, Clone)]
pub enum VisualElement {
    Text { content: String, style: TextStyle },
    Image { path: String, size: (f64, f64) },
    Shape { kind: ShapeKind, color: Color },
    Animation { frames: Vec<AnimationFrame> },
}

#[derive(Debug, Clone)]
pub enum InputMethod {
    MultipleChoice { options: Vec<String> },
    TextInput { placeholder: String },
    DragDrop { items: Vec<DraggableItem> },
    Drawing { canvas_size: (f64, f64) },
}
```

## 6. WebSocket Integration

### 6.1 WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

pub struct WebSocketClient {
    url: String,
    sender: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    receiver: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    message_queue: Arc<Mutex<VecDeque<ClientMessage>>>,
}

impl WebSocketClient {
    pub async fn connect(&mut self, token: &str) -> Result<(), WsError> {
        let url = format!("{}/api/sessions/live?token={}", self.url, token);
        let (ws_stream, _) = connect_async(url).await?;
        let (sender, receiver) = ws_stream.split();
        
        self.sender = Some(sender);
        self.receiver = Some(receiver);
        
        // Send authentication
        self.authenticate(token).await?;
        
        // Start message loop
        self.start_message_loop().await;
        
        Ok(())
    }
    
    pub async fn send_message(&mut self, msg: ClientMessage) -> Result<(), WsError> {
        if let Some(sender) = &mut self.sender {
            let json = serde_json::to_string(&msg)?;
            sender.send(Message::Text(json)).await?;
        } else {
            // Queue for later if not connected
            self.message_queue.lock().unwrap().push_back(msg);
        }
        Ok(())
    }
    
    async fn start_message_loop(&mut self) {
        if let Some(mut receiver) = self.receiver.take() {
            tokio::spawn(async move {
                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                // Send to UI thread via channel
                                handle_server_message(server_msg);
                            }
                        }
                        Ok(Message::Close(_)) => {
                            // Handle disconnection
                            break;
                        }
                        _ => {}
                    }
                }
            });
        }
    }
}
```

### 6.2 Message Handling

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { token: String },
    StartTask { payload: serde_json::Value },
    SubmitResponse {
        task_type: String,
        task_data: serde_json::Value,
        user_answer: Option<String>,
        response_time_ms: u128,
    },
    RequestHint { hint_level: Option<String> },
    Pause,
    Resume,
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    AuthenticationResult {
        success: bool,
        message: String,
        user_id: Option<Uuid>,
        timestamp: i64,
    },
    Task {
        payload: TaskMessage,
        timestamp: i64,
    },
    Hint {
        payload: HintMessage,
        timestamp: i64,
    },
    Feedback {
        payload: FeedbackMessage,
        timestamp: i64,
    },
    Intervention {
        payload: InterventionMessage,
        timestamp: i64,
    },
    StatsUpdate {
        payload: StatsMessage,
        timestamp: i64,
    },
    Error {
        message: String,
        timestamp: i64,
    },
    Heartbeat {
        timestamp: i64,
    },
}
```

## 7. Platform Abstractions

### 7.1 Platform Trait

```rust
pub trait Platform {
    fn get_device_id(&self) -> String;
    fn get_screen_size(&self) -> (f64, f64);
    fn supports_haptics(&self) -> bool;
    fn trigger_haptic(&self, pattern: HapticPattern);
    fn get_safe_area_insets(&self) -> SafeAreaInsets;
    fn request_permissions(&self, permissions: Vec<Permission>) -> Future<PermissionResult>;
    fn get_high_precision_timer(&self) -> Box<dyn PrecisionTimer>;
}

#[cfg(target_os = "ios")]
pub struct IOSPlatform;

impl Platform for IOSPlatform {
    fn supports_haptics(&self) -> bool {
        true
    }
    
    fn trigger_haptic(&self, pattern: HapticPattern) {
        // iOS-specific haptic implementation
        match pattern {
            HapticPattern::Success => {
                // UIImpactFeedbackGenerator with .light style
            }
            HapticPattern::Warning => {
                // UINotificationFeedbackGenerator with .warning
            }
            HapticPattern::Error => {
                // UINotificationFeedbackGenerator with .error
            }
        }
    }
    
    fn get_high_precision_timer(&self) -> Box<dyn PrecisionTimer> {
        Box::new(IOSPrecisionTimer::new())
    }
}

#[cfg(target_os = "android")]
pub struct AndroidPlatform;

#[cfg(not(any(target_os = "ios", target_os = "android")))]
pub struct DesktopPlatform;
```

### 7.2 Responsive Layout

```rust
pub fn responsive_layout(state: &AppState) -> impl WidgetView<AppState> {
    let screen_width = state.get_screen_width();
    
    if screen_width < 600.0 {
        // Mobile layout
        mobile_layout(state)
    } else if screen_width < 1200.0 {
        // Tablet layout
        tablet_layout(state)
    } else {
        // Desktop layout
        desktop_layout(state)
    }
}

fn mobile_layout(state: &AppState) -> impl WidgetView<AppState> {
    flex((
        navigation_bar(state),
        sized_box(content_view(state))
            .expand()
            .padding(10.0),
        bottom_tab_bar(state),
    ))
    .direction(Axis::Vertical)
}

fn desktop_layout(state: &AppState) -> impl WidgetView<AppState> {
    flex_row((
        // Sidebar navigation
        sized_box(sidebar_navigation(state))
            .width(250.0)
            .background_color(Color::from_rgb8(245, 245, 250)),
        
        // Main content
        flex((
            header_bar(state),
            sized_box(content_view(state))
                .expand()
                .padding(20.0),
        ))
        .direction(Axis::Vertical)
        .flex(1.0),
    ))
}
```

## 8. Performance Optimizations

### 8.1 High-Precision Timing

```rust
use std::time::{Instant, Duration};

pub struct PrecisionTimer {
    start: Instant,
    marks: Vec<(String, Duration)>,
}

impl PrecisionTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            marks: Vec::new(),
        }
    }
    
    pub fn mark(&mut self, label: impl Into<String>) {
        let elapsed = self.start.elapsed();
        self.marks.push((label.into(), elapsed));
    }
    
    pub fn get_elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
    
    pub fn get_elapsed_micros(&self) -> u128 {
        self.start.elapsed().as_micros()
    }
}

// Response capture with sub-millisecond precision
pub fn capture_response(state: &mut AppState, answer: String) {
    let response_time = state.current_task_timer.get_elapsed_micros();
    
    // Store with microsecond precision, report in milliseconds
    state.submit_response(answer, response_time / 1000);
    
    // Log for analysis
    state.response_times.push(response_time);
}
```

### 8.2 Virtual List Optimization

```rust
pub fn task_history_view(state: &AppState) -> impl WidgetView<AppState> {
    let responses = &state.session.as_ref().map(|s| &s.responses).unwrap_or(&vec![]);
    let count = responses.len() as i64;
    
    virtual_scroll(
        0..count,
        |state: &mut AppState, index| {
            let response = &state.session.as_ref().unwrap().responses[index as usize];
            response_item_view(response)
        }
    )
}

fn response_item_view(response: &Response) -> impl WidgetView<AppState> {
    flex_row((
        label(format!("#{}", response.sequence_number))
            .text_size(14.0)
            .color(Color::from_rgb8(128, 128, 128)),
        
        label(&response.task_type)
            .flex(1.0),
        
        if response.correct {
            Either::A(
                label("✓")
                    .color(Color::from_rgb8(0, 200, 0))
            )
        } else {
            Either::B(
                label("✗")
                    .color(Color::from_rgb8(200, 0, 0))
            )
        },
        
        label(format!("{}ms", response.response_time_ms))
            .text_size(12.0),
    ))
    .gap(10.0)
    .padding(10.0)
    .background_color(Color::from_rgb8(250, 250, 250))
    .corner_radius(5.0)
}
```

### 8.3 Caching Strategy

```rust
use lru::LruCache;

pub struct AppCache {
    // Task cache to avoid re-fetching
    tasks: LruCache<TaskKey, Task>,
    
    // Image cache for visual elements
    images: HashMap<String, vello::peniko::Image>,
    
    // Audio cache for music tasks
    audio: HashMap<String, AudioBuffer>,
    
    // Computed metrics cache
    metrics: HashMap<MetricKey, (DateTime<Utc>, f64)>,
}

impl AppCache {
    pub fn get_or_fetch_task(&mut self, key: TaskKey) -> impl Future<Output = Result<Task, ApiError>> {
        if let Some(task) = self.tasks.get(&key) {
            return future::ready(Ok(task.clone()));
        }
        
        // Fetch from API
        async move {
            let task = api::fetch_task(key).await?;
            self.tasks.put(key, task.clone());
            Ok(task)
        }
    }
}
```

## 9. Build Configuration

### 9.1 Cargo.toml

```toml
[package]
name = "abcdeez-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Xilem UI Framework
xilem = { git = "https://github.com/linebender/xilem.git" }
xilem_core = { git = "https://github.com/linebender/xilem.git" }
masonry = { git = "https://github.com/linebender/xilem.git" }
winit = { version = "0.30", features = ["android-native-activity"] }
vello = { git = "https://github.com/linebender/vello.git" }

# Core learning module
abcdeez-core = { path = "../core" }

# Async runtime
tokio = { version = "1.0", features = ["rt-multi-thread", "macros", "time"] }
futures-util = "0.3"

# Networking
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tokio-tungstenite = { version = "0.24", features = ["rustls"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Storage
rusqlite = { version = "0.32", features = ["bundled", "chrono", "serde_json"] }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# Performance
lru = "0.12"
parking_lot = "0.12"

# Visualization
plotters = { version = "0.3", default-features = false, features = ["bitmap_backend"] }

# Audio (for music tasks)
cpal = "0.16"
rodio = "0.19"

# Platform-specific
[target.'cfg(target_os = "ios")'.dependencies]
objc = "0.2"
objc-foundation = "0.1"

[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
android-activity = "0.6"

[features]
default = ["desktop"]
desktop = []
mobile = ["xilem/mobile"]
debug-overlay = []

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### 9.2 Build Script

```rust
// build.rs
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    if target.contains("ios") {
        println!("cargo:rustc-link-lib=framework=UIKit");
        println!("cargo:rustc-link-lib=framework=CoreHaptics");
    } else if target.contains("android") {
        println!("cargo:rustc-link-lib=android");
        println!("cargo:rustc-link-lib=log");
    }
    
    // Generate version info
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = get_git_hash();
    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
```

## 10. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
1. Set up project structure
2. Implement basic state management
3. Create authentication flow
4. Establish API client
5. Build navigation system

### Phase 2: Core Learning (Weeks 3-4)
1. Implement task presentation views
2. Add response capture with timing
3. Create WebSocket integration
4. Build offline queue system
5. Add basic progress tracking

### Phase 3: Polish & Features (Weeks 5-6)
1. Add animations and transitions
2. Implement charts and visualizations
3. Create settings and profile screens
4. Add error handling and recovery
5. Optimize performance

### Phase 4: Platform-Specific (Week 7)
1. iOS haptics and safe areas
2. Android back button handling
3. Desktop keyboard shortcuts
4. Platform-specific styling

### Phase 5: Testing & Release (Week 8)
1. Unit and integration tests
2. Performance profiling
3. User testing
4. Bug fixes and polish
5. Release preparation

## Summary

This specification provides a complete blueprint for implementing the ABCDEEZ Xilem app. The architecture emphasizes:

1. **Clean separation of concerns** with distinct modules for state, views, services, and models
2. **Reactive state management** following Xilem's data-down, actions-up pattern
3. **High-performance rendering** with virtual scrolling and caching
4. **Cross-platform compatibility** through platform abstractions
5. **Research-grade precision** with microsecond timing capabilities
6. **Robust offline support** with sync queue and local storage
7. **Real-time communication** via WebSocket integration
8. **Modular component architecture** for maintainability

The implementation follows Xilem best practices while providing the sophisticated features required for cognitive science research, ensuring both scientific rigor and excellent user experience across all platforms.