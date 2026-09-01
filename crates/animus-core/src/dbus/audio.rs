//! PipeWire & PulseAudio D-Bus Audio Proxy.

use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::RwLock;
use tracing::info;

pub struct AudioDbusClient {
    pub volume: RwLock<f32>,
    pub is_muted: AtomicBool,
}

impl Default for AudioDbusClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDbusClient {
    pub fn new() -> Self {
        Self {
            volume: RwLock::new(0.85),
            is_muted: AtomicBool::new(false),
        }
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.read()
    }

    pub fn set_volume(&self, vol: f32) {
        let clamped = vol.clamp(0.0, 1.0);
        *self.volume.write() = clamped;
        info!("AudioDbusClient: Master volume set to {:.2}", clamped);

        #[cfg(target_os = "linux")]
        {
            // Execute wpctl or pactl to adjust PipeWire sink volume directly
            let pct = format!("{}%", (clamped * 100.0) as u32);
            let _ = std::process::Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &pct])
                .spawn();
        }
    }

    pub fn toggle_mute(&self) -> bool {
        let current = self.is_muted.load(Ordering::SeqCst);
        let new_state = !current;
        self.is_muted.store(new_state, Ordering::SeqCst);
        info!("AudioDbusClient: Mute state toggled -> {}", new_state);

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .spawn();
        }

        new_state
    }
}
