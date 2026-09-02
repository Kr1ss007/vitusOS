//! AnimusEngine Backend Abstraction.
//!
//! Provides a unified trait over the two rendering backends:
//! - `DrmBackend`: Production bare-metal DRM/KMS output with GBM buffer allocation
//! - `WinitBackend`: WSL2/development Winit window backend for testing
//!
//! The backend is responsible for:
//! 1. Creating the Wayland display and binding outputs
//! 2. Allocating scanout buffers (GBM or Winit surface)
//! 3. Driving the frame loop (DRM vblank or Winit redraw events)
//! 4. Importing Vulkan DMA-BUF images for zero-copy scanout

pub mod drm;
pub mod winit;

#[cfg(target_os = "linux")]
pub use self::drm::AnimusDrmBackend;
pub use self::winit::AnimusWinitBackend;

use anyhow::Result;

/// Unified backend trait — implemented by DrmBackend and WinitBackend.
pub trait AnimusBackend: Send {
    /// Backend identifier (for logging/crash reports)
    fn name(&self) -> &'static str;

    /// Returns true if the backend has a GPU DRI device available.
    fn has_gpu(&self) -> bool;

    /// Requests the backend to schedule a new frame on the next vblank/redraw.
    fn schedule_frame(&mut self);

    /// Returns the current output resolution (width, height, refresh_hz).
    fn output_geometry(&self) -> (u32, u32, u32);
}
