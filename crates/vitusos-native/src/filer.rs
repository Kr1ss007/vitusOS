//! Filer: The Glass-Native Spatial File Manager & Persistent Desktop Daemon.
//!
//! Filer is always running (like macOS Finder), managing the desktop surface layer,
//! volume events, and global file system operations.
//! Filer's toolbar searchbar is physically unified with Pathfinder.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};
use animus_render::altitude::SurfaceAltitude;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarItem {
    pub label: String,
    pub icon_name: String,
    pub is_section_header: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub date_modified: String,
    pub size_bytes: u64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopIcon {
    pub name: String,
    pub path: String,
    pub grid_x: u32,
    pub grid_y: u32,
    pub is_selected: bool,
}

/// Persistent background daemon (macOS Finder equivalent) that never terminates.
pub struct FilerDaemon {
    pub is_running: bool,
    pub desktop_icons: Vec<DesktopIcon>,
    pub open_windows: Vec<FilerWindow>,
    bus: EventBus,
}

impl FilerDaemon {
    pub fn new(bus: EventBus) -> Self {
        let mut default_desktop_icons = Vec::new();
        default_desktop_icons.push(DesktopIcon {
            name: "Macintosh HD".to_string(),
            path: "/".to_string(),
            grid_x: 0,
            grid_y: 0,
            is_selected: false,
        });

        Self {
            is_running: true,
            desktop_icons: default_desktop_icons,
            open_windows: Vec::new(),
            bus,
        }
    }

    pub fn spawn_window(&mut self) -> &mut FilerWindow {
        self.open_windows.push(FilerWindow::new(self.bus.clone()));
        self.open_windows.last_mut().unwrap()
    }

    pub fn update(&mut self, dt: f32) {
        for window in &mut self.open_windows {
            window.update(dt);
        }
    }
}

pub struct FilerWindow {
    pub sidebar_altitude: SurfaceAltitude, // Mid (20px Kawase Blur, 82% Opacity)
    pub toolbar_altitude: SurfaceAltitude, // Low (8px Kawase Blur, 94% Opacity)
    pub content_altitude: SurfaceAltitude, // Grounded (#FEFEFE Opaque Canvas)
    pub sidebar_items: Vec<SidebarItem>,
    pub files: Vec<FileEntry>,
    pub selected_sidebar_idx: usize,
    pub selection_pill_y: SpringSolver,    // SPRING_SELECTION (400, 28)
    pub search_bar_width: SpringSolver,    // SPRING_HOVER (600, 40): 188 -> 260px
    pub drag_ghost_pos: SpringSolver2D,    // SPRING_WINDOW_DRAG (800, 35)
    pub is_dragging_file: bool,
    pub is_zebra_striped: bool,
    bus: EventBus,
}

impl FilerWindow {
    pub fn new(bus: EventBus) -> Self {
        let mut sidebar_items = Vec::new();
        sidebar_items.push(SidebarItem {
            label: "FAVORITES".to_string(),
            icon_name: "".to_string(),
            is_section_header: true,
        });
        sidebar_items.push(SidebarItem {
            label: "AirDrop".to_string(),
            icon_name: "airdrop".to_string(),
            is_section_header: false,
        });
        sidebar_items.push(SidebarItem {
            label: "Recents".to_string(),
            icon_name: "recents".to_string(),
            is_section_header: false,
        });
        sidebar_items.push(SidebarItem {
            label: "Desktop".to_string(),
            icon_name: "desktop".to_string(),
            is_section_header: false,
        });
        sidebar_items.push(SidebarItem {
            label: "Applications".to_string(),
            icon_name: "apps".to_string(),
            is_section_header: false,
        });
        sidebar_items.push(SidebarItem {
            label: "Documents".to_string(),
            icon_name: "documents".to_string(),
            is_section_header: false,
        });
        sidebar_items.push(SidebarItem {
            label: "Downloads".to_string(),
            icon_name: "downloads".to_string(),
            is_section_header: false,
        });

        Self {
            sidebar_altitude: SurfaceAltitude::Mid,
            toolbar_altitude: SurfaceAltitude::Low,
            content_altitude: SurfaceAltitude::Grounded,
            sidebar_items,
            files: Vec::new(),
            selected_sidebar_idx: 3, // Desktop by default
            selection_pill_y: SpringSolver::new(108.0, SpringProfile::Selection),
            search_bar_width: SpringSolver::new(188.0, SpringProfile::Hover),
            drag_ghost_pos: SpringSolver2D::new(0.0, 0.0, SpringProfile::WindowDrag),
            is_dragging_file: false,
            is_zebra_striped: true,
            bus,
        }
    }

    pub fn select_sidebar_item(&mut self, idx: usize) {
        if idx < self.sidebar_items.len() && !self.sidebar_items[idx].is_section_header {
            self.selected_sidebar_idx = idx;
            let target_y = idx as f32 * 36.0;
            self.selection_pill_y.set_target(target_y);
        }
    }

    /// Filer's searchbar IS Pathfinder: focusing/clicking search directly opens Pathfinder!
    pub fn activate_search(&mut self) {
        self.search_bar_width.set_target(260.0);
        self.bus.publish(AEEvent::PathfinderOpened);
    }

    pub fn focus_search(&mut self, focused: bool) {
        if focused {
            self.activate_search();
        } else {
            self.search_bar_width.set_target(188.0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.selection_pill_y.update(dt);
        self.search_bar_width.update(dt);
        self.drag_ghost_pos.update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filer_daemon_and_searchbar_unification() {
        let bus = EventBus::new();
        let mut daemon = FilerDaemon::new(bus.clone());
        assert!(daemon.is_running);
        assert!(!daemon.desktop_icons.is_empty());

        let window = daemon.spawn_window();
        assert_eq!(window.sidebar_altitude, SurfaceAltitude::Mid);
        assert_eq!(window.content_altitude, SurfaceAltitude::Grounded);

        // Filer searchbar activation sends AEEvent::PathfinderOpened
        window.activate_search();
        assert_eq!(window.search_bar_width.target, 260.0);
    }
}
