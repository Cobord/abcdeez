// Enhanced settings view with advanced configurations

use crate::state::{AppState, Screen, ThemeMode};
use crate::components::{Components, AppComponents, AppTheme, AppColor, ComponentOutput};

pub fn settings_view(state: &mut AppState) -> ComponentOutput {
    let header = Components::header_bar("Settings", |state| {
        state.navigate_back();
    });
    
    let profile_section = enhanced_profile_section(state);
    let learning_section = learning_preferences_section(state);
    let gamification_section = gamification_settings_section(state);
    let demo_section = demo_preferences_section(state);
    let appearance_section = appearance_section(state);
    let accessibility_section = accessibility_section(state);
    let notifications_section = notifications_section(state);
    let advanced_section = advanced_settings_section(state);
    let account_section = account_management_section(state);
    let about_section = about_section(state);
    
    let content = Components::settings_section(
        "",
        vec![
            profile_section,
            learning_section,
            gamification_section,
            demo_section,
            appearance_section,
            accessibility_section,
            notifications_section,
            advanced_section,
            account_section,
            about_section,
        ]
    );
    
    let bottom_nav = Components::bottom_nav_bar(
        Screen::Settings,
        |state, screen| {
            state.navigate(screen);
        }
    );
    
    Components::app_scaffold(header, content, Some(bottom_nav))
}

fn enhanced_profile_section(state: &AppState) -> ComponentOutput {
    let profile = state.get_current_user_profile();
    let username = state.user.as_ref()
        .map(|u| u.username.as_str())
        .unwrap_or("Guest User");
    
    let (level_text, rank_text, xp_text) = if let Some(profile) = profile {
        (
            format!("Level {}", profile.level),
            format!("{} • Tier {}", profile.rank.title, profile.rank.tier),
            format!("{}% to next level", (profile.get_progress_to_next_level() * 100.0) as u32)
        )
    } else {
        ("Level 1".to_string(), "Novice".to_string(), "0% to next level".to_string())
    };
    
    let avatar = Components::empty_state(
        "👤",
        username,
        &format!("{} • {}", rank_text, level_text),
        None
    );
    
    let xp_progress = Components::progress_bar(
        profile.map(|p| p.get_progress_to_next_level() as f64).unwrap_or(0.0),
        &xp_text
    );
    
    let edit_button = Components::action_button(
        "Edit Profile",
        AppColor::Primary,
        |state| {
            state.navigate(Screen::Profile);
        }
    );
    
    let view_achievements = Components::action_button(
        "View Achievements",
        AppColor::Success,
        |state| {
            // TODO: Navigate to achievements screen
            state.navigate(Screen::Profile);
        }
    );
    
    Components::settings_section(
        "Profile",
        vec![avatar, xp_progress, edit_button, view_achievements]
    )
}

fn learning_preferences_section(_state: &mut AppState) -> ComponentOutput {
    // Learning speed preference
    let learning_speed = Components::setting_row(
        "Learning Speed",
        Components::nav_button(
            "Normal",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show speed selector
            }
        )
    );
    
    // Difficulty preference
    let difficulty = Components::setting_row(
        "Default Difficulty",
        Components::nav_button(
            "Adaptive",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show difficulty selector
            }
        )
    );
    
    // Hint preferences
    let hints = Components::checkbox(
        true,
        "Enable Hints",
        |_state, _checked| {
            // TODO: Update hint preference
        }
    );
    
    // Domain preferences
    let domains = Components::setting_row(
        "Preferred Domains",
        Components::nav_button(
            "Select Domains",
            Screen::DomainSelection,
            false,
            |state| {
                state.navigate(Screen::DomainSelection);
            }
        )
    );
    
    // Session duration
    let session_duration = Components::setting_row(
        "Session Duration",
        Components::nav_button(
            "15 minutes",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show duration picker
            }
        )
    );
    
    // Spaced repetition
    let spaced_rep = Components::checkbox(
        true,
        "Enable Spaced Repetition",
        |_state, _checked| {
            // TODO: Toggle spaced repetition
        }
    );
    
    Components::settings_section(
        "Learning Preferences",
        vec![learning_speed, difficulty, hints, domains, session_duration, spaced_rep]
    )
}

fn gamification_settings_section(_state: &mut AppState) -> ComponentOutput {
    let profile = _state.get_current_user_profile();
    // Streak notifications
    let streak_notifications = Components::checkbox(
        true,
        "Streak Reminders",
        |_state, _checked| {
            // TODO: Toggle streak notifications
        }
    );
    
    // Weekly goals
    let weekly_goals = if let Some(profile) = profile {
        let goals = &profile.weekly_goals;
        Components::setting_row(
            "Weekly Goals",
            Components::nav_button(
                &format!("{} tasks, {}% accuracy", goals.tasks_goal, (goals.accuracy_goal * 100.0) as u32),
                Screen::Settings,
                false,
                |_state| {
                    // TODO: Show goals editor
                }
            )
        )
    } else {
        Components::empty()
    };
    
    // Leaderboard privacy
    let leaderboard_privacy = Components::setting_row(
        "Leaderboard Display",
        Components::nav_button(
            "Username Only",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show privacy options
            }
        )
    );
    
    // Power-up notifications
    let powerup_notifs = Components::checkbox(
        true,
        "Power-up Notifications",
        |_state, _checked| {
            // TODO: Toggle power-up notifications
        }
    );
    
    // Achievement sharing
    let achievement_sharing = Components::checkbox(
        false,
        "Auto-share Achievements",
        |_state, _checked| {
            // TODO: Toggle achievement sharing
        }
    );
    
    Components::settings_section(
        "Gamification",
        vec![streak_notifications, weekly_goals, leaderboard_privacy, powerup_notifs, achievement_sharing]
    )
}

fn demo_preferences_section(_state: &mut AppState) -> ComponentOutput {
    // Demo speed
    let demo_speed = Components::setting_row(
        "Demo Speed",
        Components::nav_button(
            "Normal (1.0x)",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show speed selector
            }
        )
    );
    
    // Auto-advance
    let auto_advance = Components::checkbox(
        true,
        "Auto-advance Demo Steps",
        |state, checked| {
            if let Some(ref mut demo) = state.demo_controller {
                if checked {
                    demo.demo_speed = 1.0;
                } else {
                    demo.demo_speed = 0.0; // Pause auto-advance
                }
            }
        }
    );
    
    // Voice narration
    let voice_narration = Components::checkbox(
        false,
        "Voice Narration",
        |state, checked| {
            if let Some(ref mut demo) = state.demo_controller {
                demo.voice_enabled = checked;
            }
        }
    );
    
    // Subtitles
    let subtitles = Components::checkbox(
        true,
        "Show Subtitles",
        |state, checked| {
            if let Some(ref mut demo) = state.demo_controller {
                demo.subtitle_enabled = checked;
            }
        }
    );
    
    // Keyboard shortcuts in demo
    let demo_shortcuts = Components::checkbox(
        true,
        "Enable Demo Shortcuts",
        |state, checked| {
            if let Some(ref mut demo) = state.demo_controller {
                demo.keyboard_shortcuts_enabled = checked;
            }
        }
    );
    
    // Reset demo progress
    let reset_demos = Components::action_button(
        "Reset Demo Progress",
        AppColor::Warning,
        |state| {
            if let Some(ref mut demo) = state.demo_controller {
                demo.state.completed_scenarios.clear();
                demo.state.started_scenarios.clear();
            }
        }
    );
    
    Components::settings_section(
        "Demo & Tutorials",
        vec![demo_speed, auto_advance, voice_narration, subtitles, demo_shortcuts, reset_demos]
    )
}

fn appearance_section(state: &mut AppState) -> ComponentOutput {
    let current_theme = map_theme_mode_to_app_theme(state.theme_mode);
    
    let theme_row = Components::setting_row(
        "Theme",
        Components::theme_selector(current_theme, |state, theme| {
            state.theme_mode = map_app_theme_to_theme_mode(theme);
            state.theme.mode = state.theme_mode;
        })
    );
    
    // Font size
    let font_size = Components::setting_row(
        "Font Size",
        Components::nav_button(
            "Medium",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show font size selector
            }
        )
    );
    
    // Color scheme
    let color_scheme = Components::setting_row(
        "Accent Color",
        Components::nav_button(
            "Blue",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show color picker
            }
        )
    );
    
    // Animations
    let animations = Components::checkbox(
        true,
        "Enable Animations",
        |_state, _checked| {
            // TODO: Toggle animations
        }
    );
    
    // Reduce motion
    let reduce_motion = Components::checkbox(
        false,
        "Reduce Motion",
        |_state, _checked| {
            // TODO: Toggle reduced motion
        }
    );
    
    Components::settings_section(
        "Appearance",
        vec![theme_row, font_size, color_scheme, animations, reduce_motion]
    )
}

fn accessibility_section(_state: &mut AppState) -> ComponentOutput {
    // Screen reader
    let screen_reader = Components::checkbox(
        false,
        "Screen Reader Support",
        |_state, _checked| {
            // TODO: Toggle screen reader
        }
    );
    
    // High contrast
    let high_contrast = Components::checkbox(
        false,
        "High Contrast Mode",
        |_state, _checked| {
            // TODO: Toggle high contrast
        }
    );
    
    // Keyboard navigation
    let keyboard_nav = Components::checkbox(
        true,
        "Full Keyboard Navigation",
        |_state, _checked| {
            // TODO: Toggle keyboard navigation
        }
    );
    
    // Color blind mode
    let colorblind = Components::setting_row(
        "Color Blind Mode",
        Components::nav_button(
            "Off",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show colorblind options
            }
        )
    );
    
    // Text to speech
    let tts = Components::checkbox(
        false,
        "Text to Speech",
        |_state, _checked| {
            // TODO: Toggle TTS
        }
    );
    
    Components::settings_section(
        "Accessibility",
        vec![screen_reader, high_contrast, keyboard_nav, colorblind, tts]
    )
}

fn notifications_section(_state: &mut AppState) -> ComponentOutput {
    // Push notifications
    let push_notifs = Components::checkbox(
        true,
        "Push Notifications",
        |_state, _checked| {
            // TODO: Toggle push notifications
        }
    );
    
    // Email notifications
    let email_notifs = Components::checkbox(
        false,
        "Email Notifications",
        |_state, _checked| {
            // TODO: Toggle email notifications
        }
    );
    
    // Sound effects
    let sound_effects = Components::checkbox(
        true,
        "Sound Effects",
        |_state, _checked| {
            // TODO: Toggle sound effects
        }
    );
    
    // Notification schedule
    let schedule = Components::setting_row(
        "Quiet Hours",
        Components::nav_button(
            "10 PM - 8 AM",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show schedule picker
            }
        )
    );
    
    // Weekly summary
    let weekly_summary = Components::checkbox(
        true,
        "Weekly Progress Summary",
        |_state, _checked| {
            // TODO: Toggle weekly summary
        }
    );
    
    Components::settings_section(
        "Notifications",
        vec![push_notifs, email_notifs, sound_effects, schedule, weekly_summary]
    )
}

fn advanced_settings_section(state: &mut AppState) -> ComponentOutput {
    // Data sync
    let data_sync = Components::setting_row(
        "Data Sync",
        Components::nav_button(
            if state.connection_status == crate::state::ConnectionStatus::Connected {
                "Connected"
            } else {
                "Disconnected"
            },
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show sync settings
            }
        )
    );
    
    // Cache management
    let cache = Components::setting_row(
        "Cache Size",
        Components::nav_button(
            &format!("{} items", state.task_cache.len()),
            Screen::Settings,
            false,
            |state| {
                state.task_cache.clear();
            }
        )
    );
    
    // Developer mode
    let dev_mode = Components::checkbox(
        false,
        "Developer Mode",
        |_state, _checked| {
            // TODO: Toggle developer mode
        }
    );
    
    // Debug logging
    let debug_logging = Components::checkbox(
        false,
        "Debug Logging",
        |_state, _checked| {
            // TODO: Toggle debug logging
        }
    );
    
    // Export logs
    let export_logs = Components::action_button(
        "Export Debug Logs",
        AppColor::Info,
        |_state| {
            // TODO: Export logs
        }
    );
    
    // Reset all settings
    let reset_settings = Components::action_button(
        "Reset All Settings",
        AppColor::Error,
        |state| {
            *state = AppState::default();
        }
    );
    
    Components::settings_section(
        "Advanced",
        vec![data_sync, cache, dev_mode, debug_logging, export_logs, reset_settings]
    )
}

fn account_management_section(_state: &mut AppState) -> ComponentOutput {
    // Export data
    let export_button = Components::action_button(
        "Export All Data",
        AppColor::Primary,
        |_state| {
            // TODO: Export user data
        }
    );
    
    // Privacy settings
    let privacy_button = Components::action_button(
        "Privacy & Security",
        AppColor::Info,
        |_state| {
            // TODO: Navigate to privacy settings
        }
    );
    
    // Connected accounts
    let connected_accounts = Components::setting_row(
        "Connected Accounts",
        Components::nav_button(
            "Manage",
            Screen::Settings,
            false,
            |_state| {
                // TODO: Show connected accounts
            }
        )
    );
    
    // Change password
    let change_password = Components::action_button(
        "Change Password",
        AppColor::Warning,
        |_state| {
            // TODO: Show password change dialog
        }
    );
    
    // Sign out
    let signout_button = Components::action_button(
        "Sign Out",
        AppColor::Secondary,
        |state| {
            state.user = None;
            state.auth_token = None;
            state.navigate(Screen::Login);
        }
    );
    
    // Delete account
    let delete_button = Components::error_banner(
        "Delete Account",
        |_state| {
            // TODO: Confirm and delete account
        }
    );
    
    Components::settings_section(
        "Account Management",
        vec![export_button, privacy_button, connected_accounts, change_password, signout_button, delete_button]
    )
}

fn about_section(state: &AppState) -> ComponentOutput {
    // Version info
    let version = Components::setting_row(
        "Version",
        Components::label("0.0.11")
    );
    
    // Build info
    let build = Components::setting_row(
        "Build",
        Components::label(if cfg!(debug_assertions) { "Debug" } else { "Release" })
    );
    
    // Easter egg hint
    let easter_egg_hint = if state.easter_egg_manager.crab.discovery_count > 0 {
        Components::setting_row(
            "🦀 Little Crab",
            Components::label(&format!("Found {} times!", state.easter_egg_manager.crab.discovery_count))
        )
    } else {
        Components::empty()
    };
    
    // Links
    let help_link = Components::action_button(
        "Help & Support",
        AppColor::Primary,
        |_state| {
            // TODO: Open help docs
        }
    );
    
    let feedback_link = Components::action_button(
        "Send Feedback",
        AppColor::Success,
        |_state| {
            // TODO: Open feedback form
        }
    );
    
    let terms_link = Components::action_button(
        "Terms of Service",
        AppColor::Info,
        |_state| {
            // TODO: Open terms
        }
    );
    
    let privacy_link = Components::action_button(
        "Privacy Policy",
        AppColor::Info,
        |_state| {
            // TODO: Open privacy policy
        }
    );
    
    // Credits
    let credits = Components::label("Made with ❤️ by the ABCDEEZ Team");
    
    Components::settings_section(
        "About",
        vec![version, build, easter_egg_hint, help_link, feedback_link, terms_link, privacy_link, credits]
    )
}

fn map_theme_mode_to_app_theme(mode: ThemeMode) -> AppTheme {
    match mode {
        ThemeMode::Light => AppTheme::Light,
        ThemeMode::Dark => AppTheme::Dark,
        ThemeMode::Auto => AppTheme::Auto,
    }
}

fn map_app_theme_to_theme_mode(theme: AppTheme) -> ThemeMode {
    match theme {
        AppTheme::Light => ThemeMode::Light,
        AppTheme::Dark => ThemeMode::Dark,
        AppTheme::Auto => ThemeMode::Auto,
    }
}