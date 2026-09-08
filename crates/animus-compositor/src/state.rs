//! AnimusEngine Compositor State.
//!
//! Ties together the Wayland protocols, the input seat, the active backend,
//! the rendering pipeline, and the shell data (Dock, Panel, etc.).
//!
//! On Linux: a real `wayland_server::Display<SmithayState>` is created and
//! a `ListeningSocket` is bound so Wayland clients can connect.
//! On development hosts: the state machine runs without a real display.

use crate::backend::AnimusBackend;
use crate::wayland::{
    ae_shell::AeShellManager,
    output::OutputManager,
    seat::AnimusSeat,
    xdg_shell::XdgShellState,
};
use animus_render::{RenderPipeline, AnimusVulkanRenderer};
use crate::sound_manager::SoundManager;
use tracing::info;

#[cfg(target_os = "linux")]
use crate::smithay::state::SmithayState;

/// The central state object for the entire compositor.
pub struct CompositorState {
    pub backend: Box<dyn AnimusBackend>,
    pub renderer: AnimusVulkanRenderer,
    pub render_pipeline: RenderPipeline,

    pub xdg_shell: XdgShellState,
    pub ae_shell: AeShellManager,
    pub seat: AnimusSeat,
    pub outputs: OutputManager,

    /// Sound manager with per-sound volume levels (Part 36).
    pub sounds: SoundManager,

    /// Real Wayland display (Linux only).
    /// This is the `wl_display` that clients connect to via the Wayland socket.
    #[cfg(target_os = "linux")]
    pub display: Option<wayland_server::Display<SmithayState>>,

    /// Wayland listening socket for accepting new client connections.
    #[cfg(target_os = "linux")]
    pub socket: Option<wayland_server::ListeningSocket>,

    pub is_running: bool,
    pub start_time: std::time::Instant,
}

impl CompositorState {
    /// Creates a new compositor state, initializing the GPU pipeline.
    pub fn new(backend: Box<dyn AnimusBackend>) -> Self {
        let (width, height, _hz) = backend.output_geometry();

        let mut renderer = AnimusVulkanRenderer::new(width, height);
        if let Err(e) = renderer.initialize() {
            tracing::error!("Failed to initialize Vulkan renderer: {}", e);
        }

        let render_pipeline = RenderPipeline::new(width, height);

        // On Linux, create the real Wayland display and bind a socket
        #[cfg(target_os = "linux")]
        let (display, socket) = Self::init_wayland_display();

        Self {
            backend,
            renderer,
            render_pipeline,
            xdg_shell: XdgShellState::new(),
            ae_shell: AeShellManager::new(),
            seat: AnimusSeat::new("seat0"),
            outputs: OutputManager::new(),
            sounds: SoundManager::new(),
            #[cfg(target_os = "linux")]
            display,
            #[cfg(target_os = "linux")]
            socket,
            is_running: true,
            start_time: std::time::Instant::now(),
        }
    }

    /// Creates a real `wayland_server::Display` and binds a listening socket.
    ///
    /// The socket name is `wayland-vitusos-0` (or the next available index).
    /// `WAYLAND_DISPLAY` is set so child processes and native apps find us.
    #[cfg(target_os = "linux")]
    fn init_wayland_display() -> (
        Option<wayland_server::Display<SmithayState>>,
        Option<wayland_server::ListeningSocket>,
    ) {
        use wayland_server::ListeningSocket;
        use crate::smithay::state::ClientState;

        let mut display = match wayland_server::Display::new() {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("AnimusEngine: Failed to create Wayland display: {}", e);
                return (None, None);
            }
        };

        // Initialize Smithay protocol globals on this display
        let _smithay_state = SmithayState::new(&mut display);

        // Bind the Wayland socket
        let socket = ListeningSocket::bind_auto("wayland-vitusos", 0..10)
            .ok();

        if let Some(ref sock) = socket {
            let socket_name = sock.socket_name().map(|n| n.to_string_lossy().to_string());
            if let Some(ref name) = socket_name {
                info!("AnimusEngine: Wayland display socket bound at {}", name);
                std::env::set_var("WAYLAND_DISPLAY", name);
            }
        } else {
            tracing::warn!("AnimusEngine: Failed to bind Wayland socket, running without client support");
        }

        (Some(display), socket)
    }

    /// Dispatches pending Wayland client messages (without accepting new connections).
    /// Called when the socket is owned by the calloop event source.
    #[cfg(target_os = "linux")]
    pub fn dispatch_wayland_clients_only(&mut self) {
        if let Some(ref mut display) = self.display {
            let mut state = match wayland_server::Display::new() {
                Ok(mut d) => SmithayState::new(&mut d),
                Err(_) => return,
            };
            match display.dispatch_clients(&mut state) {
                Ok(n) => {
                    if n > 0 {
                        tracing::trace!("AnimusEngine: Dispatched {} Wayland client events", n);
                    }
                }
                Err(e) => {
                    tracing::warn!("AnimusEngine: Wayland dispatch error: {}", e);
                }
            }
            let _ = display.flush_clients();
        }
    }

    /// Dispatches pending Wayland client messages.
    /// Called from the calloop event loop each frame.
    #[cfg(target_os = "linux")]
    pub fn dispatch_wayland(&mut self) {
        use std::sync::Arc;
        use crate::smithay::state::ClientState;

        // Accept new client connections
        if let Some(ref socket) = self.socket {
            while let Ok(Some(stream)) = socket.accept() {
                if let Some(ref mut display) = self.display {
                    let client_data = Arc::new(ClientState {
                        compositor_state: smithay::wayland::compositor::CompositorClientState::default(),
                    });
                    match display.handle().insert_client(stream, client_data) {
                        Ok(_client) => info!("AnimusEngine: Accepted new Wayland client connection"),
                        Err(e) => tracing::warn!("AnimusEngine: Failed to accept Wayland client: {}", e),
                    }
                }
            }
        }

        // Dispatch pending client requests
        if let Some(ref mut display) = self.display {
            // TODO: Store the SmithayState persistently and dispatch against it.
            // For now we create a fresh state each dispatch — this is not ideal
            // but allows client messages to be processed without panicking.
            // The full implementation will own the SmithayState as a field.
            let mut state = match wayland_server::Display::new() {
                Ok(mut d) => SmithayState::new(&mut d),
                Err(_) => return,
            };
            match display.dispatch_clients(&mut state) {
                Ok(n) => {
                    if n > 0 {
                        tracing::trace!("AnimusEngine: Dispatched {} Wayland client events", n);
                    }
                }
                Err(e) => {
                    tracing::warn!("AnimusEngine: Wayland dispatch error: {}", e);
                }
            }
            let _ = display.flush_clients();
        }
    }

    /// The main compositing function called once per frame.
    /// Updates physics, handles Wayland commits, renders the frame, and triggers page flip.
    pub fn composite_frame(&mut self) -> anyhow::Result<()> {
        let (_w, _h, _hz) = self.backend.output_geometry();

        // 1. Update spring physics (Dock hover, window animations)
        let _dt = 1.0 / 144.0;

        // 2. Sync Wayland windows to RenderPipeline
        let mut windows = Vec::new();
        for surface in &self.xdg_shell.surfaces {
            if surface.is_renderable() {
                let altitude = self.ae_shell.surfaces
                    .get(&surface.surface_id)
                    .map(|s| s.altitude)
                    .unwrap_or(animus_render::SurfaceAltitude::Mid);

                windows.push(animus_render::RenderWindow {
                    id: surface.surface_id as u64,
                    title: format!("Window {}", surface.surface_id),
                    x: surface.x as f32,
                    y: surface.y as f32,
                    width: surface.width as f32,
                    height: surface.height as f32,
                    shadow_x: 0.0,
                    shadow_y: 12.0,
                    corner_radius: 12.0,
                    altitude,
                    is_visible: true,
                    is_focused: surface.is_focused,
                    client_buffer: None,
                });
            }
        }

        // Render CPU software pass
        self.render_pipeline.render_frame(&windows, 0, false, false, "vitusOS");

        // 3. Render via GPU pipeline
        self.renderer.render_frame()?;

        // 4. Schedule next frame (DRM page flip)
        self.backend.schedule_frame();

        Ok(())
    }
}
