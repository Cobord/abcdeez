// Streak tracking system
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Datelike};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakInfo {
    pub current: u32,
    pub longest: u32,
    pub last_practice: Option<DateTime<Utc>>,
    pub freeze_charges: u32,
    pub frozen_days: Vec<DateTime<Utc>>,
}

impl StreakInfo {
    pub fn new() -> Self {
        Self {
            current: 0,
            longest: 0,
            last_practice: None,
            freeze_charges: 3,
            frozen_days: Vec::new(),
        }
    }
    
    pub fn update(&mut self, practiced_today: bool) {
        let _today = Utc::now();
        
        if practiced_today {
            self.practice_today();
        } else {
            self.check_streak_break();
        }
    }
    
    pub fn practice_today(&mut self) {
        let today = Utc::now();
        
        if let Some(last) = self.last_practice {
            let days_since = (today.date_naive() - last.date_naive()).num_days();
            
            match days_since {
                0 => {
                    // Already practiced today, no change
                },
                1 => {
                    // Consecutive day!
                    self.current += 1;
                    if self.current > self.longest {
                        self.longest = self.current;
                    }
                },
                2 if self.was_frozen(last, today) => {
                    // Used a freeze charge yesterday
                    // Streak continues
                },
                _ => {
                    // Streak broken, start new
                    self.current = 1;
                }
            }
        } else {
            // First practice ever
            self.current = 1;
            self.longest = 1;
        }
        
        self.last_practice = Some(today);
    }
    
    fn check_streak_break(&mut self) {
        if let Some(last) = self.last_practice {
            let today = Utc::now();
            let days_since = (today.date_naive() - last.date_naive()).num_days();
            
            if days_since > 1 && !self.was_frozen(last, today) {
                // Streak broken
                self.current = 0;
            }
        }
    }
    
    pub fn use_freeze(&mut self) -> Result<(), String> {
        if self.freeze_charges == 0 {
            return Err("No freeze charges available".to_string());
        }
        
        let today = Utc::now();
        
        // Check if already frozen today
        if self.frozen_days.iter().any(|d| d.date_naive() == today.date_naive()) {
            return Err("Already used freeze today".to_string());
        }
        
        self.freeze_charges -= 1;
        self.frozen_days.push(today);
        Ok(())
    }
    
    fn was_frozen(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> bool {
        let from_date = from.date_naive();
        let to_date = to.date_naive();
        
        self.frozen_days.iter().any(|frozen| {
            let frozen_date = frozen.date_naive();
            frozen_date > from_date && frozen_date < to_date
        })
    }
    
    pub fn days_until_break(&self) -> u32 {
        if let Some(last) = self.last_practice {
            let today = Utc::now();
            let days_since = (today.date_naive() - last.date_naive()).num_days();
            
            if days_since == 0 {
                1 // Practiced today, safe until tomorrow
            } else if days_since == 1 {
                0 // Need to practice today!
            } else {
                0 // Already broken
            }
        } else {
            0
        }
    }
    
    pub fn is_at_risk(&self) -> bool {
        self.days_until_break() == 0 && self.current > 0
    }
    
    pub fn display_status(&self) -> String {
        if self.current == 0 {
            "No streak".to_string()
        } else if self.is_at_risk() {
            format!("🔥 {} days - Practice today to continue!", self.current)
        } else {
            format!("🔥 {} days", self.current)
        }
    }
    
    pub fn get_milestone(&self) -> Option<u32> {
        match self.current {
            7 => Some(7),
            14 => Some(14),
            30 => Some(30),
            60 => Some(60),
            100 => Some(100),
            365 => Some(365),
            _ => None,
        }
    }
}

impl Default for StreakInfo {
    fn default() -> Self {
        Self::new()
    }
}