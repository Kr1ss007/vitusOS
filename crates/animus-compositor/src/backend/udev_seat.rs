//! Libinput / Udev Seat Integration for Bare Metal AnimusEngine.
//!
//! When running on real hardware (DRM backend), this module sets up `udev` 
//! to discover input devices and `libinput` to process raw evdev events.
//! These events are then translated and routed to `AnimusSeat`.

use anyhow::Result;
use input::{Libinput, LibinputInterface, event::Event, event::keyboard::KeyboardEventTrait};
use std::os::unix::io::{FromRawFd, OwnedFd};
use tracing::{info, warn};

use crate::wayland::seat::AnimusSeat;

/// Interface implementation for libinput to open/close device nodes (/dev/input/eventX).
struct AnimusLibinputInterface;

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
    pub context: Libinput,
    pub seat_id: String,
}

impl UdevLibinputSeat {
    /// Creates a new libinput context attached to a specific seat (usually "seat0").
    pub fn new(seat_id: &str) -> anyhow::Result<Self> {
        let mut context = Libinput::new_with_udev(AnimusLibinputInterface);
        context.udev_assign_seat(seat_id).map_err(|_| anyhow::anyhow!("Failed to assign seat to libinput"))?;
        
        info!("UdevLibinputSeat: Initialized libinput for '{}'", seat_id);
        
        Ok(Self {
            context,
            seat_id: seat_id.to_string(),
        })
    }

    /// Dispatches available events from libinput into the `AnimusSeat`.
    /// Called once per frame in the main event loop.
    pub fn dispatch_events(&mut self, seat: &mut AnimusSeat) {
        self.dispatch_events_full(seat, None, 1920.0, 1080.0);
    }

    /// Full event dispatch with boundary clamping and MotionWave gesture detection.
    pub fn dispatch_events_full(
        &mut self,
        seat: &mut AnimusSeat,
        mut motion_wave: Option<&mut animus_input::motion_wave::MotionWave>,
        screen_w: f32,
        screen_h: f32,
    ) {
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
                            let nx = (seat.pointer.x + dx).clamp(0.0, screen_w as f64);
                            let ny = (seat.pointer.y + dy).clamp(0.0, screen_h as f64);
                            seat.dispatch_pointer_motion(
                                nx,
                                ny,
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
                Event::Touch(touch_event) => {
                    use input::event::touch::TouchEvent;
                    if let Some(mw) = &mut motion_wave {
                        match touch_event {
                            TouchEvent::Down(_) => {
                                mw.on_swipe_begin(2);
                            }
                            TouchEvent::Up(_) => {
                                mw.on_swipe_end(false);
                            }
                            TouchEvent::Cancel(_) => {
                                mw.on_swipe_end(true);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
