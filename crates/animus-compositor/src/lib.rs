//! animus-compositor library core modules.

pub mod backend;
pub mod compositor_renderer;
pub mod shell;
pub mod shell_controller;
pub mod sound_manager;
pub mod state;
pub mod wayland;
pub mod window;
pub mod window_manager;
pub mod workspace;

pub use backend::*;
pub use shell::*;
pub use state::*;
pub use wayland::*;
pub use window::*;
pub use workspace::*;

#[cfg(target_os = "linux")]
pub mod smithay;
