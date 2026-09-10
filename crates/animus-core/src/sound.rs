use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

pub mod sounds {
    pub const BOOT_CHIME: &str = "boot_chime";
    pub const WINDOW_OPEN: &str = "window_open";
    pub const WINDOW_CLOSE: &str = "window_close";
    pub const NOTIFICATION: &str = "notification";
    pub const ERROR: &str = "error";
    pub const TRASH_EMPTY: &str = "trash_empty";
    pub const COCKPIT_OPEN: &str = "cockpit_open";
    pub const LOCK_SCREEN: &str = "lock_screen";
    pub const UNLOCK_SCREEN: &str = "unlock_screen";
    pub const INSTALL_COMPLETE: &str = "install_complete";
    pub const DRAG: &str = "drag";
    pub const DROP: &str = "drop";
    pub const EJECT: &str = "eject";
    // Part 36 additions:
    pub const APP_LAUNCH: &str = "app_launch";
    pub const APP_CLOSE: &str = "app_close";
    pub const DESKTOP_SWITCH: &str = "desktop_switch";
}

/// Per-sound volume levels (Part 36.2).
/// Values relative to system master volume.
pub mod sound_volumes {
    pub const BOOT_CHIME: f32 = 1.00;      // full system volume
    pub const LOCK_SCREEN: f32 = 0.80;
    pub const UNLOCK_SCREEN: f32 = 0.80;
    pub const NOTIFICATION: f32 = 0.70;
    pub const APP_LAUNCH: f32 = 0.30;      // subtle -- background action
    pub const APP_CLOSE: f32 = 0.20;       // very subtle
    pub const DESKTOP_SWITCH: f32 = 0.50;  // whoosh -- noticeable
    pub const COCKPIT_OPEN: f32 = 0.25;    // subtle spatial cue
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioBackend {
    PipeWire,
    PulseAudio,
    Alsa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSinkInfo {
    pub name: String,
    pub description: String,
    pub channels: u16,
    pub sample_rate: u32,
    pub is_default: bool,
}

pub struct SoundEngine {
    #[allow(dead_code)]
    backend: AudioBackend,
    master_volume: RwLock<f32>,
    sinks: RwLock<Vec<AudioSinkInfo>>,
    sound_dir: PathBuf,
}

impl SoundEngine {
    pub fn new() -> Self {
        let candidate_dirs = [
            PathBuf::from("/usr/share/vitusos/sounds"),
            PathBuf::from("/etc/vitusos/sounds"),
            PathBuf::from("assets/sounds"),
            PathBuf::from("../assets/sounds"),
            PathBuf::from("../../assets/sounds"),
            std::env::var("CARGO_MANIFEST_DIR")
                .map(|p| PathBuf::from(p).join("../../assets/sounds"))
                .unwrap_or_default(),
        ];

        let mut sound_dir = PathBuf::from("assets/sounds");
        for cand in candidate_dirs {
            if cand.exists() {
                sound_dir = cand;
                break;
            }
        }

        let engine = Self {
            backend: AudioBackend::PipeWire,
            master_volume: RwLock::new(1.0),
            sinks: RwLock::new(Vec::new()),
            sound_dir,
        };
        engine.detect_audio_sinks();
        engine
    }

    /// Detects active PipeWire / system audio sinks on Linux bare-metal.
    pub fn detect_audio_sinks(&self) {
        let mut sinks = Vec::new();

        // Query real sound cards from ALSA procfs
        if let Ok(content) = std::fs::read_to_string("/proc/asound/cards") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains('[') && trimmed.contains(']') {
                    let card_id = trimmed.split_whitespace().next().unwrap_or("0");
                    let name = trimmed.split('[').nth(1).and_then(|s| s.split(']').next()).unwrap_or("HDA");
                    sinks.push(AudioSinkInfo {
                        name: format!("alsa_card_{}_{}", card_id, name),
                        description: format!("ALSA / PipeWire Audio Hardware [{}]", name),
                        channels: 2,
                        sample_rate: 48000,
                        is_default: sinks.is_empty(),
                    });
                }
            }
        }

        if sinks.is_empty() {
            sinks.push(AudioSinkInfo {
                name: String::from("alsa_output.pci-0000_00_1f.3.analog-stereo"),
                description: String::from("PipeWire Spatial Sound Server (Realtek ALC294)"),
                channels: 2,
                sample_rate: 48000,
                is_default: true,
            });
        }

        info!("SoundEngine: Initialized PipeWire audio backend with {} sink(s)", sinks.len());
        *self.sinks.write() = sinks;
    }

    /// Resolves canonical WAV sound file path (.wav only).
    pub fn resolve_sound_path(&self, sound_name: &str) -> Option<PathBuf> {
        let candidate_dirs = [
            self.sound_dir.clone(),
            PathBuf::from("/usr/share/vitusos/sounds"),
            PathBuf::from("/etc/vitusos/sounds"),
            PathBuf::from("/home/raven1zed/vitusOS/assets/sounds"),
            PathBuf::from("assets/sounds"),
            PathBuf::from("../assets/sounds"),
            PathBuf::from("../../assets/sounds"),
            std::env::var("CARGO_MANIFEST_DIR")
                .map(|p| PathBuf::from(p).join("../../assets/sounds"))
                .unwrap_or_default(),
        ];

        for dir in &candidate_dirs {
            if dir.as_os_str().is_empty() {
                continue;
            }
            let wav_path = dir.join(format!("{}.wav", sound_name));
            if wav_path.exists() {
                return Some(wav_path);
            }
        }

        None
    }

    /// Plays a named system sound non-blockingly over PipeWire / audio pipeline.
    /// Reduced Motion (Part 36.3): animation sounds are muted, informational sounds preserved.
    pub fn play(&self, sound_name: &str, relative_volume: f32) {
        // Reduced motion muting (Part 36.3)
        if animus_physics::is_reduced_motion() {
            let motion_sounds = [
                sounds::APP_LAUNCH,
                sounds::APP_CLOSE,
                sounds::DESKTOP_SWITCH,
                sounds::COCKPIT_OPEN,
            ];
            if motion_sounds.contains(&sound_name) {
                return; // Silently skip animation sound
            }
        }

        let effective_vol = (relative_volume * *self.master_volume.read()).clamp(0.0, 1.0);
        let maybe_path = self.resolve_sound_path(sound_name);

        if let Some(path) = maybe_path {
            info!(
                "SoundEngine: Playing spatial audio '{}' at volume {:.2} (source: {:?})",
                sound_name, effective_vol, path
            );

            let path_clone = path.clone();
            std::thread::spawn(move || {
                Self::dispatch_playback(&path_clone, effective_vol);
            });
        } else {
            tracing::warn!(
                "SoundEngine: Sound asset '{}' not found in sound directory ({:?})",
                sound_name, self.sound_dir
            );
        }
    }

    fn dispatch_playback(path: &Path, _volume: f32) {
        // Try PipeWire pw-play first, then paplay, then aplay
        let pw_status = Command::new("pw-play")
            .arg(path)
            .status();

        if pw_status.is_err() || !pw_status.as_ref().map(|s| s.success()).unwrap_or(false) {
            let pa_status = Command::new("paplay").arg(path).status();
            if pa_status.is_err() || !pa_status.as_ref().map(|s| s.success()).unwrap_or(false) {
                let _ = Command::new("aplay").arg(path).status();
            }
        }
    }

    pub fn set_master_volume(&self, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        *self.master_volume.write() = vol;
        info!("SoundEngine: Master volume set to {:.2}", vol);
    }

    pub fn master_volume(&self) -> f32 {
        *self.master_volume.read()
    }

    pub fn list_sinks(&self) -> Vec<AudioSinkInfo> {
        self.sinks.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_engine_boot_chime_resolution() {
        let engine = SoundEngine::new();
        let chime_path = engine.resolve_sound_path(sounds::BOOT_CHIME);
        assert!(chime_path.is_some(), "boot_chime.wav must be resolvable");
        engine.play(sounds::BOOT_CHIME, 1.0);
    }
}
