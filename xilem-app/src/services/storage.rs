// Local storage service using SQLite for offline data persistence

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{PendingResponse, Response, Session, User};

#[derive(Clone)]
pub struct StorageService {
    conn: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for StorageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageService")
            .field("conn", &"<sqlite connection>")
            .finish()
    }
}

impl StorageService {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("abcdeez");
            std::fs::create_dir_all(&path).ok();
            path.push("local.db");
            path
        });

        let conn = Connection::open(path)?;
        
        // Initialize database schema
        Self::init_schema(&conn)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        // User table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT NOT NULL,
                created_at TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        // Sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                learner_id TEXT NOT NULL,
                topology_type TEXT NOT NULL,
                topology_data TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                status TEXT NOT NULL,
                summary TEXT
            )",
            [],
        )?;

        // Pending responses table for offline sync
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pending_responses (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                sequence_number INTEGER NOT NULL,
                task_type TEXT NOT NULL,
                task_data TEXT NOT NULL,
                user_answer TEXT,
                correct INTEGER NOT NULL,
                response_time_ms INTEGER NOT NULL,
                hint_level INTEGER,
                timestamp TEXT NOT NULL,
                synced INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Learner model cache
        conn.execute(
            "CREATE TABLE IF NOT EXISTS learner_models (
                id TEXT PRIMARY KEY,
                learner_id TEXT NOT NULL UNIQUE,
                model_data TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Generic key-value store for various data
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Task cache
        conn.execute(
            "CREATE TABLE IF NOT EXISTS task_cache (
                id TEXT PRIMARY KEY,
                task_key TEXT NOT NULL UNIQUE,
                task_data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT
            )",
            [],
        )?;

        // Gamification data
        conn.execute(
            "CREATE TABLE IF NOT EXISTS gamification (
                user_id TEXT PRIMARY KEY,
                level INTEGER NOT NULL,
                xp INTEGER NOT NULL,
                achievements TEXT NOT NULL,
                streak_current INTEGER NOT NULL,
                streak_longest INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pending_responses_session 
             ON pending_responses(session_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pending_responses_synced 
             ON pending_responses(synced)",
            [],
        )?;

        Ok(())
    }

    // User operations
    pub async fn save_user(&self, user: &User) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO users (id, username, email, created_at, metadata) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                user.id.to_string(),
                user.username,
                user.email,
                user.created_at.to_rfc3339(),
                serde_json::to_string(&user.metadata)?
            ],
        )?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, username, email, created_at, metadata FROM users WHERE id = ?1"
        )?;
        
        let user = stmt
            .query_row(params![user_id.to_string()], |row| {
                Ok(User {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    username: row.get(1)?,
                    email: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    metadata: serde_json::from_str(&row.get::<_, String>(4)?).ok(),
                })
            })
            .optional()?;
        
        Ok(user)
    }

    // Session operations
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO sessions 
             (id, learner_id, topology_type, topology_data, start_time, end_time, status, summary) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session.id.to_string(),
                session.learner_id.to_string(),
                session.topology_type,
                serde_json::to_string(&session.topology_data)?,
                session.start_time.to_rfc3339(),
                session.end_time.map(|t| t.to_rfc3339()),
                session.status,
                session.summary.as_ref().map(|s| serde_json::to_string(s).unwrap_or_default())
            ],
        )?;
        Ok(())
    }

    // Pending response operations for offline sync
    pub async fn save_pending_response(&self, response: &PendingResponse) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO pending_responses 
             (id, session_id, sequence_number, task_type, task_data, user_answer, 
              correct, response_time_ms, hint_level, timestamp, synced) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                response.id.to_string(),
                response.session_id.to_string(),
                response.sequence_number,
                response.task_type,
                serde_json::to_string(&response.task_data)?,
                response.user_answer,
                response.correct as i32,
                response.response_time_ms as i64,
                response.hint_level.map(|h| h as i32),
                response.timestamp.to_rfc3339(),
                0
            ],
        )?;
        Ok(())
    }

    pub async fn get_all_pending_responses(&self) -> Result<Vec<PendingResponse>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, sequence_number, task_type, task_data, user_answer, 
                    correct, response_time_ms, hint_level, timestamp 
             FROM pending_responses 
             WHERE synced = 0 
             ORDER BY timestamp ASC"
        )?;
        
        let responses = stmt
            .query_map([], |row| {
                Ok(PendingResponse {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    session_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    sequence_number: row.get(2)?,
                    task_type: row.get(3)?,
                    task_data: serde_json::from_str(&row.get::<_, String>(4)?).unwrap(),
                    user_answer: row.get(5)?,
                    correct: row.get::<_, i32>(6)? != 0,
                    response_time_ms: row.get::<_, i64>(7)? as u128,
                    hint_level: row.get::<_, Option<i32>>(8)?.map(|h| h as u32),
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(responses)
    }

    pub async fn delete_pending_response(&self, id: &Uuid) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM pending_responses WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    pub async fn mark_responses_as_synced(&self, ids: &[Uuid]) -> Result<()> {
        let conn = self.conn.lock().await;
        let tx = conn.unchecked_transaction()?;
        
        for id in ids {
            tx.execute(
                "UPDATE pending_responses SET synced = 1 WHERE id = ?1",
                params![id.to_string()],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    // Learner model operations
    pub async fn save_learner_model(&self, learner_id: &Uuid, model: &abcdeez_core::learning::LearnerModel) -> Result<()> {
        let conn = self.conn.lock().await;
        let model_json = serde_json::to_string(model)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO learner_models (id, learner_id, model_data, updated_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![
                Uuid::new_v4().to_string(),
                learner_id.to_string(),
                model_json,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn get_learner_model(&self, learner_id: &Uuid) -> Result<Option<abcdeez_core::learning::LearnerModel>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT model_data FROM learner_models WHERE learner_id = ?1"
        )?;
        
        let model_json = stmt
            .query_row(params![learner_id.to_string()], |row| {
                row.get::<_, String>(0)
            })
            .optional()?;
        
        if let Some(json) = model_json {
            Ok(Some(serde_json::from_str(&json)?))
        } else {
            Ok(None)
        }
    }

    // Generic key-value operations
    pub async fn save_json(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, serde_json::to_string(value)?, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub async fn get_json(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT value FROM kv_store WHERE key = ?1")?;
        
        let value_str = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .optional()?;
        
        if let Some(str) = value_str {
            Ok(Some(serde_json::from_str(&str)?))
        } else {
            Ok(None)
        }
    }

    // Task cache operations
    pub async fn cache_task(&self, key: &str, task: &serde_json::Value, ttl_seconds: Option<i64>) -> Result<()> {
        let conn = self.conn.lock().await;
        let expires_at = ttl_seconds.map(|ttl| {
            (Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339()
        });
        
        conn.execute(
            "INSERT OR REPLACE INTO task_cache (id, task_key, task_data, created_at, expires_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Uuid::new_v4().to_string(),
                key,
                serde_json::to_string(task)?,
                Utc::now().to_rfc3339(),
                expires_at
            ],
        )?;
        Ok(())
    }

    pub async fn get_cached_task(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock().await;
        
        // Clean up expired tasks first
        conn.execute(
            "DELETE FROM task_cache WHERE expires_at IS NOT NULL AND expires_at < ?1",
            params![Utc::now().to_rfc3339()],
        )?;
        
        let mut stmt = conn.prepare(
            "SELECT task_data FROM task_cache WHERE task_key = ?1"
        )?;
        
        let task_str = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .optional()?;
        
        if let Some(str) = task_str {
            Ok(Some(serde_json::from_str(&str)?))
        } else {
            Ok(None)
        }
    }

    // Gamification operations
    pub async fn save_gamification_data(&self, user_id: &Uuid, data: &GamificationData) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO gamification 
             (user_id, level, xp, achievements, streak_current, streak_longest, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                user_id.to_string(),
                data.level,
                data.xp,
                serde_json::to_string(&data.achievements)?,
                data.streak_current,
                data.streak_longest,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn get_gamification_data(&self, user_id: &Uuid) -> Result<Option<GamificationData>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT level, xp, achievements, streak_current, streak_longest 
             FROM gamification WHERE user_id = ?1"
        )?;
        
        let data = stmt
            .query_row(params![user_id.to_string()], |row| {
                Ok(GamificationData {
                    level: row.get(0)?,
                    xp: row.get(1)?,
                    achievements: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    streak_current: row.get(3)?,
                    streak_longest: row.get(4)?,
                })
            })
            .optional()?;
        
        Ok(data)
    }

    // Cleanup operations
    pub async fn cleanup_old_data(&self, days_to_keep: i64) -> Result<()> {
        let conn = self.conn.lock().await;
        let cutoff_date = (Utc::now() - chrono::Duration::days(days_to_keep)).to_rfc3339();
        
        // Clean up old synced responses
        conn.execute(
            "DELETE FROM pending_responses WHERE synced = 1 AND timestamp < ?1",
            params![cutoff_date],
        )?;
        
        // Clean up expired task cache
        conn.execute(
            "DELETE FROM task_cache WHERE expires_at IS NOT NULL AND expires_at < ?1",
            params![Utc::now().to_rfc3339()],
        )?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamificationData {
    pub level: i32,
    pub xp: i32,
    pub achievements: Vec<String>,
    pub streak_current: i32,
    pub streak_longest: i32,
}