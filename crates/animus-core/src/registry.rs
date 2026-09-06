//! RegistryManager -- Handle-Based Object Lifecycle Management (Part 27 of spec).
//!
//! Provides safe, handle-based access to live compositor objects (windows, surfaces,
//! notifications, clients). The core guarantee: `resolve(handle)` returns a valid
//! pointer or `None` -- NEVER a dangling pointer or undefined behavior.
//!
//! `RegHandle` is a `u64` that uniquely identifies a registered object within its
//! registry. Handles are stable across the object's lifetime. When an object is
//! unregistered, all subsequent `resolve()` calls return `None`.
//!
//! Registries:
//! - `WindowRegistry`: RegHandle -> AEWindow*
//! - `SurfaceRegistry`: RegHandle <-> wlr_surface* (reverse lookup)
//! - `NotificationRegistry`: RegHandle -> AENotification*
//! - `ClientRegistry`: ClientRecord with PID tracking, SIGUSR1 reconnect

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

use crate::event_bus::EventBus;
use crate::events::AEEvent;

/// A unique handle identifying a registered object.
pub type RegHandle = u64;
/// Invalid handle sentinel (Part 27).
pub const REG_INVALID: RegHandle = 0;

// ── Config Registry (system preferences, separate from object lifecycle) ──

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

// ── Window Registry ──────────────────────────────────────────────────────────
// RegHandle -> window pointer with focused handle and atomic live count.

/// Trait for objects that can be registered in a window registry.
/// The compositor's AEWindow implements this.
pub trait WindowLike: Send + Sync {
    fn handle(&self) -> RegHandle;
    fn set_handle(&mut self, handle: RegHandle);
    fn is_visible(&self) -> bool;
}

/// Thread-safe registry mapping RegHandle -> WindowLike objects.
/// `forEach` holds the lock for the duration of iteration -- safe on
/// the single-threaded compositor main thread.
pub struct WindowRegistry {
    windows: RwLock<HashMap<RegHandle, Arc<RwLock<Box<dyn WindowLike>>>>>,
    focused_handle: RwLock<Option<RegHandle>>,
    next_handle: AtomicU64,
    live_count: AtomicU64,
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            focused_handle: RwLock::new(None),
            next_handle: AtomicU64::new(1),
            live_count: AtomicU64::new(0),
        }
    }

    /// Registers a window and returns its handle.
    pub fn register(&self, mut window: Box<dyn WindowLike>) -> RegHandle {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        window.set_handle(handle);
        self.windows.write().insert(handle, Arc::new(RwLock::new(window)));
        self.live_count.fetch_add(1, Ordering::Relaxed);
        handle
    }

    /// Unregisters a window. All subsequent resolve() calls return None.
    pub fn unregister(&self, handle: RegHandle) {
        if self.windows.write().remove(&handle).is_some() {
            self.live_count.fetch_sub(1, Ordering::Relaxed);
            if *self.focused_handle.read() == Some(handle) {
                *self.focused_handle.write() = None;
            }
        }
    }

    /// Resolves a handle to a window. Returns None if the handle is invalid
    /// or the window has been unregistered. NEVER returns a dangling pointer.
    pub fn resolve(&self, handle: RegHandle) -> Option<Arc<RwLock<Box<dyn WindowLike>>>> {
        self.windows.read().get(&handle).cloned()
    }

    /// Iterates all live windows with their handles. The lock is held for
    /// the entire iteration -- safe on the single-threaded main thread.
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(RegHandle, &dyn WindowLike),
    {
        let windows = self.windows.read();
        for (handle, win) in windows.iter() {
            let guard = win.read();
            f(*handle, guard.as_ref());
        }
    }

    pub fn focused(&self) -> Option<RegHandle> {
        *self.focused_handle.read()
    }

    pub fn set_focused(&self, handle: Option<RegHandle>) {
        *self.focused_handle.write() = handle;
    }

    pub fn live_count(&self) -> u64 {
        self.live_count.load(Ordering::Relaxed)
    }
}

impl Default for WindowRegistry {
    fn default() -> Self { Self::new() }
}

// ── Surface Registry ─────────────────────────────────────────────────────────
// RegHandle <-> surface_id (u32) with reverse lookup for hit-testing.

pub struct SurfaceRegistry {
    forward: RwLock<HashMap<RegHandle, u32>>,
    reverse: RwLock<HashMap<u32, RegHandle>>,
    next_handle: AtomicU64,
}

impl SurfaceRegistry {
    pub fn new() -> Self {
        Self {
            forward: RwLock::new(HashMap::new()),
            reverse: RwLock::new(HashMap::new()),
            next_handle: AtomicU64::new(1),
        }
    }

    pub fn register(&self, surface_id: u32) -> RegHandle {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        self.forward.write().insert(handle, surface_id);
        self.reverse.write().insert(surface_id, handle);
        handle
    }

    pub fn unregister(&self, handle: RegHandle) {
        if let Some(sid) = self.forward.write().remove(&handle) {
            self.reverse.write().remove(&sid);
        }
    }

    pub fn resolve_by_handle(&self, handle: RegHandle) -> Option<u32> {
        self.forward.read().get(&handle).copied()
    }

    pub fn resolve_by_surface(&self, surface_id: u32) -> Option<RegHandle> {
        self.reverse.read().get(&surface_id).copied()
    }

    pub fn live_count(&self) -> usize {
        self.forward.read().len()
    }
}

impl Default for SurfaceRegistry {
    fn default() -> Self { Self::new() }
}

// ── Notification Registry ────────────────────────────────────────────────────

pub trait NotificationLike: Send + Sync {
    fn handle(&self) -> RegHandle;
    fn is_visible(&self) -> bool;
}

pub struct NotificationRegistry {
    notifications: RwLock<HashMap<RegHandle, Arc<RwLock<Box<dyn NotificationLike>>>>>,
    next_handle: AtomicU64,
}

impl NotificationRegistry {
    pub fn new() -> Self {
        Self {
            notifications: RwLock::new(HashMap::new()),
            next_handle: AtomicU64::new(1),
        }
    }

    pub fn register(&self, notif: Box<dyn NotificationLike>) -> RegHandle {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        self.notifications.write().insert(handle, Arc::new(RwLock::new(notif)));
        handle
    }

    pub fn unregister(&self, handle: RegHandle) {
        self.notifications.write().remove(&handle);
    }

    pub fn resolve(&self, handle: RegHandle) -> Option<Arc<RwLock<Box<dyn NotificationLike>>>> {
        self.notifications.read().get(&handle).cloned()
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(RegHandle, &dyn NotificationLike),
    {
        let notifs = self.notifications.read();
        for (handle, n) in notifs.iter() {
            let guard = n.read();
            f(*handle, guard.as_ref());
        }
    }

    pub fn live_count(&self) -> usize {
        self.notifications.read().len()
    }
}

impl Default for NotificationRegistry {
    fn default() -> Self { Self::new() }
}

// ── Client Registry ─────────────────────────────────────────────────────────
// Tracks Wayland client processes with PID, app_id, and native app flag.
// Supports SIGUSR1 reconnect on compositor restart (Part 27).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRecord {
    pub app_id: String,
    pub pid: u32,
    pub window_handle: RegHandle,
    pub is_native_app: bool,
}

pub struct ClientRegistry {
    clients: RwLock<HashMap<u32, ClientRecord>>, // key: pid
    bus: EventBus,
}

impl ClientRegistry {
    pub fn new(bus: EventBus) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            bus,
        }
    }

    pub fn register_client(&self, record: ClientRecord) {
        let pid = record.pid;
        self.clients.write().insert(pid, record);
    }

    pub fn unregister_client(&self, pid: u32) {
        if let Some(record) = self.clients.write().remove(&pid) {
            self.bus.publish(AEEvent::ClientCrashed {
                app_id: record.app_id,
                pid,
            });
        }
    }

    pub fn get_client(&self, pid: u32) -> Option<ClientRecord> {
        self.clients.read().get(&pid).cloned()
    }

    /// Returns PIDs of all live native app clients.
    /// Used for SIGUSR1 reconnect when the compositor restarts.
    pub fn live_native_client_pids(&self) -> Vec<u32> {
        self.clients
            .read()
            .values()
            .filter(|c| c.is_native_app)
            .map(|c| c.pid)
            .collect()
    }

    /// Checks if a process is still alive via kill(pid, 0).
    pub fn process_exists(pid: u32) -> bool {
        #[cfg(unix)]
        {
            // SAFETY: kill(pid, 0) is safe -- it doesn't send a signal, just checks existence
            unsafe { libc_kill(pid as i32, 0) == 0 }
        }
        #[cfg(not(unix))]
        {
            false
        }
    }

    /// Sends SIGUSR1 to all surviving native clients to reconnect them
    /// to the new compositor instance after a restart.
    pub fn signal_reconnect(&self) {
        #[cfg(unix)]
        {
            let pids = self.live_native_client_pids();
            for pid in pids {
                if Self::process_exists(pid) {
                    // SAFETY: kill is a standard POSIX call, SIGUSR1 is safe
                    let _ = unsafe { libc_kill(pid as i32, libc_SIGUSR1) };
                    info!("ClientRegistry: Sent SIGUSR1 to surviving native client PID {}", pid);
                }
            }
        }
    }

    pub fn live_count(&self) -> usize {
        self.clients.read().len()
    }
}

// Unix signal constants for ClientRegistry (avoid pulling in full nix crate here)
#[cfg(unix)]
extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}
#[cfg(unix)]
unsafe fn libc_kill(pid: i32, sig: i32) -> i32 {
    kill(pid, sig)
}
#[cfg(unix)]
const libc_SIGUSR1: i32 = 10; // SIGUSR1 on Linux x86_64

// ── RegistryManager: owns all four registries + config schemas ───────────────

/// LiveCounts for Supervisor status display (Part 27).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LiveCounts {
    pub windows: u64,
    pub surfaces: usize,
    pub notifications: usize,
    pub clients: usize,
}

pub struct RegistryManager {
    pub windows: WindowRegistry,
    pub surfaces: SurfaceRegistry,
    pub notifications: NotificationRegistry,
    pub clients: ClientRegistry,
    // Config schemas (system preferences) -- kept for backward compat
    config_schemas: RwLock<HashMap<String, RegistrySchema>>,
    config_values: RwLock<HashMap<String, RegistryValue>>,
}

impl RegistryManager {
    pub fn new(bus: EventBus) -> Self {
        let manager = Self {
            windows: WindowRegistry::new(),
            surfaces: SurfaceRegistry::new(),
            notifications: NotificationRegistry::new(),
            clients: ClientRegistry::new(bus.clone()),
            config_schemas: RwLock::new(HashMap::new()),
            config_values: RwLock::new(HashMap::new()),
        };
        manager.register_default_config_schemas();
        manager
    }

    pub fn live_counts(&self) -> LiveCounts {
        LiveCounts {
            windows: self.windows.live_count(),
            surfaces: self.surfaces.live_count(),
            notifications: self.notifications.live_count(),
            clients: self.clients.live_count(),
        }
    }

    // -- Config registry methods (system preferences) --

    pub fn register_config(&self, schema: RegistrySchema) {
        let key = schema.key.clone();
        let default_val = schema.default_value.clone();
        self.config_schemas.write().insert(key.clone(), schema);
        if !self.config_values.read().contains_key(&key) {
            self.config_values.write().insert(key, default_val);
        }
    }

    pub fn set_config(&self, key: &str, value: RegistryValue) -> bool {
        let schemas = self.config_schemas.read();
        if let Some(schema) = schemas.get(key) {
            if schema.is_readonly {
                warn!("Registry: Attempt to modify read-only key '{}'", key);
                return false;
            }
            // Validate type matches
            match (&value, &schema.default_value) {
                (RegistryValue::Bool(_), RegistryValue::Bool(_)) => {}
                (RegistryValue::Int(v), RegistryValue::Int(_)) => {
                    if let Some(min) = schema.min_int { if *v < min { return false; } }
                    if let Some(max) = schema.max_int { if *v > max { return false; } }
                }
                (RegistryValue::Float(v), RegistryValue::Float(_)) => {
                    if let Some(min) = schema.min_float { if *v < min { return false; } }
                    if let Some(max) = schema.max_float { if *v > max { return false; } }
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
        self.config_values.write().insert(key.to_string(), value);
        true
    }

    pub fn get_config(&self, key: &str) -> Option<RegistryValue> {
        self.config_values.read().get(key).cloned()
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        match self.get_config(key) {
            Some(RegistryValue::Bool(v)) => v,
            _ => default,
        }
    }

    pub fn get_int(&self, key: &str, default: i64) -> i64 {
        match self.get_config(key) {
            Some(RegistryValue::Int(v)) => v,
            _ => default,
        }
    }

    pub fn get_float(&self, key: &str, default: f64) -> f64 {
        match self.get_config(key) {
            Some(RegistryValue::Float(v)) => v,
            _ => default,
        }
    }

    pub fn get_string(&self, key: &str, default: impl Into<String>) -> String {
        match self.get_config(key) {
            Some(RegistryValue::String(v)) => v,
            _ => default.into(),
        }
    }

    fn register_default_config_schemas(&self) {
        self.register_config(RegistrySchema {
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

        self.register_config(RegistrySchema {
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

        self.register_config(RegistrySchema {
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
        assert!(reg.set_config("com.vitusos.shell.dock.magnify_size", RegistryValue::Int(80)));
        assert_eq!(reg.get_int("com.vitusos.shell.dock.magnify_size", 0), 80);

        // Invalid update (out of bounds)
        assert!(!reg.set_config("com.vitusos.shell.dock.magnify_size", RegistryValue::Int(200)));
        assert_eq!(reg.get_int("com.vitusos.shell.dock.magnify_size", 0), 80);
    }

    #[test]
    fn test_window_registry_lifecycle() {
        let registry = WindowRegistry::new();
        assert_eq!(registry.live_count(), 0);
        assert!(registry.focused().is_none());
    }

    #[test]
    fn test_surface_registry_round_trip() {
        let registry = SurfaceRegistry::new();
        let handle = registry.register(42);
        assert_eq!(registry.resolve_by_handle(handle), Some(42));
        assert_eq!(registry.resolve_by_surface(42), Some(handle));
        registry.unregister(handle);
        assert_eq!(registry.resolve_by_handle(handle), None);
        assert_eq!(registry.resolve_by_surface(42), None);
    }

    #[test]
    fn test_notification_registry_lifecycle() {
        let registry = NotificationRegistry::new();
        assert_eq!(registry.live_count(), 0);
    }

    #[test]
    fn test_client_registry() {
        let bus = EventBus::new();
        let registry = ClientRegistry::new(bus);
        assert_eq!(registry.live_count(), 0);

        registry.register_client(ClientRecord {
            app_id: "filer".to_string(),
            pid: 1234,
            window_handle: 1,
            is_native_app: true,
        });
        assert_eq!(registry.live_count(), 1);
        assert_eq!(registry.live_native_client_pids(), vec![1234]);

        registry.unregister_client(1234);
        assert_eq!(registry.live_count(), 0);
    }
}
