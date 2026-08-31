//! Shutdown Screen / Power Management Dialog (Part 29.13 of spec).
//!
//! Floating Altitude (48px Kawase Blur) with Sleep, Restart, Shutdown actions
//! and 60-second countdown timer.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerAction {
    Sleep,
    Restart,
    Shutdown,
}

pub struct ShutdownScreen {
    pub is_active: RwLock<bool>,
    pub altitude: SurfaceAltitude, // Floating (48px Kawase Blur, 64% Opacity)
    pub opacity: RwLock<SpringSolver>, // SPRING_SELECTION (400, 28): 0.0 -> 1.0
    pub countdown_seconds: RwLock<f32>, // 60.0s -> 0.0s
    pub selected_action: RwLock<PowerAction>,
    bus: EventBus,
}

impl ShutdownScreen {
    pub fn new(bus: EventBus) -> Self {
        Self {
            is_active: RwLock::new(false),
            altitude: SurfaceAltitude::Floating,
            opacity: RwLock::new(SpringSolver::new(0.0, SpringProfile::Selection)),
            countdown_seconds: RwLock::new(60.0),
            selected_action: RwLock::new(PowerAction::Shutdown),
            bus,
        }
    }

    /// Opens the power management / shutdown dialog.
    pub fn open(&self, action: PowerAction) {
        let mut active = self.is_active.write();
        *active = true;
        *self.selected_action.write() = action;
        *self.countdown_seconds.write() = 60.0;
        self.opacity.write().set_target(1.0);
        info!("ShutdownScreen: Opened with default action -> {:?}", action);
    }

    /// Cancels the power dialog.
    pub fn cancel(&self) {
        let mut active = self.is_active.write();
        *active = false;
        self.opacity.write().set_target(0.0);
        info!("ShutdownScreen: Cancelled by user.");
    }

    /// Confirms and dispatches the power action.
    pub fn confirm(&self) {
        let action = *self.selected_action.read();
        info!("ShutdownScreen: Confirmed power action -> {:?}", action);
        
        match action {
            PowerAction::Sleep => self.bus.publish(AEEvent::SystemSleep),
            PowerAction::Restart => self.bus.publish(AEEvent::ShutdownRequested),
            PowerAction::Shutdown => self.bus.publish(AEEvent::ShutdownRequested),
        }
    }

    /// Ticks the 60-second automatic countdown.
    pub fn update(&self, dt: f32) {
        self.opacity.write().update(dt);

        if *self.is_active.read() {
            let mut sec = self.countdown_seconds.write();
            *sec = (*sec - dt).max(0.0);
            if *sec <= 0.0 {
                drop(sec);
                self.confirm();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_screen_countdown_and_cancel() {
        let bus = EventBus::new();
        let screen = ShutdownScreen::new(bus);

        assert!(!*screen.is_active.read());
        screen.open(PowerAction::Restart);
        assert!(*screen.is_active.read());
        assert_eq!(*screen.selected_action.read(), PowerAction::Restart);

        screen.update(1.0);
        assert_eq!(*screen.countdown_seconds.read(), 59.0);

        screen.cancel();
        assert!(!*screen.is_active.read());
        assert_eq!(screen.opacity.read().target, 0.0);
    }
}
