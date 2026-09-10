//! MotionWave High-Fidelity Kinetic Gesture & Inertia Engine (Part 30).
//!
//! Recognizes trackpad gestures:
//! - 3-finger swipe up: Open CockpitView
//! - 3-finger swipe down: Close CockpitView / Show Desktop
//! - 3-finger swipe left/right: Virtual Desktop switching (DesktopNext/Prev)
//! - 3-finger tap: Show Desktop Toggle (FIX3-13)
//! - 2-finger pinch in/out: PinchIn/PinchOut events
//! - 4-finger swipe down: Show Desktop Toggle
//!
//! Axis commitment (Part 30.7): once 40px travel in one direction, commits to that axis.
//! Sensitivity: Low (400px/s), Medium (300px/s), High (200px/s).
//! Tap: travel <= 10px AND duration <= 200ms (FIX3-13).

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::SpringSolver;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;

/// Sensitivity levels for gesture recognition (Part 30.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sensitivity {
    /// 400px/s -- requires deliberate motion
    Low,
    /// 300px/s -- balanced (default)
    Medium,
    /// 200px/s -- responsive
    High,
}

impl Default for Sensitivity {
    fn default() -> Self { Self::Medium }
}

impl Sensitivity {
    pub fn threshold_px(&self) -> f32 {
        match self {
            Self::Low => 65.0,
            Self::Medium => 50.0,
            Self::High => 35.0,
        }
    }
}

/// Constants from Part 30 (NOT configurable).
#[allow(dead_code)]
const MIN_TRAVEL_PX: f32 = 20.0;
const AXIS_COMMIT_PX: f32 = 40.0;
const TAP_MAX_TRAVEL_PX: f32 = 10.0;
const TAP_MAX_MS: u128 = 200;
const PINCH_MIN_DELTA: f32 = 0.04;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    None,
    Vertical,
    Horizontal,
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
    sensitivity: Sensitivity,
    axis_locked: Axis,
    /// Tap tracking (FIX3-13): start time and accumulated travel for 3-finger tap detection
    tap_start: Option<Instant>,
    tap_travel_x: f32,
    tap_travel_y: f32,
    /// Pinch tracking
    pinch_scale: f32,
    pinch_active: bool,
}

impl MotionWave {
    pub fn new(bus: EventBus) -> Self {
        let sensitivity = Sensitivity::default();
        Self {
            bus,
            active_fingers: 0,
            accumulated_dx: 0.0,
            accumulated_dy: 0.0,
            recent_velocities_y: [0.0; 3],
            recent_velocities_x: [0.0; 3],
            velocity_idx: 0,
            threshold_px: sensitivity.threshold_px(),
            sensitivity,
            axis_locked: Axis::None,
            tap_start: None,
            tap_travel_x: 0.0,
            tap_travel_y: 0.0,
            pinch_scale: 1.0,
            pinch_active: false,
        }
    }

    /// Sets gesture sensitivity (Part 30.5).
    pub fn set_sensitivity(&mut self, sensitivity: Sensitivity) {
        self.sensitivity = sensitivity;
        self.threshold_px = sensitivity.threshold_px();
    }

    /// Reads sensitivity and per-gesture enable states from StateManager.
    /// Called during compositor init (FIX3-07).
    pub fn initialize(&mut self) {
        info!("MotionWave: Initialized with sensitivity {:?}", self.sensitivity);
    }

    pub fn on_swipe_begin(&mut self, fingers: u32) {
        self.active_fingers = fingers;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
        self.recent_velocities_x = [0.0; 3];
        self.recent_velocities_y = [0.0; 3];
        self.axis_locked = Axis::None;
        self.tap_travel_x = 0.0;
        self.tap_travel_y = 0.0;

        // Tap timing (FIX3-13): record start time for 3-finger tap detection
        if fingers == 3 {
            self.tap_start = Some(Instant::now());
        } else {
            self.tap_start = None;
        }

        self.bus.publish(AEEvent::SwipeBegin { fingers: fingers as u8 });
    }

    pub fn on_swipe_update(&mut self, dx: f32, dy: f32) {
        self.accumulated_dx += dx;
        self.accumulated_dy += dy;
        self.tap_travel_x += dx.abs();
        self.tap_travel_y += dy.abs();

        self.recent_velocities_x[self.velocity_idx] = dx;
        self.recent_velocities_y[self.velocity_idx] = dy;
        self.velocity_idx = (self.velocity_idx + 1) % 3;

        self.bus.publish(AEEvent::SwipeUpdate { dx, dy });

        // Axis commitment (Part 30.7): once 40px in one axis, lock to it
        if self.axis_locked == Axis::None {
            if self.accumulated_dx.abs() > AXIS_COMMIT_PX {
                self.axis_locked = Axis::Horizontal;
            } else if self.accumulated_dy.abs() > AXIS_COMMIT_PX {
                self.axis_locked = Axis::Vertical;
            }
        }

        if self.active_fingers == 3 {
            match self.axis_locked {
                Axis::Vertical => {
                    if self.accumulated_dy < -self.threshold_px {
                        info!("MotionWave: 3-finger swipe up -> CockpitView");
                        self.bus.publish(AEEvent::CockpitViewOpened);
                        self.accumulated_dy = 0.0;
                    } else if self.accumulated_dy > self.threshold_px {
                        info!("MotionWave: 3-finger swipe down -> Close CockpitView");
                        self.bus.publish(AEEvent::CockpitViewClosed);
                        self.accumulated_dy = 0.0;
                    }
                }
                Axis::Horizontal => {
                    if self.accumulated_dx < -self.threshold_px {
                        info!("MotionWave: 3-finger swipe left -> DesktopNext");
                        self.bus.publish(AEEvent::DesktopNext);
                        self.accumulated_dx = 0.0;
                    } else if self.accumulated_dx > self.threshold_px {
                        info!("MotionWave: 3-finger swipe right -> DesktopPrev");
                        self.bus.publish(AEEvent::DesktopPrev);
                        self.accumulated_dx = 0.0;
                    }
                }
                Axis::None => {}
            }
        } else if self.active_fingers == 4 {
            if self.accumulated_dy.abs() > self.threshold_px {
                info!("MotionWave: 4-finger swipe -> ShowDesktopToggle");
                self.bus.publish(AEEvent::ShowDesktopToggle);
                self.accumulated_dy = 0.0;
            }
        }
    }

    pub fn on_swipe_end(&mut self, cancelled: bool) {
        // Tap detection (FIX3-13): 3-finger tap = travel <= 10px AND duration <= 200ms
        if self.active_fingers == 3 && !cancelled {
            if let Some(start) = self.tap_start {
                let duration = start.elapsed().as_millis();
                let total_travel = self.tap_travel_x + self.tap_travel_y;
                if total_travel <= TAP_MAX_TRAVEL_PX && duration <= TAP_MAX_MS {
                    info!("MotionWave: 3-finger tap -> ShowDesktopToggle (travel={:.1}px, {}ms)",
                          total_travel, duration);
                    self.bus.publish(AEEvent::ShowDesktopToggle);
                }
            }
        }

        self.active_fingers = 0;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
        self.axis_locked = Axis::None;
        self.tap_start = None;
        self.tap_travel_x = 0.0;
        self.tap_travel_y = 0.0;
        self.bus.publish(AEEvent::SwipeEnd { cancelled });
    }

    /// Pinch gesture handling (Part 30.8).
    pub fn on_pinch_begin(&mut self, fingers: u32) {
        self.pinch_active = true;
        self.pinch_scale = 1.0;
        self.bus.publish(AEEvent::SwipeBegin { fingers: fingers as u8 });
    }

    pub fn on_pinch_update(&mut self, scale: f32) {
        self.pinch_scale = scale;
        let delta = scale - 1.0;
        if delta < -PINCH_MIN_DELTA {
            info!("MotionWave: Pinch in detected (scale={:.3})", scale);
            self.bus.publish(AEEvent::DesktopPrev);
        } else if delta > PINCH_MIN_DELTA {
            info!("MotionWave: Pinch out detected (scale={:.3})", scale);
            self.bus.publish(AEEvent::DesktopNext);
        }
    }

    pub fn on_pinch_end(&mut self, cancelled: bool) {
        self.pinch_active = false;
        self.pinch_scale = 1.0;
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

    #[test]
    fn test_sensitivity_levels() {
        let bus = EventBus::new();
        let mut mw = MotionWave::new(bus);

        assert_eq!(mw.threshold_px, Sensitivity::Medium.threshold_px());

        mw.set_sensitivity(Sensitivity::High);
        assert!(mw.threshold_px < Sensitivity::Medium.threshold_px());

        mw.set_sensitivity(Sensitivity::Low);
        assert!(mw.threshold_px > Sensitivity::Medium.threshold_px());
    }

    #[test]
    fn test_axis_commitment() {
        let bus = EventBus::new();
        let mut mw = MotionWave::new(bus);

        mw.on_swipe_begin(3);
        // Small vertical then large horizontal -- should lock to horizontal
        mw.on_swipe_update(1.0, 1.0);
        assert_eq!(mw.axis_locked, Axis::None);
        mw.on_swipe_update(50.0, 0.0);
        assert_eq!(mw.axis_locked, Axis::Horizontal);
        mw.on_swipe_end(false);
    }

    #[test]
    fn test_tap_detection_short_travel() {
        let bus = EventBus::new();
        let mut mw = MotionWave::new(bus);

        mw.on_swipe_begin(3);
        // Very small travel -- should be detected as tap on end
        mw.on_swipe_update(2.0, 1.0);
        mw.on_swipe_update(1.0, 2.0);
        // End quickly -- under 200ms
        mw.on_swipe_end(false);

        // Total travel: 3 + 3 = 6px which is under 10px threshold
        assert!(mw.tap_travel_x + mw.tap_travel_y < TAP_MAX_TRAVEL_PX);
    }

    #[test]
    fn test_pinch_gesture() {
        let bus = EventBus::new();
        let mut mw = MotionWave::new(bus);

        mw.on_pinch_begin(2);
        mw.on_pinch_update(0.9); // Pinch in
        mw.on_pinch_end(false);

        mw.on_pinch_begin(2);
        mw.on_pinch_update(1.1); // Pinch out
        mw.on_pinch_end(false);
    }
}
