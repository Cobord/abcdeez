// Power-up system
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerUp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub duration_seconds: i64,
    pub effect: PowerUpEffect,
    pub active_until: Option<DateTime<Utc>>,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerUpEffect {
    DoubleXP,       // Double experience points
    HintBoost,      // Extra hints without penalty
    TimeFreeze,     // More time to answer
    StreakShield,   // Protect streak for one day
    FocusMode,      // Hide distractions, increase rewards
}

impl PowerUp {
    pub fn new(effect: PowerUpEffect) -> Self {
        let (id, name, description, icon, duration) = match &effect {
            PowerUpEffect::DoubleXP => (
                "double_xp",
                "Double XP",
                "Earn double experience points for the next 30 minutes",
                "⚡",
                1800, // 30 minutes
            ),
            PowerUpEffect::HintBoost => (
                "hint_boost",
                "Hint Boost",
                "Get unlimited hints for the next 15 minutes",
                "💡",
                900, // 15 minutes
            ),
            PowerUpEffect::TimeFreeze => (
                "time_freeze",
                "Time Freeze",
                "Double your response time for the next 20 minutes",
                "⏰",
                1200, // 20 minutes
            ),
            PowerUpEffect::StreakShield => (
                "streak_shield",
                "Streak Shield",
                "Protect your streak if you miss a day",
                "🛡️",
                86400, // 24 hours
            ),
            PowerUpEffect::FocusMode => (
                "focus_mode",
                "Focus Mode",
                "Eliminate distractions and earn 50% more points",
                "🎯",
                2400, // 40 minutes
            ),
        };
        
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            icon: icon.to_string(),
            duration_seconds: duration,
            effect,
            active_until: None,
            quantity: 1,
        }
    }
    
    pub fn activate(&mut self) {
        if self.quantity > 0 {
            self.active_until = Some(Utc::now() + Duration::seconds(self.duration_seconds));
            self.quantity -= 1;
        }
    }
    
    pub fn is_active(&self) -> bool {
        if let Some(until) = self.active_until {
            Utc::now() < until
        } else {
            false
        }
    }
    
    pub fn time_remaining(&self) -> Option<Duration> {
        if let Some(until) = self.active_until {
            let now = Utc::now();
            if now < until {
                Some(until.signed_duration_since(now))
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn display_time_remaining(&self) -> String {
        if let Some(duration) = self.time_remaining() {
            let minutes = duration.num_minutes();
            let seconds = duration.num_seconds() % 60;
            if minutes > 0 {
                format!("{}m {}s", minutes, seconds)
            } else {
                format!("{}s", seconds)
            }
        } else {
            "Inactive".to_string()
        }
    }
    
    pub fn apply_effect(&self, base_value: f32) -> f32 {
        if !self.is_active() {
            return base_value;
        }
        
        match self.effect {
            PowerUpEffect::DoubleXP => base_value * 2.0,
            PowerUpEffect::FocusMode => base_value * 1.5,
            _ => base_value,
        }
    }
}

impl PowerUpEffect {
    pub fn color(&self) -> &str {
        match self {
            PowerUpEffect::DoubleXP => "#f6ad55",     // Gold
            PowerUpEffect::HintBoost => "#4299e1",    // Blue
            PowerUpEffect::TimeFreeze => "#9f7aea",   // Purple
            PowerUpEffect::StreakShield => "#48bb78", // Green
            PowerUpEffect::FocusMode => "#ed8936",    // Orange
        }
    }
}