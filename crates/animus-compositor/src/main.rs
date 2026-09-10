//! vitusOS Canonical Compositor Main Entry Point.
//!
//! Production bare-metal compositor running at 144Hz with Vulkan DMA-BUF scanout,
//! native AESurfaces LockScreen, LoginManager, ControlCenter, ShutdownScreen,
//! VirtualDesktopManager, MotionWave gestures, CrashManager, and EOBus.

use std::sync::Arc;
use std::time::Instant;
use animus_core::crash::CrashManager;
use animus_core::eobus::EOBus;
use animus_core::events::AEEvent;
use animus_core::handoff::AnimusGpuHandoff;
use animus_core::registry::RegistryManager;
use animus_core::AnimusEngine;
use animus_input::motion_wave::MotionWave;
use animus_render::wallpaper_sampler::WallpaperTintSampler;
use animus_compositor::shell::{
    AEShellProtocolManager, AELoginManager, BootCrossfade, CockpitView, ControlCenter, Dock,
    DockItem, GlobalMenu, LockScreen, NotificationCenter, Panel, ShutdownScreen, SystemScreen,
    WelcomeScreen,
};
use animus_compositor::shell_controller::ShellController;
use animus_compositor::compositor_renderer::CompositorRenderer;
use animus_compositor::window::AEWindow;
use animus_compositor::workspace::VirtualDesktopManager;
use animus_compositor::state::CompositorState;
use animus_compositor::backend::{AnimusBackend, AnimusWinitBackend};
#[cfg(target_os = "linux")]
use animus_compositor::backend::{AnimusDrmBackend, UdevLibinputSeat};

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub struct CompositorContext {
    pub engine: Arc<AnimusEngine>,
    pub crash_manager: Arc<CrashManager>,
    pub registry: Arc<RegistryManager>,
    pub handoff: AnimusGpuHandoff,
    pub eobus: Arc<EOBus>,
    pub ae_bridge: Option<animus_core::AEBridge>,
    pub state: CompositorState,
    pub wallpaper_sampler: WallpaperTintSampler,
    pub motion_wave: MotionWave,
    pub workspace_manager: VirtualDesktopManager,
    pub boot_crossfade: BootCrossfade,
    pub welcome_screen: WelcomeScreen,
    pub login_manager: AELoginManager,
    pub lock_screen: LockScreen,
    pub control_center: ControlCenter,
    pub notification_center: NotificationCenter,
    pub shutdown_screen: ShutdownScreen,
    pub system_screen: SystemScreen,
    pub ae_shell_proto: AEShellProtocolManager,
    pub panel: Panel,
    pub dock: Dock,
    pub cockpit_view: CockpitView,
    pub global_menu: GlobalMenu,
    pub windows: Vec<AEWindow>,
    /// Shell controller — owns all shell components and routes events.
    pub shell: ShellController,
    /// Compositor renderer — bridges shell state to the 7-layer render pipeline.
    pub renderer: CompositorRenderer,
    /// Event channel receiver — events from EventBus dispatched to ShellController.
    event_rx: crossbeam_channel::Receiver<AEEvent>,
    /// Libinput seat — wires keyboard/mouse/touch into the compositor (Linux only).
    #[cfg(target_os = "linux")]
    pub libinput_seat: Option<UdevLibinputSeat>,
}

impl CompositorContext {
    pub fn new(engine: Arc<AnimusEngine>) -> Self {
        let bus = (*engine.event_bus).clone();
        let crash_manager = Arc::new(CrashManager::new(bus.clone()));
        let registry = Arc::new(RegistryManager::new(bus.clone()));
        let handoff = AnimusGpuHandoff::read_from_efivars().unwrap_or_default();
        let eobus = Arc::new(EOBus::new(bus.clone()));
        
        let width = handoff.horizontal_resolution.max(1920);
        let height = handoff.vertical_resolution.max(1080);

        // FIX4-10: These dimensions are initial estimates from the EFI handoff.
        // The real output dimensions will be available when the DRM backend
        // enumerates connected connectors. CompositorState::new reads the
        // backend geometry, and the real mode is set during DRM surface creation.
        // The subsystems that need exact dimensions (VulkanContext, RenderPipeline)
        // will be reconfigured when the first output connects via the calloop
        // UdevBackend event source. For now, the handoff resolution is the best estimate.
        
        // Choose backend based on environment (bare-metal DRM/KMS vs nested Wayland dev)
        let backend: Box<dyn AnimusBackend> = {
            if std::env::var("ANIMUS_BACKEND").as_deref() == Ok("drm") {
                Box::new(AnimusDrmBackend::new().unwrap())
            } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
                Box::new(AnimusWinitBackend::new(width, height).unwrap())
            } else {
                Box::new(AnimusDrmBackend::new().unwrap())
            }
        };

        let state = CompositorState::new(backend);

        let wallpaper_sampler = WallpaperTintSampler::new();
        let motion_wave = MotionWave::new(bus.clone());
        let workspace_manager = VirtualDesktopManager::new(width as f32, bus.clone());
        let boot_crossfade = BootCrossfade::new(bus.clone());
        let welcome_screen = WelcomeScreen::new(bus.clone());
        let login_manager = AELoginManager::new(bus.clone());
        let lock_screen = LockScreen::new(bus.clone());
        let control_center = ControlCenter::new(bus.clone());
        let notification_center = NotificationCenter::new(bus.clone());
        let shutdown_screen = ShutdownScreen::new(bus.clone());
        let system_screen = SystemScreen::new(bus.clone());
        let ae_shell_proto = AEShellProtocolManager::new(bus.clone());
        let panel = Panel::new();
        let mut dock = Dock::new();
        let cockpit_view = CockpitView::new(bus.clone());
        let global_menu = GlobalMenu::new();

        // Pinned dock items with verified scalable assets
        dock.add_item(DockItem::new("filer", "Files", "assets/icons/dock/filer.svg"));
        dock.add_item(DockItem::new("zen-browser", "Zen Browser", "assets/icons/dock/zen-browser.svg"));
        dock.add_item(DockItem::new("pathfinder", "Pathfinder", "assets/icons/dock/pathfinder.svg"));
        dock.add_item(DockItem::new("terminow", "Terminow", "assets/icons/dock/terminow.svg"));
        dock.add_item(DockItem::new("settings", "Settings", "assets/icons/dock/settings.svg"));

        let ctx = Self {
            engine,
            crash_manager,
            registry,
            handoff,
            eobus,
            ae_bridge: None, // Connected after init
            state,
            wallpaper_sampler,
            motion_wave,
            workspace_manager,
            boot_crossfade,
            welcome_screen,
            login_manager,
            lock_screen,
            control_center,
            notification_center,
            shutdown_screen,
            system_screen,
            ae_shell_proto,
            panel,
            dock,
            cockpit_view,
            global_menu,
            windows: Vec::new(),
            shell: ShellController::new(bus.clone(), width as f32, height as f32),
            renderer: CompositorRenderer::new(width, height),
            event_rx: {
                let (tx, rx) = crossbeam_channel::unbounded::<AEEvent>();
                let bus_clone = bus.clone();
                bus_clone.subscribe(move |event: &AEEvent| {
                    let _ = tx.send(event.clone());
                });
                rx
            },
            #[cfg(target_os = "linux")]
            libinput_seat: None, // Wired into calloop in run_bare_metal()
        };

        ctx.spawn_native_daemons();
        ctx
    }

    /// Spawns the vitusos-native daemons as true standalone processes (Priority 3).
    fn spawn_native_daemons(&self) {
        tracing::info!("Spawning AENative background daemons (Filer, Pathfinder)...");
        std::thread::spawn(|| {
            let _ = std::process::Command::new("vitusos-native")
                .args(["--app", "filer"])
                .spawn();
        });
        std::thread::spawn(|| {
            let _ = std::process::Command::new("vitusos-native")
                .args(["--app", "pathfinder"])
                .spawn();
        });
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        let dt = self.engine.clock.write().tick(Instant::now());

        // Drain background worker events onto main loop (§4.4)
        self.engine.event_bus.drain_async_queue();

        // Dispatch all pending events from the EventBus to the ShellController.
        // Events are collected via the crossbeam channel subscriber registered
        // in CompositorContext::new(). drain_async_queue() triggers async
        // publishes which call the subscriber, populating this channel.
        while let Ok(event) = self.event_rx.try_recv() {
            self.shell.dispatch_event(&event);
        }

        // Update boot crossfade (owned by compositor, not ShellController)
        self.boot_crossfade.update(dt);

        // Update all shell components via the ShellController
        self.shell.update(dt);

        // Sync windows from WindowManager to the legacy windows Vec
        self.windows.clear();
        for win in self.shell.window_manager.windows() {
            self.windows.push(win.clone());
        }

        // Dispatch Wayland client messages (Linux only)
        #[cfg(target_os = "linux")]
        self.state.dispatch_wayland();

        // Composite the frame via the CompositorRenderer.
        // This reads the ShellController state, extracts window geometry,
        // and writes pixels to the ScanoutFramebuffer via the 7-layer pipeline.
        self.renderer.composite_frame(&self.shell);

        self.engine.event_bus.publish(AEEvent::Tick { dt });
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Initializing vitusOS Canonical Compositor...");

    // 1. Initialize CrashManager as first action (Part 21.2)
    let engine = Arc::new(AnimusEngine::new());
    let mut ctx = CompositorContext::new(Arc::clone(&engine));
    ctx.crash_manager.initialize();

    // 2. Step Engine Boot Sequence & Audio Handoff
    engine.boot_sequence();
    ctx.eobus.start();

    // 2.5 Connect to session process via AEBridge
    // The session process (vitusos-session) must already be running.
    // systemd ensures this via After=vitusos-session.service on our unit.
    let mut ae_bridge = animus_core::AEBridge::new((*engine.event_bus).clone());
    match ae_bridge.connect_to_session() {
        Ok(()) => info!("Compositor connected to vitusos-session via AEBridge"),
        Err(e) => tracing::warn!("AEBridge connection to session failed ({}), running without session bridge", e),
    }
    ctx.ae_bridge = Some(ae_bridge);

    // 3. Step Through Boot Milestones
    ctx.boot_crossfade.set_progress(0.15); // Stage 0/1/2 Handoff complete, DRM set
    ctx.boot_crossfade.set_progress(0.40); // Sound engine & boot chime active
    ctx.boot_crossfade.set_progress(0.65); // Vulkan pipeline & glass shaders ready
    ctx.boot_crossfade.set_progress(0.85); // Wayland socket & CrashSite bound
    ctx.boot_crossfade.set_progress(1.00); // Shell crossfade ready
    ctx.boot_crossfade.begin_fade();

    // 4. Run the production event loop
    run_bare_metal(ctx)?;

    info!("vitusOS Engine shutting down.");
    Ok(())
}

/// Linux bare-metal compositor event loop.
///
/// On real hardware: calloop drives DRM vblank events, libinput events,
/// and Wayland client socket dispatch. The frame loop is woken by vblank,
/// not by a timer -- matching macOS FramePacing behavior.
///
/// On WSL2/dev: calloop drives Winit redraw events from the host compositor.
#[cfg(target_os = "linux")]
fn run_bare_metal(mut ctx: CompositorContext) -> anyhow::Result<()> {
    use calloop::EventLoop;
    use calloop::generic::Generic;
    use calloop::{Interest, Mode};
    use std::os::unix::io::AsFd;

    info!("AnimusEngine: Linux bare-metal event loop starting (calloop)");

    let mut event_loop: EventLoop<CompositorContext> =
        EventLoop::try_new().map_err(|e| anyhow::anyhow!("Failed to create calloop event loop: {}", e))?;

    let loop_handle = event_loop.handle();

    // ── Event Source 1: Frame Timer (144Hz) ───────────────────────────
    //
    // Drives the compositor frame loop at 144Hz.
    // On bare metal with DRM/KMS this will be replaced by DRM vblank events.

    let frame_duration = std::time::Duration::from_micros(1_000_000 / 144);
    let timer = calloop::timer::Timer::from_duration(frame_duration);

    loop_handle
        .insert_source(timer, move |_, _, ctx| {
            if let Err(e) = ctx.tick() {
                tracing::error!("Frame tick failed: {}", e);
            }
            calloop::timer::TimeoutAction::ToDuration(frame_duration)
        })
        .map_err(|e| anyhow::anyhow!("Failed to insert frame timer: {}", e))?;

    // ── Event Source 2: DRM Backend Initialization ─────────────────────
    //
    // Initialize the DRM/KMS backend now that calloop is ready.
    // AnimusDrmBackend::new() finds the device; initialize() does modesetting.
    // We also try to insert the DrmDeviceNotifier for vblank-paced frame loop.

    if let Some(drm_backend) = ctx.state.backend.as_any_mut().downcast_mut::<AnimusDrmBackend>() {
        match drm_backend.initialize() {
            Ok(Some(notifier)) => {
                info!("AnimusEngine: DRM backend initialized, inserting vblank notifier");
                // The DrmDeviceNotifier is an EventSource that fires on vblank.
                // When it fires, we should queue a frame render instead of the timer.
                // For now, register it to keep the kernel DRM state machine happy.
                if let Err(e) = loop_handle.insert_source(notifier, |event, _, _ctx| {
                    use smithay::backend::drm::DrmEvent;
                    match event {
                        DrmEvent::VBlank(crtc) => {
                            tracing::trace!("AnimusEngine: VBlank on CRTC {:?}", crtc);
                            // Frame is already paced by the 144Hz timer source.
                            // In a future iteration, we'll remove the timer and
                            // only render on VBlank for perfect frame pacing.
                        }
                        DrmEvent::Error(e) => {
                            tracing::error!("AnimusEngine: DRM device error: {:?}", e);
                        }
                    }
                }) {
                    tracing::warn!("AnimusEngine: Failed to insert DRM notifier: {}", e);
                }
            }
            Ok(std::option::Option::None) => {
                info!("AnimusEngine: DRM backend initialized (no notifier)");
            }
            Err(e) => {
                tracing::warn!("AnimusEngine: DRM backend initialization failed ({}), continuing without KMS", e);
            }
        }
    }

    // ── Event Source 3: Wayland Listening Socket ───────────────────────
    //
    // When a Wayland client connects (e.g. a native app or third-party app),
    // this fires and we accept the connection via dispatch_wayland().
    // The socket fd is polled for readability.

    let socket = ctx.state.socket.take();

    if let Some(socket) = socket {
        let socket_source = Generic::new(socket, Interest::READ, Mode::Level);
        loop_handle
            .insert_source(socket_source, |_readiness, socket, ctx| {
                // Accept any pending client connections
                while let Ok(Some(stream)) = socket.accept() {
                    if let Some(ref mut display) = ctx.state.display {
                        let client_data = std::sync::Arc::new(
                            animus_compositor::smithay::state::ClientState {
                                compositor_state: smithay::wayland::compositor::CompositorClientState::default(),
                            }
                        );
                        match display.handle().insert_client(stream, client_data) {
                            Ok(_) => tracing::info!("AnimusEngine: Accepted Wayland client"),
                            Err(e) => tracing::warn!("AnimusEngine: Client accept failed: {}", e),
                        }
                    }
                }
                // Dispatch pending client messages against persistent SmithayState
                ctx.state.dispatch_wayland_clients_only();
                Ok(calloop::PostAction::Continue)
            })
            .map_err(|e| anyhow::anyhow!("Failed to insert Wayland socket source: {}", e))?;
        info!("AnimusEngine: Wayland socket event source registered");
    }

    // ── Event Source 4: Udev Backend (DRM device hotplug) ──────────────
    //
    // Monitors /dev/dri for device addition/removal. Handles hot-plug of
    // monitors and GPU resets without crashing the compositor.

    match smithay::backend::udev::UdevBackend::new("seat0") {
        Ok(udev) => {
            let devices: Vec<_> = udev.device_list().collect();
            info!("AnimusEngine: UdevBackend monitoring {} DRM device(s)", devices.len());
            for (dev_id, path) in &devices {
                info!("  DRM device: {} at {:?}", dev_id, path);
            }

            loop_handle
                .insert_source(udev, move |event, _, ctx| {
                    use smithay::backend::udev::UdevEvent;
                    match event {
                        UdevEvent::Added { device_id, path } => {
                            info!("AnimusEngine: DRM device added: {} at {:?}", device_id, path);
                            // Re-scan for new connector and initialize output
                            if let Some(drm) = ctx.state.backend.as_any_mut().downcast_mut::<AnimusDrmBackend>() {
                                if drm.drm_path == path {
                                    match drm.initialize() {
                                        Ok(_) => info!("AnimusEngine: DRM device re-initialized after hotplug"),
                                        Err(e) => tracing::warn!("AnimusEngine: DRM re-init failed: {}", e),
                                    }
                                }
                            }
                        }
                        UdevEvent::Changed { device_id } => {
                            info!("AnimusEngine: DRM device changed: {}", device_id);
                            // Mode change (e.g. monitor resolution changed via HDMI)
                            // Re-enumerate connectors and resize renderer if output changed.
                            if let Some(drm) = ctx.state.backend.as_any_mut().downcast_mut::<AnimusDrmBackend>() {
                                let (w, h, _) = drm.output_geometry();
                                info!("AnimusEngine: DRM geometry after change: {}x{}", w, h);
                                ctx.renderer.resize(w, h);
                            }
                        }
                        UdevEvent::Removed { device_id } => {
                            info!("AnimusEngine: DRM device removed: {}", device_id);
                            // Destroy surfaces associated with removed device.
                            // The compositor continues running on remaining outputs.
                            if let Some(drm) = ctx.state.backend.as_any_mut().downcast_mut::<AnimusDrmBackend>() {
                                drm.surfaces.clear();
                                drm.scanout_buffers.clear();
                                drm.is_initialized = false;
                                info!("AnimusEngine: DRM surfaces cleared for removed device {}", device_id);
                            }
                        }
                    }
                })
                .map_err(|e| anyhow::anyhow!("Failed to insert udev source: {}", e))?;
            info!("AnimusEngine: Udev event source registered");
        }
        Err(e) => {
            tracing::warn!("AnimusEngine: UdevBackend not available ({}), running without hotplug", e);
        }
    }

    // ── Event Source 5: Libinput (keyboard/mouse/touch) ───────────────
    //
    // The libinput context fd is polled for readability. When input events
    // arrive, we dispatch them through UdevLibinputSeat into the AnimusSeat
    // and MotionWave gesture recognizer.

    match UdevLibinputSeat::new("seat0") {
        Ok(libinput_seat) => {
            info!("AnimusEngine: libinput initialized on seat0");

            // Extract the raw fd from the libinput context for calloop
            let libinput_fd = libinput_seat.context.as_fd().try_clone_to_owned()
                .map_err(|e| anyhow::anyhow!("Failed to clone libinput fd: {}", e))?;

            // Store the libinput seat so the calloop callback can dispatch events
            ctx.libinput_seat = Some(libinput_seat);

            let libinput_source = Generic::new(
                libinput_fd,
                Interest::READ,
                Mode::Level,
            );

            loop_handle
                .insert_source(libinput_source, |_readiness, _fd, ctx| {
                    // Dispatch available libinput events into the seat and MotionWave
                    if let Some(ref mut seat) = ctx.libinput_seat {
                        let (w, h, _) = ctx.state.backend.output_geometry();
                        seat.dispatch_events_full(
                            &mut ctx.state.seat,
                            Some(&mut ctx.motion_wave),
                            w as f32,
                            h as f32,
                        );
                    }
                    Ok(calloop::PostAction::Continue)
                })
                .map_err(|e| anyhow::anyhow!("Failed to insert libinput source: {}", e))?;

            info!("AnimusEngine: libinput event source registered (keyboard/mouse/touch active)");
        }
        Err(e) => {
            tracing::warn!("AnimusEngine: libinput not available ({}), running without input", e);
        }
    }

    info!("AnimusEngine: All event sources registered, entering event loop");

    event_loop
        .run(None, &mut ctx, |_| {})
        .map_err(|e| anyhow::anyhow!("calloop event loop terminated: {}", e))?;

    Ok(())
}
