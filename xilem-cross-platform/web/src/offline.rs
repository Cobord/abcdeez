use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};
use serde::{Deserialize, Serialize};
use serde_json;

pub struct OfflineStorage {
    storage: Storage,
}

impl OfflineStorage {
    pub fn new() -> Result<Self, JsValue> {
        let storage = window()
            .ok_or("No window object")?
            .local_storage()
            .map_err(|_| "Failed to access localStorage")?
            .ok_or("localStorage not available")?;
        
        Ok(Self { storage })
    }
    
    pub fn clone(&self) -> Result<Self, JsValue> {
        Self::new()
    }
    
    pub fn set_item(&self, key: &str, value: &str) -> Result<(), JsValue> {
        self.storage.set_item(key, value)
    }
    
    pub fn get_item(&self, key: &str) -> Result<Option<String>, JsValue> {
        self.storage.get_item(key)
    }
    
    pub fn remove_item(&self, key: &str) -> Result<(), JsValue> {
        self.storage.remove_item(key)
    }
    
    pub fn clear(&self) -> Result<(), JsValue> {
        self.storage.clear()
    }
    
    pub fn save_json<T: Serialize>(&self, key: &str, data: &T) -> Result<(), JsValue> {
        let json = serde_json::to_string(data)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        self.set_item(key, &json)
    }
    
    pub fn load_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, JsValue> {
        match self.get_item(key)? {
            Some(json) => {
                let data = serde_json::from_str(&json)
                    .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
                Ok(Some(data))
            },
            None => Ok(None)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueue {
    items: Vec<SyncItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncItem {
    pub id: String,
    pub timestamp: f64,
    pub operation: SyncOperation,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperation {
    Create,
    Update,
    Delete,
}

impl SyncQueue {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn add(&mut self, operation: SyncOperation, data: serde_json::Value) {
        let item = SyncItem {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: js_sys::Date::now(),
            operation,
            data,
        };
        self.items.push(item);
    }
    
    pub fn clear(&mut self) {
        self.items.clear();
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    pub fn take_all(&mut self) -> Vec<SyncItem> {
        std::mem::take(&mut self.items)
    }
}

pub struct CacheManager {
    storage: OfflineStorage,
}

impl CacheManager {
    pub fn new() -> Result<Self, JsValue> {
        Ok(Self {
            storage: OfflineStorage::new()?,
        })
    }
    
    pub fn cache_user_data(&self, user_id: &str, data: &serde_json::Value) -> Result<(), JsValue> {
        let key = format!("user_data_{}", user_id);
        self.storage.save_json(&key, data)
    }
    
    pub fn get_cached_user_data(&self, user_id: &str) -> Result<Option<serde_json::Value>, JsValue> {
        let key = format!("user_data_{}", user_id);
        self.storage.load_json(&key)
    }
    
    pub fn cache_session(&self, session_id: &str, data: &serde_json::Value) -> Result<(), JsValue> {
        let key = format!("session_{}", session_id);
        self.storage.save_json(&key, data)
    }
    
    pub fn get_cached_session(&self, session_id: &str) -> Result<Option<serde_json::Value>, JsValue> {
        let key = format!("session_{}", session_id);
        self.storage.load_json(&key)
    }
    
    pub fn save_sync_queue(&self, queue: &SyncQueue) -> Result<(), JsValue> {
        self.storage.save_json("sync_queue", queue)
    }
    
    pub fn load_sync_queue(&self) -> Result<SyncQueue, JsValue> {
        match self.storage.load_json::<SyncQueue>("sync_queue")? {
            Some(queue) => Ok(queue),
            None => Ok(SyncQueue::new())
        }
    }
}