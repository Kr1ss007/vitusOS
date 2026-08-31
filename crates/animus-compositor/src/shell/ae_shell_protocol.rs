//! Custom Wayland Protocol ae-shell-v1 (Part 16 of spec).
//!
//! Exposes custom compositor interfaces for native apps:
//! Global Menu definition, Dock badges, attention requests.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AESurfaceState {
    pub surface_id: u64,
    pub app_id: String,
    pub menu_json: Option<String>,
    pub badge_count: i32,
    pub attention_requested: bool,
}

pub struct AEShellProtocolManager {
    surfaces: RwLock<HashMap<u64, AESurfaceState>>,
    bus: EventBus,
}

impl AEShellProtocolManager {
    pub fn new(bus: EventBus) -> Self {
        Self {
            surfaces: RwLock::new(HashMap::new()),
            bus,
        }
    }

    /// Registers a new managed ae-shell surface.
    pub fn register_surface(&self, surface_id: u64, app_id: String) {
        let state = AESurfaceState {
            surface_id,
            app_id: app_id.clone(),
            menu_json: None,
            badge_count: 0,
            attention_requested: false,
        };
        self.surfaces.write().insert(surface_id, state);
        info!("ae-shell-v1: Registered native surface #{} for '{}'", surface_id, app_id);
    }

    /// Handles client `set_application_menu` request.
    pub fn set_application_menu(&self, surface_id: u64, menu_json: String) {
        if let Some(surf) = self.surfaces.write().get_mut(&surface_id) {
            surf.menu_json = Some(menu_json.clone());
            let app_id = surf.app_id.clone();
            self.bus.publish(AEEvent::DBusMenuRegistered {
                app_id,
                menu_json,
            });
            info!("ae-shell-v1: Updated application menu for surface #{}", surface_id);
        }
    }

    /// Handles client `set_badge_count` request (-1 = clear badge).
    pub fn set_badge_count(&self, surface_id: u64, count: i32) {
        if let Some(surf) = self.surfaces.write().get_mut(&surface_id) {
            surf.badge_count = count.max(-1);
            info!("ae-shell-v1: Set badge count {} on surface #{}", count, surface_id);
        }
    }

    /// Handles client `request_attention` request (Dock icon bounce).
    pub fn request_attention(&self, surface_id: u64) {
        if let Some(surf) = self.surfaces.write().get_mut(&surface_id) {
            surf.attention_requested = true;
            let app_id = surf.app_id.clone();
            self.bus.publish(AEEvent::DockBounce { app_id });
            info!("ae-shell-v1: Attention requested for surface #{}", surface_id);
        }
    }

    /// Destroys a surface registration.
    pub fn destroy_surface(&self, surface_id: u64) {
        if self.surfaces.write().remove(&surface_id).is_some() {
            info!("ae-shell-v1: Destroyed surface #{}", surface_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ae_shell_protocol_lifecycle() {
        let bus = EventBus::new();
        let proto = AEShellProtocolManager::new(bus);

        proto.register_surface(1, "org.vitusos.filer".to_string());
        proto.set_application_menu(1, "{\"menu\": [\"File\", \"Edit\"]}".to_string());
        proto.set_badge_count(1, 3);
        proto.request_attention(1);

        assert_eq!(proto.surfaces.read().get(&1).unwrap().badge_count, 3);
        assert!(proto.surfaces.read().get(&1).unwrap().attention_requested);

        proto.destroy_surface(1);
        assert!(!proto.surfaces.read().contains_key(&1));
    }
}
