use serde::Serializer;
use uuid::Uuid;

pub mod math;
pub mod statistics;

// XP calculation constants
const BASE_XP: i32 = 10;
const DIFFICULTY_MULTIPLIER: f64 = 20.0;
const CONSOLATION_XP: i32 = 2;
const MAX_SPEED_BONUS: i32 = 10;
const MEDIUM_SPEED_BONUS: i32 = 5;
const SMALL_SPEED_BONUS: i32 = 2;
const SPEED_THRESHOLD_FAST: i32 = 1000;
const SPEED_THRESHOLD_MEDIUM: i32 = 2000;
const SPEED_THRESHOLD_SLOW: i32 = 3000;
const STREAK_BONUS_RATE: f64 = 0.1;
const MAX_STREAK_BONUS: i32 = 10;

// Achievement thresholds
const ACHIEVEMENT_GETTING_STARTED_TASKS: i32 = 10;
const ACHIEVEMENT_DEDICATED_LEARNER_TASKS: i32 = 100;
const ACHIEVEMENT_TASK_MASTER_TASKS: i32 = 1000;
const ACHIEVEMENT_LEGENDARY_SCHOLAR_TASKS: i32 = 10000;
const ACHIEVEMENT_WEEK_WARRIOR_STREAK: i32 = 7;
const ACHIEVEMENT_MONTH_MASTER_STREAK: i32 = 30;
const ACHIEVEMENT_CENTURY_STREAK: i32 = 100;
const ACHIEVEMENT_PERFECT_STREAK: i32 = 20;
const ACHIEVEMENT_SHARPSHOOTER_ACCURACY: f64 = 0.9;
const ACHIEVEMENT_SHARPSHOOTER_MIN_TASKS: i32 = 10;
const ACHIEVEMENT_FLAWLESS_ACCURACY: f64 = 1.0;
const ACHIEVEMENT_FLAWLESS_MIN_TASKS: i32 = 50;
const ACHIEVEMENT_QUICK_THINKER_TIME: i32 = 2000;
const ACHIEVEMENT_QUICK_THINKER_MIN_TASKS: i32 = 20;
const ACHIEVEMENT_LIGHTNING_FAST_TIME: i32 = 1000;

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

/// Calculate XP based on performance with input validation
pub fn calculate_xp_reward(
    correct: bool,
    response_time_ms: i32,
    difficulty: f64,
    streak: i32,
) -> i32 {
    // Validate inputs
    let response_time_ms = response_time_ms.max(0); // Ensure non-negative
    let difficulty = difficulty.clamp(0.0, 10.0); // Reasonable difficulty range
    let streak = streak.max(0); // Ensure non-negative

    let mut xp = if correct {
        // Base XP for correct answer
        let base = (BASE_XP as f64 + difficulty * DIFFICULTY_MULTIPLIER) as i32;

        // Speed bonus based on response time
        let speed_bonus = match response_time_ms {
            0..=SPEED_THRESHOLD_FAST => MAX_SPEED_BONUS,
            _ if response_time_ms <= SPEED_THRESHOLD_MEDIUM => MEDIUM_SPEED_BONUS,
            _ if response_time_ms <= SPEED_THRESHOLD_SLOW => SMALL_SPEED_BONUS,
            _ => 0,
        };

        base + speed_bonus
    } else {
        // Small consolation XP for trying
        CONSOLATION_XP
    };

    // Apply streak multiplier
    if streak > 0 {
        let multiplier = 1.0 + (streak.min(MAX_STREAK_BONUS) as f64 * STREAK_BONUS_RATE);
        xp = (xp as f64 * multiplier) as i32;
    }

    xp.max(1) // Ensure at least 1 XP
}

/// Check if an achievement should be unlocked
pub fn check_achievement_unlock(achievement_id: &str, user_stats: &UserStats) -> bool {
    match achievement_id {
        "first_day" => user_stats.total_tasks > 0,
        "getting_started" => user_stats.total_tasks >= ACHIEVEMENT_GETTING_STARTED_TASKS,
        "dedicated_learner" => user_stats.total_tasks >= ACHIEVEMENT_DEDICATED_LEARNER_TASKS,
        "task_master" => user_stats.total_tasks >= ACHIEVEMENT_TASK_MASTER_TASKS,
        "legendary_scholar" => user_stats.total_tasks >= ACHIEVEMENT_LEGENDARY_SCHOLAR_TASKS,
        "week_warrior" => user_stats.current_streak >= ACHIEVEMENT_WEEK_WARRIOR_STREAK,
        "month_master" => user_stats.current_streak >= ACHIEVEMENT_MONTH_MASTER_STREAK,
        "century_streak" => user_stats.current_streak >= ACHIEVEMENT_CENTURY_STREAK,
        "perfectionist" => user_stats.perfect_streak >= ACHIEVEMENT_PERFECT_STREAK,
        "sharpshooter" => {
            user_stats.session_accuracy >= ACHIEVEMENT_SHARPSHOOTER_ACCURACY
                && user_stats.session_tasks >= ACHIEVEMENT_SHARPSHOOTER_MIN_TASKS
        }
        "flawless_victory" => {
            user_stats.session_accuracy >= ACHIEVEMENT_FLAWLESS_ACCURACY
                && user_stats.session_tasks >= ACHIEVEMENT_FLAWLESS_MIN_TASKS
        }
        "quick_thinker" => {
            user_stats.avg_response_time_ms < ACHIEVEMENT_QUICK_THINKER_TIME
                && user_stats.session_tasks >= ACHIEVEMENT_QUICK_THINKER_MIN_TASKS
        }
        "lightning_fast" => {
            user_stats.avg_response_time_ms < ACHIEVEMENT_LIGHTNING_FAST_TIME
                && user_stats.session_tasks >= ACHIEVEMENT_QUICK_THINKER_MIN_TASKS
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

/// Suggest next achievement based on current progress  
pub fn suggest_next_achievement(user_stats: &UserStats) -> Option<String> {
    // Suggest achievable goals based on current progress
    if user_stats.total_tasks < ACHIEVEMENT_GETTING_STARTED_TASKS {
        Some("getting_started".to_string())
    } else if user_stats.total_tasks < ACHIEVEMENT_DEDICATED_LEARNER_TASKS {
        Some("dedicated_learner".to_string())
    } else if user_stats.current_streak < ACHIEVEMENT_WEEK_WARRIOR_STREAK {
        Some("week_warrior".to_string())
    } else if user_stats.session_accuracy < ACHIEVEMENT_SHARPSHOOTER_ACCURACY
        && user_stats.session_tasks >= ACHIEVEMENT_SHARPSHOOTER_MIN_TASKS
    {
        Some("sharpshooter".to_string())
    } else if user_stats.avg_response_time_ms > ACHIEVEMENT_QUICK_THINKER_TIME
        && user_stats.session_tasks >= ACHIEVEMENT_QUICK_THINKER_MIN_TASKS
    {
        Some("quick_thinker".to_string())
    } else {
        None
    }
}
