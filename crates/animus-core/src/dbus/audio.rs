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
            let pct = format!("{}%", (clamped * 100.0) as u32);
            let res = std::process::Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &pct])
                .status();
            if res.is_err() {
                let _ = std::process::Command::new("pactl")
                    .args(["set-sink-volume", "@DEFAULT_SINK@", &pct])
                    .status();
            }
        }
    }

    pub fn toggle_mute(&self) -> bool {
        let current = self.is_muted.load(Ordering::SeqCst);
        let new_state = !current;
        self.is_muted.store(new_state, Ordering::SeqCst);
        info!("AudioDbusClient: Mute state toggled -> {}", new_state);

        #[cfg(target_os = "linux")]
        {
            let res = std::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .status();
            if res.is_err() {
                let _ = std::process::Command::new("pactl")
                    .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                    .status();
            }
        }

        new_state
    }

    /// Queries the hardware sink volume from PipeWire/WirePlumber if available.
    pub fn query_hardware_volume(&self) -> f32 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("wpctl")
                .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(vol_str) = text.split_whitespace().nth(1) {
                    if let Ok(vol) = vol_str.parse::<f32>() {
                        *self.volume.write() = vol;
                        if text.contains("[MUTED]") {
                            self.is_muted.store(true, Ordering::SeqCst);
                        }
                        return vol;
                    }
                }
            }
        }
        *self.volume.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_dbus_client_volume_and_mute() {
        let client = AudioDbusClient::new();
        assert_eq!(client.get_volume(), 0.85);

        client.set_volume(0.5);
        assert_eq!(client.get_volume(), 0.5);

        let muted = client.toggle_mute();
        assert!(muted);
        assert!(client.is_muted.load(Ordering::SeqCst));

        let unmuted = client.toggle_mute();
        assert!(!unmuted);
    }
}
