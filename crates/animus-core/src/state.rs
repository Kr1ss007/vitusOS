//! Reactive StateManager with typed key-value storage and observer patterns.
//!
//! Aligned with FIX3-01 (get_as safe access) and FIX3-02 (Screen geometry keys).

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::event_bus::EventBus;
use crate::events::AEEvent;

pub mod state_keys {
    pub const SCREEN_WIDTH: &str = "screen_width";
    pub const SCREEN_HEIGHT: &str = "screen_height";
    pub const SCREEN_DPI_SCALE: &str = "screen_dpi_scale";
    pub const ACTIVE_WORKSPACE: &str = "active_workspace";
    pub const ACCENT_COLOR: &str = "accent_color";
    pub const REDUCED_MOTION: &str = "reduced_motion";
    pub const DARK_MODE: &str = "dark_mode";
}

/// Reactive state manager with key-value store and granular change observation.
pub struct StateManager {
    store: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    bus: EventBus,
}

impl StateManager {
    pub fn new(bus: EventBus) -> Self {
        let manager = Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            bus,
        };
        // Set default screen geometry
        manager.set(state_keys::SCREEN_WIDTH, 1920.0f32);
        manager.set(state_keys::SCREEN_HEIGHT, 1080.0f32);
        manager.set(state_keys::SCREEN_DPI_SCALE, 1.0f32);
        manager.set(state_keys::REDUCED_MOTION, false);
        manager.set(state_keys::DARK_MODE, true);
        manager
    }

    /// Sets a value in the state store and publishes `AEEvent::StateChanged`.
    pub fn set<T: Send + Sync + 'static>(&self, key: impl Into<String>, value: T) {
        let key_str = key.into();
        {
            let mut store = self.store.write();
            store.insert(key_str.clone(), Arc::new(value));
        }
        self.bus.publish(AEEvent::StateChanged { key: key_str });
    }

    /// Retrieves a cloned value from the store if it exists and type matches.
    pub fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        let store = self.store.read();
        store.get(key).and_then(|val| val.downcast_ref::<T>().cloned())
    }

    /// Safe getter returning default value if key is missing or type mismatch (FIX3-01).
    pub fn get_as<T: Clone + Send + Sync + 'static>(&self, key: &str, default: T) -> T {
        self.get::<T>(key).unwrap_or(default)
    }

    /// Observes state changes for a specific key.
    pub fn observe_state<T, F>(&self, key: impl Into<String>, handler: F) -> u64
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        let target_key = key.into();
        let store = Arc::clone(&self.store);
        let target_key_clone = target_key.clone();

        if let Some(initial_val) = self.get::<T>(&target_key) {
            handler(initial_val);
        }

        self.bus.subscribe(move |event| {
            if let AEEvent::StateChanged { key } = event {
                if key == &target_key_clone {
                    let store_guard = store.read();
                    if let Some(val) = store_guard.get(key).and_then(|v| v.downcast_ref::<T>().cloned()) {
                        handler(val);
                    }
                }
            }
        })
    }

    /// Removes a state key from the store.
    pub fn remove(&self, key: &str) -> bool {
        let mut store = self.store.write();
        store.remove(key).is_some()
    }

    /// Persists core system configuration state to JSON on disk
    pub fn save_to_disk(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut map = HashMap::new();
        if let Some(w) = self.get::<f32>(state_keys::SCREEN_WIDTH) { map.insert(state_keys::SCREEN_WIDTH.to_string(), serde_json::Value::from(w)); }
        if let Some(h) = self.get::<f32>(state_keys::SCREEN_HEIGHT) { map.insert(state_keys::SCREEN_HEIGHT.to_string(), serde_json::Value::from(h)); }
        if let Some(d) = self.get::<bool>(state_keys::DARK_MODE) { map.insert(state_keys::DARK_MODE.to_string(), serde_json::Value::from(d)); }
        if let Some(m) = self.get::<bool>(state_keys::REDUCED_MOTION) { map.insert(state_keys::REDUCED_MOTION.to_string(), serde_json::Value::from(m)); }

        let json = serde_json::to_string_pretty(&map).unwrap_or_default();
        std::fs::write(path, json)
    }

    /// Loads persisted system state from JSON on disk
    pub fn load_from_disk(&self, path: &std::path::Path) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let data = std::fs::read_to_string(path)?;
        if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&data) {
            for (k, v) in map {
                if let Some(b) = v.as_bool() {
                    self.set(k, b);
                } else if let Some(f) = v.as_f64() {
                    self.set(k, f as f32);
                } else if let Some(s) = v.as_str() {
                    self.set(k, s.to_string());
                }
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_get_as_and_defaults() {
        let bus = EventBus::new();
        let state = StateManager::new(bus);

        assert_eq!(state.get_as::<f32>(state_keys::SCREEN_WIDTH, 0.0), 1920.0);
        assert_eq!(state.get_as::<f32>(state_keys::SCREEN_HEIGHT, 0.0), 1080.0);
        assert_eq!(state.get_as::<i32>("non_existent_key", 42), 42);

        state.set("windowDesktop:100", 2);
        assert_eq!(state.get_as::<i32>("windowDesktop:100", 0), 2);
    }
}
