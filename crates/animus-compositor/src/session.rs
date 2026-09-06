//! vitusOS Session Process -- Hybrid Architecture Session Manager.
//!
//! This is the session-side process in vitusOS's two-process model (Part 28):
//!
//! vitusos-session owns:
//! - CrashManager (session-side)
//! - RegistryManager (session-side)
//! - EventBus (session-side)
//! - StateManager
//! - HEV (Hardware Encryption Vault)
//! - AEBridge (binds /run/vitusos/ae-ipc.sock)
//! - EOBus (D-Bus session bus, portals, accessibility)
//! - SeaDrop trust subsystem (planned)
//!
//! systemd starts this process BEFORE the compositor:
//!   After: dbus.service pipewire.service
//!   Restart: on-failure, RestartSec: 1s
//!   WatchdogSec: 10s
//!
//! The compositor connects to this process via AEBridge.

use std::sync::Arc;
use std::time::Duration;

use animus_core::crash::CrashManager;
use animus_core::eobus::EOBus;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_core::registry::RegistryManager;
use animus_core::state::StateManager;
use animus_core::AEBridge;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// The session process context -- owns all session-side subsystems.
pub struct SessionContext {
    pub event_bus: Arc<EventBus>,
    pub crash_manager: Arc<CrashManager>,
    pub registry: Arc<RegistryManager>,
    pub state: Arc<StateManager>,
    pub eobus: Arc<EOBus>,
    pub ae_bridge: AEBridge,
}

impl SessionContext {
    pub fn new() -> Self {
        let event_bus = Arc::new(EventBus::new());
        let crash_manager = Arc::new(CrashManager::new((*event_bus).clone()));
        let registry = Arc::new(RegistryManager::new((*event_bus).clone()));
        let state = Arc::new(StateManager::new((*event_bus).clone()));
        let eobus = Arc::new(EOBus::new((*event_bus).clone()));
        let ae_bridge = AEBridge::new((*event_bus).clone());

        Self {
            event_bus,
            crash_manager,
            registry,
            state,
            eobus,
            ae_bridge,
        }
    }

    /// Initializes all session-side subsystems in the correct order.
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        // Step 0: CrashManager -- ALWAYS FIRST
        self.crash_manager.initialize();
        info!("vitusos-session: CrashManager initialized");

        // Step 1: EventBus -- already created
        info!("vitusos-session: EventBus ready");

        // Step 2: StateManager -- already created with defaults
        info!("vitusos-session: StateManager ready");

        // Step 3: HEV -- Hardware Encryption Vault
        // HEV is initialized on first user unlock, not at session start.
        // The vault starts in Cold state and requires password entry.
        info!("vitusos-session: HEV vault in Cold state (awaiting first unlock)");

        // Step 4: AEBridge -- bind the IPC socket for the compositor
        self.ae_bridge.bind_as_session()
            .map_err(|e| anyhow::anyhow!("AEBridge bind failed: {}", e))?;
        info!("vitusos-session: AEBridge listening for compositor connection");

        // Step 5: EOBus -- start D-Bus and udev listeners
        self.eobus.start();
        info!("vitusos-session: EOBus listeners started");

        Ok(())
    }

    /// Runs the session event loop.
    ///
    /// The session process doesn't have a calloop event loop like the compositor.
    /// Instead, it uses a simple loop that drains the EventBus async queue
    /// and checks for shutdown signals. The AEBridge RX thread handles
    /// incoming events from the compositor in the background.
    pub fn run(&mut self) -> anyhow::Result<()> {
        info!("vitusos-session: Event loop started (WatchdogSec=10s)");

        loop {
            // Drain background events onto the main loop
            self.event_bus.drain_async_queue();

            // Check for shutdown
            // In production, this would also check for:
            // - systemd watchdog (sd_notify WATCHDOG=1)
            // - SIGTERM (graceful shutdown)
            // - FatalError from compositor via AEBridge
            // For now, just sleep to avoid busy-looping
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("=== vitusOS Session Process Starting ===");

    let mut ctx = SessionContext::new();
    ctx.initialize()?;

    info!("=== vitusOS Session Process Ready ===");

    ctx.run()?;

    info!("=== vitusOS Session Process Shutting Down ===");
    ctx.ae_bridge.destroy();
    ctx.crash_manager.destroy();

    Ok(())
}
