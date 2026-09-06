//! Libinput / Udev Seat Integration for Bare Metal AnimusEngine.
//!
//! When running on real hardware (DRM backend), this module sets up `udev` 
//! to discover input devices and `libinput` to process raw evdev events.
//! These events are then translated and routed to `AnimusSeat`.

#[cfg(target_os = "linux")]
use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use input::{Libinput, LibinputInterface, event::Event, event::keyboard::KeyboardEventTrait, event::pointer::PointerEventTrait};
#[cfg(target_os = "linux")]
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use tracing::{info, warn};

use crate::wayland::seat::AnimusSeat;

/// Interface implementation for libinput to open/close device nodes (/dev/input/eventX).
#[cfg(target_os = "linux")]
struct AnimusLibinputInterface;

#[cfg(target_os = "linux")]
impl LibinputInterface for AnimusLibinputInterface {
    fn open_restricted(&mut self, path: &std::path::Path, flags: i32) -> Result<OwnedFd, i32> {
        let oflags = nix::fcntl::OFlag::from_bits_truncate(flags);
        let raw = nix::fcntl::open(path, oflags, nix::sys::stat::Mode::empty())
            .map_err(|e| e as i32)?;
        // Safety: fd comes from a successful open() call
        Ok(unsafe { OwnedFd::from_raw_fd(raw) })
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        // OwnedFd closes on drop
        drop(fd);
    }
}

/// The bare-metal input processor using `udev` and `libinput`.
pub struct UdevLibinputSeat {
    #[cfg(target_os = "linux")]
    pub context: Libinput,
    pub seat_id: String,
}

impl UdevLibinputSeat {
    /// Creates a new libinput context attached to a specific seat (usually "seat0").
    pub fn new(seat_id: &str) -> anyhow::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let mut context = Libinput::new_with_udev(AnimusLibinputInterface);
            context.udev_assign_seat(seat_id).map_err(|_| anyhow::anyhow!("Failed to assign seat to libinput"))?;
            
            info!("UdevLibinputSeat: Initialized libinput for '{}'", seat_id);
            
            Ok(Self {
                context,
                seat_id: seat_id.to_string(),
            })
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            info!("UdevLibinputSeat: Stubbed for non-Linux platform.");
            Ok(Self {
                seat_id: seat_id.to_string(),
            })
        }
    }

    /// Dispatches available events from libinput into the `AnimusSeat`.
    /// Called once per frame in the main event loop.
    pub fn dispatch_events(&mut self, seat: &mut AnimusSeat) {
        #[cfg(target_os = "linux")]
        {
            if let Err(e) = self.context.dispatch() {
                warn!("UdevLibinputSeat: Dispatch error: {}", e);
                return;
            }

            for event in &mut self.context {
                match event {
                    Event::Pointer(pointer_event) => {
                        use input::event::pointer::PointerEvent;
                        match pointer_event {
                            PointerEvent::Motion(m) => {
                                let dx = m.dx();
                                let dy = m.dy();
                                seat.dispatch_pointer_motion(
                                    seat.pointer.x + dx,
                                    seat.pointer.y + dy,
                                    seat.pointer.focused_surface_id,
                                );
                            }
                            PointerEvent::Button(b) => {
                                use input::event::pointer::ButtonState as LibinputButtonState;
                                use crate::wayland::seat::ButtonState;
                                let btn = b.button();
                                let state = if b.button_state() == LibinputButtonState::Pressed {
                                    ButtonState::Pressed
                                } else {
                                    ButtonState::Released
                                };
                                let pb = match btn {
                                    0x110 => crate::wayland::seat::PointerButton::Left,
                                    0x111 => crate::wayland::seat::PointerButton::Right,
                                    0x112 => crate::wayland::seat::PointerButton::Middle,
                                    _ => crate::wayland::seat::PointerButton::Left,
                                };
                                seat.dispatch_pointer_button(pb, state, 0);
                            }
                            PointerEvent::Axis(_) => {
                                // Scroll events handled via PointerEventTrait::axis_value
                            }
                            _ => {}
                        }
                    }
                    Event::Keyboard(keyboard_event) => {
                        use input::event::keyboard::{KeyboardEvent, KeyState as LibinputKeyState};
                        use crate::wayland::seat::KeyState;
                        match keyboard_event {
                            KeyboardEvent::Key(k) => {
                                let key = k.key();
                                let state = if k.key_state() == LibinputKeyState::Pressed {
                                    KeyState::Pressed
                                } else {
                                    KeyState::Released
                                };
                                seat.dispatch_key(key, state, 0);
                            }
                            _ => {}
                        }
                    }
                    Event::Touch(_touch_event) => {
                        // Handle multi-touch gestures via MotionWave
                    }
                    _ => {}
                }
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            let _ = seat;
        }
    }
}
