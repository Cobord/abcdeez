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
    pub mood: CrabMood,
    pub friendship_level: u32,
    pub collected_treasures: Vec<CrabTreasure>,
    pub seasonal_outfit: Option<String>,
    pub favorite_domain: Option<String>,
}

/// Crab personality types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrabPersonality {
    Shy,       // Hides quickly, peeks out occasionally
    Playful,   // Dances and moves around frequently
    Helpful,   // Gives more hints and encouragement
    Mischievous, // Sometimes covers answers or UI elements
    Wise,      // Shares learning wisdom and quotes
    Energetic, // Always bouncing around with high energy
    Sleepy,    // Often napping, low energy
    Curious,   // Investigates everything
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
    Swimming,
    Digging,
    Sunbathing,
    Reading,
    Cheering,
    Thinking,
    Celebrating,
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
    ScreenEdgeHover,
    EasterEggHunt,
    BirthdayMode,
    AchievementUnlock,
    SpecialDate,
}

/// Crab mood affects behavior and messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrabMood {
    Happy,
    Excited,
    Neutral,
    Sleepy,
    Playful,
    Proud,
    Encouraging,
    Curious,
    Celebratory,
    Philosophical,
}

/// Treasures the crab can find and give
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrabTreasure {
    pub name: String,
    pub emoji: String,
    pub found_at: DateTime<Utc>,
    pub rarity: TreasureRarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreasureRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
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
            mood: CrabMood::Neutral,
            friendship_level: 0,
            collected_treasures: Vec::new(),
            seasonal_outfit: None,
            favorite_domain: None,
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
            self.energy = 100.0;
            
            // Track trigger usage
            *self.trigger_stats.entry(trigger).or_insert(0) += 1;
            
            // Set phase based on trigger and personality
            self.phase = match (trigger, self.personality) {
                (CrabTrigger::TripleClick, CrabPersonality::Shy) => AnimationPhase::Peeking,
                (CrabTrigger::TripleClick, _) => AnimationPhase::Excited,
                (CrabTrigger::PerfectStreak, _) => AnimationPhase::Cheering,
                (CrabTrigger::IdleTime, CrabPersonality::Sleepy) => AnimationPhase::Sleeping,
                (CrabTrigger::IdleTime, _) => AnimationPhase::Curious,
                (CrabTrigger::SecretWord, _) => AnimationPhase::Dancing,
                (CrabTrigger::KonamiCode, _) => AnimationPhase::Celebrating,
                (CrabTrigger::AchievementUnlock, _) => AnimationPhase::Cheering,
                (CrabTrigger::BirthdayMode, _) => AnimationPhase::Celebrating,
                _ => AnimationPhase::Peeking,
            };
            
            // Set mood based on trigger
            self.mood = match trigger {
                CrabTrigger::PerfectStreak | CrabTrigger::AchievementUnlock => CrabMood::Proud,
                CrabTrigger::SecretWord | CrabTrigger::KonamiCode => CrabMood::Playful,
                CrabTrigger::BirthdayMode => CrabMood::Celebratory,
                CrabTrigger::IdleTime if self.personality == CrabPersonality::Sleepy => CrabMood::Sleepy,
                _ => CrabMood::Happy,
            };
            
            // Set initial message based on trigger
            self.messages.push(self.get_discovery_message(trigger));
            
            // Update seasonal outfit
            self.update_seasonal_outfit();
            
            // Boost friendship for discovery
            self.friendship_level += 1;
        } else {
            // Already active - just update interaction
            self.friendship_level += 1;
            self.energy = (self.energy + 20.0).min(100.0);
        }
        
        self.last_interaction = Some(Utc::now());
    }
    
    /// Deactivate and hide the crab
    pub fn deactivate(&mut self) {
        self.active = false;
        self.phase = AnimationPhase::Hidden;
        self.messages.clear();
    }
    
    /// Update seasonal outfit based on current date
    fn update_seasonal_outfit(&mut self) {
        let now = Utc::now();
        let month = now.month();
        let day = now.day();
        
        self.seasonal_outfit = match (month, day) {
            (12, 20..=26) => Some("🎅 Santa Hat".to_string()),
            (10, 25..=31) => Some("🎃 Pumpkin Costume".to_string()),
            (7, 1..=7) => Some("🎆 Fireworks Hat".to_string()),
            (2, 14) => Some("💝 Heart Antenna".to_string()),
            (3, 17) => Some("☘️ Lucky Clover".to_string()),
            (4, 1) => Some("🤡 Jester Hat".to_string()),
            (5, _) if day <= 7 => Some("🌸 Spring Flowers".to_string()),
            (6, _) if day >= 21 => Some("☀️ Sunglasses".to_string()),
            (9, _) if day >= 22 => Some("🍂 Autumn Leaves".to_string()),
            (11, _) if day >= 24 && day <= 30 => Some("🦃 Pilgrim Hat".to_string()),
            _ => None,
        };
    }
    
    /// Celebrate user's success with personality-specific messages
    pub fn celebrate_success(&mut self, streak: u32, accuracy: f32) {
        self.mood = CrabMood::Celebratory;
        self.energy = (self.energy + 30.0).min(100.0);
        self.friendship_level += 2;
        
        // Set celebration phase
        self.phase = AnimationPhase::Celebrating;
        
        // Generate celebration message based on personality and achievement
        let message = match (self.personality, streak, accuracy) {
            (_, s, _) if s >= 20 => "🎉🦀 LEGENDARY STREAK! You're unstoppable! 🦀🎉".to_string(),
            (_, s, _) if s >= 10 => "🌟🦀 Double digits! Amazing consistency! 🦀🌟".to_string(),
            (_, _, a) if a >= 1.0 => "💯🦀 PERFECT! You're a learning machine! 🦀💯".to_string(),
            (CrabPersonality::Shy, _, _) => "🦀 *happy dance* You did so well! 🦀".to_string(),
            (CrabPersonality::Playful, _, _) => "🎊🦀 WOOHOO! Let's celebrate! 🦀🎊".to_string(),
            (CrabPersonality::Helpful, _, _) => "🦀 I knew you could do it! Great job! 🦀".to_string(),
            (CrabPersonality::Mischievous, _, _) => "🦀 Hehe, you're getting too good at this! 🦀".to_string(),
            (CrabPersonality::Wise, _, _) => "🦀 'Excellence is a habit.' - You're proving it! 🦀".to_string(),
            (CrabPersonality::Energetic, _, _) => "⚡🦀 BOOM! That was AMAZING! ⚡🦀".to_string(),
            (CrabPersonality::Sleepy, _, _) => "😊🦀 *yawn* Even I'm awake for this success! 🦀😊".to_string(),
            (CrabPersonality::Curious, _, _) => "🤔🦀 Fascinating! Your learning pattern is impressive! 🦀🤔".to_string(),
        };
        
        self.messages.push(message);
        
        // Chance to find treasure when celebrating
        if rand::random::<f32>() < 0.3 {
            self.find_treasure();
        }
    }
    
    /// Offer encouragement during struggles
    pub fn offer_encouragement(&mut self, attempts: u32) {
        self.mood = CrabMood::Encouraging;
        self.phase = AnimationPhase::Helping;
        self.friendship_level += 1;
        
        let message = match (self.personality, attempts) {
            (_, a) if a > 5 => "🦀 Learning takes time. Every attempt makes you stronger! 🦀",
            (CrabPersonality::Shy, _) => "🦀 *whispers* Don't give up... you can do it... 🦀",
            (CrabPersonality::Playful, _) => "🦀 Mistakes are just practice in disguise! Keep going! 🦀",
            (CrabPersonality::Helpful, _) => "🦀 Let me help! Try breaking it down into smaller steps! 🦀",
            (CrabPersonality::Mischievous, _) => "🦀 Tricky one, huh? Even I'd struggle with that! 🦀",
            (CrabPersonality::Wise, _) => "🦀 'The master has failed more times than the beginner has tried.' 🦀",
            (CrabPersonality::Energetic, _) => "💪🦀 You've GOT this! One more try! 💪🦀",
            (CrabPersonality::Sleepy, _) => "😴🦀 Maybe a little break would help? Rest refreshes the mind! 🦀😴",
            (CrabPersonality::Curious, _) => "🤔🦀 Hmm, what if you tried a different approach? 🦀🤔",
        };
        
        self.messages.push(message.to_string());
    }
    
    /// Find and collect treasure
    pub fn find_treasure(&mut self) {
        let mut rng = thread_rng();
        
        // Determine rarity based on friendship level
        let rarity = if self.friendship_level > 50 && rng.gen_bool(0.1) {
            TreasureRarity::Legendary
        } else if self.friendship_level > 30 && rng.gen_bool(0.15) {
            TreasureRarity::Epic
        } else if self.friendship_level > 15 && rng.gen_bool(0.25) {
            TreasureRarity::Rare
        } else if rng.gen_bool(0.4) {
            TreasureRarity::Uncommon
        } else {
            TreasureRarity::Common
        };
        
        let (name, emoji) = match rarity {
            TreasureRarity::Common => {
                let items = [("Seashell", "🐚"), ("Pebble", "🪨"), ("Seaweed", "🌿")];
                items.choose(&mut rng).unwrap().clone()
            },
            TreasureRarity::Uncommon => {
                let items = [("Starfish", "⭐"), ("Pearl", "🦪"), ("Coral", "🪸")];
                items.choose(&mut rng).unwrap().clone()
            },
            TreasureRarity::Rare => {
                let items = [("Golden Shell", "🏆"), ("Crystal", "💎"), ("Ancient Coin", "🪙")];
                items.choose(&mut rng).unwrap().clone()
            },
            TreasureRarity::Epic => {
                let items = [("Magic Conch", "🐚✨"), ("Mermaid Scale", "🧜‍♀️"), ("Neptune's Fork", "🔱")];
                items.choose(&mut rng).unwrap().clone()
            },
            TreasureRarity::Legendary => {
                let items = [("Crab Crown", "👑🦀"), ("Ocean Heart", "💙🌊"), ("Wisdom Pearl", "🧠🦪")];
                items.choose(&mut rng).unwrap().clone()
            },
        };
        
        let treasure = CrabTreasure {
            name: name.to_string(),
            emoji: emoji.to_string(),
            found_at: Utc::now(),
            rarity,
        };
        
        self.phase = AnimationPhase::Digging;
        self.messages.push(format!("🦀 Found a {} {}! 🦀", treasure.emoji, treasure.name));
        self.collected_treasures.push(treasure);
    }
    
    /// Get emoji representation with seasonal outfit
    pub fn get_emoji(&self) -> String {
        let base_emoji = match self.phase {
            AnimationPhase::Hidden => return String::new(),
            AnimationPhase::Peeking => "👀",
            AnimationPhase::Crawling => "🦀",
            AnimationPhase::Dancing => "💃🦀",
            AnimationPhase::Helping => "🦀💪",
            AnimationPhase::Sleeping => "😴🦀",
            AnimationPhase::Excited => "🎉🦀",
            AnimationPhase::Curious => "🤔🦀",
            AnimationPhase::Waving => "👋🦀",
            AnimationPhase::Swimming => "🏊🦀",
            AnimationPhase::Digging => "⛏️🦀",
            AnimationPhase::Sunbathing => "☀️🦀",
            AnimationPhase::Reading => "📚🦀",
            AnimationPhase::Cheering => "📣🦀",
            AnimationPhase::Thinking => "💭🦀",
            AnimationPhase::Celebrating => "🎊🦀🎊",
        };
        
        if let Some(outfit) = &self.seasonal_outfit {
            format!("{} {}", base_emoji, outfit)
        } else {
            base_emoji.to_string()
        }
    }
    
    /// React to learning context and domain
    pub fn react_to_domain(&mut self, domain: &str) {
        if Some(domain.to_string()) != self.favorite_domain {
            // New domain exploration
            self.favorite_domain = Some(domain.to_string());
            self.phase = AnimationPhase::Curious;
            
            let message = match domain {
                "alphabet" => "🦀 A-B-C... I love the alphabet! 26 friends to learn! 🦀",
                "numbers" => "🦀 1, 2, 3... Numbers are like counting my legs! (8!) 🦀",
                "shapes" => "🦀 Circles, squares... My shell is a dome shape! 🦀",
                "colors" => "🦀 Red like me when I'm cooked- wait, let's not think about that! 🦀",
                "music" => "🦀 ♪♫ Time for the crab rave! ♫♪ 🦀",
                _ => "🦀 Ooh, something new to explore! 🦀",
            };
            
            self.messages.push(message.to_string());
        }
    }
    
    /// Get stats summary
    pub fn get_stats_summary(&self) -> String {
        format!(
            "🦀 Little Crab Stats:\n\
            Personality: {:?}\n\
            Mood: {:?}\n\
            Energy: {:.0}%\n\
            Friendship: Level {}\n\
            Treasures: {}\n\
            Times Found: {}",
            self.personality,
            self.mood,
            self.energy,
            self.friendship_level,
            self.collected_treasures.len(),
            self.discovery_count
        )
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
            CrabPersonality::Energetic => AnimationPhase::Excited,
            CrabPersonality::Sleepy => AnimationPhase::Sleeping,
            CrabPersonality::Curious => AnimationPhase::Curious,
        };
        
        self.get_interaction_message()
    }
    
    /// Get a discovery message based on trigger
    fn get_discovery_message(&self, trigger: CrabTrigger) -> String {
        match trigger {
            CrabTrigger::TripleClick => match self.discovery_count {
                1 => "🦀 *peek* You found me! I'm Little Crab!".to_string(),
                2 => "🦀 Back again? You must really like crabs!".to_string(),
                _ => format!("🦀 We're old friends now! (Found {} times!)", self.discovery_count),
            },
            CrabTrigger::PerfectStreak => "🦀 Wow! Perfect streak! You're amazing!".to_string(),
            CrabTrigger::LongPress => "🦀 That was a long press! You must really want to see me!".to_string(),
            CrabTrigger::IdleTime => "🦀 *yawn* Were you taking a break? Me too!".to_string(),
            CrabTrigger::SecretWord => "🦀 You know the secret word! 🎉".to_string(),
            CrabTrigger::KonamiCode => "🦀 ↑↑↓↓←→←→BA! Classic gamer move!".to_string(),
            CrabTrigger::TimeOfDay => self.get_time_based_message(),
            CrabTrigger::SpecialDate => self.get_special_date_message(),
            CrabTrigger::ScreenEdgeHover => "🦀 Peeking from the edge? I see you!".to_string(),
            CrabTrigger::EasterEggHunt => "🐰🦀 Easter Crab has treasures for you!".to_string(),
            CrabTrigger::BirthdayMode => "🎂🦀 HAPPY BIRTHDAY! Special crab dance!".to_string(),
            CrabTrigger::AchievementUnlock => "🏆🦀 Achievement! I'm so proud of you!".to_string(),
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
            CrabPersonality::Energetic => "⚡ ENERGY CRAB IS HERE! Let's GO!".to_string(),
            CrabPersonality::Sleepy => "😴 *yawn* Five more minutes...".to_string(),
            CrabPersonality::Curious => "🤔 Hmm, what happens if I click this?".to_string(),
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
            CrabPersonality::Wise => AnimationPhase::Reading,
            CrabPersonality::Energetic => AnimationPhase::Excited,
            CrabPersonality::Sleepy => AnimationPhase::Sleeping,
            CrabPersonality::Curious => AnimationPhase::Thinking,
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
                    let dances = [
                        "🦀 *does the crab shuffle* 🦀",
                        "🦀 *spins in circles* 🦀",
                        "🦀 *waves claws to the beat* 🦀",
                        "🦀 *moonwalks sideways* 🦀",
                    ];
                    self.messages.push(dances.choose(&mut rng).unwrap().to_string());
                }
            },
            CrabPersonality::Mischievous => {
                // Move to random position and play tricks
                if rng.gen_bool(0.2) {
                    let new_x = rng.gen_range(-50.0..50.0);
                    let new_y = rng.gen_range(-50.0..50.0);
                    self.move_to(self.position.x + new_x, self.position.y + new_y);
                    let tricks = [
                        "🦀 *sneakily moves your cursor* 🦀",
                        "🦀 *hides behind a button* 🦀",
                        "🦀 Catch me if you can! 🦀",
                        "🦀 *giggles mischievously* 🦀",
                    ];
                    self.messages.push(tricks.choose(&mut rng).unwrap().to_string());
                }
            },
            CrabPersonality::Helpful => {
                // Offer random tips
                if rng.gen_bool(0.1) {
                    self.phase = AnimationPhase::Helping;
                    self.messages.push(self.get_random_tip());
                }
            },
            CrabPersonality::Energetic => {
                // Bounce around with high energy
                if rng.gen_bool(0.4) {
                    self.phase = AnimationPhase::Excited;
                    let new_x = rng.gen_range(-100.0..100.0);
                    let new_y = rng.gen_range(-30.0..30.0);
                    self.move_to(self.position.x + new_x, self.position.y + new_y);
                    let energy_bursts = [
                        "⚡🦀 ZOOM ZOOM! ⚡🦀",
                        "🏃‍♂️🦀 SPEEDY CRAB! 🏃‍♂️🦀",
                        "🚀🦀 TURBO MODE! 🚀🦀",
                        "💨🦀 WHOOSH! 💨🦀",
                    ];
                    self.messages.push(energy_bursts.choose(&mut rng).unwrap().to_string());
                }
            },
            CrabPersonality::Sleepy => {
                // Take frequent naps
                if rng.gen_bool(0.5) && self.energy < 50.0 {
                    self.phase = AnimationPhase::Sleeping;
                    let sleepy_sounds = [
                        "😴🦀 *zzz* 🦀😴",
                        "🦀 *snores softly* 🦀",
                        "💤🦀 *dreams of kelp* 💤🦀",
                        "🦀 *mumbles about shells* 🦀",
                    ];
                    self.messages.push(sleepy_sounds.choose(&mut rng).unwrap().to_string());
                }
            },
            CrabPersonality::Curious => {
                // Investigate and ponder
                if rng.gen_bool(0.3) {
                    self.phase = AnimationPhase::Thinking;
                    let thoughts = [
                        "🤔🦀 I wonder what happens if... 🦀🤔",
                        "🦀 Hmm, interesting pattern here... 🦀",
                        "🦀 Let me analyze this... 🦀",
                        "🔍🦀 *investigates closely* 🔍🦀",
                        "🦀 But why does it work that way? 🦀",
                    ];
                    self.messages.push(thoughts.choose(&mut rng).unwrap().to_string());
                }
            },
            CrabPersonality::Shy => {
                // Occasionally peek out and hide
                if rng.gen_bool(0.1) {
                    if self.phase == AnimationPhase::Peeking {
                        self.phase = AnimationPhase::Hidden;
                        self.messages.push("🦀 *hides* 🦀".to_string());
                    } else {
                        self.phase = AnimationPhase::Peeking;
                        self.messages.push("👀 *peeks out nervously* 👀".to_string());
                    }
                }
            },
            CrabPersonality::Wise => {
                // Share wisdom and contemplate
                if rng.gen_bool(0.15) {
                    self.phase = AnimationPhase::Reading;
                    let wisdom = [
                        "🦀 'The ocean teaches patience.' 🦀",
                        "🦀 'Every tide brings new opportunities.' 🦀",
                        "🦀 'Small steps sideways still move forward.' 🦀",
                        "🦀 'The shell protects, but also limits.' 🦀",
                        "🦀 'In stillness, we find clarity.' 🦀",
                    ];
                    self.messages.push(wisdom.choose(&mut rng).unwrap().to_string());
                }
            }
        }
    }
    
    /// Get a random learning tip
    fn get_random_tip(&self) -> String {
        let tips = match self.personality {
            CrabPersonality::Helpful => vec![
                "🦀 Tip: Take breaks to improve retention! 🦀",
                "🦀 Tip: Practice a little every day! 🦀",
                "🦀 Tip: Mistakes help you learn faster! 🦀",
                "🦀 Tip: Teaching others reinforces learning! 🦀",
                "🦀 Tip: Sleep helps consolidate memories! 🦀",
                "🦀 Tip: Celebrate small victories! 🦀",
                "🦀 Tip: Use multiple senses when learning! 🦀",
            ],
            CrabPersonality::Wise => vec![
                "🦀 'Learning is the tide that lifts all boats.' 🦀",
                "🦀 'Knowledge flows like water - let it find its way.' 🦀",
                "🦀 'The deepest learning happens in calm waters.' 🦀",
                "🦀 'Every expert was once a curious beginner.' 🦀",
            ],
            CrabPersonality::Playful => vec![
                "🦀 Learning is more fun with friends! 🦀",
                "🦀 Turn practice into a game! 🦀",
                "🦀 Dance breaks boost brain power! 🦀",
                "🦀 Silly mnemonics stick better! 🦀",
            ],
            _ => vec![
                "🦀 You're doing great! 🦀",
                "🦀 Keep going! 🦀",
                "🦀 I believe in you! 🦀",
            ],
        };
        
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
                CrabPersonality::Energetic => "💪🦀 You've GOT this! Full power! 💪🦀",
                CrabPersonality::Sleepy => "😴🦀 *yawn* Maybe after a little rest? 🦀😴",
                CrabPersonality::Curious => "🤔🦀 Hmm, what if we try differently? 🦀🤔",
            };
            self.messages.push(encouragement.to_string());
        }
    }
}

/// Easter egg manager for handling triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EasterEggManager {
    pub crab: LittleCrab,
    konami_buffer: Vec<KonamiKey>,
    click_times: Vec<DateTime<Utc>>,
    last_activity: DateTime<Utc>,
    secret_word_buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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