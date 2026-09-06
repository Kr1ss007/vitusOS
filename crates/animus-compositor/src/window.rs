//! AEWindow -- AnimusEngine Window with Fullscreen, Minimize, Size Constraints (Parts 40/41/45).
//!
//! Window size constraints (Part 45.1):
//!   Minimum: 200x100px (absolute, never configurable)
//!   Maximum: output dimensions minus Panel height
//!
//! Focus model (Part 45.3): click-to-focus ONLY. No focus-follows-mouse.
//! Z-order: most recently focused = highest z (back of vector).
//!
//! Fullscreen (Part 40): expand to fill output, Panel/Dock auto-hide,
//! floating traffic lights on hover, Esc exits.
//!
//! Minimize (Part 41): window springs toward Dock icon, scale -> 0,
//! ShowDesktopToggle minimizes/restores all in LIFO order.

use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};
use animus_render::altitude::SurfaceAltitude;
use serde::{Deserialize, Serialize};

// ── Constants (Part 45) ─────────────────────────────────────────────────────

pub const MIN_WINDOW_WIDTH: f32 = 200.0;
pub const MIN_WINDOW_HEIGHT: f32 = 100.0;
pub const TITLE_BAR_PADDING: f32 = 68.0; // traffic lights area + right padding
pub const TITLE_TRUNCATE_SUFFIX: &str = "...";

// ── Fullscreen State (Part 40) ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullscreenState {
    pub active: bool,
    pub prev_x: f32,
    pub prev_y: f32,
    pub prev_w: f32,
    pub prev_h: f32,
}

impl Default for FullscreenState {
    fn default() -> Self {
        Self { active: false, prev_x: 0.0, prev_y: 0.0, prev_w: 0.0, prev_h: 0.0 }
    }
}

// ── Fullscreen Traffic Lights (Part 40.4) ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullscreenTrafficLights {
    pub visible: bool,
    pub opacity: SpringSolver,     // SPRING_HOVER (600,40)
    pub scale: [SpringSolver; 3],  // SPRING_TRAFFIC_LIGHT per button

    pub const_hot_zone_w: f32,
    pub const_hot_zone_h: f32,
    pub const_button_x: f32,
    pub const_button_y: f32,
    pub const_button_size: f32,
    pub const_button_gap: f32,
}

impl Default for FullscreenTrafficLights {
    fn default() -> Self {
        Self {
            visible: false,
            opacity: SpringSolver::new(0.0, SpringProfile::Hover),
            scale: [
                SpringSolver::new(1.0, SpringProfile::TrafficLight),
                SpringSolver::new(1.0, SpringProfile::TrafficLight),
                SpringSolver::new(1.0, SpringProfile::TrafficLight),
            ],
            const_hot_zone_w: 60.0,
            const_hot_zone_h: 32.0,
            const_button_x: 12.0,
            const_button_y: 10.0,
            const_button_size: 12.0,
            const_button_gap: 8.0,
        }
    }
}

// ── Traffic Light Buttons ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficLightButtons {
    pub close_hover: SpringSolver,    // SPRING_TRAFFIC_LIGHT (700, 38)
    pub minimize_hover: SpringSolver, // SPRING_TRAFFIC_LIGHT (700, 38)
    pub maximize_hover: SpringSolver, // SPRING_TRAFFIC_LIGHT (700, 38)
}

impl Default for TrafficLightButtons {
    fn default() -> Self {
        Self {
            close_hover: SpringSolver::new(0.0, SpringProfile::TrafficLight),
            minimize_hover: SpringSolver::new(0.0, SpringProfile::TrafficLight),
            maximize_hover: SpringSolver::new(0.0, SpringProfile::TrafficLight),
        }
    }
}

// ── AEWindow ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AEWindow {
    pub handle: u64,
    pub title: String,
    pub app_id: String,
    pub pos: SpringSolver2D,        // SPRING_WINDOW_DRAG (800, 35)
    pub shadow_pos: SpringSolver2D, // SPRING_SHADOW (300, 25) - Lags pos
    pub scale: SpringSolver,       // SPRING_SELECTION: 0.95 -> 1.0
    pub opacity: SpringSolver,     // SPRING_SELECTION: 0.0 <-> 1.0 for minimize
    pub traffic_lights: TrafficLightButtons,
    pub altitude: SurfaceAltitude,
    pub width: f32,
    pub height: f32,
    pub min_width: f32,            // App-requested min (system min is 200px)
    pub min_height: f32,           // App-requested min (system min is 100px)
    pub corner_radius: f32,
    pub is_focused: bool,
    pub is_visible: bool,
    pub is_minimized: bool,

    // Fullscreen state (Part 40)
    pub fullscreen: FullscreenState,
    pub fullscreen_traffic_lights: FullscreenTrafficLights,

    // Minimize context (Part 41)
    pub minimize_origin_x: f32,
    pub minimize_origin_y: f32,
    pub minimize_origin_w: f32,
    pub minimize_origin_h: f32,
}

impl AEWindow {
    pub fn new(handle: u64, title: impl Into<String>, app_id: impl Into<String>, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            handle,
            title: title.into(),
            app_id: app_id.into(),
            pos: SpringSolver2D::new(x, y, SpringProfile::WindowDrag),
            shadow_pos: SpringSolver2D::new(x, y, SpringProfile::Shadow),
            scale: SpringSolver::new(0.95, SpringProfile::Selection),
            opacity: SpringSolver::new(1.0, SpringProfile::Selection),
            traffic_lights: TrafficLightButtons::default(),
            altitude: SurfaceAltitude::Mid,
            width: w.max(MIN_WINDOW_WIDTH),
            height: h.max(MIN_WINDOW_HEIGHT),
            min_width: MIN_WINDOW_WIDTH,
            min_height: MIN_WINDOW_HEIGHT,
            corner_radius: 10.0,
            is_focused: true,
            is_visible: true,
            is_minimized: false,
            fullscreen: FullscreenState::default(),
            fullscreen_traffic_lights: FullscreenTrafficLights::default(),
            minimize_origin_x: x,
            minimize_origin_y: y,
            minimize_origin_w: w,
            minimize_origin_h: h,
        }
    }

    pub fn set_target_position(&mut self, x: f32, y: f32) {
        self.pos.set_target(x, y);
        self.shadow_pos.set_target(x, y);
    }

    pub fn set_size(&mut self, w: f32, h: f32) {
        let w = w.max(self.min_width.max(MIN_WINDOW_WIDTH));
        let h = h.max(self.min_height.max(MIN_WINDOW_HEIGHT));
        self.width = w;
        self.height = h;
    }

    /// Returns truncated title for title bar display (Part 45.2).
    /// Available width = window width - 136px (traffic lights + padding).
    pub fn truncated_title(&self) -> String {
        let available = self.width - (TITLE_BAR_PADDING * 2.0);
        if available <= 0.0 || self.title.is_empty() {
            return String::new();
        }
        // Rough estimate: ~7px per character at 13px Inter Semibold
        let max_chars = (available / 7.0) as usize;
        if self.title.chars().count() <= max_chars {
            return self.title.clone();
        }
        let truncated: String = self.title.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}{}", truncated, TITLE_TRUNCATE_SUFFIX)
    }

    // -- Fullscreen (Part 40) --

    pub fn enter_fullscreen(&mut self, output_w: f32, output_h: f32) {
        if self.fullscreen.active { return; }
        self.fullscreen.active = true;
        self.fullscreen.prev_x = self.pos.x.value;
        self.fullscreen.prev_y = self.pos.y.value;
        self.fullscreen.prev_w = self.width;
        self.fullscreen.prev_h = self.height;

        self.pos.set_target(0.0, 0.0);
        self.scale.set_target(1.0);
        self.set_size(output_w, output_h);
        self.is_visible = true;
    }

    pub fn exit_fullscreen(&mut self) {
        if !self.fullscreen.active { return; }
        self.fullscreen.active = false;
        self.pos.set_target(self.fullscreen.prev_x, self.fullscreen.prev_y);
        self.set_size(self.fullscreen.prev_w, self.fullscreen.prev_h);
    }

    pub fn is_fullscreen(&self) -> bool { self.fullscreen.active }

    // -- Minimize (Part 41) --

    pub fn minimize(&mut self, icon_x: f32, icon_y: f32) {
        if self.is_minimized { return; }
        self.is_minimized = true;
        self.minimize_origin_x = self.pos.x.value;
        self.minimize_origin_y = self.pos.y.value;
        self.minimize_origin_w = self.width;
        self.minimize_origin_h = self.height;

        // Spring toward Dock icon
        self.pos.set_target(icon_x - self.width * 0.5, icon_y - self.height * 0.5);
        self.scale.set_target(0.05);
        self.opacity.set_target(0.0);
    }

    pub fn restore(&mut self) {
        if !self.is_minimized { return; }
        self.is_minimized = false;
        self.pos.set_target(self.minimize_origin_x, self.minimize_origin_y);
        self.scale.set_target(0.95);
        self.opacity.set_target(1.0);
    }

    pub fn is_renderable(&self) -> bool {
        self.is_visible && !self.is_minimized && self.opacity.value > 0.01
    }

    pub fn update(&mut self, dt: f32) {
        self.pos.update(dt);
        self.shadow_pos.update(dt);
        self.scale.update(dt);
        self.opacity.update(dt);
        self.traffic_lights.close_hover.update(dt);
        self.traffic_lights.minimize_hover.update(dt);
        self.traffic_lights.maximize_hover.update(dt);

        // Fullscreen traffic lights
        self.fullscreen_traffic_lights.opacity.update(dt);
        for s in &mut self.fullscreen_traffic_lights.scale {
            s.update(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_size_constraints() {
        let win = AEWindow::new(1, "Test", "test", 100.0, 100.0, 50.0, 50.0);
        assert_eq!(win.width, MIN_WINDOW_WIDTH);   // clamped to 200
        assert_eq!(win.height, MIN_WINDOW_HEIGHT); // clamped to 100
    }

    #[test]
    fn test_title_truncation() {
        let mut win = AEWindow::new(1, "A very long title that should be truncated", "test", 0.0, 0.0, 300.0, 200.0);
        let truncated = win.truncated_title();
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() < win.title.len());

        // Short title fits
        win.title = "Hi".to_string();
        assert_eq!(win.truncated_title(), "Hi");
    }

    #[test]
    fn test_fullscreen_lifecycle() {
        let mut win = AEWindow::new(1, "Test", "test", 100.0, 100.0, 800.0, 600.0);
        assert!(!win.is_fullscreen());

        win.enter_fullscreen(1920.0, 1080.0);
        assert!(win.is_fullscreen());
        assert_eq!(win.width, 1920.0);
        assert_eq!(win.height, 1080.0);

        win.exit_fullscreen();
        assert!(!win.is_fullscreen());
        assert_eq!(win.width, 800.0);
        assert_eq!(win.height, 600.0);
    }

    #[test]
    fn test_minimize_restore() {
        let mut win = AEWindow::new(1, "Test", "test", 100.0, 100.0, 800.0, 600.0);
        assert!(!win.is_minimized);
        assert!(win.is_renderable());

        win.minimize(960.0, 1040.0); // Dock icon position
        assert!(win.is_minimized);
        assert!(!win.is_renderable()); // opacity target is 0

        // Tick springs to settle
        for _ in 0..120 {
            win.update(0.016);
        }

        win.restore();
        assert!(!win.is_minimized);
    }
}
