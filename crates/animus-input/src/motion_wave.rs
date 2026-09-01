//! MotionWave High-Fidelity Kinetic Gesture & Inertia Engine (Part 30 & FIX3-05/07).
//!
//! Recognizes trackpad gestures:
//! - 3-finger swipe up: CockpitView
//! - 3-finger swipe down: Close CockpitView
//! - 3-finger swipe left/right: Virtual Desktop switching (DesktopNext / DesktopPrev)
//! - 4-finger pinch-out: Show Desktop Toggle

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::SpringSolver;
use serde::{Deserialize, Serialize};
use tracing::info;

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
    recent_velocities_y: [f32; 3],
    recent_velocities_x: [f32; 3],
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
            recent_velocities_y: [0.0; 3],
            recent_velocities_x: [0.0; 3],
            velocity_idx: 0,
            threshold_px: 50.0,
        }
    }

    pub fn on_swipe_begin(&mut self, fingers: u32) {
        self.active_fingers = fingers;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
        self.recent_velocities_x = [0.0; 3];
        self.recent_velocities_y = [0.0; 3];
        self.bus.publish(AEEvent::SwipeBegin { fingers: fingers as u8 });
    }

    pub fn on_swipe_update(&mut self, dx: f32, dy: f32) {
        self.accumulated_dx += dx;
        self.accumulated_dy += dy;

        self.recent_velocities_x[self.velocity_idx] = dx;
        self.recent_velocities_y[self.velocity_idx] = dy;
        self.velocity_idx = (self.velocity_idx + 1) % 3;

        self.bus.publish(AEEvent::SwipeUpdate { dx, dy });

        if self.active_fingers == 3 {
            // Vertical gesture
            if self.accumulated_dy < -self.threshold_px {
                info!("MotionWave: Recognized 3-finger swipe up -> Open CockpitView");
                self.bus.publish(AEEvent::CockpitViewOpened);
                self.accumulated_dy = 0.0;
            } else if self.accumulated_dy > self.threshold_px {
                info!("MotionWave: Recognized 3-finger swipe down -> Close CockpitView");
                self.bus.publish(AEEvent::CockpitViewClosed);
                self.accumulated_dy = 0.0;
            }

            // Horizontal desktop switching
            if self.accumulated_dx < -self.threshold_px {
                info!("MotionWave: Recognized 3-finger swipe left -> DesktopNext");
                self.bus.publish(AEEvent::DesktopNext);
                self.accumulated_dx = 0.0;
            } else if self.accumulated_dx > self.threshold_px {
                info!("MotionWave: Recognized 3-finger swipe right -> DesktopPrev");
                self.bus.publish(AEEvent::DesktopPrev);
                self.accumulated_dx = 0.0;
            }
        } else if self.active_fingers == 4 {
            if self.accumulated_dy.abs() > self.threshold_px {
                info!("MotionWave: Recognized 4-finger gesture -> ShowDesktopToggle");
                self.bus.publish(AEEvent::ShowDesktopToggle);
                self.accumulated_dy = 0.0;
            }
        }
    }

    pub fn on_swipe_end(&mut self, cancelled: bool) {
        self.active_fingers = 0;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
        self.bus.publish(AEEvent::SwipeEnd { cancelled });
    }

    /// Calculates clamped 3-frame average velocity for window throw physics.
    pub fn calculate_throw_velocity(&self) -> (f32, f32) {
        let avg_x: f32 = self.recent_velocities_x.iter().sum::<f32>() / 3.0;
        let avg_y: f32 = self.recent_velocities_y.iter().sum::<f32>() / 3.0;
        (avg_x.clamp(-2500.0, 2500.0), avg_y.clamp(-2500.0, 2500.0))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motion_wave_gesture_detection() {
        let bus = EventBus::new();
        let mut mw = MotionWave::new(bus);

        mw.on_swipe_begin(3);
        mw.on_swipe_update(0.0, -70.0);
        mw.on_swipe_end(false);

        mw.on_swipe_begin(3);
        mw.on_swipe_update(70.0, 0.0);
        mw.on_swipe_end(false);
    }
}
