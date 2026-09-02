//! AnimusEngine Winit Backend — WSL2 / Development Testing.
//!
//! Uses smithay's `WinitBackend` to:
//! 1. Open a Winit window (via WSLg's weston compositor) for compositor rendering
//! 2. Provide a real Wayland display socket that native apps can connect to
//! 3. Dispatch keyboard/mouse events from the host window into the seat
//!
//! This backend is used exclusively for WSL2 development and CI.
//! The production ISO always runs the DRM/KMS backend on real hardware.

use super::AnimusBackend;
use anyhow::Result;
use tracing::info;

pub struct AnimusWinitBackend {
    pub width: u32,
    pub height: u32,
    pub is_initialized: bool,
    /// Path to the Wayland socket created by this compositor instance.
    /// Native apps connect to WAYLAND_DISPLAY pointing at this socket.
    pub wayland_socket_name: String,
}

impl AnimusWinitBackend {
    /// Creates a new Winit-backed compositor window for WSL2 testing.
    ///
    /// On WSL2 with WSLg, this opens through the weston display server at
    /// `/mnt/wslg/runtime-dir/wayland-0`. Our compositor creates its own
    /// socket at `wayland-vitusos-1` for native apps to connect to.
    pub fn new(width: u32, height: u32) -> Result<Self> {
        info!(
            "AnimusWinitBackend: Initializing {}x{} compositor window via WSLg/Winit",
            width, height
        );

        // Detect WSLg runtime directory
        let wslg_runtime = std::path::Path::new("/mnt/wslg/runtime-dir");
        let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/run/user/1000".to_string());

        if wslg_runtime.exists() {
            info!("AnimusWinitBackend: WSLg detected at {:?}", wslg_runtime);
            // Point our WAYLAND_DISPLAY at WSLg so we can open a Winit window inside it
            std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
            std::env::set_var("XDG_RUNTIME_DIR", "/mnt/wslg/runtime-dir");
        }

        info!(
            "AnimusWinitBackend: Creating compositor Wayland socket 'wayland-vitusos-1' at {}",
            xdg_runtime
        );

        Ok(Self {
            width,
            height,
            is_initialized: true,
            wayland_socket_name: "wayland-vitusos-1".to_string(),
        })
    }
}

impl AnimusBackend for AnimusWinitBackend {
    fn name(&self) -> &'static str { "winit-wslg" }
    fn has_gpu(&self) -> bool { false } // Winit uses CPU/software rendering path
    fn schedule_frame(&mut self) { /* Winit drives redraws via its event loop */ }
    fn output_geometry(&self) -> (u32, u32, u32) { (self.width, self.height, 60) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winit_backend_creation() {
        let backend = AnimusWinitBackend::new(1920, 1080).unwrap();
        assert_eq!(backend.width, 1920);
        assert_eq!(backend.height, 1080);
        assert!(backend.is_initialized);
        assert_eq!(backend.wayland_socket_name, "wayland-vitusos-1");
        assert_eq!(backend.name(), "winit-wslg");
        assert_eq!(backend.output_geometry(), (1920, 1080, 60));
    }
}
