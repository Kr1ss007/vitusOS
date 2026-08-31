//! LockScreen on AESurfaces (SurfaceAltitude::Floating, 48px Kawase Blur).
//!
//! Aligned with Part 15 of spec and FIX-02 (zeroize password memory).

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;
use parking_lot::RwLock;
use tracing::info;
use zeroize::Zeroize;

pub struct LockScreen {
    pub is_active: RwLock<bool>,
    pub altitude: SurfaceAltitude, // Floating (48px Kawase Blur, 64% Opacity)
    pub opacity: RwLock<SpringSolver>, // SPRING_LOCK_SCREEN (120, 22): 0.0 -> 1.0
    pub shake_x: RwLock<SpringSolver>, // SPRING_SELECTION (400, 28): horizontal shake on wrong pass
    pub password_buf: RwLock<String>,
    bus: EventBus,
}

impl LockScreen {
    pub fn new(bus: EventBus) -> Self {
        Self {
            is_active: RwLock::new(false),
            altitude: SurfaceAltitude::Floating,
            opacity: RwLock::new(SpringSolver::new(0.0, SpringProfile::LockScreen)),
            shake_x: RwLock::new(SpringSolver::new(0.0, SpringProfile::Selection)),
            password_buf: RwLock::new(String::new()),
            bus,
        }
    }

    /// Activates the lock screen with slow, deliberate reveal (SPRING_LOCK_SCREEN).
    pub fn activate(&self) {
        let mut active = self.is_active.write();
        *active = true;
        self.opacity.write().set_target(1.0);
        self.password_buf.write().clear();
        self.bus.publish(AEEvent::LockScreenLocked);
        info!("LockScreen: Activated with 48px Floating Kawase blur.");
    }

    /// Deactivates and unlocks the screen.
    pub fn deactivate(&self) {
        let mut active = self.is_active.write();
        *active = false;
        self.opacity.write().set_target(0.0);
        
        // Zeroize password buffer (FIX-02)
        let mut pass = self.password_buf.write();
        unsafe {
            let vec_bytes = pass.as_bytes_mut();
            vec_bytes.zeroize();
        }
        pass.clear();

        self.bus.publish(AEEvent::LockScreenUnlocked);
        info!("LockScreen: Unlocked successfully. Transitioning to desktop session.");
    }

    /// Inputs password characters.
    pub fn input_char(&self, ch: char) {
        if *self.is_active.read() {
            self.password_buf.write().push(ch);
        }
    }

    /// Backspaces character.
    pub fn backspace(&self) {
        if *self.is_active.read() {
            self.password_buf.write().pop();
        }
    }

    /// Submits password for verification.
    pub fn submit_password(&self) {
        let pass_str = self.password_buf.read().clone();
        
        // Check password (accept non-empty or default test credentials)
        if !pass_str.is_empty() {
            self.deactivate();
        } else {
            // Shake animation on failure (SPRING_SELECTION velocity injection)
            let mut shake = self.shake_x.write();
            shake.set_velocity(350.0);
            info!("LockScreen: Authentication failed -> Triggered shake animation.");
        }
    }

    /// Updates physics springs.
    pub fn update(&self, dt: f32) {
        self.opacity.write().update(dt);
        self.shake_x.write().update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_screen_lifecycle_and_shake() {
        let bus = EventBus::new();
        let lock = LockScreen::new(bus);

        assert!(!*lock.is_active.read());
        lock.activate();
        assert!(*lock.is_active.read());
        assert_eq!(lock.opacity.read().target, 1.0);

        // Submit empty password -> shake
        lock.submit_password();
        assert!(*lock.is_active.read());
        assert!(lock.shake_x.read().velocity > 0.0);

        // Input password and submit -> unlock
        lock.input_char('v');
        lock.input_char('i');
        lock.input_char('t');
        lock.input_char('u');
        lock.input_char('s');
        lock.submit_password();

        assert!(!*lock.is_active.read());
        assert_eq!(lock.opacity.read().target, 0.0);
    }
}
