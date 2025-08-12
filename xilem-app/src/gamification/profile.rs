// User gamification profile
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::{Achievement, PowerUp, StreakInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamificationProfile {
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub level: u32,
    pub experience: u32,
    pub total_points: u32,
    pub rank: Rank,
    pub achievements: Vec<Achievement>,
    pub badges: Vec<Badge>,
    pub streak: StreakInfo,
    pub statistics: UserStatistics,
    pub weekly_goals: WeeklyGoals,
    pub power_ups: Vec<PowerUp>,
    pub friends: Vec<String>,
    pub leaderboard_rank: u32,
    pub all_achievements_count: usize,
    pub pending_friend_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rank {
    pub title: String,
    pub tier: u32,
    pub icon: String,
    pub color: String,
}

impl Rank {
    pub fn from_level(level: u32) -> Self {
        let (title, icon, color) = match level {
            0..=9 => ("Novice", "🌱", "#48bb78"),
            10..=24 => ("Apprentice", "📚", "#4299e1"),
            25..=49 => ("Scholar", "🎓", "#9f7aea"),
            50..=74 => ("Expert", "⭐", "#ed8936"),
            75..=99 => ("Master", "🏆", "#f6ad55"),
            100..=149 => ("Grandmaster", "👑", "#e53e3e"),
            150..=199 => ("Sage", "🧙", "#b794f4"),
            _ => ("Legend", "🌟", "#ffd700"),
        };
        
        Rank {
            title: title.to_string(),
            tier: (level % 10) + 1,
            icon: icon.to_string(),
            color: color.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub earned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    pub total_sessions: u32,
    pub total_tasks: u32,
    pub total_time_minutes: u32,
    pub average_accuracy: f32,
    pub best_accuracy_session: f32,
    pub fastest_response_ms: u32,
    pub domains_mastered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyGoals {
    pub tasks_goal: u32,
    pub tasks_completed: u32,
    pub accuracy_goal: f32,
    pub current_accuracy: f32,
    pub streak_goal: u32,
    pub current_streak: u32,
    pub bonus_multiplier: f32,
    pub xp_goal: u32,
    pub xp_earned: u32,
}

impl GamificationProfile {
    pub fn new(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            display_name: format!("User {}", &user_id[..8.min(user_id.len())]),
            avatar_url: None,
            level: 1,
            experience: 0,
            total_points: 0,
            rank: Rank::from_level(1),
            achievements: Vec::new(),
            badges: Vec::new(),
            streak: StreakInfo::new(),
            statistics: UserStatistics {
                total_sessions: 0,
                total_tasks: 0,
                total_time_minutes: 0,
                average_accuracy: 0.0,
                best_accuracy_session: 0.0,
                fastest_response_ms: u32::MAX,
                domains_mastered: Vec::new(),
            },
            weekly_goals: WeeklyGoals {
                tasks_goal: 100,
                tasks_completed: 0,
                accuracy_goal: 0.8,
                current_accuracy: 0.0,
                streak_goal: 7,
                current_streak: 0,
                bonus_multiplier: 1.0,
                xp_goal: 1000,
                xp_earned: 0,
            },
            power_ups: Vec::new(),
            friends: Vec::new(),
            leaderboard_rank: 0,
            all_achievements_count: 50, // Total available achievements
            pending_friend_requests: 0,
        }
    }
    
    pub fn add_experience(&mut self, xp: u32) {
        self.experience += xp;
        
        // Check for level up
        while self.should_level_up() {
            self.level_up();
        }
    }
    
    fn should_level_up(&self) -> bool {
        let xp_for_next = self.xp_for_next_level();
        self.experience >= xp_for_next
    }
    
    fn xp_for_next_level(&self) -> u32 {
        // XP required for next level: 100 * level^1.5
        (100.0 * (self.level as f32 + 1.0).powf(1.5)) as u32
    }
    
    fn level_up(&mut self) {
        let xp_for_next = self.xp_for_next_level();
        self.level += 1;
        self.experience -= xp_for_next;
        self.rank = Rank::from_level(self.level);
        
        // Award badge for milestone levels
        if self.level % 10 == 0 {
            self.badges.push(Badge {
                id: format!("level_{}", self.level),
                name: format!("Level {} Milestone", self.level),
                icon: "🎖️".to_string(),
                description: format!("Reached level {}", self.level),
                earned_at: Utc::now(),
            });
        }
    }
    
    pub fn update_streak(&mut self, practiced_today: bool) {
        self.streak.update(practiced_today);
        self.weekly_goals.current_streak = self.streak.current;
    }
    
    pub fn update_statistics(&mut self, session_data: SessionData) {
        self.statistics.total_sessions += 1;
        self.statistics.total_tasks += session_data.tasks_completed;
        self.statistics.total_time_minutes += session_data.duration_minutes;
        
        // Update average accuracy
        let total_accuracy = self.statistics.average_accuracy * (self.statistics.total_sessions - 1) as f32;
        self.statistics.average_accuracy = (total_accuracy + session_data.accuracy) / self.statistics.total_sessions as f32;
        
        // Update best accuracy
        if session_data.accuracy > self.statistics.best_accuracy_session {
            self.statistics.best_accuracy_session = session_data.accuracy;
        }
        
        // Update fastest response
        if session_data.fastest_response_ms < self.statistics.fastest_response_ms {
            self.statistics.fastest_response_ms = session_data.fastest_response_ms;
        }
        
        // Update weekly goals
        self.weekly_goals.tasks_completed += session_data.tasks_completed;
        self.weekly_goals.current_accuracy = self.statistics.average_accuracy;
        
        // Check for weekly goal completion bonus
        self.check_weekly_goals();
    }
    
    fn check_weekly_goals(&mut self) {
        let mut goals_met = 0;
        
        if self.weekly_goals.tasks_completed >= self.weekly_goals.tasks_goal {
            goals_met += 1;
        }
        
        if self.weekly_goals.current_accuracy >= self.weekly_goals.accuracy_goal {
            goals_met += 1;
        }
        
        if self.weekly_goals.current_streak >= self.weekly_goals.streak_goal {
            goals_met += 1;
        }
        
        // Update bonus multiplier based on goals met
        self.weekly_goals.bonus_multiplier = match goals_met {
            0 => 1.0,
            1 => 1.25,
            2 => 1.5,
            3 => 2.0,
            _ => 1.0,
        };
    }
    
    pub fn activate_power_up(&mut self, power_up_id: &str) -> Result<(), String> {
        let power_up = self.power_ups.iter_mut()
            .find(|p| p.id == power_up_id)
            .ok_or("PowerUp not found")?;
        
        if power_up.is_active() {
            return Err("PowerUp already active".to_string());
        }
        
        power_up.activate();
        Ok(())
    }
    
    pub fn get_active_power_ups(&self) -> Vec<&PowerUp> {
        self.power_ups.iter()
            .filter(|p| p.is_active())
            .collect()
    }
    
    pub fn get_progress_to_next_level(&self) -> f32 {
        let xp_for_next = self.xp_for_next_level();
        self.experience as f32 / xp_for_next as f32
    }
}

/// Session data for updating statistics
pub struct SessionData {
    pub tasks_completed: u32,
    pub duration_minutes: u32,
    pub accuracy: f32,
    pub fastest_response_ms: u32,
}