// Enhanced dashboard view with live metrics, gamification, and analytics

use crate::state::{AppState, Screen};
use crate::components::{Components, AppComponents, AppColor, ComponentOutput, responsive, SpacerSize};
use crate::components::layout::{dashboard_grid, page_layout};
use crate::components::cards::{stat_card_with_trend, Trend, metric_card, achievement_card};
use chrono::Utc;

pub fn dashboard_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    // Build the enhanced dashboard with live metrics
    let username = state.user.as_ref()
        .map(|u| u.username.as_str())
        .unwrap_or("Guest");
    
    let header = Components::header_bar(
        "ABCDEEZ Dashboard",
        |state| {
            state.navigate(Screen::Settings);
        }
    );
    
    // Gamification profile info
    let profile = state.get_current_user_profile();
    let (level, xp_progress, rank) = if let Some(profile) = profile {
        (
            format!("Level {}", profile.level),
            format!("{}% to next level", (profile.get_progress_to_next_level() * 100.0) as u32),
            profile.rank.title.clone()
        )
    } else {
        ("Level 1".to_string(), "0% to next level".to_string(), "Novice".to_string())
    };
    
    // Enhanced welcome card with rank and level
    let welcome_content = format!("{} • {}", rank, level);
    let welcome = Components::welcome_card(
        username,
        &welcome_content,
        |state| {
            state.navigate(Screen::Learning);
        }
    );
    
    // Live session metrics
    let session_metrics = if let Some(session) = &state.session {
        let duration = Utc::now().signed_duration_since(session.start_time);
        let minutes = duration.num_minutes();
        let accuracy = if session.responses_total > 0 {
            (session.responses_correct as f32 / session.responses_total as f32 * 100.0) as u32
        } else {
            0
        };
        
        vec![
            Components::stat_card(&format!("{}m", minutes), "Session Time", AppColor::Primary),
            Components::stat_card(&format!("{}%", accuracy), "Accuracy", get_accuracy_color(accuracy)),
            Components::stat_card(&format!("{}", session.responses_total), "Tasks", AppColor::Info),
        ]
    } else {
        // Default metrics when no active session
        vec![
            Components::stat_card("--", "Session Time", AppColor::Secondary),
            Components::stat_card("--", "Accuracy", AppColor::Secondary),
            Components::stat_card("--", "Tasks", AppColor::Secondary),
        ]
    };
    
    // Gamification stats
    let gamification_stats = if let Some(profile) = profile {
        vec![
            Components::stat_card(
                &profile.streak.display_status(),
                "Streak",
                if profile.streak.is_at_risk() { AppColor::Error } else { AppColor::Success }
            ),
            Components::stat_card(
                &format!("{}", profile.achievements.len()),
                "Achievements",
                AppColor::Warning
            ),
            Components::stat_card(
                &xp_progress,
                "Progress",
                AppColor::Info
            ),
        ]
    } else {
        vec![
            Components::stat_card("No streak", "Streak", AppColor::Secondary),
            Components::stat_card("0", "Achievements", AppColor::Secondary),
            Components::stat_card("0%", "Progress", AppColor::Secondary),
        ]
    };
    
    // Cognitive metrics (if available from session)
    let cognitive_metrics = if let Some(session) = &state.session {
        let avg_response_time = if !state.response_times.is_empty() {
            let sum: u128 = state.response_times.iter().sum();
            sum / state.response_times.len() as u128
        } else {
            0
        };
        
        vec![
            Components::stat_card(
                &format!("{}ms", avg_response_time),
                "Avg Response",
                AppColor::Primary
            ),
            Components::stat_card(
                &get_performance_rating(session),
                "Performance",
                AppColor::Success
            ),
            Components::stat_card(
                &format!("{}", session.total_hints_used),
                "Hints Used",
                AppColor::Info
            ),
        ]
    } else {
        vec![]
    };
    
    // Weekly goals progress
    let weekly_goals = if let Some(profile) = profile {
        let goals = &profile.weekly_goals;
        vec![
            Components::progress_card(
                "Weekly Tasks",
                goals.tasks_completed as f32 / goals.tasks_goal as f32,
                &format!("{}/{}", goals.tasks_completed, goals.tasks_goal),
                AppColor::Primary
            ),
            Components::progress_card(
                "Accuracy Goal",
                goals.current_accuracy / goals.accuracy_goal,
                &format!("{:.0}%/{:.0}%", goals.current_accuracy * 100.0, goals.accuracy_goal * 100.0),
                AppColor::Success
            ),
            Components::progress_card(
                "Streak Goal",
                goals.current_streak as f32 / goals.streak_goal as f32,
                &format!("{}/{} days", goals.current_streak, goals.streak_goal),
                AppColor::Warning
            ),
        ]
    } else {
        vec![]
    };
    
    // Recent achievements
    let recent_achievements = if let Some(profile) = profile {
        let mut achievements = profile.achievements.clone();
        achievements.sort_by(|a, b| b.unlocked_at.cmp(&a.unlocked_at));
        achievements.truncate(3);
        
        achievements.into_iter().map(|achievement| {
            let time_ago = format_time_ago(achievement.unlocked_at.unwrap_or(Utc::now()));
            Components::activity_card(
                &format!("🏆 {}", achievement.name),
                &time_ago,
                false
            )
        }).collect()
    } else {
        vec![]
    };
    
    // Active power-ups
    let active_powerups = if let Some(profile) = profile {
        profile.get_active_power_ups().into_iter().map(|powerup| {
            Components::activity_card(
                &format!("{} {}", powerup.icon, powerup.name),
                &powerup.display_time_remaining(),
                true
            )
        }).collect()
    } else {
        vec![]
    };
    
    // Build responsive grid layout for main stats only
    let stats_grid = dashboard_grid(session_metrics, window_width);
    
    // Quick actions with new features
    let quick_actions = Components::settings_section(
        "Quick Actions",
        vec![
            Components::action_button("Start Training", AppColor::Primary, |state| {
                state.navigate(Screen::Training);
            }),
            Components::action_button("View Progress", AppColor::Success, |state| {
                state.navigate(Screen::Progress);
            }),
            Components::action_button("Leaderboard", AppColor::Warning, |state| {
                state.navigate(Screen::Leaderboard);
            }),
            Components::action_button("Challenges", AppColor::Info, |state| {
                state.navigate(Screen::Challenges);
            }),
        ]
    );
    
    let bottom_nav = Components::bottom_nav_bar(
        Screen::Dashboard,
        |state, screen| {
            state.navigate(screen);
        }
    );
    
    // Build main content with responsive layout
    let mut content_items = vec![
        welcome,
        Components::spacer(SpacerSize::Medium),
        stats_grid,
        Components::spacer(SpacerSize::Large),
    ];
    
    // Add additional sections
    if !gamification_stats.is_empty() {
        content_items.push(Components::settings_section("Gamification", gamification_stats));
    }
    if !cognitive_metrics.is_empty() {
        content_items.push(Components::settings_section("Cognitive Performance", cognitive_metrics));
    }
    if !weekly_goals.is_empty() {
        content_items.push(Components::settings_section("Weekly Goals", weekly_goals));
    }
    if !recent_achievements.is_empty() {
        content_items.push(Components::settings_section("Recent Achievements", recent_achievements));
    }
    if !active_powerups.is_empty() {
        content_items.push(Components::settings_section("Active Power-ups", active_powerups));
    }
    
    // Add quick actions
    content_items.push(quick_actions);

    // Append Little Crab card if active
    if state.easter_egg_manager.crab.active {
        content_items.push(render_little_crab(state));
    }

    let content = Components::simple_flex_column(content_items);
    
    // Use page layout for consistent structure
    let page_content = page_layout(
        "ABCDEEZ Dashboard",
        Some(&format!("Welcome back, {}!", username)),
        content
    );
    
    // Fill screen; center content with max-width in components
    Components::app_scaffold(header, page_content, Some(bottom_nav))
}

// Helper functions

fn get_accuracy_color(accuracy: u32) -> AppColor {
    match accuracy {
        90..=100 => AppColor::Success,
        70..=89 => AppColor::Warning,
        50..=69 => AppColor::Info,
        _ => AppColor::Error,
    }
}

fn get_performance_rating(session: &crate::state::SessionState) -> String {
    let accuracy = if session.responses_total > 0 {
        session.responses_correct as f32 / session.responses_total as f32
    } else {
        0.0
    };
    
    match accuracy {
        x if x >= 0.9 => "Mastery Level",
        x if x >= 0.8 => "Proficient",
        x if x >= 0.7 => "Developing",
        x if x >= 0.6 => "Practicing",
        _ => "Learning",
    }.to_string()
}

fn format_time_ago(time: chrono::DateTime<chrono::Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(time);
    
    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "Just now".to_string()
    }
}

// Render the Little Crab overlay
pub fn render_little_crab(state: &AppState) -> ComponentOutput {
    let crab = &state.easter_egg_manager.crab;
    
    if !crab.active {
        return Components::empty();
    }
    
    // Get crab emoji with seasonal outfit
    let crab_emoji = crab.get_emoji();
    
    // Get latest message
    let message = crab.messages.last()
        .map(|msg| msg.as_str())
        .unwrap_or("");
    
    // Create crab display with personality-based styling
    let _color = match crab.mood {
        crate::utils::easter_egg::CrabMood::Happy => AppColor::Success,
        crate::utils::easter_egg::CrabMood::Excited => AppColor::Warning,
        crate::utils::easter_egg::CrabMood::Celebratory => AppColor::Success,
        crate::utils::easter_egg::CrabMood::Encouraging => AppColor::Info,
        crate::utils::easter_egg::CrabMood::Curious => AppColor::Primary,
        _ => AppColor::Secondary,
    };
    
    // Display stats if friendship is high enough
    let stats_display = if crab.friendship_level >= 10 {
        Components::label(&format!(
            "Friendship: {} • Energy: {:.0}%", 
            crab.friendship_level, 
            crab.energy
        ))
    } else {
        Components::empty()
    };
    
    // Display treasures count if any collected
    let treasure_display = if !crab.collected_treasures.is_empty() {
        Components::label(&format!(
            "Treasures: {} collected", 
            crab.collected_treasures.len()
        ))
    } else {
        Components::empty()
    };
    
    Components::card(
        &crab_emoji,
        Components::simple_flex_column(vec![
            Components::label(message),
            stats_display,
            treasure_display,
        ])
    )
}