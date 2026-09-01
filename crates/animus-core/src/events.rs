//! Complete AEEvent (AnimusEngine Event) Enumeration across all architectural subsystems.
//!
//! Aligned with ANIMUSENGINE_COMPLETE_ARCHITECTURE.md & Fixes Volumes 1–4.

use serde::{Deserialize, Serialize};

/// Strongly typed notification payload (FIX-03).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub timeout_ms: i32,      // -1 = persistent
    pub is_persistent: bool,
    pub action_keys: Vec<String>,
    pub action_labels: Vec<String>,
}

impl Default for NotificationPayload {
    fn default() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            timeout_ms: 5000,
            is_persistent: false,
            action_keys: Vec::new(),
            action_labels: Vec::new(),
        }
    }
}

/// Drag and drop payload (FIX-04).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DragPayloadType {
    File,
    Text,
    UriList,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragPayload {
    pub payload_type: DragPayloadType,
    pub data: Vec<u8>,
    pub mime_types: Vec<String>,
    pub origin_x: f32,
    pub origin_y: f32,
}

/// AnimusContext passed into CockpitView and Shell Overlays.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AnimusContext {
    pub active_window_handle: Option<u64>,
    pub screen_width: f32,
    pub screen_height: f32,
    pub dpi_scale: f32,
    pub workspace_index: usize,
}

/// Complete architectural event stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AEEvent {
    // ── Frame & Engine Lifecycle ──────────────────────────────────
    Tick { dt: f32 },
    SpringSettled { settle_id: u64 },
    BootCrossfadeComplete,
    StateChanged { key: String },
    ShutdownRequested,
    ConfigReload,

    // ── Window Management (Part 40 / 41 / FIX2-01) ───────────────
    WindowOpened { handle: u64, app_id: String },
    WindowClosed { handle: u64 },
    WindowFocused { handle: u64, app_id: String },
    WindowBlurred { handle: u64 },
    WindowMoved { handle: u64, x: f32, y: f32 },
    WindowResized { handle: u64, width: f32, height: f32 },
    WindowMaximized { handle: u64 },
    WindowUnmaximized { handle: u64 },
    WindowMinimized { handle: u64 },
    WindowDeminimized { handle: u64 },
    FullscreenEntered { handle: u64 },
    FullscreenExited { handle: u64 },
    WindowAltAltitudeEnter { handle: u64 },
    WindowAltAltitudeExit { handle: u64 },

    // ── Shell Overlays & CockpitView (Part 29 / FIX4-03) ──────────
    DockBounce { app_id: String },
    PanelMenuActivated,
    CockpitViewOpen { ctx: AnimusContext },
    CockpitViewClose,
    CockpitViewOpened,
    CockpitViewClosed,
    LockScreenActivate,
    LockScreenLocked,
    LockScreenUnlocked,
    WelcomeScreenCompleted,
    NotificationPosted(NotificationPayload),
    NotificationDismissed { id: u64 },

    // ── Input & Gestures (Part 30 MotionWave) ─────────────────────
    KeyDown { keycode: u32, modifiers: u32 },
    KeyUp { keycode: u32, modifiers: u32 },
    MouseMoved { x: f32, y: f32 },
    MouseButtonDown { button: u32, x: f32, y: f32 },
    MouseButtonUp { button: u32, x: f32, y: f32 },
    ScrollDelta { dx: f32, dy: f32 },
    SwipeBegin { fingers: u8 },
    SwipeUpdate { dx: f32, dy: f32 },
    SwipeEnd { cancelled: bool },
    DesktopPrev,
    DesktopNext,
    ShowDesktopToggle,
    DragStart(DragPayload),
    DragMotion { x: f32, y: f32 },
    DragDrop { x: f32, y: f32 },
    DragCancel,

    // ── Global Menu & D-Bus / EO-Bus ──────────────────────────────
    DBusMenuRegistered { app_id: String, menu_json: String },
    DBusMenuUpdated { app_id: String, item_path: String },
    DBusMenuChanged,
    StatusNotifierChanged,
    AccessibilityTreeChanged,
    ReducedMotionChanged { enabled: bool },
    OpenURI { uri: String },
    PortalFileChosen { paths: Vec<String> },
    PortalScreenCastStarted,
    GlobalMenuActivated,
    GlobalMenuDeactivated,

    // ── Pathfinder & Search ───────────────────────────────────────
    PathfinderOpened,
    PathfinderClosed,
    PathfinderQueryChanged { query: String },
    PathfinderResultsReady { count: usize },

    // ── HEV Vault & Security (Part 37) ────────────────────────────
    HEVUnlocked,
    HEVLocked,
    HEVSealed,
    HEVAccessDenied,
    ProximityUnlockReady,

    // ── Crash & Subsystem Diagnostics ─────────────────────────────
    ResourcePressure { level: u8 },
    SubsystemHealthChanged { name: String, healthy: bool },
    ClientCrashed { app_id: String, pid: u32 },
    MemoryPressure { available_mb: u64 },

    // ── Install / Package Lifecycle ───────────────────────────────
    InstallProgress { app_id: String, progress: f32 },
    InstallComplete { app_id: String },
    InstallFailed { app_id: String, error: String },
    RemoveComplete { app_id: String },
    RemoveFailed { app_id: String, error: String },

    // ── Hardware & Power ──────────────────────────────────────────
    BatteryLevelChanged { percentage: f32, is_charging: bool },
    BatteryCritical,
    VolumeChanged { volume: f32, muted: bool },
    BrightnessChanged { brightness: f32 },
    NetworkStatusChanged { connected: bool, ssid: Option<String> },
    LidClosed,
    SystemSleep,
    SystemShutdown,
    SystemRestart,
    DisplaySleep,
    DisplayWake,

    // ── Engine & Hardware Lifecycle ───────────────────────────────
    EngineReady,
    GpuTopologyChanged,
    FontInstalled { family: String },

    // ── File System & Background IO ───────────────────────────────
    DirectoryChanged { path: String },
    DirectoryLoaded { path: String, count: usize },
    ThumbnailReady { path: String },
    FileOpProgress { op_id: u64, progress: f32 },
    FileOpComplete { op_id: u64 },
    FileOpConflict { op_id: u64, file_name: String },
}
