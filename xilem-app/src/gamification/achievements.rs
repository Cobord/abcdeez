// Achievement system
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

use super::profile::GamificationProfile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub icon: String,
    pub points: u32,
    pub rarity: Rarity,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub progress: f32,
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn color(&self) -> &str {
        match self {
            Rarity::Common => "#808080",
            Rarity::Uncommon => "#48bb78",
            Rarity::Rare => "#4299e1",
            Rarity::Epic => "#9f7aea",
            Rarity::Legendary => "#f6ad55",
        }
    }

    pub fn multiplier(&self) -> f32 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 1.5,
            Rarity::Rare => 2.0,
            Rarity::Epic => 3.0,
            Rarity::Legendary => 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AchievementEvent {
    DailyPractice,
    SessionComplete {
        accuracy: f32,
        duration: Duration,
    },
    TaskComplete {
        correct: bool,
        response_time: Duration,
        total_tasks: u32,
    },
    DomainMastered {
        domain: String,
    },
    LeaderboardRankChange {
        old_rank: u32,
        new_rank: u32,
    },
    SpecialCondition {
        condition: String,
    },
}

pub struct AchievementManager {
    achievements: Vec<Rc<RefCell<Achievement>>>,
}

impl AchievementManager {
    pub fn new() -> Self {
        let mut manager = Self {
            achievements: Vec::new(),
        };
        manager.initialize_achievements();
        manager
    }

    fn add_achievement(&mut self, achievement: Achievement) {
        self.achievements.push(Rc::new(RefCell::new(achievement)));
    }

    fn initialize_achievements(&mut self) {
        // Streak achievements
        self.add_achievement(Achievement {
            id: "streak_7".to_string(),
            name: "Week Warrior".to_string(),
            description: "Practice for 7 days in a row".to_string(),
            category: AchievementCategory::Streak,
            icon: "🔥".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "7 day streak".to_string(),
        });

        self.add_achievement(Achievement {
            id: "streak_30".to_string(),
            name: "Monthly Master".to_string(),
            description: "Practice for 30 days in a row".to_string(),
            category: AchievementCategory::Streak,
            icon: "🌟".to_string(),
            points: 200,
            rarity: Rarity::Rare,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "30 day streak".to_string(),
        });

        // Accuracy achievements
        self.add_achievement(Achievement {
            id: "accuracy_90".to_string(),
            name: "Sharp Shooter".to_string(),
            description: "Complete a session with 90% accuracy".to_string(),
            category: AchievementCategory::Accuracy,
            icon: "🎯".to_string(),
            points: 75,
            rarity: Rarity::Uncommon,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "90% accuracy in session".to_string(),
        });

        self.add_achievement(Achievement {
            id: "perfect_session".to_string(),
            name: "Perfectionist".to_string(),
            description: "Complete a session with 100% accuracy".to_string(),
            category: AchievementCategory::Accuracy,
            icon: "💯".to_string(),
            points: 150,
            rarity: Rarity::Epic,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "100% accuracy in session".to_string(),
        });

        // Speed achievements
        self.add_achievement(Achievement {
            id: "speed_demon".to_string(),
            name: "Speed Demon".to_string(),
            description: "Answer 10 questions in under 1 second each".to_string(),
            category: AchievementCategory::Speed,
            icon: "⚡".to_string(),
            points: 100,
            rarity: Rarity::Rare,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "10 fast responses".to_string(),
        });

        // Volume achievements
        self.add_achievement(Achievement {
            id: "century".to_string(),
            name: "Centurion".to_string(),
            description: "Complete 100 tasks".to_string(),
            category: AchievementCategory::Volume,
            icon: "💪".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "100 tasks completed".to_string(),
        });

        self.add_achievement(Achievement {
            id: "millennium".to_string(),
            name: "Task Master".to_string(),
            description: "Complete 1000 tasks".to_string(),
            category: AchievementCategory::Volume,
            icon: "🏅".to_string(),
            points: 250,
            rarity: Rarity::Epic,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "1000 tasks completed".to_string(),
        });

        // Mastery achievements
        self.add_achievement(Achievement {
            id: "alphabet_master".to_string(),
            name: "Alphabet Master".to_string(),
            description: "Master the alphabet domain".to_string(),
            category: AchievementCategory::Mastery,
            icon: "🔤".to_string(),
            points: 100,
            rarity: Rarity::Uncommon,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Complete alphabet with 95% accuracy".to_string(),
        });

        // Explorer achievements
        self.add_achievement(Achievement {
            id: "curious_cat".to_string(),
            name: "Curious Cat".to_string(),
            description: "Try all learning domains".to_string(),
            category: AchievementCategory::Explorer,
            icon: "🐱".to_string(),
            points: 75,
            rarity: Rarity::Uncommon,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Try all 4 domains".to_string(),
        });

        // Social achievements
        self.add_achievement(Achievement {
            id: "leaderboard_top10".to_string(),
            name: "Elite Player".to_string(),
            description: "Reach top 10 on any leaderboard".to_string(),
            category: AchievementCategory::Social,
            icon: "🏆".to_string(),
            points: 200,
            rarity: Rarity::Rare,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Top 10 leaderboard position".to_string(),
        });

        // Special achievements
        self.add_achievement(Achievement {
            id: "early_bird".to_string(),
            name: "Early Bird".to_string(),
            description: "Practice before 6 AM".to_string(),
            category: AchievementCategory::Special,
            icon: "🌅".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Practice before 6 AM".to_string(),
        });

        self.add_achievement(Achievement {
            id: "night_owl".to_string(),
            name: "Night Owl".to_string(),
            description: "Practice after midnight".to_string(),
            category: AchievementCategory::Special,
            icon: "🦉".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Practice after midnight".to_string(),
        });
    }

    pub fn check_achievements(
        &mut self,
        profile: &mut GamificationProfile,
        event: &AchievementEvent,
    ) -> Vec<Achievement> {
        let mut unlocked = Vec::new();

        // FIXME: cloning this vector so we can update_progress later.
        for achievement in self.achievements.clone() {
            let achievement = achievement.clone();
            let mut achievement = achievement.borrow_mut();
            if !achievement.unlocked && self.check_condition(&achievement, event, profile) {
                achievement.unlocked = true;
                achievement.unlocked_at = Some(Utc::now());
                achievement.progress = 1.0;

                // Award points with rarity multiplier
                let points = (achievement.points as f32 * achievement.rarity.multiplier()) as u32;
                profile.total_points += points;
                profile.add_experience(points);

                // Add to profile's achievements
                profile.achievements.push(achievement.clone());

                unlocked.push(achievement.clone());
            } else if !achievement.unlocked {
                // Update progress for incomplete achievements
                self.update_progress(&mut *achievement, event, profile);
            }
        }

        unlocked
    }

    fn check_condition(
        &self,
        achievement: &Achievement,
        event: &AchievementEvent,
        profile: &GamificationProfile,
    ) -> bool {
        match (&achievement.id[..], event) {
            ("streak_7", AchievementEvent::DailyPractice) => profile.streak.current >= 7,
            ("streak_30", AchievementEvent::DailyPractice) => profile.streak.current >= 30,
            ("accuracy_90", AchievementEvent::SessionComplete { accuracy, .. }) => *accuracy >= 0.9,
            ("perfect_session", AchievementEvent::SessionComplete { accuracy, .. }) => {
                *accuracy >= 1.0
            }
            ("century", AchievementEvent::TaskComplete { total_tasks, .. }) => *total_tasks >= 100,
            ("millennium", AchievementEvent::TaskComplete { total_tasks, .. }) => {
                *total_tasks >= 1000
            }
            ("speed_demon", AchievementEvent::TaskComplete { response_time, .. }) => {
                response_time.num_milliseconds() < 1000
            }
            ("alphabet_master", AchievementEvent::DomainMastered { domain }) => {
                domain == "alphabet"
            }
            ("leaderboard_top10", AchievementEvent::LeaderboardRankChange { new_rank, .. }) => {
                *new_rank <= 10
            }
            ("early_bird", AchievementEvent::SpecialCondition { condition }) => {
                condition == "early_morning"
            }
            ("night_owl", AchievementEvent::SpecialCondition { condition }) => {
                condition == "late_night"
            }
            _ => false,
        }
    }

    fn update_progress(
        &mut self,
        achievement: &mut Achievement,
        event: &AchievementEvent,
        profile: &GamificationProfile,
    ) {
        match (&achievement.id[..], event) {
            ("streak_7", AchievementEvent::DailyPractice) => {
                achievement.progress = (profile.streak.current as f32 / 7.0).min(1.0);
            }
            ("streak_30", AchievementEvent::DailyPractice) => {
                achievement.progress = (profile.streak.current as f32 / 30.0).min(1.0);
            }
            ("century", AchievementEvent::TaskComplete { total_tasks, .. }) => {
                achievement.progress = (*total_tasks as f32 / 100.0).min(1.0);
            }
            ("millennium", AchievementEvent::TaskComplete { total_tasks, .. }) => {
                achievement.progress = (*total_tasks as f32 / 1000.0).min(1.0);
            }
            _ => {}
        }
    }

    pub fn get_all_achievements(&self) -> &[Rc<RefCell<Achievement>>] {
        &self.achievements
    }

    pub fn get_unlocked_achievements(&self) -> Vec<Rc<RefCell<Achievement>>> {
        self.achievements
            .iter()
            .filter(|a| a.borrow().unlocked)
            .map(|a| a.clone())
            .collect()
    }
}

impl Achievement {
    pub fn display_progress(&self) -> String {
        if self.unlocked {
            "✅ Unlocked".to_string()
        } else {
            format!("{:.0}%", self.progress * 100.0)
        }
    }

    pub fn rarity_color(&self) -> &str {
        self.rarity.color()
    }
}
