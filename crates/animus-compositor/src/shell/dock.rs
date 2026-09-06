//! Floating Dock Component (64px Height, 16px Radius, Mid Altitude 20px Blur).

use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockItem {
    pub app_id: String,
    pub display_name: String,
    pub icon_path: String,
    pub is_running: bool,
    pub badge_count: Option<u32>,
    pub magnify: SpringSolver, // SPRING_DOCK_MAGNIFY (450, 32)
    pub bounce: SpringSolver,  // SPRING_SELECTION (400, 28)
}

impl DockItem {
    pub fn new(app_id: impl Into<String>, display_name: impl Into<String>, icon_path: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            display_name: display_name.into(),
            icon_path: icon_path.into(),
            is_running: false,
            badge_count: None,
            magnify: SpringSolver::new(48.0, SpringProfile::DockMagnify),
            bounce: SpringSolver::new(0.0, SpringProfile::Selection),
        }
    }
}

pub struct Dock {
    pub items: Vec<DockItem>,
    pub height: f32,
    pub icon_size: f32,
    pub max_magnify: f32,
    pub corner_radius: f32,
    // Fullscreen auto-hide (Part 40.3) -- mirrored: hot zone at bottom
    pub is_fullscreen: bool,
    pub hide_y: SpringSolver,  // 0 = visible, +HEIGHT = hidden below screen
    pub cursor_y: f32,
    // Auto-hide when window overlaps (Addendum I.2)
    pub auto_hiding: bool,
    pub auto_hide: SpringSolver,  // SPRING_HOVER for show/hide
}

impl Default for Dock {
    fn default() -> Self {
        Self::new()
    }
}

impl Dock {
    pub const HEIGHT: f32 = 64.0;
    pub const ICON_SIZE: f32 = 48.0;
    pub const MAX_MAGNIFY: f32 = 72.0; // 1.5x Peak
    pub const CORNER_RADIUS: f32 = 16.0;
    pub const HOT_ZONE_PX: f32 = 4.0; // Bottom 4px hot zone in fullscreen

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            height: Self::HEIGHT,
            icon_size: Self::ICON_SIZE,
            max_magnify: Self::MAX_MAGNIFY,
            corner_radius: Self::CORNER_RADIUS,
            is_fullscreen: false,
            hide_y: SpringSolver::new(0.0, SpringProfile::Selection),
            cursor_y: 0.0,
            auto_hiding: false,
            auto_hide: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    /// Enters fullscreen mode (Part 40.3).
    pub fn enter_fullscreen_mode(&mut self) {
        self.is_fullscreen = true;
    }

    /// Exits fullscreen mode (Part 40.3).
    pub fn exit_fullscreen_mode(&mut self) {
        self.is_fullscreen = false;
        self.hide_y.set_target(0.0);
    }

    /// Updates cursor Y for hot zone detection (Part 40.3).
    pub fn on_pointer_motion(&mut self, _x: f32, y: f32, screen_h: f32) {
        self.cursor_y = y;
        if self.is_fullscreen {
            // Hot zone: cursor within 4px of bottom edge
            let target = if y >= screen_h - Self::HOT_ZONE_PX { 0.0 } else { Self::HEIGHT };
            self.hide_y.set_target(target);
        }
    }

    /// Returns render Y offset: 0 = visible, +HEIGHT = hidden.
    pub fn render_offset_y(&self) -> f32 {
        if self.is_fullscreen { self.hide_y.value } else { self.auto_hide.value }
    }

    pub fn is_hidden(&self) -> bool {
        if self.is_fullscreen { self.hide_y.value > Self::HEIGHT * 0.5 }
        else if self.auto_hiding { self.auto_hide.value > Self::HEIGHT * 0.5 }
        else { false }
    }

    pub fn add_item(&mut self, item: DockItem) {
        self.items.push(item);
    }

    /// Triggers bounce spring and spawns native process on user click
    pub fn launch_item(&mut self, idx: usize) -> bool {
        if let Some(item) = self.items.get_mut(idx) {
            item.bounce.set_target(1.0);
            item.is_running = true;
            tracing::info!("Dock: User clicked '{}' -> launching process", item.app_id);

            let app_id = item.app_id.clone();
            std::thread::spawn(move || {
                let _ = std::process::Command::new("vitusos-native")
                    .args(["--app", &app_id])
                    .spawn();
            });
            return true;
        }
        false
    }

    /// Applies Gaussian magnification across icons as cursor hovers over the Dock
    pub fn handle_pointer_motion(&mut self, cursor_x: f32, dock_x: f32) {
        let item_width = 56.0f32;
        for (i, item) in self.items.iter_mut().enumerate() {
            let item_center = dock_x + 16.0 + (i as f32 * item_width) + (item_width * 0.5);
            let dist = (cursor_x - item_center).abs();
            let sigma = 64.0f32;
            let factor = (- (dist * dist) / (2.0 * sigma * sigma)).exp();
            let target_size = Self::ICON_SIZE + (Self::MAX_MAGNIFY - Self::ICON_SIZE) * factor;
            item.magnify.set_target(target_size);
        }
    }

    pub fn reset_magnification(&mut self) {
        for item in &mut self.items {
            item.magnify.set_target(Self::ICON_SIZE);
        }
    }

    pub fn update(&mut self, dt: f32) {
        for item in &mut self.items {
            item.magnify.update(dt);
            item.bounce.update(dt);
        }
        self.hide_y.update(dt);
        self.auto_hide.update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_magnification_and_launch() {
        let mut dock = Dock::new();
        dock.add_item(DockItem::new("filer", "Files", "assets/icons/dock/filer.svg"));
        dock.add_item(DockItem::new("terminow", "Terminow", "assets/icons/dock/terminow.svg"));

        dock.handle_pointer_motion(40.0, 0.0);
        assert!(dock.items[0].magnify.target > Dock::ICON_SIZE);

        assert!(dock.launch_item(0));
        assert!(dock.items[0].is_running);
    }
}

