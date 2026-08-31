//! MotionWave High-Fidelity Gesture & Inertia Engine.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::SpringSolver;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GestureType {
    None,
    Swipe3Up,
    Swipe3Down,
    Swipe3Left,
    Swipe3Right,
    PinchIn,
    PinchOut,
}

pub struct MotionWave {
    bus: EventBus,
    active_fingers: u32,
    accumulated_dx: f32,
    accumulated_dy: f32,
    recent_velocities: [f32; 3],
    velocity_idx: usize,
    threshold_px: f32,
}

impl MotionWave {
    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            active_fingers: 0,
            accumulated_dx: 0.0,
            accumulated_dy: 0.0,
            recent_velocities: [0.0; 3],
            velocity_idx: 0,
            threshold_px: 60.0,
        }
    }

    pub fn on_swipe_begin(&mut self, fingers: u32) {
        self.active_fingers = fingers;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
    }

    pub fn on_swipe_update(&mut self, dx: f32, dy: f32) {
        self.accumulated_dx += dx;
        self.accumulated_dy += dy;

        // Record velocity history for throw momentum
        self.recent_velocities[self.velocity_idx] = dy;
        self.velocity_idx = (self.velocity_idx + 1) % 3;

        if self.active_fingers == 3 {
            if self.accumulated_dy < -self.threshold_px {
                self.bus.publish(AEEvent::CockpitViewOpened);
                self.accumulated_dy = 0.0;
            } else if self.accumulated_dy > self.threshold_px {
                self.bus.publish(AEEvent::CockpitViewClosed);
                self.accumulated_dy = 0.0;
            }
        }
    }

    pub fn on_swipe_end(&mut self, _cancelled: bool) {
        self.active_fingers = 0;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
    }

    /// Calculates clamped 3-frame average velocity for window throw physics.
    pub fn calculate_throw_velocity(&self) -> f32 {
        let avg: f32 = self.recent_velocities.iter().sum::<f32>() / 3.0;
        avg.clamp(-2000.0, 2000.0)
    }

    /// Applies 32px soft boundary resistance to spring targets at screen edges.
    pub fn apply_edge_resistance(pos: f32, min_bound: f32, max_bound: f32, spring: &mut SpringSolver) {
        const RESIST_ZONE: f32 = 32.0;
        const RESIST_K: f32 = 0.3;

        if pos < min_bound {
            let penetration = min_bound - pos;
            let force = RESIST_K * (penetration / RESIST_ZONE) * spring.stiffness;
            spring.velocity += force;
        } else if pos > max_bound {
            let penetration = pos - max_bound;
            let force = -RESIST_K * (penetration / RESIST_ZONE) * spring.stiffness;
            spring.velocity += force;
        }
    }
}
