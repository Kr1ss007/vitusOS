//! CrashSite — Wayland Client Failure & Auto-Respawn Policy (Part 21.6 of spec).
//!
//! Tracks client disconnects, handles crashes, and enforces rate-limited auto-respawn policies.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

use crate::event_bus::EventBus;
use crate::events::{AEEvent, NotificationPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCrashRecord {
    pub respawn_count: u32,
    #[serde(skip, default = "Instant::now")]
    pub first_crash_time: Instant,
}

pub struct CrashSite {
    records: Arc<RwLock<HashMap<String, AppCrashRecord>>>,
    bus: EventBus,
}

impl CrashSite {
    pub const MAX_RESPAWNS: u32 = 3;
    pub const RESPAWN_WINDOW_SECS: f32 = 10.0;

    pub fn new(bus: EventBus) -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            bus,
        }
    }

    /// Handles normal, graceful Wayland client exit.
    pub fn on_clean_exit(&self, app_id: &str) {
        self.records.write().remove(app_id);
        info!("CrashSite: Client '{}' exited cleanly.", app_id);
    }

    /// Handles dirty client crash or sudden socket disconnect.
    pub fn on_client_crash(&self, app_id: &str, pid: u32) {
        warn!("CrashSite: Client '{}' (PID {}) crashed unexpectedly.", app_id, pid);
        self.bus.publish_async(AEEvent::ClientCrashed {
            app_id: app_id.to_string(),
            pid,
        });

        if self.should_respawn(app_id) {
            self.record_respawn(app_id);
            info!("CrashSite: Triggering auto-respawn for '{}'...", app_id);
        } else {
            warn!("CrashSite: Client '{}' exceeded max respawn threshold. Marked as dead.", app_id);
            self.bus.publish_async(AEEvent::NotificationPosted(NotificationPayload {
                title: format!("{} Quit Unexpectedly", app_id),
                body: "Click to view diagnostic logs or reopen the app.".to_string(),
                timeout_ms: 8000,
                is_persistent: false,
                action_keys: vec!["reopen".to_string(), "ignore".to_string()],
                action_labels: vec!["Reopen".to_string(), "Ignore".to_string()],
            }));
        }
    }

    /// Checks if app is eligible for auto-respawn within rate limit window.
    pub fn should_respawn(&self, app_id: &str) -> bool {
        let records = self.records.read();
        if let Some(rec) = records.get(app_id) {
            let elapsed = rec.first_crash_time.elapsed().as_secs_f32();
            if elapsed > Self::RESPAWN_WINDOW_SECS {
                true // Window reset
            } else {
                rec.respawn_count < Self::MAX_RESPAWNS
            }
        } else {
            true // First crash
        }
    }

    /// Records an auto-respawn attempt.
    pub fn record_respawn(&self, app_id: &str) {
        let mut records = self.records.write();
        let now = Instant::now();
        if let Some(rec) = records.get_mut(app_id) {
            let elapsed = rec.first_crash_time.elapsed().as_secs_f32();
            if elapsed > Self::RESPAWN_WINDOW_SECS {
                rec.respawn_count = 1;
                rec.first_crash_time = now;
            } else {
                rec.respawn_count += 1;
            }
        } else {
            records.insert(
                app_id.to_string(),
                AppCrashRecord {
                    respawn_count: 1,
                    first_crash_time: now,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_site_respawn_rate_limit() {
        let bus = EventBus::new();
        let site = CrashSite::new(bus);

        assert!(site.should_respawn("org.vitusos.filer"));
        site.record_respawn("org.vitusos.filer"); // 1
        assert!(site.should_respawn("org.vitusos.filer"));
        site.record_respawn("org.vitusos.filer"); // 2
        assert!(site.should_respawn("org.vitusos.filer"));
        site.record_respawn("org.vitusos.filer"); // 3
        assert!(!site.should_respawn("org.vitusos.filer")); // 4th attempt rejected
    }
}
