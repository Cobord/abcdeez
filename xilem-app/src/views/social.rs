// Social features views: Leaderboard, Challenges, and Friends

use crate::state::{AppState, Screen};
use crate::components::{Components, AppComponents, AppColor, ComponentOutput, SpacerSize};
use crate::components::layout::{page_layout, dashboard_grid, tab_layout_with_state};
use crate::components::cards::{leaderboard_card, challenge_card, ChallengeDifficulty, metric_card};
use crate::components::feedback::{status_badge, StatusType};

pub fn leaderboard_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    // Build tabs for different leaderboards
    let tabs = vec![
        ("Global", global_leaderboard_tab(state, window_width)),
        ("Weekly", weekly_leaderboard_tab(state)),
        ("Friends", friends_leaderboard_tab(state)),
        ("Domain", domain_leaderboard_tab(state)),
    ];
    
    let tabbed_content = tab_layout_with_state(state, "leaderboard", tabs);
    
    page_layout(
        "Leaderboard",
        Some("Compete with learners worldwide"),
        tabbed_content
    )
}

fn global_leaderboard_tab(state: &AppState, window_width: f64) -> ComponentOutput {
    let mut items = vec![];
    
    // Your stats at the top
    if let Some(profile) = state.get_current_user_profile() {
        let your_stats = vec![
            metric_card("🏆", "Your Rank", &format!("#{}", profile.leaderboard_rank), None, AppColor::Primary),
            metric_card("⭐", "Total XP", &profile.experience.to_string(), None, AppColor::Success),
            metric_card("📊", "This Week", &format!("+{} XP", profile.weekly_goals.xp_earned), None, AppColor::Warning),
            metric_card("🔥", "Streak", &format!("{} days", profile.streak.current), None, AppColor::Error),
        ];
        
        items.push(dashboard_grid(your_stats, window_width));
        items.push(Components::spacer(SpacerSize::Large));
    }
    
    items.push(Components::label("Top Players"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Mock leaderboard data
    let leaders = vec![
        (1, "AlexMaster", 15420, Some("🦸"), false),
        (2, "StudyNinja", 14850, Some("🥷"), false),
        (3, "BrainPower", 14200, Some("🧠"), false),
        (4, "You", 12500, Some("🎯"), true),
        (5, "QuizWhiz", 11800, Some("✨"), false),
        (6, "LearnBot", 11200, Some("🤖"), false),
        (7, "SmartCookie", 10500, Some("🍪"), false),
    ];
    
    for (rank, username, score, avatar, is_you) in leaders {
        items.push(leaderboard_card(rank, username, score, avatar, is_you));
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    Components::simple_flex_column(items)
}

fn weekly_leaderboard_tab(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("This Week's Competition"));
    items.push(Components::spacer(SpacerSize::Small));
    
    // Week progress
    items.push(Components::progress_bar(0.4, "3 days remaining"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Weekly prizes
    items.push(Components::card(
        "Weekly Prizes",
        Components::simple_flex_column(vec![
            Components::label("🥇 1st Place: 500 bonus XP + Champion Badge"),
            Components::label("🥈 2nd Place: 300 bonus XP + Silver Badge"),
            Components::label("🥉 3rd Place: 150 bonus XP + Bronze Badge"),
            Components::label("🎖️ Top 10: 50 bonus XP"),
        ])
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Weekly leaders
    let weekly_leaders = vec![
        (1, "SpeedLearner", 3420, Some("⚡"), false),
        (2, "WeekWarrior", 3150, Some("⚔️"), false),
        (3, "You", 2800, Some("🎯"), true),
    ];
    
    for (rank, username, score, avatar, is_you) in weekly_leaders {
        items.push(leaderboard_card(rank, username, score, avatar, is_you));
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    Components::simple_flex_column(items)
}

fn friends_leaderboard_tab(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    if let Some(profile) = state.get_current_user_profile() {
        if profile.friends.is_empty() {
            items.push(Components::empty_state(
                "👥",
                "No Friends Yet",
                "Add friends to see how you compare!",
                Some(Components::action_button(
                    "Find Friends",
                    AppColor::Primary,
                    |state| state.navigate(Screen::Friends)
                ))
            ));
        } else {
            items.push(Components::label("Friends Competition"));
            items.push(Components::spacer(SpacerSize::Medium));
            
            // Mock friends data
            let friends = vec![
                (1, "BestFriend", 8500, Some("💙"), false),
                (2, "You", 7200, Some("🎯"), true),
                (3, "StudyBuddy", 6800, Some("📚"), false),
            ];
            
            for (rank, username, score, avatar, is_you) in friends {
                items.push(leaderboard_card(rank, username, score, avatar, is_you));
                items.push(Components::spacer(SpacerSize::Small));
            }
        }
    } else {
        items.push(Components::empty_state(
            "👥",
            "Sign in to compete with friends",
            "",
            None
        ));
    }
    
    Components::simple_flex_column(items)
}

fn domain_leaderboard_tab(state: &mut AppState) -> ComponentOutput {
    let mut items = vec![];
    
    // Domain selector
    items.push(Components::label("Select Domain"));
    items.push(Components::spacer(SpacerSize::Small));
    
    let domains = vec!["Alphabet", "Numbers", "Colors", "Days"];
    let current_domain = state.selected_domain.as_deref().unwrap_or("Alphabet");
    
    for domain in domains {
        let is_selected = domain == current_domain;
        let domain_str = domain.to_string();
        items.push(Components::simple_button(
            &format!("{} {}", if is_selected { "✓" } else { "○" }, domain),
            move |state| {
                state.selected_domain = Some(domain_str.clone());
            }
        ));
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    items.push(Components::spacer(SpacerSize::Medium));
    items.push(Components::divider(crate::components::Orientation::Horizontal));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Domain-specific leaders
    items.push(Components::label("Alphabet Masters"));
    items.push(Components::spacer(SpacerSize::Small));
    
    let domain_leaders = vec![
        (1, "AlphaKing", 4500, Some("👑"), false),
        (2, "LetterPro", 4200, Some("🔤"), false),
        (3, "ABCMaster", 3900, Some("🎓"), false),
    ];
    
    for (rank, username, score, avatar, is_you) in domain_leaders {
        items.push(leaderboard_card(rank, username, score, avatar, is_you));
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    Components::simple_flex_column(items)
}

pub fn challenges_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    let tabs = vec![
        ("Active", active_challenges_tab(state, window_width)),
        ("Daily", daily_challenges_tab(state)),
        ("Weekly", weekly_challenges_tab(state)),
        ("Completed", completed_challenges_tab(state)),
    ];
    
    let tabbed_content = tab_layout_with_state(state, "challenges", tabs);
    
    page_layout(
        "Challenges",
        Some("Test your skills with special challenges"),
        tabbed_content
    )
}

fn active_challenges_tab(state: &AppState, window_width: f64) -> ComponentOutput {
    let mut items = vec![];
    
    // Active challenge stats
    let stats = vec![
        metric_card("🎯", "Active", "3", Some("challenges"), AppColor::Primary),
        metric_card("✅", "Completed Today", "2", None, AppColor::Success),
        metric_card("🏆", "Total Completed", "47", None, AppColor::Warning),
        metric_card("⭐", "Challenge Points", "2,850", None, AppColor::Info),
    ];
    
    items.push(dashboard_grid(stats, window_width));
    items.push(Components::spacer(SpacerSize::Large));
    
    // Active challenges
    items.push(Components::label("Your Active Challenges"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    let challenges = vec![
        ("Speed Demon", "Complete 20 tasks in under 10 seconds each", ChallengeDifficulty::Medium, 200, Some("5 minutes"), Some(0.65)),
        ("Perfectionist", "Achieve 100% accuracy in 5 consecutive sessions", ChallengeDifficulty::Hard, 500, None, Some(0.4)),
        ("Marathon", "Complete 100 tasks in a single day", ChallengeDifficulty::Expert, 1000, Some("18 hours"), Some(0.25)),
    ];
    
    for (title, desc, difficulty, xp, time_limit, progress) in challenges {
        items.push(challenge_card(
            title,
            desc,
            difficulty,
            xp,
            time_limit,
            progress,
            |state| {
                state.navigate(Screen::Learning);
            }
        ));
        items.push(Components::spacer(SpacerSize::Medium));
    }
    
    Components::simple_flex_column(items)
}

fn daily_challenges_tab(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Today's Challenges"));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(Components::label("Resets in 14h 23m"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    let daily = vec![
        ("Morning Warm-up", "Complete 10 tasks before noon", ChallengeDifficulty::Easy, 50, Some("4 hours"), None),
        ("Accuracy Focus", "Maintain 90% accuracy for 20 tasks", ChallengeDifficulty::Medium, 100, None, Some(0.0)),
        ("Speed Run", "Complete 5 tasks in under 5 seconds each", ChallengeDifficulty::Hard, 150, None, None),
    ];
    
    for (title, desc, difficulty, xp, time_limit, progress) in daily {
        items.push(challenge_card(
            title,
            desc,
            difficulty,
            xp,
            time_limit,
            progress,
            |state| {
                state.navigate(Screen::Learning);
            }
        ));
        items.push(Components::spacer(SpacerSize::Medium));
    }
    
    Components::simple_flex_column(items)
}

fn weekly_challenges_tab(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Weekly Challenges"));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(Components::label("Resets in 4 days"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    let weekly = vec![
        ("Consistency King", "Maintain a 7-day streak", ChallengeDifficulty::Medium, 500, Some("4 days"), Some(0.43)),
        ("Domain Master", "Complete 50 tasks in each domain", ChallengeDifficulty::Hard, 1000, Some("4 days"), Some(0.2)),
        ("Elite Performance", "Achieve 95% overall accuracy", ChallengeDifficulty::Expert, 1500, Some("4 days"), Some(0.88)),
    ];
    
    for (title, desc, difficulty, xp, time_limit, progress) in weekly {
        items.push(challenge_card(
            title,
            desc,
            difficulty,
            xp,
            time_limit,
            progress,
            |state| {
                state.navigate(Screen::Learning);
            }
        ));
        items.push(Components::spacer(SpacerSize::Medium));
    }
    
    Components::simple_flex_column(items)
}

fn completed_challenges_tab(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Completed Challenges"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    let completed = vec![
        ("First Steps", "Complete your first task", ChallengeDifficulty::Easy, 25, "2 days ago"),
        ("Quick Learner", "Complete 10 tasks in 5 minutes", ChallengeDifficulty::Medium, 100, "Yesterday"),
        ("Streak Builder", "Maintain a 3-day streak", ChallengeDifficulty::Easy, 75, "Today"),
    ];
    
    for (title, desc, difficulty, xp, completed_when) in completed {
        items.push(Components::card(
            title,
            Components::simple_flex_column(vec![
                Components::label(desc),
                Components::spacer(SpacerSize::Small),
                Components::simple_flex_row(vec![
                    status_badge(&format!("{} XP earned", xp), StatusType::Completed),
                    Components::label(completed_when),
                ]),
            ])
        ));
        items.push(Components::spacer(SpacerSize::Medium));
    }
    
    Components::simple_flex_column(items)
}

pub fn friends_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    let tabs = vec![
        ("Friends", friends_list_tab(state, window_width)),
        ("Requests", friend_requests_tab(state)),
        ("Find Friends", find_friends_tab(state)),
    ];
    
    let tabbed_content = tab_layout_with_state(state, "friends", tabs);
    
    page_layout(
        "Friends",
        Some("Connect with other learners"),
        tabbed_content
    )
}

fn friends_list_tab(state: &AppState, window_width: f64) -> ComponentOutput {
    let mut items = vec![];
    
    if let Some(profile) = state.get_current_user_profile() {
        if profile.friends.is_empty() {
            items.push(Components::empty_state(
                "👥",
                "No Friends Yet",
                "Start adding friends to learn together!",
                Some(Components::action_button(
                    "Find Friends",
                    AppColor::Primary,
                    |_state| {
                        // Switch to find friends tab
                    }
                ))
            ));
        } else {
            // Friend stats
            let stats = vec![
                metric_card("👥", "Friends", &profile.friends.len().to_string(), None, AppColor::Primary),
                metric_card("🏆", "Top Friend Rank", "#12", None, AppColor::Success),
                metric_card("📊", "Avg Friend Level", "8", None, AppColor::Info),
            ];
            
            items.push(dashboard_grid(stats, window_width));
            items.push(Components::spacer(SpacerSize::Large));
            
            items.push(Components::label("Your Friends"));
            items.push(Components::spacer(SpacerSize::Medium));
            
            // Mock friends list
            let friends = vec![
                ("BestFriend", "Level 12", "Online now", true, 8500),
                ("StudyBuddy", "Level 9", "2 hours ago", false, 6800),
                ("LearningPal", "Level 7", "Yesterday", false, 5200),
            ];
            
            for (name, level, last_seen, is_online, xp) in friends {
                items.push(friend_card(name, level, last_seen, is_online, xp));
                items.push(Components::spacer(SpacerSize::Small));
            }
        }
    } else {
        items.push(Components::empty_state(
            "👥",
            "Sign in to add friends",
            "",
            None
        ));
    }
    
    Components::simple_flex_column(items)
}

fn friend_requests_tab(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    // Mock pending requests
    let requests = vec![
        ("NewFriend", "Level 5", "Sent 2 hours ago"),
        ("Challenger", "Level 10", "Sent yesterday"),
    ];
    
    if requests.is_empty() {
        items.push(Components::empty_state(
            "📬",
            "No Pending Requests",
            "Friend requests will appear here",
            None
        ));
    } else {
        items.push(Components::label("Pending Friend Requests"));
        items.push(Components::spacer(SpacerSize::Medium));
        
        for (name, level, when) in requests {
            items.push(Components::card(
                name,
                Components::simple_flex_column(vec![
                    Components::label(level),
                    Components::label(when),
                    Components::spacer(SpacerSize::Small),
                    Components::simple_flex_row(vec![
                        Components::action_button("Accept", AppColor::Success, |state| {
                            tracing::info!("Friend request accepted");
                            state.add_error("Friend request accepted!".to_string(), true);
                            // TODO: Actually accept the friend request
                        }),
                        Components::action_button("Decline", AppColor::Secondary, |state| {
                            tracing::info!("Friend request declined");
                            // TODO: Actually decline the friend request
                        }),
                    ]),
                ])
            ));
            items.push(Components::spacer(SpacerSize::Medium));
        }
    }
    
    Components::simple_flex_column(items)
}

fn find_friends_tab(state: &mut AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Find Friends"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Search input
    items.push(Components::labeled_input(
        "Search by username...",
        state.search_query.clone(),
        |state, query| {
            state.search_query = query;
            // In a real app, this would trigger a search
        }
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Suggested friends
    items.push(Components::label("Suggested Friends"));
    items.push(Components::spacer(SpacerSize::Small));
    
    let suggestions = vec![
        ("TopLearner", "Level 15", "Similar learning pace", 12000),
        ("QuizMaster", "Level 11", "In your domain", 9500),
        ("StudyGroup", "Level 8", "Active daily", 7200),
    ];
    
    for (name, level, reason, xp) in suggestions {
        items.push(Components::card(
            name,
            Components::simple_flex_column(vec![
                Components::label(level),
                Components::label(&format!("🎯 {}", reason)),
                Components::label(&format!("⭐ {} XP", xp)),
                Components::spacer(SpacerSize::Small),
                Components::action_button("Add Friend", AppColor::Primary, |state| {
                    tracing::info!("Sending friend request to {}", name);
                    state.add_error(format!("Friend request sent to {}", name), true);
                    // TODO: Actually send friend request
                }),
            ])
        ));
        items.push(Components::spacer(SpacerSize::Medium));
    }
    
    Components::simple_flex_column(items)
}

// Helper function for friend cards
fn friend_card(name: &str, level: &str, last_seen: &str, is_online: bool, xp: u32) -> ComponentOutput {
    Components::card(
        name,
        Components::simple_flex_column(vec![
            Components::simple_flex_row(vec![
                Components::label(level),
                Components::label("•"),
                Components::label(&format!("{} XP", xp)),
            ]),
            Components::spacer(SpacerSize::Small),
            if is_online {
                status_badge("Online", StatusType::Active)
            } else {
                Components::label(last_seen)
            },
            Components::spacer(SpacerSize::Small),
            Components::simple_flex_row(vec![
                Components::action_button("Challenge", AppColor::Primary, |state| {
                    tracing::info!("Challenging friend {}", name);
                    state.navigate(Screen::Challenges);
                }),
                Components::action_button("Compare", AppColor::Info, |state| {
                    tracing::info!("Comparing stats with {}", name);
                    state.navigate(Screen::Analytics);
                }),
            ]),
        ])
    )
}