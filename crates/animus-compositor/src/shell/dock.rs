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

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            height: Self::HEIGHT,
            icon_size: Self::ICON_SIZE,
            max_magnify: Self::MAX_MAGNIFY,
            corner_radius: Self::CORNER_RADIUS,
        }
    }

    pub fn add_item(&mut self, item: DockItem) {
        self.items.push(item);
    }

    pub fn update(&mut self, dt: f32) {
        for item in &mut self.items {
            item.magnify.update(dt);
            item.bounce.update(dt);
        }
    }
}
