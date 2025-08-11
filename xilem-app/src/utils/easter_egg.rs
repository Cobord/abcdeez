// Easter Egg - Little Crab System
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Datelike, Timelike};
use rand::prelude::*;
use std::collections::HashMap;

/// The Little Crab - a friendly companion that appears during learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LittleCrab {
    pub active: bool,
    pub personality: CrabPersonality,
    pub phase: AnimationPhase,
    pub position: CrabPosition,
    pub energy: f32,
    pub messages: Vec<String>,
    pub discovery_count: u32,
    pub last_interaction: Option<DateTime<Utc>>,
    pub trigger_stats: HashMap<CrabTrigger, u32>,
}

/// Crab personality types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrabPersonality {
    Shy,       // Hides quickly, peeks out occasionally
    Playful,   // Dances and moves around frequently
    Helpful,   // Gives more hints and encouragement
    Mischievous, // Sometimes covers answers or UI elements
    Wise,      // Shares learning wisdom and quotes
}

/// Animation phases for the crab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationPhase {
    Hidden,
    Peeking,
    Crawling,
    Dancing,
    Helping,
    Sleeping,
    Excited,
    Curious,
    Waving,
}

/// Position of the crab on screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrabPosition {
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub speed: f32,
}

/// Triggers that can activate the crab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrabTrigger {
    TripleClick,
    LongPress,
    PerfectStreak,
    IdleTime,
    SecretWord,
    KonamiCode,
    TimeOfDay,
    SpecialDate,
}

impl Default for LittleCrab {
    fn default() -> Self {
        Self {
            active: false,
            personality: CrabPersonality::Playful,
            phase: AnimationPhase::Hidden,
            position: CrabPosition {
                x: 0.0,
                y: 0.0,
                target_x: 0.0,
                target_y: 0.0,
                speed: 2.0,
            },
            energy: 100.0,
            messages: Vec::new(),
            discovery_count: 0,
            last_interaction: None,
            trigger_stats: HashMap::new(),
        }
    }
}

impl LittleCrab {
    /// Create a new crab with random personality
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let personality = match rng.gen_range(0..5) {
            0 => CrabPersonality::Shy,
            1 => CrabPersonality::Playful,
            2 => CrabPersonality::Helpful,
            3 => CrabPersonality::Mischievous,
            _ => CrabPersonality::Wise,
        };
        
        Self {
            personality,
            ..Default::default()
        }
    }
    
    /// Activate the crab with a specific trigger
    pub fn activate(&mut self, trigger: CrabTrigger) {
        if !self.active {
            self.active = true;
            self.discovery_count += 1;
            self.phase = AnimationPhase::Peeking;
            self.energy = 100.0;
            
            // Track trigger usage
            *self.trigger_stats.entry(trigger).or_insert(0) += 1;
            
            // Set initial message based on trigger
            self.messages.push(self.get_discovery_message(trigger));
        }
        
        self.last_interaction = Some(Utc::now());
    }
    
    /// Deactivate and hide the crab
    pub fn deactivate(&mut self) {
        self.active = false;
        self.phase = AnimationPhase::Hidden;
        self.messages.clear();
    }
    
    /// Update crab state (call this regularly)
    pub fn update(&mut self, delta_time: f32) {
        if !self.active {
            return;
        }
        
        // Update position with smooth movement
        let dx = self.position.target_x - self.position.x;
        let dy = self.position.target_y - self.position.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance > 0.1 {
            let move_speed = self.position.speed * delta_time;
            self.position.x += (dx / distance) * move_speed.min(distance);
            self.position.y += (dy / distance) * move_speed.min(distance);
            
            // Update phase based on movement
            if self.phase != AnimationPhase::Crawling && distance > 10.0 {
                self.phase = AnimationPhase::Crawling;
            }
        } else if self.phase == AnimationPhase::Crawling {
            // Reached target, do something based on personality
            self.phase = self.get_idle_phase();
        }
        
        // Decrease energy over time
        self.energy = (self.energy - delta_time * 0.5).max(0.0);
        
        if self.energy <= 0.0 {
            self.phase = AnimationPhase::Sleeping;
        }
        
        // Random behaviors based on personality
        if rand::random::<f32>() < 0.01 * delta_time {
            self.perform_random_behavior();
        }
    }
    
    /// Move crab to a new position
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.position.target_x = x;
        self.position.target_y = y;
        
        // Adjust speed based on personality
        self.position.speed = match self.personality {
            CrabPersonality::Shy => 1.5,
            CrabPersonality::Playful => 3.0,
            CrabPersonality::Mischievous => 2.5,
            _ => 2.0,
        };
    }
    
    /// React to user interaction
    pub fn interact(&mut self) -> String {
        self.energy = (self.energy + 20.0).min(100.0);
        self.last_interaction = Some(Utc::now());
        
        // Change phase based on interaction
        self.phase = match self.personality {
            CrabPersonality::Shy => AnimationPhase::Peeking,
            CrabPersonality::Playful => AnimationPhase::Dancing,
            CrabPersonality::Helpful => AnimationPhase::Helping,
            CrabPersonality::Mischievous => AnimationPhase::Excited,
            CrabPersonality::Wise => AnimationPhase::Waving,
        };
        
        self.get_interaction_message()
    }
    
    /// Get a discovery message based on trigger
    fn get_discovery_message(&self, trigger: CrabTrigger) -> String {
        match trigger {
            CrabTrigger::TripleClick => match self.discovery_count {
                1 => "🦀 *peek* You found me! I'm Little Crab!".to_string(),
                2 => "🦀 Back again? You must really like crabs!".to_string(),
                _ => "🦀 We're old friends now!".to_string(),
            },
            CrabTrigger::PerfectStreak => "🦀 Wow! Perfect streak! You're amazing!".to_string(),
            CrabTrigger::IdleTime => "🦀 *yawn* Were you taking a break? Me too!".to_string(),
            CrabTrigger::SecretWord => "🦀 You know the secret word! 🎉".to_string(),
            CrabTrigger::KonamiCode => "🦀 ↑↑↓↓←→←→BA! Classic!".to_string(),
            CrabTrigger::TimeOfDay => self.get_time_based_message(),
            CrabTrigger::SpecialDate => self.get_special_date_message(),
            _ => "🦀 Hello there!".to_string(),
        }
    }
    
    /// Get interaction message based on personality
    fn get_interaction_message(&self) -> String {
        match self.personality {
            CrabPersonality::Shy => "🦀 *blushes* Oh, you clicked me...".to_string(),
            CrabPersonality::Playful => "🦀 Wheee! That tickles!".to_string(),
            CrabPersonality::Helpful => "🦀 Need help? I'm here for you!".to_string(),
            CrabPersonality::Mischievous => "🦀 Hehe, can't catch me!".to_string(),
            CrabPersonality::Wise => "🦀 'The journey of learning begins with curiosity.'".to_string(),
        }
    }
    
    /// Get time-based greeting
    fn get_time_based_message(&self) -> String {
        let hour = Utc::now().time().hour();
        match hour {
            5..=11 => "🦀 Good morning! Ready to learn?".to_string(),
            12..=16 => "🦀 Afternoon learning session!".to_string(),
            17..=20 => "🦀 Evening practice time!".to_string(),
            _ => "🦀 Late night learning? Impressive!".to_string(),
        }
    }
    
    /// Get special date message
    fn get_special_date_message(&self) -> String {
        let now = Utc::now();
        let month = now.month();
        let day = now.day();
        
        match (month, day) {
            (1, 1) => "🦀 Happy New Year! New year, new learning!".to_string(),
            (2, 14) => "🦀 Happy Valentine's! I ❤️ learning!".to_string(),
            (3, 14) => "🦀 Happy Pi Day! 3.14159...".to_string(),
            (4, 1) => "🦀 No fooling - you're doing great!".to_string(),
            (10, 31) => "🦀 Boo! Happy Halloween! 👻".to_string(),
            (12, 25) => "🦀 Merry Christmas! 🎄".to_string(),
            _ => "🦀 Today is special because you're learning!".to_string(),
        }
    }
    
    /// Get idle phase based on personality
    fn get_idle_phase(&self) -> AnimationPhase {
        match self.personality {
            CrabPersonality::Shy => AnimationPhase::Peeking,
            CrabPersonality::Playful => AnimationPhase::Dancing,
            CrabPersonality::Helpful => AnimationPhase::Waving,
            CrabPersonality::Mischievous => AnimationPhase::Curious,
            CrabPersonality::Wise => AnimationPhase::Waving,
        }
    }
    
    /// Perform random behavior based on personality
    fn perform_random_behavior(&mut self) {
        let mut rng = thread_rng();
        
        match self.personality {
            CrabPersonality::Playful => {
                // Random dance moves
                if rng.gen_bool(0.3) {
                    self.phase = AnimationPhase::Dancing;
                    self.messages.push("🦀 *does a little dance*".to_string());
                }
            },
            CrabPersonality::Mischievous => {
                // Move to random position
                if rng.gen_bool(0.2) {
                    let new_x = rng.gen_range(-50.0..50.0);
                    let new_y = rng.gen_range(-50.0..50.0);
                    self.move_to(self.position.x + new_x, self.position.y + new_y);
                }
            },
            CrabPersonality::Helpful => {
                // Offer random tips
                if rng.gen_bool(0.1) {
                    self.phase = AnimationPhase::Helping;
                    self.messages.push(self.get_random_tip());
                }
            },
            _ => {}
        }
    }
    
    /// Get a random learning tip
    fn get_random_tip(&self) -> String {
        let tips = [
            "🦀 Tip: Take breaks to improve retention!",
            "🦀 Tip: Practice a little every day!",
            "🦀 Tip: Mistakes help you learn faster!",
            "🦀 Tip: Teaching others reinforces learning!",
            "🦀 Tip: Sleep helps consolidate memories!",
        ];
        
        let mut rng = thread_rng();
        tips.choose(&mut rng).unwrap().to_string()
    }
    
    /// Check if crab should appear based on learning performance
    pub fn check_performance_trigger(&mut self, accuracy: f32, streak: u32) {
        if streak >= 10 && !self.active {
            self.activate(CrabTrigger::PerfectStreak);
            self.messages.push(format!("🦀 {} in a row! Incredible!", streak));
        } else if self.active && accuracy > 0.9 {
            self.phase = AnimationPhase::Excited;
            self.messages.push("🦀 You're on fire! 🔥".to_string());
        }
    }
    
    /// React to learning struggles
    pub fn encourage(&mut self) {
        if self.active {
            self.phase = AnimationPhase::Helping;
            let encouragement = match self.personality {
                CrabPersonality::Helpful => "🦀 Don't give up! You've got this!",
                CrabPersonality::Wise => "🦀 'Every expert was once a beginner.'",
                CrabPersonality::Playful => "🦀 Oopsie! Try again - you can do it!",
                CrabPersonality::Shy => "🦀 *whispers* I believe in you...",
                CrabPersonality::Mischievous => "🦀 That was tricky! Even I got confused!",
            };
            self.messages.push(encouragement.to_string());
        }
    }
}

/// Easter egg manager for handling triggers
pub struct EasterEggManager {
    pub crab: LittleCrab,
    konami_buffer: Vec<KonamiKey>,
    click_times: Vec<DateTime<Utc>>,
    last_activity: DateTime<Utc>,
    secret_word_buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum KonamiKey {
    Up, Down, Left, Right, B, A
}

impl Default for EasterEggManager {
    fn default() -> Self {
        Self {
            crab: LittleCrab::new(),
            konami_buffer: Vec::new(),
            click_times: Vec::new(),
            last_activity: Utc::now(),
            secret_word_buffer: String::new(),
        }
    }
}

impl EasterEggManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Handle a click event for triple-click detection
    pub fn handle_click(&mut self, x: f32, y: f32) -> bool {
        let now = Utc::now();
        
        // Clean old clicks (older than 500ms)
        self.click_times.retain(|t| (now - *t).num_milliseconds() < 500);
        self.click_times.push(now);
        
        // Check for triple click
        if self.click_times.len() >= 3 {
            self.crab.activate(CrabTrigger::TripleClick);
            self.crab.move_to(x, y);
            self.click_times.clear();
            return true;
        }
        
        false
    }
    
    /// Handle keyboard input for Konami code and secret words
    pub fn handle_key(&mut self, key: &str) -> bool {
        // Update activity
        self.last_activity = Utc::now();
        
        // Check for Konami code
        let konami_key = match key {
            "ArrowUp" => Some(KonamiKey::Up),
            "ArrowDown" => Some(KonamiKey::Down),
            "ArrowLeft" => Some(KonamiKey::Left),
            "ArrowRight" => Some(KonamiKey::Right),
            "b" | "B" => Some(KonamiKey::B),
            "a" | "A" => Some(KonamiKey::A),
            _ => None,
        };
        
        if let Some(k) = konami_key {
            self.konami_buffer.push(k);
            if self.konami_buffer.len() > 10 {
                self.konami_buffer.remove(0);
            }
            
            if self.check_konami_code() {
                self.crab.activate(CrabTrigger::KonamiCode);
                self.konami_buffer.clear();
                return true;
            }
        }
        
        // Check for secret word "crab"
        if key.len() == 1 {
            self.secret_word_buffer.push_str(key);
            if self.secret_word_buffer.len() > 10 {
                self.secret_word_buffer = self.secret_word_buffer[1..].to_string();
            }
            
            if self.secret_word_buffer.to_lowercase().contains("crab") {
                self.crab.activate(CrabTrigger::SecretWord);
                self.secret_word_buffer.clear();
                return true;
            }
        }
        
        false
    }
    
    /// Check if Konami code was entered
    fn check_konami_code(&self) -> bool {
        let konami = vec![
            KonamiKey::Up, KonamiKey::Up,
            KonamiKey::Down, KonamiKey::Down,
            KonamiKey::Left, KonamiKey::Right,
            KonamiKey::Left, KonamiKey::Right,
            KonamiKey::B, KonamiKey::A,
        ];
        
        self.konami_buffer.len() >= konami.len() &&
            self.konami_buffer[self.konami_buffer.len() - konami.len()..] == konami
    }
    
    /// Check for idle trigger
    pub fn check_idle(&mut self) -> bool {
        let now = Utc::now();
        let idle_duration = (now - self.last_activity).num_seconds();
        
        if idle_duration > 30 && !self.crab.active {
            self.crab.activate(CrabTrigger::IdleTime);
            return true;
        }
        
        false
    }
    
    /// Update the easter egg system
    pub fn update(&mut self, delta_time: f32) {
        self.crab.update(delta_time);
        
        // Check for idle trigger periodically
        if rand::random::<f32>() < 0.01 {
            self.check_idle();
        }
        
        // Check for special times
        if !self.crab.active && rand::random::<f32>() < 0.001 {
            let hour = Utc::now().time().hour();
            if hour == 13 || hour == 23 {  // 1 PM or 11 PM
                self.crab.activate(CrabTrigger::TimeOfDay);
            }
        }
    }
}