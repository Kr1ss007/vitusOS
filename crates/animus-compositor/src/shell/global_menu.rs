//! Global Menu System with Keyboard Navigation (F10 / Alt).

use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub is_separator: bool,
    pub is_enabled: bool,
    pub sub_items: Vec<MenuItem>,
    pub hover_alpha: SpringSolver, // SPRING_HOVER (600, 40)
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            is_separator: false,
            is_enabled: true,
            sub_items: Vec::new(),
            hover_alpha: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }
}

pub struct GlobalMenu {
    pub items: Vec<MenuItem>,
    pub active_index: Option<usize>,
    pub is_keyboard_active: bool,
    pub submenu_clip_h: SpringSolver, // SPRING_SHEET (420, 30)
}

impl Default for GlobalMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalMenu {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active_index: None,
            is_keyboard_active: false,
            submenu_clip_h: SpringSolver::new(0.0, SpringProfile::Sheet),
        }
    }

    pub fn set_items(&mut self, items: Vec<MenuItem>) {
        self.items = items;
    }

    pub fn update(&mut self, dt: f32) {
        self.submenu_clip_h.update(dt);
        for item in &mut self.items {
            item.hover_alpha.update(dt);
        }
    }
}
