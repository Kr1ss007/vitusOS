//! VirtualDesktopManager -- Spatial Desktop Transitions (Part 31 of spec).
//!
//! macOS Spaces model: windows slide at full velocity, wallpaper at 40% (parallax depth).
//! Boundary: bounce (not wrap). Max 6 desktops, min 1.
//! Window assignment via StateManager key "windowDesktop:{handle}".
//! Springs: SPRING_DESKTOP_SWITCH (350,26) for windows, DesktopSwitch for wallpaper.
//! Reduced Motion (Part 38): slide springs marked for elimination.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

pub const MAX_DESKTOPS: usize = 6;
pub const MIN_DESKTOPS: usize = 1;
pub const PARALLAX_FACTOR: f32 = 0.4;       // wallpaper travels 40% of window travel
pub const MAX_NAME_LEN: usize = 24;
const BOUNCE_PX: f32 = 32.0;                // boundary bounce displacement

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualDesktop {
    pub id: usize,
    pub name: String,
    pub window_handles: Vec<u64>,
}

pub struct VirtualDesktopManager {
    pub current_index: RwLock<usize>,
    pub names: RwLock<Vec<String>>,
    pub slide_offset_x: RwLock<SpringSolver>,       // windows layer
    pub bg_slide_offset_x: RwLock<SpringSolver>,     // wallpaper parallax layer
    pub bounce_x: RwLock<SpringSolver>,             // boundary bounce
    pub bouncing: RwLock<bool>,
    pub previous_index: RwLock<usize>,              // for transition rendering
    pub window_desktop_map: RwLock<HashMap<u64, usize>>,
    pub screen_width: f32,
    bus: EventBus,
}

impl VirtualDesktopManager {
    pub fn new(screen_width: f32, bus: EventBus) -> Self {
        Self {
            current_index: RwLock::new(0),
            names: RwLock::new(vec!["Desktop 1".to_string()]),
            slide_offset_x: RwLock::new(
                SpringSolver::new(0.0, SpringProfile::DesktopSwitch)
                    .eliminate_on_reduced_motion(true)
            ),
            bg_slide_offset_x: RwLock::new(
                SpringSolver::with_params(0.0, 180.0, 24.0)
                    .eliminate_on_reduced_motion(true)
            ),
            bounce_x: RwLock::new(SpringSolver::new(0.0, SpringProfile::Selection)),
            bouncing: RwLock::new(false),
            previous_index: RwLock::new(0),
            window_desktop_map: RwLock::new(HashMap::new()),
            screen_width,
            bus,
        }
    }

    pub fn current_index(&self) -> usize { *self.current_index.read() }
    pub fn count(&self) -> usize { self.names.read().len() }
    pub fn slide_offset_x(&self) -> f32 { self.slide_offset_x.read().value }
    pub fn bg_slide_offset_x(&self) -> f32 { self.bg_slide_offset_x.read().value }
    pub fn bounce_offset_x(&self) -> f32 { self.bounce_x.read().value }
    pub fn is_bouncing(&self) -> bool { *self.bouncing.read() }

    pub fn is_at_left_boundary(&self) -> bool { *self.current_index.read() == 0 }
    pub fn is_at_right_boundary(&self) -> bool { *self.current_index.read() >= self.count() - 1 }

    fn default_name(index: usize) -> String {
        format!("Desktop {}", index + 1)
    }

    /// Switches to a specific desktop index.
    pub fn switch_to(&self, idx: usize) {
        self.switch_to_with_velocity(idx, 600.0);
    }

    pub fn switch_to_with_velocity(&self, idx: usize, velocity: f32) {
        if idx >= self.count() { return; }
        let current = *self.current_index.read();
        if idx == current { return; }

        let direction = if idx > current { -1.0 } else { 1.0 };
        *self.previous_index.write() = current;
        *self.current_index.write() = idx;

        self.slide_offset_x.write().set_target(0.0);
        self.slide_offset_x.write().set_velocity(direction * velocity);

        self.bg_slide_offset_x.write().set_target(0.0);
        self.bg_slide_offset_x.write().set_velocity(direction * velocity * PARALLAX_FACTOR);

        info!("DesktopManager: Switched to Workspace {} (velocity={:.0})", idx + 1, velocity);
        self.bus.publish(AEEvent::StateChanged {
            key: format!("active_workspace:{}", idx),
        });
    }

    /// Switches to the previous desktop (left). Bounces if at left boundary.
    pub fn switch_prev(&self) {
        self.switch_prev_with_velocity(600.0);
    }

    pub fn switch_prev_with_velocity(&self, velocity: f32) {
        let current = *self.current_index.read();
        if current == 0 {
            self.trigger_bounce(1.0);
            return;
        }
        *self.previous_index.write() = current;
        *self.current_index.write() = current - 1;

        self.slide_offset_x.write().snap(0.0);
        self.slide_offset_x.write().set_target(self.screen_width);
        self.slide_offset_x.write().set_velocity(velocity);

        self.bg_slide_offset_x.write().snap(0.0);
        self.bg_slide_offset_x.write().set_target(self.screen_width * PARALLAX_FACTOR);
        self.bg_slide_offset_x.write().set_velocity(velocity * PARALLAX_FACTOR);

        info!("DesktopManager: Switched prev to Workspace {}", current);
        self.bus.publish(AEEvent::StateChanged {
            key: format!("active_workspace:{}", current - 1),
        });
    }

    /// Switches to the next desktop (right). Bounces if at right boundary.
    pub fn switch_next(&self) {
        self.switch_next_with_velocity(600.0);
    }

    pub fn switch_next_with_velocity(&self, velocity: f32) {
        let current = *self.current_index.read();
        if current >= self.count() - 1 {
            self.trigger_bounce(-1.0);
            return;
        }
        *self.previous_index.write() = current;
        *self.current_index.write() = current + 1;

        self.slide_offset_x.write().snap(0.0);
        self.slide_offset_x.write().set_target(-self.screen_width);
        self.slide_offset_x.write().set_velocity(-velocity);

        self.bg_slide_offset_x.write().snap(0.0);
        self.bg_slide_offset_x.write().set_target(-self.screen_width * PARALLAX_FACTOR);
        self.bg_slide_offset_x.write().set_velocity(-velocity * PARALLAX_FACTOR);

        info!("DesktopManager: Switched next to Workspace {}", current + 2);
        self.bus.publish(AEEvent::StateChanged {
            key: format!("active_workspace:{}", current + 1),
        });
    }

    /// Triggers a boundary bounce in the given direction (Part 31.3).
    fn trigger_bounce(&self, direction: f32) {
        self.bounce_x.write().snap(0.0);
        self.bounce_x.write().set_target(direction * BOUNCE_PX);
        *self.bouncing.write() = true;
        info!("DesktopManager: Boundary bounce (direction={:.0})", direction);
    }

    /// Adds a new desktop (max 6).
    pub fn add_desktop(&self) {
        if self.count() >= MAX_DESKTOPS { return; }
        let idx = self.count();
        self.names.write().push(Self::default_name(idx));
        info!("DesktopManager: Added Desktop {}", idx + 1);
        self.bus.publish(AEEvent::StateChanged {
            key: "desktop_count".to_string(),
        });
    }

    /// Removes a desktop. Windows moved to Desktop 0. Cannot remove last desktop.
    pub fn remove_desktop(&self, idx: usize) {
        if self.count() <= MIN_DESKTOPS { return; }
        if idx >= self.count() { return; }

        // Reassign windows from removed desktop to Desktop 0
        let mut map = self.window_desktop_map.write();
        let entries: Vec<(u64, usize)> = map.iter().map(|(k, v)| (*k, *v)).collect();
        for (handle, desktop) in entries {
            if desktop == idx {
                map.insert(handle, 0);
            } else if desktop > idx {
                map.insert(handle, desktop - 1);
            }
        }
        drop(map);

        self.names.write().remove(idx);
        let current = *self.current_index.read();
        if current >= self.count() {
            *self.current_index.write() = self.count() - 1;
        }

        info!("DesktopManager: Removed Desktop {}", idx + 1);
        self.bus.publish(AEEvent::StateChanged {
            key: "desktop_count".to_string(),
        });
    }

    /// Renames a desktop (max 24 chars, empty = default name).
    pub fn rename_desktop(&self, idx: usize, name: &str) {
        if idx >= self.count() { return; }
        let mut names = self.names.write();
        if name.is_empty() {
            names[idx] = Self::default_name(idx);
        } else {
            names[idx] = name.chars().take(MAX_NAME_LEN).collect();
        }
    }

    pub fn name_for_desktop(&self, idx: usize) -> String {
        self.names.read().get(idx).cloned().unwrap_or_else(|| Self::default_name(idx))
    }

    /// Assigns a window to a specific desktop.
    pub fn assign_window(&self, window_handle: u64, desktop_idx: usize) {
        self.window_desktop_map.write().insert(window_handle, desktop_idx);
    }

    pub fn desktop_for_window(&self, window_handle: u64) -> usize {
        self.window_desktop_map.read().get(&window_handle).copied().unwrap_or(0)
    }

    pub fn window_visible_on_current(&self, window_handle: u64) -> bool {
        self.desktop_for_window(window_handle) == *self.current_index.read()
    }

    /// Called each frame to tick springs.
    pub fn update(&self, dt: f32) {
        self.slide_offset_x.write().update(dt);
        self.bg_slide_offset_x.write().update(dt);

        if *self.bouncing.read() {
            self.bounce_x.write().update(dt);
            let bounce_val = self.bounce_x.read().value;
            if self.bounce_x.read().is_settled() && bounce_val.abs() > 1.0 {
                self.bounce_x.write().set_target(0.0);
            } else if self.bounce_x.read().is_settled() && bounce_val.abs() <= 1.0 {
                *self.bouncing.write() = false;
                self.bounce_x.write().snap(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_desktop_switching() {
        let bus = EventBus::new();
        let vdm = VirtualDesktopManager::new(1920.0, bus);

        assert_eq!(vdm.current_index(), 0);
        assert_eq!(vdm.count(), 1);

        vdm.add_desktop();
        vdm.add_desktop();
        assert_eq!(vdm.count(), 3);

        vdm.switch_next();
        assert_eq!(vdm.current_index(), 1);
        assert_eq!(vdm.slide_offset_x.read().target, -1920.0);

        vdm.switch_prev();
        assert_eq!(vdm.current_index(), 0);
        assert_eq!(vdm.slide_offset_x.read().target, 1920.0);
    }

    #[test]
    fn test_boundary_bounce() {
        let bus = EventBus::new();
        let vdm = VirtualDesktopManager::new(1920.0, bus);
        vdm.add_desktop();

        // At left boundary, try to go prev -- should bounce
        vdm.switch_prev();
        assert!(vdm.is_bouncing());
        assert_eq!(vdm.current_index(), 0); // Didn't switch
    }

    #[test]
    fn test_window_assignment() {
        let bus = EventBus::new();
        let vdm = VirtualDesktopManager::new(1920.0, bus);
        vdm.add_desktop();

        vdm.assign_window(1001, 1);
        assert_eq!(vdm.desktop_for_window(1001), 1);
        assert!(!vdm.window_visible_on_current(1001));

        vdm.switch_next();
        assert!(vdm.window_visible_on_current(1001));
    }

    #[test]
    fn test_add_remove_desktop() {
        let bus = EventBus::new();
        let vdm = VirtualDesktopManager::new(1920.0, bus);

        vdm.add_desktop();
        assert_eq!(vdm.count(), 2);

        vdm.remove_desktop(1);
        assert_eq!(vdm.count(), 1);

        // Cannot remove last desktop
        vdm.remove_desktop(0);
        assert_eq!(vdm.count(), 1);
    }

    #[test]
    fn test_rename_desktop() {
        let bus = EventBus::new();
        let vdm = VirtualDesktopManager::new(1920.0, bus);

        vdm.add_desktop();
        vdm.rename_desktop(1, "Code");
        assert_eq!(vdm.name_for_desktop(1), "Code");

        // Empty name reverts to default
        vdm.rename_desktop(1, "");
        assert_eq!(vdm.name_for_desktop(1), "Desktop 2");
    }

    #[test]
    fn test_max_desktops() {
        let bus = EventBus::new();
        let vdm = VirtualDesktopManager::new(1920.0, bus);

        for _ in 0..10 {
            vdm.add_desktop();
        }
        assert_eq!(vdm.count(), MAX_DESKTOPS);
    }
}
