use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;

// ============= Core Gamification Types =============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AchievementCategory {
    Streak,
    Accuracy,
    Speed,
    Volume,
    Mastery,
    Explorer,
    Social,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl FromStr for Rarity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "common" => Ok(Rarity::Common),
            "uncommon" => Ok(Rarity::Uncommon),
            "rare" => Ok(Rarity::Rare),
            "epic" => Ok(Rarity::Epic),
            "legendary" => Ok(Rarity::Legendary),
            _ => Err(format!("Unknown rarity: {}", s)),
        }
    }
}

impl Rarity {
    pub fn points_multiplier(&self) -> f32 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 1.5,
            Rarity::Rare => 2.0,
            Rarity::Epic => 3.0,
            Rarity::Legendary => 5.0,
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Rarity::Common => "#808080",
            Rarity::Uncommon => "#48bb78",
            Rarity::Rare => "#4299e1",
            Rarity::Epic => "#9f7aea",
            Rarity::Legendary => "#f6ad55",
        }
    }
}

// ============= Database Models =============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGamification {
    pub user_id: Uuid,
    pub level: i32,
    pub experience: i32,
    pub total_points: i32,
    pub current_streak: i32,
    pub best_streak: i32,
    pub last_activity_date: Option<NaiveDate>,
    pub rank_title: String,
    pub rank_tier: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub achievement_id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub rarity: String,
    pub points: i32,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub progress: f32,
    pub requirement_data: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaderboardEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>, // Joined from users table
    pub leaderboard_type: String,
    pub score: i32,
    pub rank: Option<i32>,
    pub week_number: Option<i32>,
    pub month_year: Option<String>,
    pub domain: Option<String>,
    pub metadata: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Badge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub badge_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub earned_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WeeklyGoal {
    pub id: Uuid,
    pub user_id: Uuid,
    pub week_number: i32,
    pub year: i32,
    pub tasks_goal: i32,
    pub tasks_completed: i32,
    pub accuracy_goal: f32,
    pub current_accuracy: f32,
    pub streak_goal: i32,
    pub current_streak: i32,
    pub completed: bool,
    pub reward_claimed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PowerUp {
    pub id: Uuid,
    pub user_id: Uuid,
    pub power_up_type: String,
    pub quantity: i32,
    pub active_until: Option<DateTime<Utc>>,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct XpTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: i32,
    pub reason: String,
    pub source_id: Option<String>,
    pub multiplier: f32,
    pub created_at: DateTime<Utc>,
}

// ============= API Request/Response Types =============

#[derive(Debug, Deserialize)]
pub struct UnlockAchievementRequest {
    pub achievement_id: String,
    pub progress: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct GamificationProfile {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub level: i32,
    pub experience: i32,
    pub experience_to_next_level: i32,
    pub total_points: i32,
    pub rank: Rank,
    pub current_streak: i32,
    pub best_streak: i32,
    pub achievements: Vec<Achievement>,
    pub badges: Vec<Badge>,
    pub weekly_goal: Option<WeeklyGoal>,
    pub power_ups: Vec<PowerUp>,
    pub leaderboard_position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rank {
    pub title: String,
    pub tier: i32,
    pub icon: String,
    pub color: String,
    pub min_level: i32,
    pub max_level: i32,
}

impl Rank {
    pub fn from_level(level: i32) -> Self {
        let (title, icon, color, min, max) = match level {
            0..=9 => ("Novice", "🌱", "#48bb78", 0, 9),
            10..=24 => ("Apprentice", "📚", "#4299e1", 10, 24),
            25..=49 => ("Scholar", "🎓", "#9f7aea", 25, 49),
            50..=74 => ("Expert", "⭐", "#ed8936", 50, 74),
            75..=99 => ("Master", "🏆", "#f6ad55", 75, 99),
            100..=149 => ("Grandmaster", "👑", "#e53e3e", 100, 149),
            150..=199 => ("Legend", "✨", "#d69e2e", 150, 199),
            _ => ("Mythic", "🔥", "#9b2c2c", 200, 999),
        };

        let tier = ((level - min) / ((max - min + 1) / 10).max(1)) + 1;

        Rank {
            title: title.to_string(),
            tier: tier.min(10),
            icon: icon.to_string(),
            color: color.to_string(),
            min_level: min,
            max_level: max,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddXpRequest {
    pub amount: i32,
    pub reason: String,
    pub source_id: Option<String>,
    pub multiplier: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct XpResponse {
    pub xp_gained: i32,
    pub total_xp: i32,
    pub level: i32,
    pub level_up: bool,
    pub new_achievements: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardRequest {
    pub leaderboard_type: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub domain: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub user_rank: Option<i32>,
    pub total_participants: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStreakRequest {
    pub increment: bool,
}

#[derive(Debug, Serialize)]
pub struct StreakResponse {
    pub current_streak: i32,
    pub best_streak: i32,
    pub streak_bonus_xp: i32,
    pub new_achievement: Option<String>,
}

// ============= Helper Functions =============

impl UserGamification {
    pub fn calculate_level_from_xp(xp: i32) -> i32 {
        // Level progression: 100 XP for level 1, 150 for level 2, etc.
        let mut level = 1;
        let mut required_xp = 100;
        let mut total_required = 0;

        while total_required + required_xp <= xp {
            total_required += required_xp;
            level += 1;
            required_xp = 100 + (level - 1) * 50;
        }

        level
    }

    pub fn xp_for_next_level(&self) -> i32 {
        100 + self.level * 50
    }

    pub fn xp_progress_percentage(&self) -> f32 {
        let xp_for_current = if self.level == 1 {
            0
        } else {
            (1..self.level).map(|l| 100 + (l - 1) * 50).sum()
        };

        let xp_in_current_level = self.experience - xp_for_current;
        let xp_needed = self.xp_for_next_level();

        (xp_in_current_level as f32 / xp_needed as f32) * 100.0
    }
}

// Default achievements that should be created for new users
pub fn get_default_achievements() -> Vec<(String, String, String, AchievementCategory, Rarity, i32)>
{
    vec![
        // Streak achievements
        (
            "first_day".to_string(),
            "First Day".to_string(),
            "Complete your first day of practice".to_string(),
            AchievementCategory::Streak,
            Rarity::Common,
            10,
        ),
        (
            "week_warrior".to_string(),
            "Week Warrior".to_string(),
            "Maintain a 7-day streak".to_string(),
            AchievementCategory::Streak,
            Rarity::Uncommon,
            50,
        ),
        (
            "month_master".to_string(),
            "Month Master".to_string(),
            "Maintain a 30-day streak".to_string(),
            AchievementCategory::Streak,
            Rarity::Rare,
            200,
        ),
        (
            "century_streak".to_string(),
            "Century Streak".to_string(),
            "Maintain a 100-day streak".to_string(),
            AchievementCategory::Streak,
            Rarity::Epic,
            1000,
        ),
        // Accuracy achievements
        (
            "sharpshooter".to_string(),
            "Sharpshooter".to_string(),
            "Achieve 90% accuracy in a session".to_string(),
            AchievementCategory::Accuracy,
            Rarity::Common,
            20,
        ),
        (
            "perfectionist".to_string(),
            "Perfectionist".to_string(),
            "Complete 20 tasks without a mistake".to_string(),
            AchievementCategory::Accuracy,
            Rarity::Uncommon,
            75,
        ),
        (
            "flawless_victory".to_string(),
            "Flawless Victory".to_string(),
            "100% accuracy with 50+ tasks".to_string(),
            AchievementCategory::Accuracy,
            Rarity::Rare,
            300,
        ),
        // Volume achievements
        (
            "getting_started".to_string(),
            "Getting Started".to_string(),
            "Complete 10 tasks".to_string(),
            AchievementCategory::Volume,
            Rarity::Common,
            10,
        ),
        (
            "dedicated_learner".to_string(),
            "Dedicated Learner".to_string(),
            "Complete 100 tasks".to_string(),
            AchievementCategory::Volume,
            Rarity::Uncommon,
            50,
        ),
        (
            "task_master".to_string(),
            "Task Master".to_string(),
            "Complete 1000 tasks".to_string(),
            AchievementCategory::Volume,
            Rarity::Rare,
            250,
        ),
        (
            "legendary_scholar".to_string(),
            "Legendary Scholar".to_string(),
            "Complete 10000 tasks".to_string(),
            AchievementCategory::Volume,
            Rarity::Legendary,
            5000,
        ),
        // Speed achievements
        (
            "quick_thinker".to_string(),
            "Quick Thinker".to_string(),
            "Average response time under 2 seconds".to_string(),
            AchievementCategory::Speed,
            Rarity::Common,
            30,
        ),
        (
            "lightning_fast".to_string(),
            "Lightning Fast".to_string(),
            "Average response time under 1 second".to_string(),
            AchievementCategory::Speed,
            Rarity::Rare,
            150,
        ),
    ]
}
