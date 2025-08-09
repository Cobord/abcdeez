-- Gamification System Migration
-- Adds support for achievements, leaderboards, XP, and user progression

-- User gamification profile
CREATE TABLE IF NOT EXISTS user_gamification (
    user_id BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    level INTEGER NOT NULL DEFAULT 1,
    experience INTEGER NOT NULL DEFAULT 0,
    total_points INTEGER NOT NULL DEFAULT 0,
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    last_activity_date DATE,
    rank_title TEXT DEFAULT 'Novice',
    rank_tier INTEGER DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Achievements definition and tracking
CREATE TABLE IF NOT EXISTS achievements (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL, -- Streak, Accuracy, Speed, Volume, Mastery, Explorer, Social, Special
    rarity TEXT NOT NULL, -- Common, Uncommon, Rare, Epic, Legendary
    points INTEGER NOT NULL DEFAULT 10,
    unlocked BOOLEAN NOT NULL DEFAULT FALSE,
    unlocked_at TIMESTAMP,
    progress REAL NOT NULL DEFAULT 0.0, -- 0.0 to 1.0
    requirement_data TEXT, -- JSON with specific requirements
    UNIQUE(user_id, achievement_id)
);

-- Leaderboards
CREATE TABLE IF NOT EXISTS leaderboards (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    leaderboard_type TEXT NOT NULL, -- weekly, monthly, all_time, domain_specific
    score INTEGER NOT NULL DEFAULT 0,
    rank INTEGER,
    week_number INTEGER, -- For weekly leaderboards
    month_year TEXT, -- For monthly leaderboards (YYYY-MM)
    domain TEXT, -- For domain-specific leaderboards
    metadata TEXT, -- JSON with additional data
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, leaderboard_type, week_number, month_year, domain)
);

-- User badges (special achievements)
CREATE TABLE IF NOT EXISTS badges (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    icon_url TEXT,
    earned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT, -- JSON with badge-specific data
    UNIQUE(user_id, badge_id)
);

-- Weekly goals and challenges
CREATE TABLE IF NOT EXISTS weekly_goals (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_number INTEGER NOT NULL,
    year INTEGER NOT NULL,
    tasks_goal INTEGER NOT NULL DEFAULT 100,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    accuracy_goal REAL NOT NULL DEFAULT 0.8,
    current_accuracy REAL NOT NULL DEFAULT 0.0,
    streak_goal INTEGER NOT NULL DEFAULT 7,
    current_streak INTEGER NOT NULL DEFAULT 0,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    reward_claimed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, week_number, year)
);

-- Power-ups and boosts
CREATE TABLE IF NOT EXISTS power_ups (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    power_up_type TEXT NOT NULL, -- double_xp, hint_boost, time_freeze, etc.
    quantity INTEGER NOT NULL DEFAULT 0,
    active_until TIMESTAMP, -- For time-based power-ups
    metadata TEXT, -- JSON with power-up specific data
    UNIQUE(user_id, power_up_type)
);

-- XP transactions log (for tracking XP gains)
CREATE TABLE IF NOT EXISTS xp_transactions (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount INTEGER NOT NULL,
    reason TEXT NOT NULL, -- task_completion, achievement_unlock, daily_bonus, etc.
    source_id TEXT, -- Reference to the source (task_id, achievement_id, etc.)
    multiplier REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Daily challenges
CREATE TABLE IF NOT EXISTS daily_challenges (
    id BLOB PRIMARY KEY,
    challenge_date DATE NOT NULL,
    challenge_type TEXT NOT NULL,
    requirements TEXT NOT NULL, -- JSON with challenge requirements
    reward_xp INTEGER NOT NULL DEFAULT 50,
    reward_points INTEGER NOT NULL DEFAULT 10,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE(challenge_date, challenge_type)
);

-- User daily challenge progress
CREATE TABLE IF NOT EXISTS user_daily_challenges (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    challenge_id BLOB NOT NULL REFERENCES daily_challenges(id) ON DELETE CASCADE,
    progress REAL NOT NULL DEFAULT 0.0,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    completed_at TIMESTAMP,
    reward_claimed BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, challenge_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_achievements_user ON achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_achievements_unlocked ON achievements(unlocked);
CREATE INDEX IF NOT EXISTS idx_achievements_category ON achievements(category);
CREATE INDEX IF NOT EXISTS idx_leaderboards_user ON leaderboards(user_id);
CREATE INDEX IF NOT EXISTS idx_leaderboards_type ON leaderboards(leaderboard_type);
CREATE INDEX IF NOT EXISTS idx_leaderboards_rank ON leaderboards(rank);
CREATE INDEX IF NOT EXISTS idx_badges_user ON badges(user_id);
CREATE INDEX IF NOT EXISTS idx_weekly_goals_user ON weekly_goals(user_id);
CREATE INDEX IF NOT EXISTS idx_xp_transactions_user ON xp_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_xp_transactions_created ON xp_transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_user_daily_challenges_user ON user_daily_challenges(user_id);

-- Initialize default achievements
INSERT OR IGNORE INTO daily_challenges (id, challenge_date, challenge_type, requirements, reward_xp, reward_points)
VALUES 
    (randomblob(16), date('now'), 'accuracy_master', '{"min_accuracy": 0.9, "min_tasks": 20}', 100, 20),
    (randomblob(16), date('now'), 'speed_demon', '{"max_avg_time_ms": 2000, "min_tasks": 15}', 75, 15),
    (randomblob(16), date('now'), 'consistent_practice', '{"min_tasks": 50}', 50, 10);