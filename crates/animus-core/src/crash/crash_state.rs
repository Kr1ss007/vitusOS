//! CrashStateBlock -- Static Intel Block for Signal Handler (Part 23 of spec).
//!
//! Written continuously by subsystems during normal operation.
//! Read by the signal handler at crash time -- no locks, torn reads accepted.
//! Allocated once at startup. NEVER heap-allocated after that point.
//!
//! Constraint 1: async-signal-safe only in Phase 1 (signal handler).
//! Constraint 2: pre-allocation before first frame -- everything the signal
//! handler needs must already exist in static memory before backend start.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

/// Magic number for crash dump format validation: "VTOS" = 0x56544F53.
pub const CRASHDUMP_MAGIC: u32 = 0x56544F53;
pub const CRASHDUMP_VERSION: u32 = 1;

pub const MAX_STACK_FRAMES: usize = 64;
pub const MAX_VESSEL_NAME: usize = 32;
pub const MAX_VESSEL_COUNT: usize = 32;
pub const MAX_EVENT_RING: usize = 64;
pub const MAX_WAYLAND_CLIENTS: usize = 32;
pub const MAX_CLIENT_APPID: usize = 64;
pub const MAX_MAPS_SNAPSHOT: usize = 65536; // 64KB
pub const MAX_SOUND_NAME: usize = 32;

/// Frame state written by AnimationClock::onPresent() every frame.
#[derive(Debug)]
pub struct CrashFrame {
    pub frame_number: AtomicU64,
    pub total_time_s: AtomicU64,
    pub last_dt: AtomicU32,
    pub refresh_hz: AtomicU32,
}

/// Active spring state written by AnimationEngine::tick().
#[derive(Debug, Clone, Copy)]
pub struct CrashSpringEntry {
    pub name: [u8; 32],
    pub value: f32,
    pub target: f32,
    pub velocity: f32,
    pub is_resting: bool,
}

/// Vessel state written by Vessels on each state change.
#[derive(Debug, Clone, Copy)]
pub struct CrashVesselEntry {
    pub name: [u8; MAX_VESSEL_NAME],
    pub state: u8,
}

/// Event ring buffer entry written by EventBus::publish().
#[derive(Debug, Clone, Copy)]
pub struct CrashEventEntry {
    pub event_id: u32,
    pub time_s: f64,
}

/// Connected Wayland client written by CrashSite.
#[derive(Debug, Clone, Copy)]
pub struct CrashClientEntry {
    pub app_id: [u8; MAX_CLIENT_APPID],
    pub connected: bool,
    pub pid: u32,
}

/// Resource snapshot written by GlobalFeed::monitorLoop().
#[derive(Debug, Clone, Copy)]
pub struct CrashResourceEntry {
    pub vm_rss_kb: u64,
    pub open_fd_count: u32,
    pub gpu_used_bytes: u64,
    pub pw_underruns: u32,
    pub mem_pressure: u8,
    pub fd_pressure: u8,
    pub gpu_pressure: u8,
    pub audio_pressure: u8,
}

/// Last sound played written by SoundEngine::play().
#[derive(Debug, Clone, Copy)]
pub struct CrashSoundEntry {
    pub name: [u8; MAX_SOUND_NAME],
    pub time_s: f64,
}

/// StateManager well-known keys snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct CrashStateKeys {
    pub focused_window_id: u64,
    pub lock_screen_visible: bool,
    pub cockpit_view_open: bool,
    pub pathfinder_open: bool,
    pub wallpaper_tint_r: f32,
    pub wallpaper_tint_g: f32,
    pub wallpaper_tint_b: f32,
    pub system_volume: f32,
    pub dock_visible: bool,
}

/// The complete static intel block -- one global instance.
/// Allocated in static memory, never heap-allocated.
pub struct CrashStateBlock {
    pub magic: AtomicU32,
    pub version: AtomicU32,

    pub frame: CrashFrame,

    pub springs: [CrashSpringEntry; MAX_STACK_FRAMES],
    pub spring_count: AtomicU32,

    pub vessels: [CrashVesselEntry; MAX_VESSEL_COUNT],
    pub vessel_count: AtomicU32,

    pub events: [CrashEventEntry; MAX_EVENT_RING],
    pub event_head: AtomicU32,

    pub clients: [CrashClientEntry; MAX_WAYLAND_CLIENTS],
    pub client_count: AtomicU32,

    pub resources: std::sync::atomic::AtomicPtr<CrashResourceEntry>,
    pub last_sound: std::sync::atomic::AtomicPtr<CrashSoundEntry>,
    pub state_keys: std::sync::atomic::AtomicPtr<CrashStateKeys>,

    pub maps_snapshot: std::sync::atomic::AtomicPtr<[u8; MAX_MAPS_SNAPSHOT]>,
    pub maps_snapshot_len: AtomicU32,
}

// Static storage -- allocated in BSS, never heap
static mut G_CRASH_STATE: Option<CrashStateBlock> = None;
static INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl CrashStateBlock {
    /// Returns the global instance. Must call initialize() first.
    pub fn global() -> &'static mut CrashStateBlock {
        unsafe {
            G_CRASH_STATE.as_mut().expect("CrashStateBlock not initialized")
        }
    }

    /// Initializes the static block. Must be called before wlr_backend_start().
    /// Safe to call multiple times -- subsequent calls are no-ops.
    pub fn initialize() {
        if INITIALIZED.swap(true, Ordering::SeqCst) { return; }

        unsafe {
            G_CRASH_STATE = Some(CrashStateBlock {
                magic: AtomicU32::new(CRASHDUMP_MAGIC),
                version: AtomicU32::new(CRASHDUMP_VERSION),
                frame: CrashFrame {
                    frame_number: AtomicU64::new(0),
                    total_time_s: AtomicU64::new(0),
                    last_dt: AtomicU32::new(0),
                    refresh_hz: AtomicU32::new(0),
                },
                springs: std::array::from_fn(|_| CrashSpringEntry {
                    name: [0; 32], value: 0.0, target: 0.0, velocity: 0.0, is_resting: true,
                }),
                spring_count: AtomicU32::new(0),
                vessels: std::array::from_fn(|_| CrashVesselEntry {
                    name: [0; MAX_VESSEL_NAME], state: 0,
                }),
                vessel_count: AtomicU32::new(0),
                events: std::array::from_fn(|_| CrashEventEntry {
                    event_id: 0, time_s: 0.0,
                }),
                event_head: AtomicU32::new(0),
                clients: std::array::from_fn(|_| CrashClientEntry {
                    app_id: [0; MAX_CLIENT_APPID], connected: false, pid: 0,
                }),
                client_count: AtomicU32::new(0),
                resources: std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()),
                last_sound: std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()),
                state_keys: std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()),
                maps_snapshot: std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()),
                maps_snapshot_len: AtomicU32::new(0),
            });
        }
    }

    /// Updates frame state. Called by AnimationClock::onPresent() every frame.
    pub fn update_frame(frame_number: u64, total_time_s: f64, last_dt: f32, refresh_hz: f32) {
        if !INITIALIZED.load(Ordering::Relaxed) { return; }
        let cs = Self::global();
        cs.frame.frame_number.store(frame_number, Ordering::Relaxed);
        cs.frame.total_time_s.store(total_time_s.to_bits(), Ordering::Relaxed);
        cs.frame.last_dt.store(last_dt.to_bits(), Ordering::Relaxed);
        cs.frame.refresh_hz.store(refresh_hz.to_bits(), Ordering::Relaxed);
    }

    /// Updates a spring entry. Called by AnimationEngine::tick().
    pub fn update_spring(name: &str, value: f32, target: f32, velocity: f32, resting: bool) {
        if !INITIALIZED.load(Ordering::Relaxed) { return; }
        let cs = Self::global();
        let idx = cs.spring_count.load(Ordering::Relaxed) as usize % MAX_STACK_FRAMES;
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(31);
        cs.springs[idx].name[..len].copy_from_slice(&name_bytes[..len]);
        cs.springs[idx].value = value;
        cs.springs[idx].target = target;
        cs.springs[idx].velocity = velocity;
        cs.springs[idx].is_resting = resting;
        cs.spring_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Updates vessel states. Called by Vessels::syncToCrashState().
    pub fn update_vessels(entries: &[(String, u8)]) {
        if !INITIALIZED.load(Ordering::Relaxed) { return; }
        let cs = Self::global();
        let count = entries.len().min(MAX_VESSEL_COUNT);
        for i in 0..count {
            let name_bytes = entries[i].0.as_bytes();
            let len = name_bytes.len().min(MAX_VESSEL_NAME - 1);
            cs.vessels[i].name[..len].copy_from_slice(&name_bytes[..len]);
            cs.vessels[i].name[len] = 0;
            cs.vessels[i].state = entries[i].1;
        }
        cs.vessel_count.store(count as u32, Ordering::Relaxed);
    }

    /// Records an event in the ring buffer. Called by EventBus::publish().
    pub fn record_event(event_id: u32, time_s: f64) {
        if !INITIALIZED.load(Ordering::Relaxed) { return; }
        let cs = Self::global();
        let slot = cs.event_head.fetch_add(1, Ordering::Relaxed) as usize % MAX_EVENT_RING;
        cs.events[slot].event_id = event_id;
        cs.events[slot].time_s = time_s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize test access to the global crash state
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_crash_state_block_init() {
        let _lock = TEST_LOCK.lock().unwrap();
        CrashStateBlock::initialize();
        let cs = CrashStateBlock::global();
        assert_eq!(cs.magic.load(Ordering::Relaxed), CRASHDUMP_MAGIC);
        assert_eq!(cs.version.load(Ordering::Relaxed), CRASHDUMP_VERSION);
    }

    #[test]
    fn test_crash_state_update_frame() {
        let _lock = TEST_LOCK.lock().unwrap();
        CrashStateBlock::initialize();
        CrashStateBlock::update_frame(99, 2.5, 0.007, 144.0);
        let cs = CrashStateBlock::global();
        let fn_val = cs.frame.frame_number.load(Ordering::Relaxed);
        assert!(fn_val >= 42);
    }

    #[test]
    fn test_crash_state_update_spring() {
        let _lock = TEST_LOCK.lock().unwrap();
        CrashStateBlock::initialize();
        let before = CrashStateBlock::global().spring_count.load(Ordering::Relaxed);
        CrashStateBlock::update_spring("TestSpring", 100.0, 200.0, 50.0, false);
        let after = CrashStateBlock::global().spring_count.load(Ordering::Relaxed);
        assert!(after > before);
    }

    #[test]
    fn test_crash_state_record_event() {
        let _lock = TEST_LOCK.lock().unwrap();
        CrashStateBlock::initialize();
        let before = CrashStateBlock::global().event_head.load(Ordering::Relaxed);
        CrashStateBlock::record_event(5, 1.234);
        let after = CrashStateBlock::global().event_head.load(Ordering::Relaxed);
        assert!(after > before);
    }

    #[test]
    fn test_crash_state_update_vessels() {
        let _lock = TEST_LOCK.lock().unwrap();
        CrashStateBlock::initialize();
        let entries = vec![
            ("TestCompositor".to_string(), 0u8),
            ("TestSoundEngine".to_string(), 0u8),
        ];
        CrashStateBlock::update_vessels(&entries);
        let cs = CrashStateBlock::global();
        assert_eq!(cs.vessel_count.load(Ordering::Relaxed), 2);
    }
}
