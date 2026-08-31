//! AEWindow Component (AnimusEngine Window) with Lagged Shadow and Spatial Elevation.

use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};
use animus_render::altitude::SurfaceAltitude;
use serde::{Deserialize, Serialize};

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

pub struct AEWindow {
    pub handle: u64,
    pub title: String,
    pub app_id: String,
    pub pos: SpringSolver2D,       // SPRING_WINDOW_DRAG (800, 35)
    pub shadow_pos: SpringSolver2D, // SPRING_SHADOW (300, 25) - Lags pos
    pub scale: SpringSolver,       // SPRING_SELECTION: 0.95 -> 1.0
    pub traffic_lights: TrafficLightButtons,
    pub altitude: SurfaceAltitude,
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub is_focused: bool,
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
            traffic_lights: TrafficLightButtons::default(),
            altitude: SurfaceAltitude::Mid,
            width: w,
            height: h,
            corner_radius: 10.0,
            is_focused: true,
        }
    }

    pub fn set_target_position(&mut self, x: f32, y: f32) {
        self.pos.set_target(x, y);
        self.shadow_pos.set_target(x, y);
    }

    pub fn update(&mut self, dt: f32) {
        self.pos.update(dt);
        self.shadow_pos.update(dt);
        self.scale.update(dt);
        self.traffic_lights.close_hover.update(dt);
        self.traffic_lights.minimize_hover.update(dt);
        self.traffic_lights.maximize_hover.update(dt);
    }
}
