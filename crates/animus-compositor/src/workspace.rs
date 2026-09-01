//! VirtualDesktopManager — Multi-Workspace Orchestration (Part 31 of spec).
//!
//! Manages virtual workspaces (1-10), horizontal sliding parallax with `SpringProfile::DesktopSwitch`,
//! and window workspace migration.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualDesktop {
    pub id: usize,
    pub name: String,
    pub window_handles: Vec<u64>,
}

pub struct VirtualDesktopManager {
    pub active_desktop_idx: RwLock<usize>,
    pub desktops: RwLock<Vec<VirtualDesktop>>,
    pub slide_offset_x: RwLock<SpringSolver>, // SPRING_DESKTOP_SWITCH (350, 26)
    pub window_desktop_map: RwLock<HashMap<u64, usize>>,
    pub screen_width: f32,
    bus: EventBus,
}

impl VirtualDesktopManager {
    pub const MAX_DESKTOPS: usize = 10;

    pub fn new(screen_width: f32, bus: EventBus) -> Self {
        let initial_desktops = vec![
            VirtualDesktop {
                id: 0,
                name: "Workspace 1".to_string(),
                window_handles: Vec::new(),
            },
            VirtualDesktop {
                id: 1,
                name: "Workspace 2".to_string(),
                window_handles: Vec::new(),
            },
            VirtualDesktop {
                id: 2,
                name: "Workspace 3".to_string(),
                window_handles: Vec::new(),
            },
        ];

        Self {
            active_desktop_idx: RwLock::new(0),
            desktops: RwLock::new(initial_desktops),
            slide_offset_x: RwLock::new(SpringSolver::new(0.0, SpringProfile::DesktopSwitch)),
            window_desktop_map: RwLock::new(HashMap::new()),
            screen_width,
            bus,
        }
    }

    /// Switches to a specific desktop index with horizontal spring sliding.
    pub fn switch_to_desktop(&self, idx: usize) {
        let desktops_len = self.desktops.read().len();
        if idx >= desktops_len {
            return;
        }

        let mut active = self.active_desktop_idx.write();
        *active = idx;
        let target_x = -(idx as f32) * self.screen_width;
        self.slide_offset_x.write().set_target(target_x);

        info!("VirtualDesktopManager: Switched to Workspace {} (offset_x -> {})", idx + 1, target_x);
        self.bus.publish(AEEvent::StateChanged {
            key: format!("active_workspace:{}", idx),
        });
    }

    /// Moves to the next virtual desktop.
    pub fn next_desktop(&self) {
        let current = *self.active_desktop_idx.read();
        let total = self.desktops.read().len();
        if current + 1 < total {
            self.switch_to_desktop(current + 1);
        }
    }

    /// Moves to the previous virtual desktop.
    pub fn prev_desktop(&self) {
        let current = *self.active_desktop_idx.read();
        if current > 0 {
            self.switch_to_desktop(current - 1);
        }
    }

    /// Assigns a window to a specific desktop.
    pub fn assign_window_to_desktop(&self, window_handle: u64, desktop_idx: usize) {
        let mut map = self.window_desktop_map.write();
        map.insert(window_handle, desktop_idx);

        let mut desktops = self.desktops.write();
        for d in desktops.iter_mut() {
            d.window_handles.retain(|&h| h != window_handle);
        }
        if let Some(target) = desktops.get_mut(desktop_idx) {
            target.window_handles.push(window_handle);
        }
        info!("VirtualDesktopManager: Window #{} moved to Workspace {}", window_handle, desktop_idx + 1);
    }

    /// Checks if a window is visible on the currently active desktop.
    pub fn is_window_visible_on_active(&self, window_handle: u64) -> bool {
        let active = *self.active_desktop_idx.read();
        let map = self.window_desktop_map.read();
        map.get(&window_handle).map(|&d| d == active).unwrap_or(true)
    }

    pub fn update(&self, dt: f32) {
        self.slide_offset_x.write().update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_desktop_manager_switching() {
        let bus = EventBus::new();
        let vdm = VirtualDesktopManager::new(1920.0, bus);

        assert_eq!(*vdm.active_desktop_idx.read(), 0);
        vdm.next_desktop();
        assert_eq!(*vdm.active_desktop_idx.read(), 1);
        assert_eq!(vdm.slide_offset_x.read().target, -1920.0);

        vdm.prev_desktop();
        assert_eq!(*vdm.active_desktop_idx.read(), 0);
        assert_eq!(vdm.slide_offset_x.read().target, 0.0);

        vdm.assign_window_to_desktop(1001, 1);
        assert!(!vdm.is_window_visible_on_active(1001));
    }
}
