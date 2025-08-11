/*!
In-memory cache module with a tiny Redis-like API used by the codebase.

This replaces the need for a real Redis server at development time while
keeping the existing call sites unchanged:

- Create/clone a connection manager:
    let mut conn = state.cache_conn.clone();

- Execute commands:
    crate::cache::cmd("GET").arg("some:key").query_async::<String>(&mut conn).await?;
    crate::cache::cmd("SETEX").arg("some:key").arg(300).arg("value").query_async::<()>(&mut conn).await?;
    crate::cache::cmd("DEL").arg("some:key").query_async::<()>(&mut conn).await?;

Supported commands:
- GET key -> returns String (error if missing) or Option<String> (None if missing)
- SETEX key ttl_seconds value -> returns "OK" (String) or unit ()
- DEL key -> returns unit ()

Data is stored in-memory and TTLs are enforced on read.
*/

use std::{
    collections::HashMap,
    fmt,
    str::FromStr,
    time::{Duration, Instant},
};

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum CacheError {
    NotFound,
    InvalidCommand(String),
    InvalidArgs(&'static str),
    ParseError(&'static str),
    Internal(&'static str),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::NotFound => write!(f, "not found"),
            CacheError::InvalidCommand(c) => write!(f, "invalid command: {}", c),
            CacheError::InvalidArgs(m) => write!(f, "invalid args: {}", m),
            CacheError::ParseError(m) => write!(f, "parse error: {}", m),
            CacheError::Internal(m) => write!(f, "internal error: {}", m),
        }
    }
}

impl std::error::Error for CacheError {}

#[derive(Clone)]
pub struct ConnectionManager {
    inner: Arc<Mutex<InMemoryCache>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InMemoryCache::new())),
        }
    }
}

#[derive(Default)]
struct InMemoryCache {
    map: Arc<Mutex<HashMap<String, Entry>>>,
}

impl InMemoryCache {
    fn new() -> Self {
        let cache = Self {
            map: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start background cleanup task
        let map_clone = Arc::clone(&cache.map);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Clean every 5 minutes
            loop {
                interval.tick().await;
                let mut map = map_clone.lock().await;
                map.retain(|_, entry| !entry.is_expired());
            }
        });
        
        cache
    }

    async fn get(&self, key: &str) -> Option<String> {
        let mut map = self.map.lock().await;
        // Purge expired values on access
        if let Some(entry) = map.get(key) {
            if entry.is_expired() {
                map.remove(key);
                return None;
            }
            Some(entry.value.clone())
        } else {
            None
        }
    }

    async fn set_ex(&self, key: String, ttl_secs: u64, value: String) {
        let expires_at = Instant::now().checked_add(Duration::from_secs(ttl_secs));
        let mut map = self.map.lock().await;
        map.insert(key, Entry { value, expires_at });
    }

    async fn del(&self, key: &str) -> bool {
        let mut map = self.map.lock().await;
        map.remove(key).is_some()
    }

    async fn exists(&self, key: &str) -> bool {
        let mut map = self.map.lock().await;
        // Check existence and remove if expired
        if let Some(entry) = map.get(key) {
            if entry.is_expired() {
                map.remove(key);
                return false;
            }
            true
        } else {
            false
        }
    }

    async fn incr(&self, key: &str) -> Result<i64, CacheError> {
        let mut map = self.map.lock().await;
        // Get current value or default to 0
        let current = if let Some(entry) = map.get(key) {
            if entry.is_expired() {
                map.remove(key);
                0
            } else {
                entry.value.parse::<i64>().unwrap_or(0)
            }
        } else {
            0
        };

        let new_value = current + 1;
        // Store with no expiry (will be set by EXPIRE if needed)
        map.insert(
            key.to_string(),
            Entry {
                value: new_value.to_string(),
                expires_at: None,
            },
        );
        Ok(new_value)
    }

    async fn expire(&self, key: &str, ttl_secs: u64) -> bool {
        let mut map = self.map.lock().await;
        if let Some(entry) = map.get_mut(key) {
            entry.expires_at = Instant::now().checked_add(Duration::from_secs(ttl_secs));
            true
        } else {
            false
        }
    }
}

struct Entry {
    value: String,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at
            .map(|t| Instant::now() >= t)
            .unwrap_or(false)
    }
}

/// Command builder mirroring the small surface we use from redis::cmd.
pub struct Command {
    name: String,
    args: Vec<String>,
}

/// Entry point that mirrors redis::cmd("...").
pub fn cmd(name: &str) -> Command {
    Command {
        name: name.to_string(),
        args: Vec::new(),
    }
}

/// Convert arguments passed to Command::arg into Strings.
pub trait ToArg {
    fn to_arg(self) -> String;
}

impl ToArg for String {
    fn to_arg(self) -> String {
        self
    }
}

impl ToArg for &str {
    fn to_arg(self) -> String {
        self.to_string()
    }
}

impl ToArg for i32 {
    fn to_arg(self) -> String {
        self.to_string()
    }
}

impl ToArg for i64 {
    fn to_arg(self) -> String {
        self.to_string()
    }
}

impl ToArg for u64 {
    fn to_arg(self) -> String {
        self.to_string()
    }
}

impl ToArg for usize {
    fn to_arg(self) -> String {
        self.to_string()
    }
}

impl ToArg for f64 {
    fn to_arg(self) -> String {
        self.to_string()
    }
}

impl ToArg for &String {
    fn to_arg(self) -> String {
        self.clone()
    }
}

impl Command {
    pub fn arg<T: ToArg>(mut self, value: T) -> Self {
        self.args.push(value.to_arg());
        self
    }

    /// Execute the command against the in-memory cache.
    ///
    /// This mirrors the generic signature used at call sites in the codebase.
    pub async fn query_async<T>(self, conn: &mut ConnectionManager) -> Result<T, CacheError>
    where
        T: FromCacheResponse,
    {
        let name = self.name.to_uppercase();
        match name.as_str() {
            "PING" => {
                // Respond with a simple heartbeat. Most callers only check is_ok().
                T::from_get(Some("PONG".to_string()))
            }
            "SET" => {
                if self.args.len() != 2 {
                    return Err(CacheError::InvalidArgs(
                        "SET requires 2 arguments: key value",
                    ));
                }
                let key = self.args[0].clone();
                let value = self.args[1].clone();

                let mut guard = conn.inner.lock().await;
                // Store without expiry by using a very long TTL (~100 years)
                guard.set_ex(key, 31_536_000 * 100, value);
                T::from_set()
            }
            "GET" => {
                if self.args.len() != 1 {
                    return Err(CacheError::InvalidArgs("GET requires 1 argument: key"));
                }
                let key = &self.args[0];
                let mut guard = conn.inner.lock().await;
                let val = guard.get(key).await;
                T::from_get(val)
            }
            "SETEX" => {
                if self.args.len() != 3 {
                    return Err(CacheError::InvalidArgs(
                        "SETEX requires 3 arguments: key ttl_seconds value",
                    ));
                }
                let key = self.args[0].clone();
                let ttl_secs = u64::from_str(&self.args[1])
                    .map_err(|_| CacheError::ParseError("ttl_seconds must be an integer"))?;
                let value = self.args[2].clone();

                let guard = conn.inner.lock().await;
                guard.set_ex(key, ttl_secs, value).await;
                T::from_set()
            }
            "DEL" => {
                if self.args.len() != 1 {
                    return Err(CacheError::InvalidArgs("DEL requires 1 argument: key"));
                }
                let key = &self.args[0];
                let guard = conn.inner.lock().await;
                let deleted = guard.del(key).await;
                T::from_del(deleted)
            }
            "EXISTS" => {
                if self.args.len() != 1 {
                    return Err(CacheError::InvalidArgs("EXISTS requires 1 argument: key"));
                }
                let key = &self.args[0];
                let guard = conn.inner.lock().await;
                let exists = guard.exists(key).await;
                T::from_exists(exists)
            }
            "INCR" => {
                if self.args.len() != 1 {
                    return Err(CacheError::InvalidArgs("INCR requires 1 argument: key"));
                }
                let key = &self.args[0];
                let guard = conn.inner.lock().await;
                let value = guard.incr(key).await?;
                T::from_incr(value)
            }
            "EXPIRE" => {
                if self.args.len() != 2 {
                    return Err(CacheError::InvalidArgs(
                        "EXPIRE requires 2 arguments: key ttl_seconds",
                    ));
                }
                let key = &self.args[0];
                let ttl_secs = u64::from_str(&self.args[1])
                    .map_err(|_| CacheError::ParseError("ttl_seconds must be an integer"))?;
                let guard = conn.inner.lock().await;
                let success = guard.expire(key, ttl_secs).await;
                T::from_expire(success)
            }
            other => Err(CacheError::InvalidCommand(other.to_string())),
        }
    }
}

/// Adapts the internal cache results for different expected return types
/// at the call sites (String, Option<String>, unit).
pub trait FromCacheResponse: Sized {
    fn from_get(v: Option<String>) -> Result<Self, CacheError>;
    fn from_set() -> Result<Self, CacheError>;
    fn from_del(_deleted: bool) -> Result<Self, CacheError>;
    fn from_exists(_exists: bool) -> Result<Self, CacheError> {
        Err(CacheError::InvalidCommand(
            "EXISTS not supported for this type".to_string(),
        ))
    }
    fn from_incr(_value: i64) -> Result<Self, CacheError> {
        Err(CacheError::InvalidCommand(
            "INCR not supported for this type".to_string(),
        ))
    }
    fn from_expire(_success: bool) -> Result<Self, CacheError> {
        Err(CacheError::InvalidCommand(
            "EXPIRE not supported for this type".to_string(),
        ))
    }
}

impl FromCacheResponse for String {
    fn from_get(v: Option<String>) -> Result<Self, CacheError> {
        v.ok_or(CacheError::NotFound)
    }

    fn from_set() -> Result<Self, CacheError> {
        Ok("OK".to_string())
    }

    fn from_del(_deleted: bool) -> Result<Self, CacheError> {
        Ok("OK".to_string())
    }

    fn from_exists(exists: bool) -> Result<Self, CacheError> {
        Ok(if exists {
            "1".to_string()
        } else {
            "0".to_string()
        })
    }

    fn from_incr(value: i64) -> Result<Self, CacheError> {
        Ok(value.to_string())
    }

    fn from_expire(success: bool) -> Result<Self, CacheError> {
        Ok(if success {
            "1".to_string()
        } else {
            "0".to_string()
        })
    }
}

impl FromCacheResponse for () {
    fn from_get(_v: Option<String>) -> Result<Self, CacheError> {
        // Unused in codebase; return Ok(())
        Ok(())
    }

    fn from_set() -> Result<Self, CacheError> {
        Ok(())
    }

    fn from_del(_deleted: bool) -> Result<Self, CacheError> {
        Ok(())
    }

    fn from_expire(_success: bool) -> Result<Self, CacheError> {
        Ok(())
    }
}

impl FromCacheResponse for Option<String> {
    fn from_get(v: Option<String>) -> Result<Self, CacheError> {
        Ok(v)
    }

    fn from_set() -> Result<Self, CacheError> {
        Ok(None)
    }

    fn from_del(_deleted: bool) -> Result<Self, CacheError> {
        Ok(None)
    }
}

/// Helper to create a fresh connection manager (not strictly required,
/// but convenient if needed externally).
pub fn connection_manager() -> ConnectionManager {
    ConnectionManager::new()
}
