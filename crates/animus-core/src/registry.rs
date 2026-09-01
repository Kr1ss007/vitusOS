//! RegistryManager — System-wide configuration registry with schemas (Part 27 of spec).
//!
//! Provides schema-validated key-value persistence, type-safety, and reactive observation.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

use crate::event_bus::EventBus;
use crate::events::AEEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegistryValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySchema {
    pub key: String,
    pub description: String,
    pub default_value: RegistryValue,
    pub is_readonly: bool,
    pub min_int: Option<i64>,
    pub max_int: Option<i64>,
    pub min_float: Option<f64>,
    pub max_float: Option<f64>,
    pub allowed_strings: Option<Vec<String>>,
}

pub struct RegistryManager {
    schemas: Arc<RwLock<HashMap<String, RegistrySchema>>>,
    values: Arc<RwLock<HashMap<String, RegistryValue>>>,
    bus: EventBus,
}

impl RegistryManager {
    pub fn new(bus: EventBus) -> Self {
        let manager = Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            values: Arc::new(RwLock::new(HashMap::new())),
            bus,
        };

        manager.register_default_schemas();
        manager
    }

    /// Registers a key schema with validation constraints.
    pub fn register_schema(&self, schema: RegistrySchema) {
        let key = schema.key.clone();
        let default_val = schema.default_value.clone();

        self.schemas.write().insert(key.clone(), schema);
        if !self.values.read().contains_key(&key) {
            self.values.write().insert(key, default_val);
        }
    }

    /// Validates and sets a registry value.
    pub fn set(&self, key: &str, value: RegistryValue) -> bool {
        let schemas = self.schemas.read();
        if let Some(schema) = schemas.get(key) {
            if schema.is_readonly {
                warn!("Registry: Attempt to modify read-only key '{}'", key);
                return false;
            }

            // Validate against schema constraints
            match (&value, &schema.default_value) {
                (RegistryValue::Bool(_), RegistryValue::Bool(_)) => {}
                (RegistryValue::Int(v), RegistryValue::Int(_)) => {
                    if let Some(min) = schema.min_int {
                        if *v < min { return false; }
                    }
                    if let Some(max) = schema.max_int {
                        if *v > max { return false; }
                    }
                }
                (RegistryValue::Float(v), RegistryValue::Float(_)) => {
                    if let Some(min) = schema.min_float {
                        if *v < min { return false; }
                    }
                    if let Some(max) = schema.max_float {
                        if *v > max { return false; }
                    }
                }
                (RegistryValue::String(v), RegistryValue::String(_)) => {
                    if let Some(allowed) = &schema.allowed_strings {
                        if !allowed.contains(v) { return false; }
                    }
                }
                (RegistryValue::Binary(_), RegistryValue::Binary(_)) => {}
                _ => {
                    warn!("Registry: Type mismatch for key '{}'", key);
                    return false;
                }
            }
        }

        self.values.write().insert(key.to_string(), value);
        self.bus.publish(AEEvent::StateChanged { key: key.to_string() });
        true
    }

    /// Gets a value from the registry.
    pub fn get(&self, key: &str) -> Option<RegistryValue> {
        self.values.read().get(key).cloned()
    }

    /// Gets a boolean with default fallback.
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        match self.get(key) {
            Some(RegistryValue::Bool(v)) => v,
            _ => default,
        }
    }

    /// Gets an integer with default fallback.
    pub fn get_int(&self, key: &str, default: i64) -> i64 {
        match self.get(key) {
            Some(RegistryValue::Int(v)) => v,
            _ => default,
        }
    }

    /// Gets a float with default fallback.
    pub fn get_float(&self, key: &str, default: f64) -> f64 {
        match self.get(key) {
            Some(RegistryValue::Float(v)) => v,
            _ => default,
        }
    }

    /// Gets a string with default fallback.
    pub fn get_string(&self, key: &str, default: impl Into<String>) -> String {
        match self.get(key) {
            Some(RegistryValue::String(v)) => v,
            _ => default.into(),
        }
    }

    fn register_default_schemas(&self) {
        self.register_schema(RegistrySchema {
            key: "com.vitusos.shell.dock.magnify_size".to_string(),
            description: "Maximum dock icon magnification size in pixels".to_string(),
            default_value: RegistryValue::Int(64),
            is_readonly: false,
            min_int: Some(48),
            max_int: Some(128),
            min_float: None,
            max_float: None,
            allowed_strings: None,
        });

        self.register_schema(RegistrySchema {
            key: "com.vitusos.render.glass.blur_intensity".to_string(),
            description: "Global Kawase glass blur multiplier".to_string(),
            default_value: RegistryValue::Float(1.0),
            is_readonly: false,
            min_int: None,
            max_int: None,
            min_float: Some(0.0),
            max_float: Some(2.0),
            allowed_strings: None,
        });

        self.register_schema(RegistrySchema {
            key: "com.vitusos.system.ota_channel".to_string(),
            description: "Active OTA update release channel".to_string(),
            default_value: RegistryValue::String("UpstreamColor".to_string()),
            is_readonly: false,
            min_int: None,
            max_int: None,
            min_float: None,
            max_float: None,
            allowed_strings: Some(vec!["UpstreamColor".to_string(), "UpstreamOne".to_string()]),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_manager_schema_validation() {
        let bus = EventBus::new();
        let reg = RegistryManager::new(bus);

        assert_eq!(reg.get_int("com.vitusos.shell.dock.magnify_size", 0), 64);
        assert_eq!(reg.get_string("com.vitusos.system.ota_channel", ""), "UpstreamColor");

        // Valid update
        assert!(reg.set("com.vitusos.shell.dock.magnify_size", RegistryValue::Int(80)));
        assert_eq!(reg.get_int("com.vitusos.shell.dock.magnify_size", 0), 80);

        // Invalid update (out of bounds)
        assert!(!reg.set("com.vitusos.shell.dock.magnify_size", RegistryValue::Int(200)));
        assert_eq!(reg.get_int("com.vitusos.shell.dock.magnify_size", 0), 80);
    }
}
