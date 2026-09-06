//! AEBridge -- Cross-Process EventBus Bridge via Unix Domain Socket.
//!
//! vitusOS runs as two processes (Part 28):
//! - vitusos-session: owns HEV, StateManager, EventBus (session-side), EOBus, SeaDrop
//! - vitusos-compositor: owns AnimusEngine, CrashManager, CacheKeepr, Shell, EventBus (compositor-side)
//!
//! AEBridge connects the two via a Unix domain socket at `/run/vitusos/ae-ipc.sock`.
//! Only events marked BRIDGED cross the bridge. LOCAL events stay in-process.
//!
//! Wire format (length-prefixed binary):
//!   [4 bytes] u32 message length (LE, not including this field)
//!   [4 bytes] u32 event discriminant (BridgedEvent variant index)
//!   [N bytes] serde-serialized payload (or empty if no payload)
//!
//! Session process binds the socket. Compositor process connects to it.
//! Reconnect: compositor attempts reconnect every 500ms on connection loss.
//!
//! Compiled on WSL2/Linux (the vitusOS build environment). No stubs needed.

use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::event_bus::EventBus;
use crate::events::AEEvent;

/// The socket path for the AEBridge IPC.
pub const AE_IPC_SOCKET_PATH: &str = "/run/vitusos/ae-ipc.sock";

/// Maximum message size for the bridge wire protocol.
const MAX_MSG_SIZE: usize = 4096;

/// Events that cross the session-compositor boundary.
///
/// Only these events are forwarded via the Unix socket.
/// All other AEEvent variants are LOCAL — they stay in their originating process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BridgedEvent {
    // -- Session to Compositor --
    /// HEV vault unlocked — LockScreen dismisses
    HEVUnlocked,
    /// HEV vault locked — LockScreen appears
    HEVLocked,
    /// Wallpaper path changed — RenderPipeline + WallpaperTintSampler reload
    WallpaperChanged { path: String },
    /// A state key changed — compositor reads new value from its own StateManager
    StateChanged { key: String },
    /// Package install completed — CacheKeepr invalidates
    InstallComplete { app_id: String },
    /// App index rebuild done — Pathfinder refreshes
    AppIndexReady,
    /// Config file changed (SIGHUP received)
    ConfigReload,

    // -- Compositor to Session --
    /// A Wayland client connected — session: ClientRegistry.registerClient()
    ClientConnected { window_handle: u64 },
    /// A Wayland client crashed — session: ClientRegistry.unregisterClient()
    ClientCrashed { app_id: String, pid: u32 },
    /// Focused window changed — session: StateManager updates
    WindowFocusChanged { window_handle: u64 },
    /// Compositor is ready (first frame rendered) — session: log + unblock
    CompositorReady,
    /// Fatal error in compositor — session: log + prepare for restart
    FatalError { description: String },

    // -- Bidirectional --
    /// System shutdown requested (can originate from either process)
    SystemShutdown,
    /// System restart requested
    SystemRestart,
}

impl BridgedEvent {
    /// Converts an AEEvent to a BridgedEvent if it is a BRIDGED event.
    /// Returns None for LOCAL events.
    pub fn from_ae_event(event: &AEEvent) -> Option<Self> {
        match event {
            AEEvent::HEVUnlocked => Some(Self::HEVUnlocked),
            AEEvent::HEVLocked => Some(Self::HEVLocked),
            AEEvent::HEVSealed => Some(Self::HEVLocked),
            AEEvent::InstallComplete { app_id } => Some(Self::InstallComplete { app_id: app_id.clone() }),
            AEEvent::ConfigReload => Some(Self::ConfigReload),
            AEEvent::ClientCrashed { app_id, pid } => Some(Self::ClientCrashed { app_id: app_id.clone(), pid: *pid }),
            AEEvent::SystemShutdown => Some(Self::SystemShutdown),
            AEEvent::SystemRestart => Some(Self::SystemRestart),
            AEEvent::ShutdownRequested => Some(Self::SystemShutdown),
            _ => None,
        }
    }

    /// Converts a BridgedEvent back into an AEEvent for the receiving process.
    pub fn to_ae_event(&self) -> AEEvent {
        match self {
            Self::HEVUnlocked => AEEvent::HEVUnlocked,
            Self::HEVLocked => AEEvent::HEVLocked,
            Self::WallpaperChanged { path } => AEEvent::StateChanged { key: path.clone() },
            Self::StateChanged { key } => AEEvent::StateChanged { key: key.clone() },
            Self::InstallComplete { app_id } => AEEvent::InstallComplete { app_id: app_id.clone() },
            Self::AppIndexReady => AEEvent::EngineReady,
            Self::ConfigReload => AEEvent::ConfigReload,
            Self::ClientConnected { window_handle } => AEEvent::WindowOpened {
                handle: *window_handle,
                app_id: String::new(),
            },
            Self::ClientCrashed { app_id, pid } => AEEvent::ClientCrashed {
                app_id: app_id.clone(),
                pid: *pid,
            },
            Self::WindowFocusChanged { window_handle } => AEEvent::WindowFocused {
                handle: *window_handle,
                app_id: String::new(),
            },
            Self::CompositorReady => AEEvent::EngineReady,
            Self::FatalError { description: _ } => AEEvent::ShutdownRequested,
            Self::SystemShutdown => AEEvent::SystemShutdown,
            Self::SystemRestart => AEEvent::SystemRestart,
        }
    }
}

/// The cross-process AEBridge.
///
/// On the session side: `bind_as_session()` creates the listening socket.
/// On the compositor side: `connect_to_session()` connects to it.
/// Both sides spawn a background RX thread that deserializes incoming
/// BridgedEvents and publishes them to the local EventBus.
pub struct AEBridge {
    socket_path: PathBuf,
    is_running: Arc<AtomicBool>,
    rx_thread: Option<thread::JoinHandle<()>>,
    tx_stream: Arc<parking_lot::Mutex<Option<UnixStream>>>,
    bus: EventBus,
}

impl AEBridge {
    /// Creates a new AEBridge bound to the given EventBus.
    /// The socket path defaults to `/run/vitusos/ae-ipc.sock`.
    pub fn new(bus: EventBus) -> Self {
        Self {
            socket_path: PathBuf::from(AE_IPC_SOCKET_PATH),
            is_running: Arc::new(AtomicBool::new(false)),
            rx_thread: None,
            tx_stream: Arc::new(parking_lot::Mutex::new(None)),
            bus,
        }
    }

    /// Session side: binds the Unix socket and waits for the compositor to connect.
    ///
    /// This must be called before the compositor starts. systemd ensures this
    /// ordering via `After=vitusos-session.service` on the compositor unit.
    pub fn bind_as_session(&mut self) -> io::Result<()> {
        // Ensure /run/vitusos exists
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Remove stale socket if present
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = UnixListener::bind(&self.socket_path)?;
        info!("AEBridge: Session bound socket at {:?}", self.socket_path);

        self.is_running.store(true, Ordering::SeqCst);
        let bus = self.bus.clone();
        let is_running = self.is_running.clone();
        let tx_stream = self.tx_stream.clone();

        self.rx_thread = Some(thread::spawn(move || {
            // Accept a single connection (the compositor)
            match listener.accept() {
                Ok((stream, _addr)) => {
                    info!("AEBridge: Compositor connected to session");

                    // Store the stream for TX (session -> compositor)
                    *tx_stream.lock() = stream.try_clone().ok();

                    // RX loop: receive BridgedEvents from compositor
                    let mut reader = stream;
                    Self::rx_loop(&mut reader, &bus, &is_running, "session");
                }
                Err(e) => {
                    error!("AEBridge: Failed to accept compositor connection: {}", e);
                }
            }
        }));

        Ok(())
    }

    /// Compositor side: connects to the session's Unix socket.
    ///
    /// Retries every 500ms until the session is available (max 30s).
    pub fn connect_to_session(&mut self) -> io::Result<()> {
        let max_retries = 60;
        let mut stream = None;

        for attempt in 0..max_retries {
            match UnixStream::connect(&self.socket_path) {
                Ok(s) => {
                    stream = Some(s);
                    info!("AEBridge: Compositor connected to session on attempt {}", attempt + 1);
                    break;
                }
                Err(_) if attempt < max_retries - 1 => {
                    thread::sleep(std::time::Duration::from_millis(500));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        let stream = stream.ok_or_else(|| {
            io::Error::new(io::ErrorKind::ConnectionRefused, "Session not reachable after 30s")
        })?;

        self.is_running.store(true, Ordering::SeqCst);

        // Store for TX (compositor -> session)
        *self.tx_stream.lock() = stream.try_clone().ok();

        let bus = self.bus.clone();
        let is_running = self.is_running.clone();

        self.rx_thread = Some(thread::spawn(move || {
            let mut reader = stream;
            Self::rx_loop(&mut reader, &bus, &is_running, "compositor");
        }));

        Ok(())
    }

    /// Forwards a BridgedEvent to the other process via the socket.
    ///
    /// Called from either process. Serializes the event and writes it
    /// to the TX stream. If the connection is lost, the event is dropped.
    pub fn send(&self, event: &BridgedEvent) {
        let mut stream_guard = self.tx_stream.lock();
        if let Some(ref mut stream) = *stream_guard {
            match Self::serialize_event(event) {
                Ok(buf) => {
                    if let Err(e) = stream.write_all(&buf) {
                        warn!("AEBridge: TX write failed: {}", e);
                    }
                }
                Err(e) => {
                    warn!("AEBridge: Serialization failed: {}", e);
                }
            }
        }
    }

    /// Stops the bridge and closes the connection.
    pub fn destroy(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        *self.tx_stream.lock() = None;
        let _ = std::fs::remove_file(&self.socket_path);
        info!("AEBridge: Bridge destroyed");
    }

    /// RX loop: reads length-prefixed BridgedEvent messages from the socket
    /// and publishes them to the local EventBus via publish_async.
    fn rx_loop(
        reader: &mut UnixStream,
        bus: &EventBus,
        is_running: &AtomicBool,
        role: &str,
    ) {
        let mut len_buf = [0u8; 4];

        while is_running.load(Ordering::Relaxed) {
            match reader.read_exact(&mut len_buf) {
                Ok(_) => {
                    let msg_len = u32::from_le_bytes(len_buf) as usize;
                    if msg_len > MAX_MSG_SIZE || msg_len == 0 {
                        warn!("AEBridge [{}]: Invalid message length {}, closing", role, msg_len);
                        break;
                    }

                    let mut msg_buf = vec![0u8; msg_len];
                    match reader.read_exact(&mut msg_buf) {
                        Ok(_) => {
                            match Self::deserialize_event(&msg_buf) {
                                Ok(event) => {
                                    let ae_event = event.to_ae_event();
                                    bus.publish_async(ae_event);
                                }
                                Err(e) => {
                                    warn!("AEBridge [{}]: Deserialization failed: {}", role, e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("AEBridge [{}]: Read body failed: {}", role, e);
                            break;
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    info!("AEBridge [{}]: Connection closed by peer", role);
                    break;
                }
                Err(e) => {
                    warn!("AEBridge [{}]: Read header failed: {}", role, e);
                    break;
                }
            }
        }

        info!("AEBridge [{}]: RX loop terminated", role);
    }

    /// Serializes a BridgedEvent to a length-prefixed binary message.
    fn serialize_event(event: &BridgedEvent) -> io::Result<Vec<u8>> {
        let payload = serde_json::to_vec(event)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let total_len = payload.len() as u32;
        let mut buf = Vec::with_capacity(4 + payload.len());
        buf.extend_from_slice(&total_len.to_le_bytes());
        buf.extend_from_slice(&payload);
        Ok(buf)
    }

    /// Deserializes a BridgedEvent from a binary message buffer.
    fn deserialize_event(buf: &[u8]) -> io::Result<BridgedEvent> {
        serde_json::from_slice(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl Drop for AEBridge {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridged_event_serialization() {
        let event = BridgedEvent::HEVUnlocked;
        let buf = AEBridge::serialize_event(&event).unwrap();
        assert!(buf.len() > 4);

        let payload = &buf[4..];
        let deserialized = AEBridge::deserialize_event(payload).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_bridged_event_with_payload() {
        let event = BridgedEvent::ClientCrashed {
            app_id: "org.vitusos.filer".to_string(),
            pid: 12345,
        };
        let buf = AEBridge::serialize_event(&event).unwrap();
        let payload = &buf[4..];
        let deserialized = AEBridge::deserialize_event(payload).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_bridged_event_from_ae_event() {
        assert_eq!(
            BridgedEvent::from_ae_event(&AEEvent::HEVUnlocked),
            Some(BridgedEvent::HEVUnlocked)
        );
        assert_eq!(
            BridgedEvent::from_ae_event(&AEEvent::HEVLocked),
            Some(BridgedEvent::HEVLocked)
        );
        // LOCAL event -- should return None
        assert!(BridgedEvent::from_ae_event(&AEEvent::Tick { dt: 0.016 }).is_none());
    }

    #[test]
    fn test_bridged_event_to_ae_event() {
        let event = BridgedEvent::HEVUnlocked;
        let ae_event = event.to_ae_event();
        match ae_event {
            AEEvent::HEVUnlocked => {}
            _ => panic!("Expected HEVUnlocked"),
        }
    }
}
