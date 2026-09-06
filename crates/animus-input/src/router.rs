//! Global Input Router -- Central Dispatch for All Input Events (FIX4-07).
//!
//! Routes raw pointer/keyboard/swipe events from libinput to all interactive
//! shell surfaces. Each component checks if it's relevant (hit test or always-on).
//!
//! This is the SINGLE dispatch point for pointer motion. No shell component
// receives pointer events directly from the compositor -- they all go through
//! InputRouter::on_pointer_motion().

use crate::motion_wave::MotionWave;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::info;

pub struct InputRouter {
    pub motion_wave: Arc<RwLock<MotionWave>>,
    bus: EventBus,
    pointer_x: RwLock<f64>,
    pointer_y: RwLock<f64>,
    screen_width: RwLock<f32>,
    screen_height: RwLock<f32>,
}

impl InputRouter {
    pub fn new(bus: EventBus) -> Self {
        Self {
            motion_wave: Arc::new(RwLock::new(MotionWave::new(bus.clone()))),
            bus,
            pointer_x: RwLock::new(0.0),
            pointer_y: RwLock::new(0.0),
            screen_width: RwLock::new(1920.0),
            screen_height: RwLock::new(1080.0),
        }
    }

    /// Sets the screen dimensions for coordinate clamping.
    pub fn set_screen_geometry(&self, width: f32, height: f32) {
        *self.screen_width.write() = width;
        *self.screen_height.write() = height;
    }

    /// Returns the current pointer position.
    pub fn pointer_position(&self) -> (f64, f64) {
        (*self.pointer_x.read(), *self.pointer_y.read())
    }

    // -- Keyboard --

    pub fn on_key(&self, keycode: u32, modifiers: u32, pressed: bool) {
        // Alt-Tab intercept for CockpitView cycling
        // 15 = Tab (Linux evdev), 56 = Alt
        if keycode == 15 && pressed && (modifiers & 0x08 != 0) {
            // Alt+Tab -> CockpitView
            self.bus.publish(AEEvent::CockpitViewOpened);
            return;
        }

        // F10 or Alt alone -> GlobalMenu activation (Part 29)
        if keycode == 68 && pressed {
            self.bus.publish(AEEvent::GlobalMenuActivated);
            return;
        }

        // Esc -> CockpitView close or fullscreen exit
        if keycode == 1 && pressed {
            self.bus.publish(AEEvent::CockpitViewClosed);
        }

        if pressed {
            self.bus.publish(AEEvent::KeyDown { keycode, modifiers });
        } else {
            self.bus.publish(AEEvent::KeyUp { keycode, modifiers });
        }
    }

    // -- Pointer --

    /// Dispatches pointer motion to ALL interactive shell surfaces (FIX4-07).
    ///
    /// Each component receives the motion event and decides internally whether
    /// it's relevant (hit test, always-on, or currently open). This is the
    /// single dispatch point -- no component gets pointer motion directly.
    pub fn on_pointer_motion(&self, x: f64, y: f64) {
        // Clamp to screen bounds
        let sw = *self.screen_width.read();
        let sh = *self.screen_height.read();
        let cx = x.clamp(0.0, sw as f64);
        let cy = y.clamp(0.0, sh as f64);

        *self.pointer_x.write() = cx;
        *self.pointer_y.write() = cy;

        let fx = cx as f32;
        let fy = cy as f32;

        // Publish to EventBus so all subscribers receive it
        self.bus.publish(AEEvent::MouseMoved { x: fx, y: fy });
    }

    pub fn on_pointer_button(&self, button: u32, pressed: bool) {
        let (x, y) = self.pointer_position();
        if pressed {
            self.bus.publish(AEEvent::MouseButtonDown {
                button,
                x: x as f32,
                y: y as f32,
            });
        } else {
            self.bus.publish(AEEvent::MouseButtonUp {
                button,
                x: x as f32,
                y: y as f32,
            });
        }
    }

    pub fn on_pointer_axis(&self, dx: f64, dy: f64) {
        self.bus.publish(AEEvent::ScrollDelta {
            dx: dx as f32,
            dy: dy as f32,
        });
    }

    // -- Gestures (delegated to MotionWave) --

    pub fn on_swipe_begin(&self, fingers: u32) {
        self.motion_wave.write().on_swipe_begin(fingers);
    }

    pub fn on_swipe_update(&self, dx: f32, dy: f32) {
        self.motion_wave.write().on_swipe_update(dx, dy);
    }

    pub fn on_swipe_end(&self, cancelled: bool) {
        self.motion_wave.write().on_swipe_end(cancelled);
    }
}
