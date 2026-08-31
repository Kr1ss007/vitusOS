//! Spring Physics Solver using Semi-Implicit Euler integration.
//!
//! Implements canonical motion profiles and 2D edge-resistance solvers (FIX3-03, FIX3-04).

use serde::{Deserialize, Serialize};

/// Named motion profiles with exact stiffness and damping ratios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpringProfile {
    /// General selection, pills, popovers, cockpit transitions (400, 28)
    Selection,
    /// Fast snappy shake / snap (500, 32)
    Snap,
    /// Window dragging with snappy follow (800, 35)
    WindowDrag,
    /// Dual-layer shadow lag (300, 25)
    Shadow,
    /// Micro-interactions, button hover, search expansion (600, 40)
    Hover,
    /// Kinetic fling and table scrolling (80, 18)
    Scroll,
    /// Interactive window resizing (500, 30)
    Resize,
    /// Modal sheets and dropdown menus (420, 30)
    Sheet,
    /// Stage 0/2 boot crossfade and reveal (200, 22)
    Boot,
    /// Notification banner slide and vertical stack (380, 26)
    Notification,
    /// Traffic light hover magnification (700, 38)
    TrafficLight,
    /// Dock icon magnification curve (450, 32)
    DockMagnify,
    /// Lock screen blur reveal (120, 22)
    LockScreen,
    /// Virtual desktop horizontal switcher (350, 26)
    DesktopSwitch,
}

impl SpringProfile {
    #[inline]
    pub const fn params(&self) -> (f32, f32) {
        match self {
            Self::Selection => (400.0, 28.0),
            Self::Snap => (500.0, 32.0),
            Self::WindowDrag => (800.0, 35.0),
            Self::Shadow => (300.0, 25.0),
            Self::Hover => (600.0, 40.0),
            Self::Scroll => (80.0, 18.0),
            Self::Resize => (500.0, 30.0),
            Self::Sheet => (420.0, 30.0),
            Self::Boot => (200.0, 22.0),
            Self::Notification => (380.0, 26.0),
            Self::TrafficLight => (700.0, 38.0),
            Self::DockMagnify => (450.0, 32.0),
            Self::LockScreen => (120.0, 22.0),
            Self::DesktopSwitch => (350.0, 26.0),
        }
    }
}

/// 1D Spring Solver driven by semi-implicit Euler integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringSolver {
    pub value: f32,
    pub target: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub epsilon: f32,
}

impl SpringSolver {
    pub fn new(initial_value: f32, profile: SpringProfile) -> Self {
        let (stiffness, damping) = profile.params();
        Self {
            value: initial_value,
            target: initial_value,
            velocity: 0.0,
            stiffness,
            damping,
            epsilon: 0.001,
        }
    }

    pub fn with_params(initial_value: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            value: initial_value,
            target: initial_value,
            velocity: 0.0,
            stiffness,
            damping,
            epsilon: 0.001,
        }
    }

    #[inline]
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    #[inline]
    pub fn snap(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
    }

    #[inline]
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }

    #[inline]
    pub fn is_settled(&self) -> bool {
        (self.value - self.target).abs() < self.epsilon && self.velocity.abs() < self.epsilon
    }

    /// Advances the spring by dt (in seconds) using semi-implicit Euler integration.
    pub fn update(&mut self, dt: f32) -> f32 {
        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
            return self.value;
        }

        let substeps = ((dt / (1.0 / 120.0)).ceil() as usize).clamp(1, 4);
        let sub_dt = dt / substeps as f32;

        for _ in 0..substeps {
            let displacement = self.value - self.target;
            let spring_force = -self.stiffness * displacement;
            let damping_force = -self.damping * self.velocity;
            let acceleration = spring_force + damping_force;

            self.velocity += acceleration * sub_dt;
            self.value += self.velocity * sub_dt;
        }

        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
        }

        self.value
    }
}

/// 2D Spring Solver with edge resistance boundaries (FIX3-03, FIX3-04).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringSolver2D {
    pub x: SpringSolver,
    pub y: SpringSolver,
    pub min_x: Option<f32>,
    pub max_x: Option<f32>,
    pub min_y: Option<f32>,
    pub max_y: Option<f32>,
    pub resistance_px: f32,
}

impl SpringSolver2D {
    pub fn new(x: f32, y: f32, profile: SpringProfile) -> Self {
        Self {
            x: SpringSolver::new(x, profile),
            y: SpringSolver::new(y, profile),
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
            resistance_px: 32.0,
        }
    }

    #[inline]
    pub fn set_target(&mut self, x: f32, y: f32) {
        self.x.set_target(x);
        self.y.set_target(y);
    }

    #[inline]
    pub fn snap(&mut self, x: f32, y: f32) {
        self.x.snap(x);
        self.y.snap(y);
    }

    #[inline]
    pub fn set_velocity(&mut self, vx: f32, vy: f32) {
        self.x.set_velocity(vx);
        self.y.set_velocity(vy);
    }

    pub fn enable_edge_resistance_x(&mut self, min: f32, max: f32, resistance_px: f32) {
        self.min_x = Some(min);
        self.max_x = Some(max);
        self.resistance_px = resistance_px;
    }

    pub fn enable_edge_resistance_y(&mut self, min: f32, max: f32, resistance_px: f32) {
        self.min_y = Some(min);
        self.max_y = Some(max);
        self.resistance_px = resistance_px;
    }

    #[inline]
    pub fn is_settled(&self) -> bool {
        self.x.is_settled() && self.y.is_settled()
    }

    #[inline]
    pub fn update(&mut self, dt: f32) -> (f32, f32) {
        let mut px = self.x.update(dt);
        let mut py = self.y.update(dt);

        if let (Some(min), Some(max)) = (self.min_x, self.max_x) {
            px = px.clamp(min - self.resistance_px, max + self.resistance_px);
        }
        if let (Some(min), Some(max)) = (self.min_y, self.max_y) {
            py = py.clamp(min - self.resistance_px, max + self.resistance_px);
        }

        (px, py)
    }

    #[inline]
    pub fn values(&self) -> (f32, f32) {
        (self.x.value, self.y.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_settling() {
        let mut spring = SpringSolver::new(0.0, SpringProfile::Selection);
        spring.set_target(100.0);

        let dt = 1.0 / 60.0;
        for _ in 0..120 {
            spring.update(dt);
        }

        assert!(spring.is_settled());
        assert!((spring.value - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_injection_and_edge_resistance() {
        let mut spring2d = SpringSolver2D::new(0.0, 0.0, SpringProfile::WindowDrag);
        spring2d.enable_edge_resistance_x(0.0, 1920.0, 32.0);
        spring2d.enable_edge_resistance_y(28.0, 1080.0, 32.0);

        spring2d.set_velocity(500.0, 200.0);
        spring2d.update(1.0 / 60.0);
        assert!(spring2d.x.value > 0.0);
        assert!(spring2d.y.value > 0.0);
    }
}
