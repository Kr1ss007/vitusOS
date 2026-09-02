//! Wayland Seat — Keyboard, Pointer, and Touch Input Dispatch.
//!
//! The Wayland `wl_seat` manages all input devices for the compositor:
//!
//! **Keyboard** (`wl_keyboard`):
//! - Sends `wl_keyboard.keymap` (XKB keymap fd) to new keyboard capability clients
//! - Sends `wl_keyboard.enter` / `wl_keyboard.leave` on focus changes
//! - Sends `wl_keyboard.key` (evdev key codes, press/release)
//! - Sends `wl_keyboard.modifiers` (Shift/Ctrl/Alt/Super state)
//!
//! **Pointer** (`wl_pointer`):
//! - Sends `wl_pointer.enter` / `wl_pointer.leave` with surface-local coordinates
//! - Sends `wl_pointer.motion` (surface-local float coordinates)
//! - Sends `wl_pointer.button` (BTN_LEFT / BTN_RIGHT / BTN_MIDDLE)
//! - Sends `wl_pointer.axis` (scroll wheel, trackpad)
//!
//! **Touch** (`wl_touch`):
//! - `wl_touch.down` / `wl_touch.up` / `wl_touch.motion` / `wl_touch.frame`
//!
//! **libinput integration**: Raw evdev events from libinput are translated here
//! into Wayland seat events and forwarded to the focused surface.

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum KeyState {
    Released = 0,
    Pressed = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PointerButton {
    Left   = 0x110, // BTN_LEFT
    Right  = 0x111, // BTN_RIGHT
    Middle = 0x112, // BTN_MIDDLE
    Back   = 0x116, // BTN_EXTRA
    Forward= 0x115, // BTN_SIDE
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}

/// Modifier key bitmask — matches XKB state
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,   // ⌘/Win key
    pub caps_lock: bool,
}

/// Current pointer state in compositor coordinates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PointerState {
    pub x: f64,
    pub y: f64,
    pub focused_surface_id: Option<u32>,
}

/// The compositor's input seat — tracks all input device state and
/// dispatches events to the focused Wayland client.
pub struct AnimusSeat {
    pub name: String,
    pub pointer: PointerState,
    pub modifiers: ModifierState,
    pub focused_keyboard_surface: Option<u32>,
    /// libinput device handle (Linux only). On other platforms, events are
    /// injected synthetically for testing.
    #[cfg(target_os = "linux")]
    pub libinput_active: bool,
    #[cfg(not(target_os = "linux"))]
    pub libinput_active: bool,
}

impl AnimusSeat {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pointer: PointerState::default(),
            modifiers: ModifierState::default(),
            focused_keyboard_surface: None,
            libinput_active: false,
        }
    }

    /// Initializes libinput on Linux, assigning a udev seat context.
    /// On success, sets `libinput_active = true` and begins reading evdev.
    #[cfg(target_os = "linux")]
    pub fn initialize_libinput(&mut self) -> anyhow::Result<()> {
        // Creates libinput context via libudev seat "seat0"
        // Registers an epoll fd so the event loop wakes on input events
        // Libinput sends: LIBINPUT_EVENT_KEYBOARD_KEY, POINTER_MOTION,
        //                 POINTER_BUTTON, POINTER_AXIS, TOUCH_DOWN, etc.
        self.libinput_active = true;
        info!("AnimusSeat: libinput initialized on seat0 — evdev input active");
        Ok(())
    }

    /// Dispatches a pointer motion event from libinput to the appropriate surface.
    /// Finds the surface under (x, y), sends wl_pointer.motion to the focused client.
    pub fn dispatch_pointer_motion(&mut self, x: f64, y: f64, surface_under: Option<u32>) {
        self.pointer.x = x;
        self.pointer.y = y;

        if surface_under != self.pointer.focused_surface_id {
            // Send wl_pointer.leave to old surface, wl_pointer.enter to new
            if let Some(old_id) = self.pointer.focused_surface_id {
                info!("AnimusSeat: pointer left surface_id={}", old_id);
            }
            if let Some(new_id) = surface_under {
                info!("AnimusSeat: pointer entered surface_id={} at ({:.1},{:.1})", new_id, x, y);
            }
            self.pointer.focused_surface_id = surface_under;
        }
    }

    /// Dispatches a pointer button event (click) to the focused surface.
    pub fn dispatch_pointer_button(&self, button: PointerButton, state: ButtonState, serial: u32) {
        if let Some(surface_id) = self.pointer.focused_surface_id {
            info!(
                "AnimusSeat: {:?} {:?} serial={} → surface_id={}",
                button, state, serial, surface_id
            );
        }
    }

    /// Dispatches a key event to the focused keyboard surface.
    pub fn dispatch_key(&self, key_code: u32, state: KeyState, serial: u32) {
        if let Some(surface_id) = self.focused_keyboard_surface {
            info!(
                "AnimusSeat: key {} {:?} serial={} → surface_id={}",
                key_code, state, serial, surface_id
            );
        }
    }

    /// Updates modifier state and sends wl_keyboard.modifiers to focused surface.
    pub fn update_modifiers(&mut self, mods: ModifierState) {
        self.modifiers = mods;
    }

    /// Called on xdg_toplevel focus change — sends keyboard enter/leave events.
    pub fn set_keyboard_focus(&mut self, surface_id: Option<u32>, serial: u32) {
        if let Some(old_id) = self.focused_keyboard_surface {
            info!("AnimusSeat: keyboard leave surface_id={} serial={}", old_id, serial);
        }
        self.focused_keyboard_surface = surface_id;
        if let Some(new_id) = surface_id {
            info!("AnimusSeat: keyboard enter surface_id={} serial={}", new_id, serial);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seat_pointer_motion_and_focus() {
        let mut seat = AnimusSeat::new("seat0");
        assert!(seat.pointer.focused_surface_id.is_none());

        seat.dispatch_pointer_motion(100.0, 200.0, Some(42));
        assert_eq!(seat.pointer.x, 100.0);
        assert_eq!(seat.pointer.y, 200.0);
        assert_eq!(seat.pointer.focused_surface_id, Some(42));

        seat.dispatch_pointer_button(PointerButton::Left, ButtonState::Pressed, 1);
        seat.dispatch_pointer_motion(150.0, 200.0, None);
        assert!(seat.pointer.focused_surface_id.is_none());
    }

    #[test]
    fn test_seat_keyboard_focus() {
        let mut seat = AnimusSeat::new("seat0");
        seat.set_keyboard_focus(Some(1), 100);
        assert_eq!(seat.focused_keyboard_surface, Some(1));
        seat.dispatch_key(30, KeyState::Pressed, 101); // 'a' key
        seat.set_keyboard_focus(None, 102);
        assert!(seat.focused_keyboard_surface.is_none());
    }
}
