// Leaderboard system
use serde::{Deserialize, Serialize};
use super::profile::GamificationProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub score: u32,
    pub level: u32,
    pub achievement_count: u32,
    pub trend: Trend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Up(u32),    // Positions gained
    Down(u32),  // Positions lost
    Stable,
}

impl Trend {
    pub fn display(&self) -> String {
        match self {
            Trend::Up(n) => format!("↑ {}", n),
            Trend::Down(n) => format!("↓ {}", n),
            Trend::Stable => "→".to_string(),
        }
    }
    
    pub fn color(&self) -> &str {
        match self {
            Trend::Up(_) => "#48bb78",    // Green
            Trend::Down(_) => "#f56565",  // Red
            Trend::Stable => "#a0aec0",   // Gray
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LeaderboardType {
    Global,
    Weekly,
    Monthly,
    Friends,
    Domain(String),
}

impl LeaderboardType {
    pub fn display_name(&self) -> String {
        match self {
            LeaderboardType::Global => "Global Rankings".to_string(),
            LeaderboardType::Weekly => "This Week".to_string(),
            LeaderboardType::Monthly => "This Month".to_string(),
            LeaderboardType::Friends => "Friends".to_string(),
            LeaderboardType::Domain(domain) => format!("{} Masters", domain),
        }
    }
    
    pub fn icon(&self) -> &str {
        match self {
            LeaderboardType::Global => "🌍",
            LeaderboardType::Weekly => "📅",
            LeaderboardType::Monthly => "📆",
            LeaderboardType::Friends => "👥",
            LeaderboardType::Domain(_) => "🎯",
        }
    }
}

impl LeaderboardEntry {
    pub fn from_profile(profile: &GamificationProfile) -> Self {
        Self {
            rank: 0, // Will be set when sorting
            user_id: profile.user_id.clone(),
            display_name: profile.display_name.clone(),
            avatar_url: profile.avatar_url.clone(),
            score: profile.total_points,
            level: profile.level,
            achievement_count: profile.achievements.len() as u32,
            trend: Trend::Stable,
        }
    }
    
    pub fn display_rank(&self) -> String {
        match self.rank {
            1 => "🥇".to_string(),
            2 => "🥈".to_string(),
            3 => "🥉".to_string(),
            _ => format!("#{}", self.rank),
        }
    }
    
    pub fn is_top_three(&self) -> bool {
        self.rank <= 3
    }
}