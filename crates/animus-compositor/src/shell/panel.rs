//! Top Panel Component (28px Height, Low Altitude 8px Blur) -- Part 15 + 40.3.
//!
//! Panel auto-hides in fullscreen mode: cursor within 4px of top -> slide down.
//! Cursor leaves hot zone -> slide up (hidden above screen).
//! Uses SPRING_SELECTION (400,28) for hideY: 0 = visible, -HEIGHT = hidden.

use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

/// Hot zone for Panel auto-hide in fullscreen (Part 40.3).
pub const HOT_ZONE_PX: f32 = 4.0;

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

/// Traffic light hover springs for focused window (Part 29).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficLightHover {
    pub close: SpringSolver,     // SPRING_TRAFFIC_LIGHT (700,38)
    pub minimize: SpringSolver,  // SPRING_TRAFFIC_LIGHT (700,38)
    pub maximize: SpringSolver,  // SPRING_TRAFFIC_LIGHT (700,38)
}

impl Default for TrafficLightHover {
    fn default() -> Self {
        Self {
            close: SpringSolver::new(0.0, SpringProfile::TrafficLight),
            minimize: SpringSolver::new(0.0, SpringProfile::TrafficLight),
            maximize: SpringSolver::new(0.0, SpringProfile::TrafficLight),
        }
    }
}

/// System tray items (Part 15.1, Addendum I).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTrayItem {
    pub icon_name: String,
    pub tooltip: String,
    pub hover_alpha: SpringSolver,
}

pub struct Panel {
    pub height: f32,
    pub orange_box: OrangeBoxButton,
    pub focused_app_title: String,
    pub is_clock_visible: bool,
    pub traffic_lights: TrafficLightHover,

    // Fullscreen auto-hide (Part 40.3)
    pub is_fullscreen: bool,
    pub hide_y: SpringSolver,       // SPRING_SELECTION (400,28): 0 = visible, -HEIGHT = hidden
    pub cursor_y: f32,              // updated from InputRouter

    // System tray
    pub tray_items: Vec<SystemTrayItem>,
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
            focused_app_title: "Filer".to_string(),
            is_clock_visible: true,
            traffic_lights: TrafficLightHover::default(),
            is_fullscreen: false,
            hide_y: SpringSolver::new(0.0, SpringProfile::Selection),
            cursor_y: 999.0,
            tray_items: Vec::new(),
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

    /// Updates cursor Y position for hot zone detection (Part 40.3).
    pub fn on_pointer_motion(&mut self, y: f32) {
        self.cursor_y = y;

        if self.is_fullscreen {
            let target = if y <= HOT_ZONE_PX { 0.0 } else { -Self::HEIGHT };
            self.hide_y.set_target(target);
        }
    }

    /// Returns the current Y offset for rendering (Part 40.3).
    /// 0 = fully visible, -HEIGHT = fully hidden.
    pub fn render_offset_y(&self) -> f32 {
        self.hide_y.value
    }

    pub fn is_hidden(&self) -> bool {
        self.is_fullscreen && self.hide_y.value < -Self::HEIGHT * 0.5
    }

    pub fn update(&mut self, dt: f32) {
        self.orange_box.hover_alpha.update(dt);
        self.hide_y.update(dt);
        self.traffic_lights.close.update(dt);
        self.traffic_lights.minimize.update(dt);
        self.traffic_lights.maximize.update(dt);
        for item in &mut self.tray_items {
            item.hover_alpha.update(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_fullscreen_auto_hide() {
        let mut panel = Panel::new();
        assert!(!panel.is_fullscreen);
        assert_eq!(panel.render_offset_y(), 0.0);

        panel.enter_fullscreen_mode();
        assert!(panel.is_fullscreen);

        // Cursor not near top -- should hide
        panel.on_pointer_motion(500.0);
        assert_eq!(panel.hide_y.target, -Panel::HEIGHT);

        // Tick spring to settle
        for _ in 0..120 { panel.update(0.016); }
        assert!(panel.is_hidden());

        // Cursor near top -- should show
        panel.on_pointer_motion(2.0);
        assert_eq!(panel.hide_y.target, 0.0);

        // Tick spring to settle
        for _ in 0..120 { panel.update(0.016); }
        assert!(!panel.is_hidden());

        panel.exit_fullscreen_mode();
        assert!(!panel.is_fullscreen);
    }

    #[test]
    fn test_panel_non_fullscreen_always_visible() {
        let mut panel = Panel::new();
        panel.on_pointer_motion(0.0);  // Even at top
        assert_eq!(panel.render_offset_y(), 0.0); // No hide in non-fullscreen
    }
}
