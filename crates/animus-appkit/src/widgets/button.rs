use animus_physics::spring::{SpringSolver, SpringProfile};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small = 24,
    Medium = 32,
    Large = 40,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Destructive,
    Ghost,
}

pub struct AEButton {
    pub label: String,
    pub size: ButtonSize,
    pub style: ButtonStyle,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    
    // Springs for interaction
    pub hover_spring: SpringSolver, // 0.0 -> 1.0
    pub press_spring: SpringSolver, // 1.0 -> 0.96 -> 1.0
}

impl AEButton {
    pub fn new(label: impl Into<String>, size: ButtonSize, style: ButtonStyle) -> Self {
        Self {
            label: label.into(),
            size,
            style,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            hover_spring: SpringSolver::new(0.0, SpringProfile::Hover),
            press_spring: SpringSolver::new(1.0, SpringProfile::Selection),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.hover_spring.update(dt);
        self.press_spring.update(dt);
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let height = self.size as i32 as f32;
        mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + height
    }

    pub fn on_pointer_motion(&mut self, mx: f32, my: f32) {
        if self.hit_test(mx, my) {
            self.hover_spring.set_target(1.0);
        } else {
            self.hover_spring.set_target(0.0);
        }
    }

    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) -> bool {
        if self.hit_test(mx, my) {
            if pressed {
                self.press_spring.set_target(0.96);
            } else {
                self.press_spring.set_target(1.0);
                info!("AEButton '{}' clicked", self.label);
                return true; // Action triggered
            }
        } else if !pressed {
            self.press_spring.set_target(1.0);
        }
        false
    }
}
