use serde::Serializer;
use uuid::Uuid;

pub mod math;
pub mod statistics;

/// Serialize UUID as string for compatibility with the app
/// The app expects string IDs, not UUID objects
pub fn serialize_uuid_as_string<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

/// Helper to convert Option<Uuid> to Option<String>
pub fn serialize_option_uuid_as_string<S>(
    uuid: &Option<Uuid>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match uuid {
        Some(id) => serializer.serialize_str(&id.to_string()),
        None => serializer.serialize_none(),
    }
}

/// Parse UUID from string, handling both hyphenated and non-hyphenated formats
pub fn parse_uuid_flexible(s: &str) -> Result<Uuid, uuid::Error> {
    // Try parsing as-is first
    if let Ok(uuid) = Uuid::parse_str(s) {
        return Ok(uuid);
    }

    // If it's 32 characters without hyphens, add them
    if s.len() == 32 {
        let formatted = format!(
            "{}-{}-{}-{}-{}",
            &s[0..8],
            &s[8..12],
            &s[12..16],
            &s[16..20],
            &s[20..32]
        );
        Uuid::parse_str(&formatted)
    } else {
        Uuid::parse_str(s)
    }
}

/// Calculate XP based on performance
pub fn calculate_xp_reward(
    correct: bool,
    response_time_ms: i32,
    difficulty: f64,
    streak: i32,
) -> i32 {
    let mut xp = if correct {
        // Base XP for correct answer
        let base = (10.0 + difficulty * 20.0) as i32;

        // Speed bonus
        let speed_bonus = match response_time_ms {
            0..=1000 => 10,
            1001..=2000 => 5,
            2001..=3000 => 2,
            _ => 0,
        };

        base + speed_bonus
    } else {
        // Small consolation XP for trying
        2
    };

    // Streak multiplier
    if streak > 0 {
        let multiplier = 1.0 + (streak.min(10) as f64 * 0.1);
        xp = (xp as f64 * multiplier) as i32;
    }

    xp
}

/// Check if an achievement should be unlocked
pub fn check_achievement_unlock(achievement_id: &str, user_stats: &UserStats) -> bool {
    match achievement_id {
        "first_day" => user_stats.total_tasks > 0,
        "getting_started" => user_stats.total_tasks >= 10,
        "dedicated_learner" => user_stats.total_tasks >= 100,
        "task_master" => user_stats.total_tasks >= 1000,
        "legendary_scholar" => user_stats.total_tasks >= 10000,
        "week_warrior" => user_stats.current_streak >= 7,
        "month_master" => user_stats.current_streak >= 30,
        "century_streak" => user_stats.current_streak >= 100,
        "sharpshooter" => user_stats.session_accuracy >= 0.9 && user_stats.session_tasks >= 10,
        "perfectionist" => user_stats.perfect_streak >= 20,
        "flawless_victory" => user_stats.session_accuracy >= 1.0 && user_stats.session_tasks >= 50,
        "quick_thinker" => user_stats.avg_response_time_ms < 2000 && user_stats.session_tasks >= 20,
        "lightning_fast" => {
            user_stats.avg_response_time_ms < 1000 && user_stats.session_tasks >= 20
        }
        _ => false,
    }
}

#[derive(Debug)]
pub struct UserStats {
    pub total_tasks: i32,
    pub current_streak: i32,
    pub perfect_streak: i32,
    pub session_tasks: i32,
    pub session_accuracy: f64,
    pub avg_response_time_ms: i32,
}

/// Generate a random achievement based on current progress
pub fn suggest_next_achievement(user_stats: &UserStats) -> Option<String> {
    // Suggest achievable goals based on current progress
    if user_stats.total_tasks < 10 {
        Some("getting_started".to_string())
    } else if user_stats.total_tasks < 100 {
        Some("dedicated_learner".to_string())
    } else if user_stats.current_streak < 7 {
        Some("week_warrior".to_string())
    } else if user_stats.session_accuracy < 0.9 {
        Some("sharpshooter".to_string())
    } else if user_stats.avg_response_time_ms > 2000 {
        Some("quick_thinker".to_string())
    } else {
        None
    }
}
