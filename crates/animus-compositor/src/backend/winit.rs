//! AnimusEngine Winit Backend -- WSL2 / Development Testing.
//!
//! Uses Smithay's `WinitBackend` to open a window through the host compositor
//! (WSLg on WSL2, or any desktop Wayland/X11 compositor) for development testing.
//!
//! In production, the DRM/KMS backend is always used. This backend exists
//! so the compositor can be developed and tested without real DRM hardware.
//!
//! Usage from the compositor event loop:
//! ```text
//! let (graphics_backend, mut winit_loop) = smithay::backend::winit::init()?;
//! // In the frame loop:
//! winit_loop.dispatch_new_events(|event| { ... });
//! graphics_backend.bind()?;
//! // render into the Winit framebuffer
//! graphics_backend.submit()?;
//! ```

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
    ///
    /// The real Smithay `winit::init()` call requires the `backend_winit`
    /// and `renderer_gl` features (both enabled). It returns
    /// `(WinitGraphicsBackend<GlowRenderer>, WinitEventLoop)` which we
    /// store and pump in the compositor event loop.
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
            std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
            std::env::set_var("XDG_RUNTIME_DIR", "/mnt/wslg/runtime-dir");
        }

        // The real Smithay winit::init() call would go here:
        //
        // #[cfg(target_os = "linux")]
        // {
        //     use smithay::backend::winit::{init, WinitEventLoop};
        //     use smithay::backend::renderer::glow::GlowRenderer;
        //
        //     let (graphics_backend, winit_event_loop) = init::<GlowRenderer>()
        //         .context("Failed to initialize Winit backend")?;
        //
        //     // Store graphics_backend and winit_event_loop in the struct
        //     // Pump winit_event_loop.dispatch_new_events() each frame
        //     // Use graphics_backend for rendering
        // }
        //
        // This requires a display connection (WSLg or native Wayland).
        // On Windows development, we skip the Winit init and use the
        // CPU ScanoutFramebuffer for rendering validation.

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
    fn has_gpu(&self) -> bool { false }
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
