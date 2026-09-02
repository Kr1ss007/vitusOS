//! XDG Shell Handler — Window Management for AnimusEngine Compositor.
//!
//! Handles the `xdg_wm_base` and `xdg_toplevel` Wayland protocols:
//!
//! - `xdg_surface.get_toplevel()` — Creates a managed application window
//! - `xdg_toplevel.set_title()`   — Updates the Panel's focused app name
//! - `xdg_toplevel.set_app_id()` — Maps to Dock icon and ae_shell surface
//! - `xdg_toplevel.set_maximized()` / `set_fullscreen()` — Window state
//! - `xdg_surface.ack_configure()` — Client acknowledges resize event
//! - `wl_surface.commit()`         — Applies pending surface damage
//!
//! Window lifecycle:
//! `xdg_surface created` → `configure sent (size + state)` → `ack_configure`
//! → `wl_surface.commit` with a `wl_buffer` attached → added to scene graph
//! → `RenderPipeline` composites on next frame.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Represents a managed XDG toplevel window in the compositor's scene graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdgToplevelSurface {
    /// Unique Wayland object ID for this surface
    pub surface_id: u32,
    /// Application title (shown in Panel global menu bar)
    pub title: String,
    /// XDG app-id (maps to .desktop file, Dock icon, ae_shell registration)
    pub app_id: String,
    /// Current window geometry in compositor coordinates
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Whether the client has acknowledged the latest configure event
    pub configure_acked: bool,
    /// Whether the client has committed a valid wl_buffer
    pub has_committed_buffer: bool,
    /// Whether this surface is currently focused (receives keyboard input)
    pub is_focused: bool,
    /// Window state flags
    pub is_maximized: bool,
    pub is_fullscreen: bool,
    pub is_minimized: bool,
}

impl XdgToplevelSurface {
    pub fn new(surface_id: u32, app_id: impl Into<String>) -> Self {
        let app_id = app_id.into();
        info!(
            "XdgShell: New xdg_toplevel created — surface_id={} app_id='{}'",
            surface_id, app_id
        );
        Self {
            surface_id,
            title: app_id.clone(),
            app_id,
            x: 64.0,
            y: 64.0,
            width: 800.0,
            height: 600.0,
            configure_acked: false,
            has_committed_buffer: false,
            is_focused: false,
            is_maximized: false,
            is_fullscreen: false,
            is_minimized: false,
        }
    }

    /// Sends a `xdg_surface.configure` event to the client with current geometry.
    /// The client must respond with `xdg_surface.ack_configure(serial)`.
    pub fn send_configure(&mut self, serial: u32, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.configure_acked = false;
        info!(
            "XdgShell: Sent configure serial={} size={}x{} to surface_id={}",
            serial, width, height, self.surface_id
        );
    }

    /// Called when client sends `xdg_surface.ack_configure(serial)`.
    pub fn ack_configure(&mut self, serial: u32) {
        self.configure_acked = true;
        info!(
            "XdgShell: surface_id={} acked configure serial={}",
            self.surface_id, serial
        );
    }

    /// Called when client sends `wl_surface.commit` with a new `wl_buffer`.
    /// After this, the surface is visible in the next composited frame.
    pub fn commit_buffer(&mut self) {
        self.has_committed_buffer = true;
    }

    /// Returns true if this surface is ready to be composited this frame.
    pub fn is_renderable(&self) -> bool {
        self.configure_acked && self.has_committed_buffer && !self.is_minimized
    }

    /// Positions the window with the standard vitusOS window placement algorithm:
    /// new windows cascade from top-left with 32px offset per window, staying
    /// within the panel (36px) and dock (80px) safe areas.
    pub fn place_default(&mut self, screen_w: f32, screen_h: f32, window_index: usize) {
        let cascade_offset = 32.0 * window_index as f32;
        self.x = (64.0 + cascade_offset).min(screen_w - self.width - 32.0);
        self.y = (36.0 + cascade_offset + 32.0).min(screen_h - self.height - 96.0);
    }
}

/// Tracks all active XDG toplevel windows and their focus order.
pub struct XdgShellState {
    pub surfaces: Vec<XdgToplevelSurface>,
    pub focus_stack: Vec<u32>,  // surface_id order, last = frontmost
    pub configure_serial: u32,
}

impl XdgShellState {
    pub fn new() -> Self {
        Self {
            surfaces: Vec::new(),
            focus_stack: Vec::new(),
            configure_serial: 1,
        }
    }

    /// Creates a new xdg_toplevel, assigns initial geometry, and sends configure.
    pub fn create_toplevel(&mut self, app_id: impl Into<String>, screen_w: f32, screen_h: f32) -> u32 {
        let surface_id = self.configure_serial;
        let mut surface = XdgToplevelSurface::new(surface_id, app_id);
        surface.place_default(screen_w, screen_h, self.surfaces.len());

        let serial = self.next_serial();
        surface.send_configure(serial, surface.width, surface.height);

        self.surfaces.push(surface);
        self.focus(surface_id);
        surface_id
    }

    /// Focuses a surface — raises it to the top of the focus stack and
    /// sends keyboard focus enter to its seat.
    pub fn focus(&mut self, surface_id: u32) {
        self.focus_stack.retain(|id| *id != surface_id);
        self.focus_stack.push(surface_id);

        for s in &mut self.surfaces {
            s.is_focused = s.surface_id == surface_id;
        }
        info!("XdgShell: Focus → surface_id={}", surface_id);
    }

    /// Destroys an xdg_toplevel and removes it from the scene graph.
    pub fn destroy_toplevel(&mut self, surface_id: u32) {
        self.surfaces.retain(|s| s.surface_id != surface_id);
        self.focus_stack.retain(|id| *id != surface_id);
        info!("XdgShell: Destroyed surface_id={}", surface_id);
    }

    /// Returns the currently focused surface.
    pub fn focused(&self) -> Option<&XdgToplevelSurface> {
        let focused_id = self.focus_stack.last()?;
        self.surfaces.iter().find(|s| s.surface_id == *focused_id)
    }

    fn next_serial(&mut self) -> u32 {
        let s = self.configure_serial;
        self.configure_serial += 1;
        s
    }
}

impl Default for XdgShellState {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdg_shell_window_lifecycle() {
        let mut state = XdgShellState::new();

        let filer_id = state.create_toplevel("filer", 1920.0, 1080.0);
        assert_eq!(state.surfaces.len(), 1);
        assert!(!state.surfaces[0].configure_acked);

        // Client acks configure
        state.surfaces[0].ack_configure(1);
        assert!(state.surfaces[0].configure_acked);

        // Client commits buffer
        state.surfaces[0].commit_buffer();
        assert!(state.surfaces[0].is_renderable());

        // Open second window
        let term_id = state.create_toplevel("terminow", 1920.0, 1080.0);
        assert_eq!(state.surfaces.len(), 2);
        assert!(state.focused().is_some());
        assert_eq!(state.focused().unwrap().app_id, "terminow");

        // Focus first window
        state.focus(filer_id);
        assert_eq!(state.focused().unwrap().app_id, "filer");

        // Destroy window
        state.destroy_toplevel(filer_id);
        assert_eq!(state.surfaces.len(), 1);
        assert_eq!(state.surfaces[0].app_id, "terminow");
    }

    #[test]
    fn test_window_placement_cascade() {
        let mut state = XdgShellState::new();
        let id0 = state.create_toplevel("app0", 1920.0, 1080.0);
        let id1 = state.create_toplevel("app1", 1920.0, 1080.0);

        let s0 = state.surfaces.iter().find(|s| s.surface_id == id0).unwrap();
        let s1 = state.surfaces.iter().find(|s| s.surface_id == id1).unwrap();
        // Second window should cascade to a different position
        assert!(s1.x > s0.x || s1.y > s0.y);
    }
}
