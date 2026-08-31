//! AEAppKit Canonical Widget & Control System (AnimusEngine AppKit).
//!
//! Provides mathematically precise Apple-grade widgets driven by SpringSolver physics.

use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonVariant {
    Primary,   // Space Orange #FF6B00
    Secondary, // Translucent Glass Low (8px blur)
    Destructive, // Red #FF3B30
    Ghost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AEButton {
    pub label: String,
    pub variant: ButtonVariant,
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub scale: SpringSolver,      // 1.0 -> 0.96 (pressed) -> 1.0 (released)
    pub hover_alpha: SpringSolver,// 0.0 -> 1.0
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub is_enabled: bool,
}

impl AEButton {
    pub fn new(label: impl Into<String>, variant: ButtonVariant) -> Self {
        Self {
            label: label.into(),
            variant,
            width: 120.0,
            height: 32.0,
            corner_radius: 6.0,
            scale: SpringSolver::new(1.0, SpringProfile::Selection),
            hover_alpha: SpringSolver::new(0.0, SpringProfile::Hover),
            is_hovered: false,
            is_pressed: false,
            is_enabled: true,
        }
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
        self.hover_alpha.set_target(if hovered { 1.0 } else { 0.0 });
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
        self.scale.set_target(if pressed { 0.96 } else { 1.0 });
    }

    pub fn update(&mut self, dt: f32) {
        self.scale.update(dt);
        self.hover_alpha.update(dt);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AESegmentedControl {
    pub segments: Vec<String>,
    pub selected_index: usize,
    pub indicator_x: SpringSolver, // SPRING_SELECTION (400, 28) sliding pill
    pub segment_width: f32,
    pub height: f32,
}

impl AESegmentedControl {
    pub fn new(segments: Vec<String>, segment_width: f32) -> Self {
        Self {
            segments,
            selected_index: 0,
            indicator_x: SpringSolver::new(0.0, SpringProfile::Selection),
            segment_width,
            height: 28.0,
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.segments.len() {
            self.selected_index = index;
            self.indicator_x.set_target(index as f32 * self.segment_width);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.indicator_x.update(dt);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AETextField {
    pub text: String,
    pub placeholder: String,
    pub is_focused: bool,
    pub focus_glow: SpringSolver, // 0.0 -> 1.0 (Space Orange 2px glow ring)
    pub width: f32,
    pub height: f32,
}

impl AETextField {
    pub fn new(placeholder: impl Into<String>, width: f32) -> Self {
        Self {
            text: String::new(),
            placeholder: placeholder.into(),
            is_focused: false,
            focus_glow: SpringSolver::new(0.0, SpringProfile::Selection),
            width,
            height: 30.0,
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        self.focus_glow.set_target(if focused { 1.0 } else { 0.0 });
    }

    pub fn update(&mut self, dt: f32) {
        self.focus_glow.update(dt);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AETrafficLights {
    pub diameter: f32,
    pub spacing: f32,
    pub close_color: [f32; 4],    // #FF5F56
    pub minimize_color: [f32; 4], // #FFBD2E
    pub maximize_color: [f32; 4], // #007AFF Blue (Canonical)
    pub hover_scale: SpringSolver,
}

impl Default for AETrafficLights {
    fn default() -> Self {
        Self {
            diameter: 12.0,
            spacing: 8.0,
            close_color: [1.0, 0.373, 0.337, 1.0],      // #FF5F56
            minimize_color: [1.0, 0.741, 0.180, 1.0],   // #FFBD2E
            maximize_color: [0.0, 0.478, 1.0, 1.0],     // #007AFF
            hover_scale: SpringSolver::new(1.0, SpringProfile::TrafficLight),
        }
    }
}
