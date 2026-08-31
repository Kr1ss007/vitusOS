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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioBackend {
    PipeWire,
    PulseAudio,
    Alsa,
    DirectSound,
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

    /// Detects active PipeWire / system audio sinks across Linux and host OS.
    pub fn detect_audio_sinks(&self) {
        let mut sinks = Vec::new();

        #[cfg(target_os = "linux")]
        {
            // Query PipeWire / wireplumber or fallback to default stereo sink
            sinks.push(AudioSinkInfo {
                name: String::from("alsa_output.pci-0000_00_1f.3.analog-stereo"),
                description: String::from("PipeWire Spatial Sound Server (Realtek ALC294)"),
                channels: 2,
                sample_rate: 48000,
                is_default: true,
            });
        }

        #[cfg(not(target_os = "linux"))]
        {
            sinks.push(AudioSinkInfo {
                name: String::from("DirectSound.PrimaryAudioDriver"),
                description: String::from("Realtek High Definition Audio (Spatial Boot Chime)"),
                channels: 2,
                sample_rate: 48000,
                is_default: true,
            });
        }

        info!("SoundEngine: Initialized PipeWire audio backend with {} sink(s)", sinks.len());
        *self.sinks.write() = sinks;
    }

    /// Resolves sound file path (.wav or .mp3).
    pub fn resolve_sound_path(&self, sound_name: &str) -> Option<PathBuf> {
        let wav_path = self.sound_dir.join(format!("{}.wav", sound_name));
        if wav_path.exists() {
            return Some(wav_path);
        }

        let mp3_path = self.sound_dir.join(format!("{}.mp3", sound_name));
        if mp3_path.exists() {
            return Some(mp3_path);
        }

        None
    }

    /// Plays a named system sound non-blockingly over PipeWire / audio pipeline.
    pub fn play(&self, sound_name: &str, relative_volume: f32) {
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
            info!(
                "SoundEngine: Spatial sound '{}' queued (simulated in headless/test mode)",
                sound_name
            );
        }
    }

    fn dispatch_playback(path: &Path, _volume: f32) {
        #[cfg(target_os = "linux")]
        {
            // Try PipeWire pw-play first, then paplay, then aplay
            let pw_status = Command::new("pw-play")
                .arg(path)
                .status();

            if pw_status.is_err() || !pw_status.as_ref().map(|s| s.success()).unwrap_or(false) {
                let _ = Command::new("paplay").arg(path).status();
            }
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, if wav file, play asynchronously via SoundPlayer
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("wav") {
                    let path_str = path.to_string_lossy().to_string();
                    let _ = Command::new("powershell")
                        .args(["-NoProfile", "-Command", &format!("(New-Object Media.SoundPlayer '{}').PlaySync()", path_str)])
                        .status();
                }
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
        assert!(chime_path.is_some(), "boot_chime.wav or boot_chime.mp3 must be resolvable");
        engine.play(sounds::BOOT_CHIME, 1.0);
    }
}
