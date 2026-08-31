//! Top Panel Component (28px Height, Low Altitude 8px Blur).

use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrangeBoxButton {
    pub width: f32,
    pub height: f32,
    pub hover_alpha: SpringSolver, // SPRING_HOVER (600, 40)
}

impl Default for OrangeBoxButton {
    fn default() -> Self {
        Self {
            width: 42.0,
            height: 28.0,
            hover_alpha: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }
}

pub struct Panel {
    pub height: f32,
    pub orange_box: OrangeBoxButton,
    pub focused_app_title: String,
    pub is_clock_visible: bool,
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel {
    pub const HEIGHT: f32 = 28.0;

    pub fn new() -> Self {
        Self {
            height: Self::HEIGHT,
            orange_box: OrangeBoxButton::default(),
            focused_app_title: "Finder".to_string(),
            is_clock_visible: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.orange_box.hover_alpha.update(dt);
    }
}
