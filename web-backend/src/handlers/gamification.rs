use axum::{
    extract::{Query, State},
    Extension, Json,
};
use chrono::{Datelike, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::Claims,
    models::gamification::*,
    state::AppState,
};

// ============= Profile Endpoints =============

/// Get user's complete gamification profile
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<GamificationProfile>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    // Get or create user gamification data
    let user_gamification =
        sqlx::query_as::<_, UserGamification>("SELECT * FROM user_gamification WHERE user_id = ?")
            .bind(user_id.as_bytes().as_slice())
            .fetch_optional(&mut *conn)
            .await?
            .unwrap_or_else(|| {
                // Create default profile if doesn't exist
                UserGamification {
                    user_id,
                    level: 1,
                    experience: 0,
                    total_points: 0,
                    current_streak: 0,
                    best_streak: 0,
                    last_activity_date: None,
                    rank_title: "Novice".to_string(),
                    rank_tier: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            });

    // Get achievements
    let achievements = sqlx::query_as::<_, Achievement>(
        "SELECT * FROM achievements WHERE user_id = ? ORDER BY unlocked DESC, points DESC",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_all(&mut *conn)
    .await?;

    // Get badges
    let badges = sqlx::query_as::<_, Badge>(
        "SELECT * FROM badges WHERE user_id = ? ORDER BY earned_at DESC",
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_all(&mut *conn)
    .await?;

    // Get current weekly goal
    let current_week = Utc::now().iso_week().week();
    let current_year = Utc::now().year();

    let weekly_goal = sqlx::query_as::<_, WeeklyGoal>(
        "SELECT * FROM weekly_goals WHERE user_id = ? AND week_number = ? AND year = ?",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(current_week as i32)
    .bind(current_year)
    .fetch_optional(&mut *conn)
    .await?;

    // Get power-ups
    let power_ups = sqlx::query_as::<_, PowerUp>(
        "SELECT * FROM power_ups WHERE user_id = ? AND (active_until IS NULL OR active_until > ?)",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(Utc::now())
    .fetch_all(&mut *conn)
    .await?;

    // Get leaderboard position
    let leaderboard_position = sqlx::query_scalar::<_, i32>(
        "SELECT rank FROM leaderboards 
         WHERE user_id = ? AND leaderboard_type = 'weekly' 
         AND week_number = ?",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(current_week as i32)
    .fetch_optional(&mut *conn)
    .await?;

    // Get user display name
    let display_name = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
        .bind(user_id.as_bytes().as_slice())
        .fetch_one(&mut *conn)
        .await?;

    let rank = Rank::from_level(user_gamification.level);
    let xp_to_next = user_gamification.xp_for_next_level();

    Ok(Json(GamificationProfile {
        user_id,
        display_name,
        avatar_url: None,
        level: user_gamification.level,
        experience: user_gamification.experience,
        experience_to_next_level: xp_to_next,
        total_points: user_gamification.total_points,
        rank,
        current_streak: user_gamification.current_streak,
        best_streak: user_gamification.best_streak,
        achievements,
        badges,
        weekly_goal,
        power_ups,
        leaderboard_position,
    }))
}

// ============= XP and Level Endpoints =============

/// Add XP to user (usually called after task completion)
pub async fn add_xp(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AddXpRequest>,
) -> AppResult<Json<XpResponse>> {
    let user_id = claims.sub;
    let conn = state.db_pool.acquire().await?;

    // Start transaction
    let mut tx = state.db_pool.begin().await?;

    // Get current user gamification data
    let mut user_gamification =
        sqlx::query_as::<_, UserGamification>("SELECT * FROM user_gamification WHERE user_id = ?")
            .bind(user_id.as_bytes().as_slice())
            .fetch_optional(&mut *tx)
            .await?
            .unwrap_or_else(|| UserGamification {
                user_id,
                level: 1,
                experience: 0,
                total_points: 0,
                current_streak: 0,
                best_streak: 0,
                last_activity_date: None,
                rank_title: "Novice".to_string(),
                rank_tier: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

    // Calculate actual XP with multiplier
    let multiplier = req.multiplier.unwrap_or(1.0);
    let xp_gained = (req.amount as f32 * multiplier) as i32;

    // Update XP and check for level up
    let old_level = user_gamification.level;
    user_gamification.experience += xp_gained;
    user_gamification.total_points += xp_gained;
    user_gamification.level =
        UserGamification::calculate_level_from_xp(user_gamification.experience);

    let level_up = user_gamification.level > old_level;

    // Update rank if leveled up
    if level_up {
        let new_rank = Rank::from_level(user_gamification.level);
        user_gamification.rank_title = new_rank.title;
        user_gamification.rank_tier = new_rank.tier;
    }

    // Save updated gamification data
    sqlx::query(
        "INSERT INTO user_gamification (user_id, level, experience, total_points, current_streak, 
         best_streak, last_activity_date, rank_title, rank_tier, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET
         level = excluded.level,
         experience = excluded.experience,
         total_points = excluded.total_points,
         rank_title = excluded.rank_title,
         rank_tier = excluded.rank_tier,
         updated_at = excluded.updated_at",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(user_gamification.level)
    .bind(user_gamification.experience)
    .bind(user_gamification.total_points)
    .bind(user_gamification.current_streak)
    .bind(user_gamification.best_streak)
    .bind(user_gamification.last_activity_date)
    .bind(&user_gamification.rank_title)
    .bind(user_gamification.rank_tier)
    .bind(user_gamification.created_at)
    .bind(Utc::now())
    .execute(&mut *tx)
    .await?;

    // Log XP transaction
    let transaction_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO xp_transactions (id, user_id, amount, reason, source_id, multiplier, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(transaction_id.as_bytes().as_slice())
    .bind(user_id.as_bytes().as_slice())
    .bind(xp_gained)
    .bind(&req.reason)
    .bind(req.source_id.as_deref())
    .bind(multiplier)
    .bind(Utc::now())
    .execute(&mut *tx)
    .await?;

    // Check for new achievements
    let mut new_achievements = Vec::new();

    if level_up {
        // Check level-based achievements
        let level_achievements = vec![
            (10, "level_10"),
            (25, "level_25"),
            (50, "level_50"),
            (100, "level_100"),
        ];

        for (level, achievement_id) in level_achievements {
            if user_gamification.level >= level && old_level < level {
                new_achievements.push(achievement_id.to_string());
                unlock_achievement_internal(&mut tx, user_id, achievement_id).await?;
            }
        }
    }

    // Commit transaction
    tx.commit().await?;

    Ok(Json(XpResponse {
        xp_gained,
        total_xp: user_gamification.experience,
        level: user_gamification.level,
        level_up,
        new_achievements,
    }))
}

// ============= Achievement Endpoints =============

/// Get all achievements (both locked and unlocked)
pub async fn get_achievements(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<Achievement>>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    let achievements = sqlx::query_as::<_, Achievement>(
        "SELECT * FROM achievements WHERE user_id = ? ORDER BY unlocked DESC, category, points DESC"
    )
    .bind(user_id.as_bytes().as_slice())
    .fetch_all(&mut *conn)
    .await?;

    // If no achievements exist, create default ones
    if achievements.is_empty() {
        create_default_achievements_for_user(&state.db_pool, user_id).await?;

        // Fetch again
        let achievements = sqlx::query_as::<_, Achievement>(
            "SELECT * FROM achievements WHERE user_id = ? ORDER BY unlocked DESC, category, points DESC"
        )
        .bind(user_id.as_bytes().as_slice())
        .fetch_all(&mut *conn)
        .await?;

        return Ok(Json(achievements));
    }

    Ok(Json(achievements))
}

/// Unlock or update progress on an achievement
pub async fn unlock_achievement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UnlockAchievementRequest>,
) -> AppResult<Json<Achievement>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    // Update achievement
    let now = Utc::now();
    let unlocked = req.progress.unwrap_or(1.0) >= 1.0;

    sqlx::query(
        "UPDATE achievements 
         SET progress = ?, unlocked = ?, unlocked_at = ?
         WHERE user_id = ? AND achievement_id = ?",
    )
    .bind(req.progress.unwrap_or(1.0))
    .bind(unlocked)
    .bind(if unlocked { Some(now) } else { None })
    .bind(user_id.as_bytes().as_slice())
    .bind(&req.achievement_id)
    .execute(&mut *conn)
    .await?;

    // Fetch updated achievement
    let achievement = sqlx::query_as::<_, Achievement>(
        "SELECT * FROM achievements WHERE user_id = ? AND achievement_id = ?",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(&req.achievement_id)
    .fetch_one(&mut *conn)
    .await?;

    // If unlocked, add XP bonus
    if unlocked && req.progress.unwrap_or(0.0) < 1.0 {
        let xp_bonus = (achievement.points as f32
            * Rarity::from(achievement.rarity.parse().unwrap_or(Rarity::Common))
                .points_multiplier()) as i32;

        // Add XP through the XP endpoint logic
        let _xp_result = add_xp(
            State(state.clone()),
            Extension(claims),
            Json(AddXpRequest {
                amount: xp_bonus,
                reason: format!("Achievement unlocked: {}", achievement.name),
                source_id: Some(achievement.achievement_id.clone()),
                multiplier: Some(1.0),
            }),
        )
        .await?;
    }

    Ok(Json(achievement))
}

// ============= Leaderboard Endpoints =============

/// Get leaderboard entries
pub async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<LeaderboardRequest>,
) -> AppResult<Json<LeaderboardResponse>> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Build query based on leaderboard type
    let (query, week_param) = match params.leaderboard_type.as_str() {
        "weekly" => {
            let current_week = Utc::now().iso_week().week() as i32;
            (
                "SELECT l.*, u.username 
                 FROM leaderboards l
                 JOIN users u ON l.user_id = u.id
                 WHERE l.leaderboard_type = ? AND l.week_number = ?
                 ORDER BY l.score DESC
                 LIMIT ? OFFSET ?",
                Some(current_week),
            )
        }
        _ => (
            "SELECT l.*, u.username 
                 FROM leaderboards l
                 JOIN users u ON l.user_id = u.id
                 WHERE l.leaderboard_type = ?
                 ORDER BY l.score DESC
                 LIMIT ? OFFSET ?",
            None,
        ),
    };

    let entries = if let Some(week) = week_param {
        sqlx::query_as::<_, LeaderboardEntry>(query)
            .bind(&params.leaderboard_type)
            .bind(week)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *conn)
            .await?
    } else {
        sqlx::query_as::<_, LeaderboardEntry>(query)
            .bind(&params.leaderboard_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *conn)
            .await?
    };

    // Get user's rank
    let user_rank = sqlx::query_scalar::<_, i32>(
        "SELECT rank FROM leaderboards WHERE user_id = ? AND leaderboard_type = ?",
    )
    .bind(user_id.as_bytes().as_slice())
    .bind(&params.leaderboard_type)
    .fetch_optional(&mut *conn)
    .await?;

    // Get total participants
    let total_participants = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM leaderboards WHERE leaderboard_type = ?",
    )
    .bind(&params.leaderboard_type)
    .fetch_one(&mut *conn)
    .await? as i32;

    Ok(Json(LeaderboardResponse {
        entries,
        user_rank,
        total_participants,
    }))
}

/// Update leaderboard score
#[derive(Deserialize)]
pub struct UpdateScoreRequest {
    score_delta: i32,
    leaderboard_type: String,
}

pub async fn update_leaderboard_score(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateScoreRequest>,
) -> AppResult<()> {
    let user_id = claims.sub;
    let mut conn = state.db_pool.acquire().await?;

    let current_week = Utc::now().iso_week().week() as i32;
    let current_year = Utc::now().year();
    let month_year = format!("{}-{:02}", current_year, Utc::now().month());

    let leaderboard_id = Uuid::new_v4();

    // Upsert leaderboard entry
    sqlx::query(
        "INSERT INTO leaderboards (id, user_id, leaderboard_type, score, week_number, month_year, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id, leaderboard_type, week_number, month_year, domain) DO UPDATE SET
         score = leaderboards.score + excluded.score,
         updated_at = excluded.updated_at"
    )
    .bind(leaderboard_id.as_bytes().as_slice())
    .bind(user_id.as_bytes().as_slice())
    .bind(&req.leaderboard_type)
    .bind(req.score_delta)
    .bind(if req.leaderboard_type == "weekly" { Some(current_week) } else { None })
    .bind(if req.leaderboard_type == "monthly" { Some(month_year) } else { None })
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&mut *conn)
    .await?;

    // Update ranks (simplified - in production, use a more efficient ranking query)
    sqlx::query(
        "UPDATE leaderboards 
         SET rank = (
            SELECT COUNT(*) + 1 
            FROM leaderboards l2 
            WHERE l2.leaderboard_type = leaderboards.leaderboard_type
            AND l2.score > leaderboards.score
         )
         WHERE leaderboard_type = ?",
    )
    .bind(&req.leaderboard_type)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

// ============= Helper Functions =============

async fn unlock_achievement_internal(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user_id: Uuid,
    achievement_id: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE achievements 
         SET unlocked = true, unlocked_at = ?, progress = 1.0
         WHERE user_id = ? AND achievement_id = ? AND NOT unlocked",
    )
    .bind(Utc::now())
    .bind(user_id.as_bytes().as_slice())
    .bind(achievement_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn create_default_achievements_for_user(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> AppResult<()> {
    let mut conn = pool.acquire().await?;

    for (id, name, desc, category, rarity, points) in get_default_achievements() {
        let achievement_id = Uuid::new_v4();
        sqlx::query(
            "INSERT OR IGNORE INTO achievements 
             (id, user_id, achievement_id, name, description, category, rarity, points, 
              unlocked, progress, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, false, 0.0, ?)",
        )
        .bind(achievement_id.as_bytes().as_slice())
        .bind(user_id.as_bytes().as_slice())
        .bind(id)
        .bind(name)
        .bind(desc)
        .bind(format!("{:?}", category))
        .bind(format!("{:?}", rarity))
        .bind(points)
        .bind(Utc::now())
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}
