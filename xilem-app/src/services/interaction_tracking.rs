// Interaction tracking service for capturing user behavior data

use abcdeez_core::data::interaction_tracking::{
    InteractionTracker, InteractionSession, InteractionMetrics as CoreMetrics,
    KeyEventType, KeystrokeContext,
    MouseEventType, Position, MouseButton as CoreMouseButton,
    HesitationAnalysis, TypingDynamicsProfile, InteractionQuality,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

const MAX_EVENT_BUFFER: usize = 1000;
const KEYSTROKE_WINDOW_MS: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMetrics {
    pub typing_speed_wpm: f64,
    pub average_dwell_time_ms: f64,
    pub average_flight_time_ms: f64,
    pub pause_count: usize,
    pub correction_count: usize,
    pub mouse_distance_pixels: f64,
    pub mouse_velocity_avg: f64,
    pub hesitation_events: usize,
}

pub struct InteractionTrackingService {
    interaction_tracker: InteractionTracker,
    session: Option<InteractionSession>,
    session_start: Instant,
    event_buffer: VecDeque<TrackedEvent>,
    current_metrics: InteractionMetrics,
}

#[derive(Debug, Clone)]
pub struct TrackedEvent {
    pub event_type: EventType,
    pub timestamp: Instant,
    pub task_context: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EventType {
    KeyPress { key: String, duration_ms: u64 },
    MouseMove { x: f64, y: f64 },
    MouseClick { x: f64, y: f64, button: MouseButton },
    MouseScroll { delta_y: f64 },
    FocusChange { target: String },
    Pause { duration_ms: u64 },
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl InteractionTrackingService {
    pub fn new() -> Self {
        let session_id = Uuid::new_v4().to_string();
        let participant_id = Uuid::new_v4().to_string();
        let tracker = InteractionTracker::new(session_id.clone(), participant_id.clone());
        
        // Create a session directly from the tracker's internal session
        let session = InteractionSession {
            session_id,
            participant_id,
            start_time: Utc::now(),
            end_time: None,
            keystroke_events: Vec::new(),
            mouse_events: Vec::new(),
            focus_events: Vec::new(),
            scroll_events: Vec::new(),
            behavioral_patterns: Vec::new(),
            hesitation_analysis: HesitationAnalysis {
                total_hesitations: 0,
                average_hesitation_duration: Duration::from_secs(0),
                hesitation_locations: HashMap::new(),
                hesitation_triggers: Vec::new(),
                hesitation_patterns: Vec::new(),
            },
            typing_dynamics: TypingDynamicsProfile {
                words_per_minute: 0.0,
                characters_per_minute: 0.0,
                average_dwell_time: Duration::from_secs(0),
                average_flight_time: Duration::from_secs(0),
                typing_rhythm: abcdeez_core::data::interaction_tracking::TypingRhythm {
                    rhythm_score: 0.0,
                    burst_patterns: Vec::new(),
                    pause_patterns: Vec::new(),
                    tempo_changes: Vec::new(),
                },
                keystroke_intensity: abcdeez_core::data::interaction_tracking::KeystrokeIntensity {
                    pressure_variation: 0.0,
                    timing_precision: 0.0,
                    force_consistency: 0.0,
                    stress_indicators: Vec::new(),
                },
                error_rate: 0.0,
                correction_latency: Duration::from_secs(0),
                finger_usage_pattern: HashMap::new(),
                typing_consistency: 0.0,
            },
            interaction_quality: InteractionQuality {
                overall_score: 0.0,
                engagement_level: abcdeez_core::data::interaction_tracking::EngagementLevel::Low,
                attention_consistency: 0.0,
                task_focus: 0.0,
                interaction_efficiency: 0.0,
                error_handling_quality: 0.0,
                learning_indicators: Vec::new(),
            },
        };
        
        Self {
            interaction_tracker: tracker,
            session: Some(session),
            session_start: Instant::now(),
            event_buffer: VecDeque::with_capacity(MAX_EVENT_BUFFER),
            current_metrics: InteractionMetrics {
                typing_speed_wpm: 0.0,
                average_dwell_time_ms: 0.0,
                average_flight_time_ms: 0.0,
                pause_count: 0,
                correction_count: 0,
                mouse_distance_pixels: 0.0,
                mouse_velocity_avg: 0.0,
                hesitation_events: 0,
            },
        }
    }

    pub fn track_keypress(&mut self, key: String, duration_ms: u64) {
        // Track in core systems
        if let Some(ref mut _session) = self.session {
            self.interaction_tracker.record_keystroke(
                KeyEventType::KeyPress,
                key.clone(),
                None,
                KeystrokeContext {
                    task_id: None,
                    input_field: None,
                    cursor_position: None,
                    text_length: None,
                    is_correction: false,
                    correction_type: None,
                },
            );
        }

        // Track in local buffer
        let tracked = TrackedEvent {
            event_type: EventType::KeyPress { key, duration_ms },
            timestamp: Instant::now(),
            task_context: None,
        };
        self.add_event(tracked);

        // Update metrics
        self.update_typing_metrics();
    }

    pub fn track_mouse_move(&mut self, x: f64, y: f64) {
        // Track in core system
        if let Some(ref mut _session) = self.session {
            self.interaction_tracker.record_mouse_event(
                MouseEventType::Move,
                Position { x, y, timestamp: Utc::now() },
                None,
                None,
            );
        }

        // Track locally
        let tracked = TrackedEvent {
            event_type: EventType::MouseMove { x, y },
            timestamp: Instant::now(),
            task_context: None,
        };
        self.add_event(tracked);

        // Update mouse metrics
        self.update_mouse_metrics();
    }

    pub fn track_mouse_click(&mut self, x: f64, y: f64, button: MouseButton) {
        // Track in core system
        if let Some(ref mut _session) = self.session {
            self.interaction_tracker.record_mouse_event(
                MouseEventType::Click,
                Position { x, y, timestamp: Utc::now() },
                None,
                Some(match button {
                    MouseButton::Left => CoreMouseButton::Left,
                    MouseButton::Right => CoreMouseButton::Right,
                    MouseButton::Middle => CoreMouseButton::Middle,
                }),
            );
        }

        // Track locally
        let tracked = TrackedEvent {
            event_type: EventType::MouseClick { x, y, button },
            timestamp: Instant::now(),
            task_context: None,
        };
        self.add_event(tracked);
    }

    pub fn track_pause(&mut self, duration_ms: u64) {
        if duration_ms > 1000 {
            self.current_metrics.pause_count += 1;
            
            if duration_ms > 3000 {
                self.current_metrics.hesitation_events += 1;
            }
        }

        let tracked = TrackedEvent {
            event_type: EventType::Pause { duration_ms },
            timestamp: Instant::now(),
            task_context: None,
        };
        self.add_event(tracked);
    }

    pub fn track_correction(&mut self) {
        self.current_metrics.correction_count += 1;
        
        // Track as keystroke event (backspace)
        if let Some(ref mut _session) = self.session {
            self.interaction_tracker.record_keystroke(
                KeyEventType::KeyPress,
                "Backspace".to_string(),
                None,
                KeystrokeContext {
                    task_id: None,
                    input_field: None,
                    cursor_position: None,
                    text_length: None,
                    is_correction: false,
                    correction_type: None,
                },
            );
        }
    }

    fn add_event(&mut self, event: TrackedEvent) {
        if self.event_buffer.len() >= MAX_EVENT_BUFFER {
            self.event_buffer.pop_front();
        }
        self.event_buffer.push_back(event);
    }

    fn update_typing_metrics(&mut self) {
        // Calculate typing speed from tracked events
        let keystroke_count = self.event_buffer.iter()
            .filter(|e| matches!(e.event_type, EventType::KeyPress { .. }))
            .count() as f64;
        
        let session_duration = self.session_start.elapsed().as_secs_f64() / 60.0;
        self.current_metrics.typing_speed_wpm = (keystroke_count / 5.0) / session_duration.max(0.1);
    }

    fn update_mouse_metrics(&mut self) {
        // Calculate mouse metrics from tracked events
        let mouse_events: Vec<_> = self.event_buffer.iter()
            .filter_map(|e| match &e.event_type {
                EventType::MouseMove { x, y } => Some((*x, *y)),
                _ => None,
            })
            .collect();
        
        if mouse_events.len() > 1 {
            let mut total_distance = 0.0;
            for i in 1..mouse_events.len() {
                let dx = mouse_events[i].0 - mouse_events[i-1].0;
                let dy = mouse_events[i].1 - mouse_events[i-1].1;
                total_distance += (dx * dx + dy * dy).sqrt();
            }
            self.current_metrics.mouse_distance_pixels = total_distance;
            
            if mouse_events.len() > 0 {
                let duration_secs = self.session_start.elapsed().as_secs_f64();
                self.current_metrics.mouse_velocity_avg = total_distance / duration_secs.max(0.1);
            }
        }
    }

    pub fn get_metrics(&self) -> InteractionMetrics {
        self.current_metrics.clone()
    }

    pub fn get_recent_events(&self, count: usize) -> Vec<TrackedEvent> {
        self.event_buffer.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    pub fn export_session_data(&self) -> SessionInteractionData {
        SessionInteractionData {
            session_duration_ms: self.session_start.elapsed().as_millis(),
            metrics: self.current_metrics.clone(),
            core_metrics: None, // Metrics not directly available from session
            total_events: self.event_buffer.len(),
            timestamp: Utc::now(),
        }
    }

    pub fn reset(&mut self) {
        self.event_buffer.clear();
        let session_id = Uuid::new_v4().to_string();
        let participant_id = Uuid::new_v4().to_string();
        self.interaction_tracker = InteractionTracker::new(session_id.clone(), participant_id.clone());
        
        self.session = Some(InteractionSession {
            session_id,
            participant_id,
            start_time: Utc::now(),
            end_time: None,
            keystroke_events: Vec::new(),
            mouse_events: Vec::new(),
            focus_events: Vec::new(),
            scroll_events: Vec::new(),
            behavioral_patterns: Vec::new(),
            hesitation_analysis: HesitationAnalysis {
                total_hesitations: 0,
                average_hesitation_duration: Duration::from_secs(0),
                hesitation_locations: HashMap::new(),
                hesitation_triggers: Vec::new(),
                hesitation_patterns: Vec::new(),
            },
            typing_dynamics: TypingDynamicsProfile {
                words_per_minute: 0.0,
                characters_per_minute: 0.0,
                average_dwell_time: Duration::from_secs(0),
                average_flight_time: Duration::from_secs(0),
                typing_rhythm: abcdeez_core::data::interaction_tracking::TypingRhythm {
                    rhythm_score: 0.0,
                    burst_patterns: Vec::new(),
                    pause_patterns: Vec::new(),
                    tempo_changes: Vec::new(),
                },
                keystroke_intensity: abcdeez_core::data::interaction_tracking::KeystrokeIntensity {
                    pressure_variation: 0.0,
                    timing_precision: 0.0,
                    force_consistency: 0.0,
                    stress_indicators: Vec::new(),
                },
                error_rate: 0.0,
                correction_latency: Duration::from_secs(0),
                finger_usage_pattern: HashMap::new(),
                typing_consistency: 0.0,
            },
            interaction_quality: InteractionQuality {
                overall_score: 0.0,
                engagement_level: abcdeez_core::data::interaction_tracking::EngagementLevel::Low,
                attention_consistency: 0.0,
                task_focus: 0.0,
                interaction_efficiency: 0.0,
                error_handling_quality: 0.0,
                learning_indicators: Vec::new(),
            },
        });
        
        self.session_start = Instant::now();
        self.current_metrics = InteractionMetrics {
            typing_speed_wpm: 0.0,
            average_dwell_time_ms: 0.0,
            average_flight_time_ms: 0.0,
            pause_count: 0,
            correction_count: 0,
            mouse_distance_pixels: 0.0,
            mouse_velocity_avg: 0.0,
            hesitation_events: 0,
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInteractionData {
    pub session_duration_ms: u128,
    pub metrics: InteractionMetrics,
    pub core_metrics: Option<CoreMetrics>,
    pub total_events: usize,
    pub timestamp: DateTime<Utc>,
}

// Helper for integrating with Xilem event system
pub struct XilemInteractionHandler {
    service: InteractionTrackingService,
    last_key_time: Option<Instant>,
    last_mouse_pos: Option<(f64, f64)>,
}

impl XilemInteractionHandler {
    pub fn new() -> Self {
        Self {
            service: InteractionTrackingService::new(),
            last_key_time: None,
            last_mouse_pos: None,
        }
    }

    pub fn handle_key_event(&mut self, key: String) {
        let now = Instant::now();
        
        // Calculate dwell time if we have a previous key
        let duration_ms = if let Some(last) = self.last_key_time {
            (now - last).as_millis() as u64
        } else {
            KEYSTROKE_WINDOW_MS
        };

        // Check for pause
        if duration_ms > 1000 {
            self.service.track_pause(duration_ms);
        }

        self.service.track_keypress(key, duration_ms.min(KEYSTROKE_WINDOW_MS));
        self.last_key_time = Some(now);
    }

    pub fn handle_mouse_move(&mut self, x: f64, y: f64) {
        self.service.track_mouse_move(x, y);
        self.last_mouse_pos = Some((x, y));
    }

    pub fn handle_mouse_click(&mut self, x: f64, y: f64, button: MouseButton) {
        self.service.track_mouse_click(x, y, button);
    }

    pub fn handle_backspace(&mut self) {
        self.service.track_correction();
    }

    pub fn get_service(&self) -> &InteractionTrackingService {
        &self.service
    }

    pub fn get_service_mut(&mut self) -> &mut InteractionTrackingService {
        &mut self.service
    }
}