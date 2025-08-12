use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;

pub type AppCallbackFn0 = fn(&mut crate::state::AppState);

// Global registry for zero-arg callbacks by name
pub static APP_CALLBACKS_0: Lazy<RwLock<HashMap<&'static str, AppCallbackFn0>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_callback0(name: &'static str, f: AppCallbackFn0) {
    let mut map = APP_CALLBACKS_0
        .write()
        .expect("Failed to acquire write lock for APP_CALLBACKS_0");
    map.insert(name, f);
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AppCallback0 {
    pub fun_name: Cow<'static, str>,
    #[serde(skip)]
    fun_ptr: Option<AppCallbackFn0>,
}

impl AppCallback0 {
    pub fn new(name: &'static str, ptr: AppCallbackFn0) -> Self {
        Self {
            fun_name: Cow::Borrowed(name),
            fun_ptr: Some(ptr),
        }
    }

    pub fn invoke(&self, state: &mut crate::state::AppState) {
        if let Some(f) = self.fun_ptr {
            (f)(state);
            return;
        }
        let map = APP_CALLBACKS_0
            .read()
            .expect("Failed to acquire read lock for APP_CALLBACKS_0");
        if let Some(f) = map.get(self.fun_name.as_ref()) {
            (f)(state);
        } else {
            panic!("callback '{}' not found in registry", self.fun_name);
        }
    }
}


