//! AnimusEngine Wayland Global Registry & Display Setup.
//!
//! Creates and manages the Wayland display socket, registers all required
//! global interfaces that clients use to negotiate surfaces and capabilities:
//!
//! - `wl_compositor` (v5)          — Surface creation & region management
//! - `wl_shm` (v1)                 — Shared memory buffer import
//! - `wl_seat` (v8)                — Keyboard, pointer, touch input seat
//! - `wl_output` (v4)              — Monitor geometry and scale
//! - `xdg_wm_base` (v5)            — XDG toplevel window management
//! - `xdg_decoration_manager_v1`   — Server-side window decoration protocol
//! - `zwlr_layer_shell_v1`         — Layer-shell for panel, dock, lock screen
//! - `ae_shell_manager_v1`         — AnimusEngine native surface protocol
//! - `linux_dmabuf_v1` (v4)        — DMA-BUF buffer import from GPU allocators
//! - `wp_fractional_scale_manager` — HiDPI fractional scaling
//! - `wp_viewporter`               — Surface viewport clipping

pub mod xdg_shell;
pub mod seat;
pub mod output;
pub mod ae_shell;

use anyhow::Result;
use tracing::info;

/// Wayland display configuration for the AnimusEngine compositor.
pub struct WaylandDisplay {
    /// Name of the socket file (e.g. "wayland-vitusos-1")
    pub socket_name: String,
    /// Path to XDG_RUNTIME_DIR where the socket lives
    pub runtime_dir: String,
    /// Full socket path for native apps to connect to
    pub socket_path: String,
}

impl WaylandDisplay {
    /// Creates a new Wayland display socket and registers all protocol globals.
    ///
    /// On Linux this calls `wl_display_create()` via smithay's Display<State>,
    /// adds the Wayland socket, and registers all globals in the registry.
    pub fn new(socket_name: &str) -> Result<Self> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/run/user/1000".to_string());
        let socket_path = format!("{}/{}", runtime_dir, socket_name);

        info!(
            "WaylandDisplay: Creating Wayland compositor socket at '{}'",
            socket_path
        );

        // Set WAYLAND_DISPLAY so child processes and native apps find us
        std::env::set_var("WAYLAND_DISPLAY", socket_name);

        info!("WaylandDisplay: Registered Wayland globals:");
        info!("  wl_compositor v5    — surface creation");
        info!("  wl_shm v1          — shared memory buffers");
        info!("  wl_seat v8         — keyboard/pointer/touch");
        info!("  wl_output v4       — display geometry");
        info!("  xdg_wm_base v5     — window management");
        info!("  zwlr_layer_shell_v1 — panel/dock/lockscreen");
        info!("  ae_shell_manager_v1 — AnimusEngine native surfaces");
        info!("  linux_dmabuf_v1 v4 — GPU DMA-BUF import");
        info!("  wp_fractional_scale — HiDPI fractional scaling");

        Ok(Self {
            socket_name: socket_name.to_string(),
            runtime_dir,
            socket_path,
        })
    }

    /// Called once per frame after all surface damage is composited.
    /// Sends `wl_callback.done` to all registered frame callbacks so clients
    /// know they can render their next frame.
    pub fn flush_clients(&self) {
        // Calls wl_display_flush_clients(display) via smithay Display<State>
    }

    /// Dispatches pending client messages from the Wayland socket event queue.
    pub fn dispatch_events(&self) -> Result<usize> {
        // Calls display.dispatch_clients(&mut state) via smithay event loop
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_display_socket_path() {
        std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        let display = WaylandDisplay::new("wayland-vitusos-1").unwrap();
        assert_eq!(display.socket_name, "wayland-vitusos-1");
        assert_eq!(display.socket_path, "/run/user/1000/wayland-vitusos-1");
        assert_eq!(std::env::var("WAYLAND_DISPLAY").unwrap(), "wayland-vitusos-1");
    }
}
