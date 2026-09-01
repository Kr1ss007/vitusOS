//! System Screen for Shutdown & Restart (Part 29.7 of specification).
//!
//! Full-screen pure black (#000000) surface shown during shutdown and restart.
//! Replaces everything. Zero TTY. Zero systemd journal. Zero kernel messages.
//!
//! EXACT STRINGS — locked. Never change. Never localize. Never capitalize:
//! - Shutdown: "goodbye"
//! - Restart:  "i'll see you in a bit"

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

pub const SHUTDOWN_MESSAGE: &str = "goodbye";
pub const RESTART_MESSAGE: &str = "i'll see you in a bit";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemScreenMode {
    Shutdown,
    Restart,
}

pub struct SystemScreen {
    pub mode: RwLock<SystemScreenMode>,
    pub is_active: RwLock<bool>,
    pub opacity: RwLock<SpringSolver>, // SPRING_BOOT (200, 22): 0.0 -> 1.0
    pub is_invoked: RwLock<bool>,
    bus: EventBus,
}

impl SystemScreen {
    pub fn new(bus: EventBus) -> Self {
        Self {
            mode: RwLock::new(SystemScreenMode::Shutdown),
            is_active: RwLock::new(false),
            opacity: RwLock::new(SpringSolver::new(0.0, SpringProfile::Boot)),
            is_invoked: RwLock::new(false),
            bus,
        }
    }

    /// Shows the black system screen and begins the slow, deliberate fade.
    pub fn show(&self, mode: SystemScreenMode) {
        let mut active = self.is_active.write();
        *active = true;
        *self.mode.write() = mode;
        *self.is_invoked.write() = false;
        self.opacity.write().set_target(1.0);

        info!("SystemScreen: Active in {:?} mode. Fading desktop to pure black (#000000).", mode);
    }

    /// Returns the exact canonical string for the current mode.
    pub fn message(&self) -> &'static str {
        match *self.mode.read() {
            SystemScreenMode::Shutdown => SHUTDOWN_MESSAGE,
            SystemScreenMode::Restart => RESTART_MESSAGE,
        }
    }

    /// Ticks the spring animation and dispatches systemd poweroff/reboot upon completion.
    pub fn update(&self, dt: f32) {
        if !*self.is_active.read() {
            return;
        }

        let mut opacity = self.opacity.write();
        opacity.update(dt);

        // When black surface has settled at >= 99% opacity, trigger system power transition
        if opacity.value >= 0.99 && !*self.is_invoked.read() {
            *self.is_invoked.write() = true;
            let mode = *self.mode.read();
            info!("SystemScreen: Blackout settled. Dispatching power transition -> {:?}", mode);

            match mode {
                SystemScreenMode::Shutdown => {
                    self.bus.publish(AEEvent::SystemShutdown);
                }
                SystemScreenMode::Restart => {
                    self.bus.publish(AEEvent::ShutdownRequested);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_screen_exact_strings_and_fade() {
        let bus = EventBus::new();
        let screen = SystemScreen::new(bus);

        assert_eq!(SHUTDOWN_MESSAGE, "goodbye");
        assert_eq!(RESTART_MESSAGE, "i'll see you in a bit");

        screen.show(SystemScreenMode::Shutdown);
        assert_eq!(screen.message(), "goodbye");
        assert!(*screen.is_active.read());

        // Simulate 800ms fade (144Hz ticks)
        for _ in 0..120 {
            screen.update(1.0 / 144.0);
        }

        assert!(*screen.is_invoked.read());
        assert!(screen.opacity.read().value >= 0.99);
    }
}
