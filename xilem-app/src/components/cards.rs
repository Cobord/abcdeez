// Card components for displaying data and metrics

use crate::state::AppState;
use crate::components::{AppColor, AppComponents, ComponentOutput, Components, SpacerSize};

/// Create an enhanced stat card with trend indicator
pub fn stat_card_with_trend(
    value: &str,
    label: &str,
    trend: Trend,
    color: AppColor,
) -> ComponentOutput {
    let trend_indicator = match trend {
        Trend::Up(pct) => format!("↑ {}%", pct),
        Trend::Down(pct) => format!("↓ {}%", pct),
        Trend::Neutral => "—".to_string(),
    };
    
    let items = vec![
        Components::label(value),
        Components::label(label),
        Components::label(&trend_indicator),
    ];
    
    Components::card(label, Components::simple_flex_column(items))
}

#[derive(Debug, Clone)]
pub enum Trend {
    Up(u32),
    Down(u32),
    Neutral,
}

/// Create a metric card with icon
pub fn metric_card(
    icon: &str,
    title: &str,
    value: &str,
    subtitle: Option<&str>,
    color: AppColor,
) -> ComponentOutput {
    let mut items = vec![
        Components::label(icon),
        Components::label(title),
        Components::label(value),
    ];
    
    if let Some(subtitle) = subtitle {
        items.push(Components::label(subtitle));
    }
    
    Components::card(title, Components::simple_flex_column(items))
}

/// Create an achievement card
pub fn achievement_card(
    icon: &str,
    name: &str,
    description: &str,
    unlocked: bool,
    progress: Option<f32>,
) -> ComponentOutput {
    let status = if unlocked {
        "✓ Unlocked"
    } else if let Some(progress) = progress {
        &format!("{:.0}% Complete", progress * 100.0)
    } else {
        "Locked"
    };
    
    let items = vec![
        Components::label(icon),
        Components::label(name),
        Components::label(description),
        Components::label(status),
    ];
    
    if let Some(progress) = progress {
        if !unlocked {
            let progress_bar = Components::progress_bar(progress as f64, "");
            let mut all_items = items;
            all_items.push(progress_bar);
            return Components::card(name, Components::simple_flex_column(all_items));
        }
    }
    
    Components::card(name, Components::simple_flex_column(items))
}

/// Create a leaderboard entry card
pub fn leaderboard_card(
    rank: u32,
    username: &str,
    score: u32,
    avatar: Option<&str>,
    is_current_user: bool,
) -> ComponentOutput {
    let rank_display = match rank {
        1 => "🥇".to_string(),
        2 => "🥈".to_string(),
        3 => "🥉".to_string(),
        _ => format!("#{}", rank),
    };
    
    let avatar_display = avatar.unwrap_or("👤");
    
    let highlight = if is_current_user {
        " (You)"
    } else {
        ""
    };
    
    let items = vec![
        Components::label(&rank_display),
        Components::label(avatar_display),
        Components::label(&format!("{}{}", username, highlight)),
        Components::label(&format!("{} pts", score)),
    ];
    
    Components::simple_flex_row(items)
}

/// Create a session summary card
pub fn session_summary_card(
    duration_minutes: u32,
    tasks_completed: u32,
    accuracy: f32,
    xp_earned: u32,
) -> ComponentOutput {
    let items = vec![
        metric_row("Duration", &format!("{} min", duration_minutes)),
        metric_row("Tasks", &tasks_completed.to_string()),
        metric_row("Accuracy", &format!("{:.0}%", accuracy * 100.0)),
        metric_row("XP Earned", &format!("+{}", xp_earned)),
    ];
    
    Components::card("Session Summary", Components::simple_flex_column(items))
}

/// Create a simple metric row
fn metric_row(label: &str, value: &str) -> ComponentOutput {
    let items = vec![
        Components::label(label),
        Components::spacer(SpacerSize::Small),
        Components::label(value),
    ];
    Components::simple_flex_row(items)
}

/// Create a feature card for showcasing app features
pub fn feature_card<F>(
    icon: &str,
    title: &str,
    description: &str,
    action_label: &str,
    on_action: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let items = vec![
        Components::label(icon),
        Components::label(title),
        Components::label(description),
        Components::spacer(SpacerSize::Medium),
        Components::action_button(action_label, AppColor::Primary, on_action),
    ];
    
    Components::card(title, Components::simple_flex_column(items))
}

/// Create a notification card
pub fn notification_card<F>(
    icon: &str,
    title: &str,
    message: &str,
    timestamp: &str,
    is_read: bool,
    on_click: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let unread_indicator = if !is_read { "•" } else { "" };
    
    let items = vec![
        Components::simple_flex_row(vec![
            Components::label(icon),
            Components::label(&format!("{} {}", title, unread_indicator)),
            Components::spacer(SpacerSize::Small),
            Components::label(timestamp),
        ]),
        Components::label(message),
    ];
    
    let card_content = Components::simple_flex_column(items);
    
    // For now, just return the card content with a click handler
    // TODO: Make the whole card clickable
    card_content
}

/// Create a domain selection card
pub fn domain_card_enhanced<F>(
    icon: &str,
    name: &str,
    description: &str,
    difficulty: &str,
    is_selected: bool,
    stats: Option<DomainStats>,
    on_select: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let mut items = vec![
        Components::label(icon),
        Components::label(name),
        Components::label(description),
        Components::label(&format!("Difficulty: {}", difficulty)),
    ];
    
    // Add stats if available
    if let Some(stats) = stats {
        items.push(Components::divider(crate::components::Orientation::Horizontal));
        items.push(metric_row("Sessions", &stats.sessions.to_string()));
        items.push(metric_row("Best Streak", &stats.best_streak.to_string()));
        items.push(metric_row("Mastery", &format!("{:.0}%", stats.mastery * 100.0)));
    }
    
    // Selection indicator
    if is_selected {
        items.push(Components::label("✓ Selected"));
    }
    
    items.push(Components::action_button(
        if is_selected { "Selected" } else { "Select" },
        if is_selected { AppColor::Success } else { AppColor::Primary },
        on_select
    ));
    
    Components::card(name, Components::simple_flex_column(items))
}

#[derive(Debug, Clone)]
pub struct DomainStats {
    pub sessions: u32,
    pub best_streak: u32,
    pub mastery: f32,
}

/// Create a challenge card
pub fn challenge_card<F>(
    title: &str,
    description: &str,
    difficulty: ChallengeDifficulty,
    reward_xp: u32,
    time_limit: Option<&str>,
    progress: Option<f32>,
    on_start: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    let difficulty_display = match difficulty {
        ChallengeDifficulty::Easy => "⭐ Easy",
        ChallengeDifficulty::Medium => "⭐⭐ Medium",
        ChallengeDifficulty::Hard => "⭐⭐⭐ Hard",
        ChallengeDifficulty::Expert => "⭐⭐⭐⭐ Expert",
    };
    
    let mut items = vec![
        Components::label(title),
        Components::label(description),
        Components::label(difficulty_display),
        Components::label(&format!("Reward: {} XP", reward_xp)),
    ];
    
    if let Some(time_limit) = time_limit {
        items.push(Components::label(&format!("⏱ {}", time_limit)));
    }
    
    if let Some(progress) = progress {
        items.push(Components::progress_bar(progress as f64, "Progress"));
        if progress >= 1.0 {
            items.push(Components::label("✓ Completed"));
        } else {
            items.push(Components::action_button("Continue", AppColor::Primary, on_start));
        }
    } else {
        items.push(Components::action_button("Start Challenge", AppColor::Primary, on_start));
    }
    
    Components::card(title, Components::simple_flex_column(items))
}

#[derive(Debug, Clone, Copy)]
pub enum ChallengeDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}