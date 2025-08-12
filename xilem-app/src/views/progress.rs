// Progress tracking view with enhanced metrics and visualizations

use crate::state::{AppState, Screen};
use crate::components::{Components, AppComponents, AppColor, ComponentOutput, SpacerSize, TabItem};
use crate::components::layout::{page_layout, dashboard_grid, tab_layout_with_state};
use crate::components::cards::{stat_card_with_trend, Trend, metric_card, achievement_card};
use crate::components::feedback::{step_indicator};

pub fn progress_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    // Get user profile data
    let profile = state.get_current_user_profile();
    
    // Build tabs for different progress views
    let tabs = vec![
        ("Overview", overview_tab(state, &profile, window_width)),
        ("Daily", daily_progress_tab(state, &profile)),
        ("Weekly", weekly_progress_tab(state, &profile)),
        ("Achievements", achievements_tab(state, &profile)),
    ];
    
    // Create tabbed layout with state management
    let tabbed_content = tab_layout_with_state(state, "progress", tabs);
    
    // Use page layout for consistent structure
    page_layout(
        "Progress Tracking",
        Some("Track your learning journey"),
        tabbed_content
    )
}

fn overview_tab(state: &AppState, profile: &Option<crate::gamification::profile::GamificationProfile>, window_width: f64) -> ComponentOutput {
    let mut items = vec![];
    
    if let Some(profile) = profile {
        // Progress to next level
        let level_progress = Components::progress_bar(
            profile.get_progress_to_next_level(),
            &format!("Level {} → Level {}", profile.level, profile.level + 1)
        );
        items.push(level_progress);
        items.push(Components::spacer(SpacerSize::Medium));
        
        // Key metrics with trends
        let metrics = vec![
            stat_card_with_trend(
                &format!("{}", profile.level),
                "Level",
                Trend::Up(10),
                AppColor::Primary
            ),
            stat_card_with_trend(
                &format!("{} XP", profile.experience),
                "Total XP",
                Trend::Up(250),
                AppColor::Success
            ),
            stat_card_with_trend(
                &format!("{} days", profile.streak.current),
                "Current Streak",
                if profile.streak.is_at_risk() { Trend::Down(1) } else { Trend::Up(1) },
                if profile.streak.is_at_risk() { AppColor::Warning } else { AppColor::Success }
            ),
            metric_card(
                "🏆",
                "Achievements",
                &format!("{}", profile.achievements.len()),
                Some(&format!("{} this week", profile.achievements.iter().filter(|a| {
                    a.unlocked_at.map_or(false, |t| {
                        let days_ago = chrono::Utc::now().signed_duration_since(t).num_days();
                        days_ago <= 7
                    })
                }).count())),
                AppColor::Warning
            ),
        ];
        
        items.push(dashboard_grid(metrics, window_width));
        items.push(Components::spacer(SpacerSize::Large));
        
        // Weekly goals progress
        let goals = &profile.weekly_goals;
        items.push(Components::label("Weekly Goals"));
        items.push(Components::spacer(SpacerSize::Small));
        
        let goal_cards = vec![
            Components::progress_card(
                "Tasks",
                goals.tasks_completed as f32 / goals.tasks_goal as f32,
                &format!("{}/{} completed", goals.tasks_completed, goals.tasks_goal),
                AppColor::Primary
            ),
            Components::progress_card(
                "Accuracy",
                goals.current_accuracy / goals.accuracy_goal,
                &format!("{:.0}%/{:.0}%", goals.current_accuracy * 100.0, goals.accuracy_goal * 100.0),
                AppColor::Success
            ),
            Components::progress_card(
                "Streak",
                goals.current_streak as f32 / goals.streak_goal as f32,
                &format!("{}/{} days", goals.current_streak, goals.streak_goal),
                AppColor::Warning
            ),
            Components::progress_card(
                "XP",
                goals.xp_earned as f32 / goals.xp_goal as f32,
                &format!("{}/{} XP", goals.xp_earned, goals.xp_goal),
                AppColor::Info
            ),
        ];
        
        items.push(dashboard_grid(goal_cards, window_width));
    } else {
        items.push(Components::empty_state(
            "📊",
            "No Progress Data",
            "Start learning to track your progress",
            Some(Components::action_button(
                "Start Learning",
                AppColor::Primary,
                |state| state.navigate(Screen::Learning)
            ))
        ));
    }
    
    Components::simple_flex_column(items)
}

fn daily_progress_tab(_state: &AppState, profile: &Option<crate::gamification::profile::GamificationProfile>) -> ComponentOutput {
    let mut items = vec![];
    
    if let Some(profile) = profile {
        // Daily streak status
        items.push(Components::card(
            "Daily Streak",
            Components::simple_flex_column(vec![
                Components::label(&profile.streak.display_status()),
                Components::spacer(SpacerSize::Small),
                if profile.streak.completed_today {
                    Components::label("✅ Completed today!")
                } else {
                    Components::label("⏰ Complete today's session to maintain streak")
                },
            ])
        ));
        
        items.push(Components::spacer(SpacerSize::Medium));
        
        // Today's progress
        let today_stats = vec![
            Components::stat_card("Tasks", "0", AppColor::Info), // Would get from session data
            Components::stat_card("Accuracy", "0%", AppColor::Success),
            Components::stat_card("Time", "0m", AppColor::Primary),
            Components::stat_card("XP Earned", "0", AppColor::Warning),
        ];
        
        items.push(Components::label("Today's Progress"));
        items.push(Components::spacer(SpacerSize::Small));
        items.push(Components::stats_grid(today_stats));
        
        // Daily goals checklist
        items.push(Components::spacer(SpacerSize::Medium));
        items.push(Components::label("Daily Goals"));
        items.push(Components::spacer(SpacerSize::Small));
        
        let daily_goals = vec![
            ("Complete 10 tasks", false),
            ("Maintain 80% accuracy", false),
            ("Practice for 15 minutes", false),
            ("Earn 100 XP", false),
        ];
        
        for (goal, completed) in daily_goals {
            items.push(Components::checkbox(
                completed,
                goal,
                |state, checked| {
                    tracing::info!("Daily goal '{}' marked as {}", goal, if checked { "complete" } else { "incomplete" });
                    // TODO: Actually update goal completion status
                }
            ));
        }
    } else {
        items.push(Components::empty_state(
            "📅",
            "No Daily Data",
            "Complete sessions to see daily progress",
            None
        ));
    }
    
    Components::simple_flex_column(items)
}

fn weekly_progress_tab(_state: &AppState, profile: &Option<crate::gamification::profile::GamificationProfile>) -> ComponentOutput {
    let mut items = vec![];
    
    if let Some(profile) = profile {
        // Week progress overview
        let week_progress = (profile.weekly_goals.tasks_completed as f32 / profile.weekly_goals.tasks_goal.max(1) as f32).min(1.0);
        
        items.push(step_indicator(
            (week_progress * 7.0) as usize,
            7,
            Some(vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"])
        ));
        
        items.push(Components::spacer(SpacerSize::Medium));
        
        // Weekly stats comparison
        items.push(Components::label("This Week vs Last Week"));
        items.push(Components::spacer(SpacerSize::Small));
        
        let comparison = vec![
            stat_card_with_trend("450", "Tasks", Trend::Up(12), AppColor::Info),
            stat_card_with_trend("87%", "Accuracy", Trend::Up(3), AppColor::Success),
            stat_card_with_trend("3.5h", "Time", Trend::Neutral, AppColor::Primary),
            stat_card_with_trend("2,100", "XP", Trend::Up(350), AppColor::Warning),
        ];
        
        items.push(Components::stats_grid(comparison));
        
        // Weekly leaderboard position
        items.push(Components::spacer(SpacerSize::Medium));
        items.push(Components::card(
            "Weekly Leaderboard",
            Components::simple_flex_column(vec![
                Components::label(&format!("Your Position: #{}", profile.leaderboard_rank)),
                Components::spacer(SpacerSize::Small),
                Components::progress_bar(0.75, "Top 25% of learners"),
            ])
        ));
    } else {
        items.push(Components::empty_state(
            "📈",
            "No Weekly Data",
            "Complete a full week to see trends",
            None
        ));
    }
    
    Components::simple_flex_column(items)
}

fn achievements_tab(_state: &AppState, profile: &Option<crate::gamification::profile::GamificationProfile>) -> ComponentOutput {
    let mut items = vec![];
    
    if let Some(profile) = profile {
        // Achievement stats
        let total = profile.all_achievements_count;
        let unlocked = profile.achievements.len();
        let progress = unlocked as f32 / total.max(1) as f32;
        
        items.push(Components::progress_bar(
            progress as f64,
            &format!("{}/{} Achievements Unlocked", unlocked, total)
        ));
        items.push(Components::spacer(SpacerSize::Medium));
        
        // Recent achievements
        items.push(Components::label("Recent Achievements"));
        items.push(Components::spacer(SpacerSize::Small));
        
        let mut recent = profile.achievements.clone();
        recent.sort_by(|a, b| b.unlocked_at.cmp(&a.unlocked_at));
        recent.truncate(5);
        
        for achievement in recent {
            items.push(achievement_card(
                &achievement.icon,
                &achievement.name,
                &achievement.description,
                true,
                None
            ));
            items.push(Components::spacer(SpacerSize::Small));
        }
        
        // Locked achievements preview
        items.push(Components::spacer(SpacerSize::Medium));
        items.push(Components::label("Next Achievements"));
        items.push(Components::spacer(SpacerSize::Small));
        
        // Mock locked achievements
        let locked = vec![
            ("🎯", "Sharpshooter", "Achieve 100% accuracy in 10 sessions", Some(0.7)),
            ("📚", "Scholar", "Complete 1000 total tasks", Some(0.45)),
            ("⚡", "Speed Demon", "Answer 50 questions in under 1 second each", Some(0.2)),
        ];
        
        for (icon, name, desc, progress) in locked {
            items.push(achievement_card(icon, name, desc, false, progress));
            items.push(Components::spacer(SpacerSize::Small));
        }
    } else {
        items.push(Components::empty_state(
            "🏆",
            "No Achievements Yet",
            "Start learning to unlock achievements",
            None
        ));
    }
    
    Components::simple_flex_column(items)
}