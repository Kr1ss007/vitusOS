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
//! 
//! systemd starts this process BEFORE the compositor:
//!   After: dbus.service pipewire.service
//!   Type: notify (sd_notify READY=1 on init complete)
//!   WatchdogSec: 10s (sd_notify WATCHDOG=1 every 5s)
//!   Restart: on-failure, RestartSec: 1s
//!
//! The compositor connects to this process via AEBridge.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use animus_core::crash::CrashManager;
use animus_core::eobus::EOBus;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_core::registry::RegistryManager;
use animus_core::state::StateManager;
use animus_core::AEBridge;

use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// Shared shutdown flag — set to true by the SIGTERM handler.
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

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
    /// Uses a condition-variable sleep instead of a busy-loop to avoid
    /// wasting CPU while waiting for events. Notifies systemd watchdog
    /// every 5 seconds (half of WatchdogSec=10s).
    ///
    /// Exits cleanly on SIGTERM (shutdown flag set by signal handler).
    pub fn run(&mut self) -> anyhow::Result<()> {
        info!("vitusos-session: Event loop started (WatchdogSec=10s, watchdog ping=5s)");

        // Notify systemd: we are fully initialized and ready.
        // This transitions the service from activating → active.
        notify_systemd_ready();

        let watchdog_interval = Duration::from_secs(5);
        let mut last_watchdog = Instant::now();

        while !SHUTDOWN_REQUESTED.load(Ordering::Relaxed) {
            // Drain background events onto the main loop
            self.event_bus.drain_async_queue();

            // Systemd watchdog ping every 5s (WatchdogSec=10s, ping at half-period)
            if last_watchdog.elapsed() >= watchdog_interval {
                notify_systemd_watchdog();
                last_watchdog = Instant::now();
            }

            // Sleep just below the watchdog threshold (8ms tick, wakes on SIGTERM via atomic)
            // This gives us ~125 Hz event loop responsiveness without busy-spinning.
            std::thread::sleep(Duration::from_millis(8));
        }

        info!("vitusos-session: Shutdown requested — draining final events");
        self.event_bus.drain_async_queue();

        Ok(())
    }
}

/// Notify systemd that the service is ready (Type=notify).
///
/// Sends "READY=1\n" to the systemd socket specified by NOTIFY_SOCKET.
/// This is safe to call even if not running under systemd — it silently
/// does nothing if NOTIFY_SOCKET is not set.
fn notify_systemd_ready() {
    match sd_notify::notify(false, &[sd_notify::NotifyState::Ready]) {
        Ok(()) => info!("vitusos-session: sd_notify READY=1 sent"),
        Err(e) => warn!("vitusos-session: sd_notify READY=1 failed (not under systemd?): {}", e),
    }
}

/// Notify systemd watchdog (prevents service restart due to timeout).
fn notify_systemd_watchdog() {
    match sd_notify::notify(false, &[sd_notify::NotifyState::Watchdog]) {
        Ok(()) => {}
        Err(e) => warn!("vitusos-session: sd_notify WATCHDOG=1 failed: {}", e),
    }
}

/// Install the SIGTERM handler using nix.
///
/// Uses a simple `AtomicBool` flag to signal the event loop to exit.
/// This is async-signal-safe: we only write to an atomic inside the handler.
#[cfg(unix)]
fn install_signal_handlers() {
    use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};

    extern "C" fn sigterm_handler(_: i32) {
        SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
    }

    let action = SigAction::new(
        SigHandler::Handler(sigterm_handler),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );

    unsafe {
        // Handle both SIGTERM (systemd stop) and SIGINT (Ctrl-C in dev)
        signal::sigaction(Signal::SIGTERM, &action)
            .expect("Failed to install SIGTERM handler");
        signal::sigaction(Signal::SIGINT, &action)
            .expect("Failed to install SIGINT handler");
    }

    info!("vitusos-session: SIGTERM/SIGINT handlers installed");
}

#[cfg(not(unix))]
fn install_signal_handlers() {
    // Non-Unix: no signal handling needed
}

fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("=== vitusOS Session Process Starting ===");

    // Install signal handlers BEFORE initializing subsystems.
    // This ensures a SIGTERM during init is caught cleanly.
    install_signal_handlers();

    let mut ctx = SessionContext::new();

    match ctx.initialize() {
        Ok(()) => info!("=== vitusOS Session Process Ready ==="),
        Err(e) => {
            error!("vitusos-session: Initialization failed: {}", e);
            // Notify systemd of failure before exiting
            let _ = sd_notify::notify(false, &[
                sd_notify::NotifyState::Status("Initialization failed"),
            ]);
            return Err(e);
        }
    }

    ctx.run()?;

    info!("=== vitusOS Session Process Shutting Down ===");
    ctx.ae_bridge.destroy();
    ctx.crash_manager.destroy();

    // Notify systemd that we stopped cleanly
    let _ = sd_notify::notify(false, &[
        sd_notify::NotifyState::Stopping,
    ]);

    Ok(())
}
