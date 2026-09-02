//! ae-shell-v1 Protocol Handler — AnimusEngine Native Surface Extensions.
//!
//! This module implements the compositor-side handler for the `ae-shell-v1`
//! Wayland protocol defined in `protocol/ae-shell-v1.xml`.
//!
//! Protocol requests from native app clients:
//!
//! `ae_shell_manager_v1.get_ae_surface(id, surface)`:
//!   - Associates an ae_surface_v1 with a wl_surface
//!   - Client must have an xdg_toplevel on the same wl_surface
//!
//! `ae_surface_v1.set_application_menu(menu_json)`:
//!   - Provides the structured menu JSON (File, Edit, View, …)
//!   - The Panel's GlobalMenu bar displays this when the window is focused
//!
//! `ae_surface_v1.set_badge_count(count)`:
//!   - Sets the Dock icon's badge number (e.g. unread notifications)
//!   - count=0 removes the badge
//!
//! `ae_surface_v1.set_altitude(altitude)`:
//!   - Requests a specific Kawase blur radius from the compositor
//!   - Maps to SurfaceAltitude: 0=Grounded, 1=Low, 2=Mid, 3=High, 4=Floating
//!
//! `ae_surface_v1.request_attention(type)`:
//!   - Triggers the Dock icon bounce animation
//!   - type=0: gentle pulse, type=1: critical bounce

use animus_render::altitude::SurfaceAltitude;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Parsed application menu structure received from ae_surface_v1.set_application_menu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMenuDefinition {
    pub app_name: String,
    pub menus: Vec<MenuGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuGroup {
    pub title: String,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub action_id: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub is_separator: bool,
}

/// State for one native ae_surface_v1 object
#[derive(Debug, Clone)]
pub struct AeSurfaceState {
    pub surface_id: u32,
    pub app_id: String,
    pub altitude: SurfaceAltitude,
    pub badge_count: u32,
    pub menu: Option<AppMenuDefinition>,
    pub is_requesting_attention: bool,
    pub attention_critical: bool,
}

impl AeSurfaceState {
    pub fn new(surface_id: u32, app_id: impl Into<String>) -> Self {
        Self {
            surface_id,
            app_id: app_id.into(),
            altitude: SurfaceAltitude::Mid,
            badge_count: 0,
            menu: None,
            is_requesting_attention: false,
            attention_critical: false,
        }
    }

    /// Handles ae_surface_v1.set_application_menu(menu_json)
    pub fn handle_set_menu(&mut self, menu_json: &str) {
        match serde_json::from_str::<AppMenuDefinition>(menu_json) {
            Ok(menu) => {
                info!(
                    "AeShell: surface_id={} '{}' set menu with {} groups",
                    self.surface_id, self.app_id, menu.menus.len()
                );
                self.menu = Some(menu);
            }
            Err(e) => {
                tracing::warn!("AeShell: Invalid menu JSON from surface_id={}: {}", self.surface_id, e);
            }
        }
    }

    /// Handles ae_surface_v1.set_badge_count(count)
    pub fn handle_set_badge(&mut self, count: u32) {
        self.badge_count = count;
        if count == 0 {
            info!("AeShell: surface_id={} badge cleared", self.surface_id);
        } else {
            info!("AeShell: surface_id={} badge={}", self.surface_id, count);
        }
    }

    /// Handles ae_surface_v1.set_altitude(altitude_u32)
    pub fn handle_set_altitude(&mut self, altitude: u32) {
        self.altitude = match altitude {
            0 => SurfaceAltitude::Grounded,
            1 => SurfaceAltitude::Low,
            2 => SurfaceAltitude::Mid,
            3 => SurfaceAltitude::High,
            4 => SurfaceAltitude::Floating,
            _ => SurfaceAltitude::Mid,
        };
        info!(
            "AeShell: surface_id={} altitude → {:?} (blur {}px)",
            self.surface_id, self.altitude,
            match self.altitude {
                SurfaceAltitude::Grounded => 0,
                SurfaceAltitude::Low => 8,
                SurfaceAltitude::Mid => 20,
                SurfaceAltitude::High => 32,
                SurfaceAltitude::Floating => 48,
            }
        );
    }

    /// Handles ae_surface_v1.request_attention(attention_type)
    pub fn handle_request_attention(&mut self, attention_type: u32) {
        self.is_requesting_attention = true;
        self.attention_critical = attention_type == 1;
        info!(
            "AeShell: surface_id='{}' requests {} attention",
            self.app_id,
            if self.attention_critical { "CRITICAL" } else { "gentle" }
        );
    }
}

/// Compositor-side ae_shell_manager_v1 global handler.
/// Tracks all registered ae_surface_v1 objects and provides
/// the Panel and Dock with menu/badge data.
pub struct AeShellManager {
    pub surfaces: HashMap<u32, AeSurfaceState>,
}

impl AeShellManager {
    pub fn new() -> Self {
        Self { surfaces: HashMap::new() }
    }

    /// Called when a client requests ae_shell_manager_v1.get_ae_surface
    pub fn register_surface(&mut self, surface_id: u32, app_id: impl Into<String>) {
        let state = AeSurfaceState::new(surface_id, app_id);
        info!("AeShellManager: Registered ae_surface for surface_id={} '{}'", surface_id, state.app_id);
        self.surfaces.insert(surface_id, state);
    }

    /// Returns the active menu definition for the focused surface (used by Panel)
    pub fn get_menu_for_surface(&self, surface_id: u32) -> Option<&AppMenuDefinition> {
        self.surfaces.get(&surface_id)?.menu.as_ref()
    }

    /// Returns badge count for a given app_id (used by Dock)
    pub fn get_badge_for_app(&self, app_id: &str) -> u32 {
        self.surfaces.values()
            .find(|s| s.app_id == app_id)
            .map(|s| s.badge_count)
            .unwrap_or(0)
    }

    /// Returns surfaces requesting attention (for Dock bounce animation)
    pub fn surfaces_requesting_attention(&self) -> Vec<&AeSurfaceState> {
        self.surfaces.values()
            .filter(|s| s.is_requesting_attention)
            .collect()
    }

    /// Clears attention state after the Dock has played the bounce animation
    pub fn clear_attention(&mut self, surface_id: u32) {
        if let Some(s) = self.surfaces.get_mut(&surface_id) {
            s.is_requesting_attention = false;
        }
    }
}

impl Default for AeShellManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ae_shell_surface_menu_and_badge() {
        let mut mgr = AeShellManager::new();
        mgr.register_surface(1, "filer");

        // Set menu
        let menu_json = r#"{
            "app_name": "Filer",
            "menus": [
                {"title": "File", "items": [
                    {"label": "New Window", "action_id": "file.new_window", "shortcut": "⌘N", "enabled": true, "is_separator": false}
                ]},
                {"title": "Edit", "items": [
                    {"label": "", "action_id": "", "shortcut": null, "enabled": false, "is_separator": true}
                ]}
            ]
        }"#;
        mgr.surfaces.get_mut(&1).unwrap().handle_set_menu(menu_json);
        let menu = mgr.get_menu_for_surface(1).unwrap();
        assert_eq!(menu.app_name, "Filer");
        assert_eq!(menu.menus.len(), 2);
        assert_eq!(menu.menus[0].items[0].shortcut, Some("⌘N".to_string()));

        // Set badge
        mgr.surfaces.get_mut(&1).unwrap().handle_set_badge(3);
        assert_eq!(mgr.get_badge_for_app("filer"), 3);
        mgr.surfaces.get_mut(&1).unwrap().handle_set_badge(0);
        assert_eq!(mgr.get_badge_for_app("filer"), 0);

        // Request attention
        mgr.surfaces.get_mut(&1).unwrap().handle_request_attention(1);
        assert_eq!(mgr.surfaces_requesting_attention().len(), 1);
        assert!(mgr.surfaces_requesting_attention()[0].attention_critical);
        mgr.clear_attention(1);
        assert!(mgr.surfaces_requesting_attention().is_empty());
    }

    #[test]
    fn test_ae_shell_altitude_mapping() {
        let mut surface = AeSurfaceState::new(1, "pathfinder");
        surface.handle_set_altitude(4);
        assert_eq!(surface.altitude, SurfaceAltitude::Floating);
        surface.handle_set_altitude(2);
        assert_eq!(surface.altitude, SurfaceAltitude::Mid);
    }
}
