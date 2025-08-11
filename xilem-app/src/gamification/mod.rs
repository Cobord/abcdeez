// Gamification system for ABCDEEZ
pub mod achievements;
pub mod leaderboard;
pub mod profile;
pub mod powerups;
pub mod streaks;

pub use achievements::{Achievement, AchievementCategory, AchievementEvent, AchievementManager, Rarity};
pub use leaderboard::{LeaderboardEntry, LeaderboardType, Trend};
pub use profile::{GamificationProfile, Rank, UserStatistics, WeeklyGoals};
pub use powerups::{PowerUp, PowerUpEffect};
pub use streaks::StreakInfo;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{DateTime, Utc};

/// Main gamification controller
pub struct GamificationManager {
    pub achievement_manager: AchievementManager,
    pub user_profiles: HashMap<String, Rc<RefCell<GamificationProfile>>>,
    pub leaderboards: HashMap<LeaderboardType, Vec<LeaderboardEntry>>,
}

impl GamificationManager {
    pub fn new() -> Self {
        Self {
            achievement_manager: AchievementManager::new(),
            user_profiles: HashMap::new(),
            leaderboards: HashMap::new(),
        }
    }
    
    pub fn get_or_create_profile(&mut self, user_id: &str) -> &RefCell<GamificationProfile> {
        self.user_profiles.entry(user_id.to_string())
            .or_insert_with(|| Rc::new(RefCell::new(GamificationProfile::new(user_id))))
    }
    
    pub fn check_achievements(&mut self, user_id: &str, event: &AchievementEvent) -> Vec<Achievement> {
        let profile = self.get_or_create_profile(user_id).clone();
        let mut profile = profile.borrow_mut();
        self.achievement_manager.check_achievements(&mut *profile, event)
    }
    
    pub fn update_leaderboard(&mut self, leaderboard_type: LeaderboardType) {
        let mut entries: Vec<LeaderboardEntry> = self.user_profiles.values()
            .map(|profile| LeaderboardEntry::from_profile(&profile.borrow()))
            .collect();
        
        // Sort by score descending
        entries.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Assign ranks and detect trends
        let old_leaderboard = self.leaderboards.get(&leaderboard_type);
        for (i, entry) in entries.iter_mut().enumerate() {
            let new_rank = (i + 1) as u32;
            
            // Calculate trend based on previous position
            if let Some(old_board) = old_leaderboard {
                if let Some(old_entry) = old_board.iter().find(|e| e.user_id == entry.user_id) {
                    entry.trend = match new_rank.cmp(&old_entry.rank) {
                        std::cmp::Ordering::Less => Trend::Up(old_entry.rank - new_rank),
                        std::cmp::Ordering::Greater => Trend::Down(new_rank - old_entry.rank),
                        std::cmp::Ordering::Equal => Trend::Stable,
                    };
                }
            }
            
            entry.rank = new_rank;
        }
        
        self.leaderboards.insert(leaderboard_type, entries);
    }
    
    pub fn get_leaderboard(&self, leaderboard_type: &LeaderboardType, limit: usize) -> Vec<LeaderboardEntry> {
        self.leaderboards.get(leaderboard_type)
            .map(|entries| entries.iter().take(limit).cloned().collect())
            .unwrap_or_default()
    }
    
    pub fn award_experience(&mut self, user_id: &str, xp: u32) {
        let profile = self.get_or_create_profile(user_id);
        let mut profile = profile.borrow_mut();
        profile.add_experience(xp);
    }
    
    pub fn activate_power_up(&mut self, user_id: &str, power_up_id: &str) -> Result<(), String> {
        let profile = self.get_or_create_profile(user_id);
        let mut profile = profile.borrow_mut();
        profile.activate_power_up(power_up_id)
    }
}

impl Default for GamificationManager {
    fn default() -> Self {
        Self::new()
    }
}