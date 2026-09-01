//! Settings: Split-Pane System Configuration App for vitusOS.
//!
//! Aligned with Part 33 of specification.
//! Features 9 comprehensive system preference sections, Spring-driven sidebar navigation,
//! and live system state synchronization.

use animus_core::dbus::SystemDbusManager;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsSection {
    Wallpaper,
    Appearance,
    Displays,
    Sound,
    Keyboard,
    MotionWave,
    SecurityVault,
    Updates,
    About,
}

impl SettingsSection {
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Wallpaper => "Wallpaper",
            Self::Appearance => "Appearance",
            Self::Displays => "Displays",
            Self::Sound => "Sound & Spatial Audio",
            Self::Keyboard => "Keyboard & Input",
            Self::MotionWave => "MotionWave & Gestures",
            Self::SecurityVault => "Security & HEV Vault",
            Self::Updates => "Software Update & Channel",
            Self::About => "About vitusOS",
        }
    }

    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Wallpaper => "wallpaper",
            Self::Appearance => "appearance",
            Self::Displays => "display",
            Self::Sound => "sound",
            Self::Keyboard => "keyboard",
            Self::MotionWave => "gestures",
            Self::SecurityVault => "vault",
            Self::Updates => "update",
            Self::About => "info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OTAChannel {
    /// The latest & experimental rolling build (OTA through GitHub)
    UpstreamColor,
    /// The stable verified production releases (OTA through GitHub)
    UpstreamOne,
}

impl std::fmt::Display for OTAChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UpstreamColor => write!(f, "vitusOS Upstream Color (Experimental Rolling)"),
            Self::UpstreamOne => write!(f, "vitusOS Upstream One (Stable Verified)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSettingsState {
    // Appearance
    pub is_dark_mode: bool,
    pub accent_color_hex: String, // e.g. "#FF6B00" (Space Orange)
    pub reduce_motion: bool,
    pub reduce_transparency: bool,

    // Displays
    pub display_resolution: String,
    pub refresh_rate_hz: f32,
    pub ui_scale: f64,
    pub night_shift_enabled: bool,
    pub color_temperature_k: u32,

    // Sound
    pub output_volume: f32,
    pub boot_chime_enabled: bool,
    pub spatial_audio_dsp: bool,

    // MotionWave
    pub trackpad_natural_scroll: bool,
    pub three_finger_swipe_enabled: bool,
    pub fling_friction: f32,

    // Security & HEV
    pub hev_encryption_active: bool,
    pub tpm_pcr_sealed: bool,
    pub proximity_lock_enabled: bool,

    // OTA Updates
    pub active_channel: OTAChannel,
    pub is_checking_ota: bool,
    pub update_available: bool,
    pub remote_version: Option<String>,
}

impl Default for SystemSettingsState {
    fn default() -> Self {
        Self {
            is_dark_mode: true,
            accent_color_hex: "#FF6B00".to_string(),
            reduce_motion: false,
            reduce_transparency: false,
            display_resolution: "1920x1080".to_string(),
            refresh_rate_hz: 144.0,
            ui_scale: 1.0,
            night_shift_enabled: false,
            color_temperature_k: 6500,
            output_volume: 0.85,
            boot_chime_enabled: true,
            spatial_audio_dsp: true,
            trackpad_natural_scroll: true,
            three_finger_swipe_enabled: true,
            fling_friction: 0.985,
            hev_encryption_active: true,
            tpm_pcr_sealed: true,
            proximity_lock_enabled: false,
            active_channel: OTAChannel::UpstreamColor,
            is_checking_ota: false,
            update_available: false,
            remote_version: None,
        }
    }
}

pub struct SettingsApp {
    pub altitude: SurfaceAltitude, // Mid (20px Kawase Blur, 82% Opacity)
    pub current_section: RwLock<SettingsSection>,
    pub selection_pill_y: RwLock<SpringSolver>, // SPRING_SELECTION (400, 28)
    pub state: RwLock<SystemSettingsState>,
    pub dbus: Arc<SystemDbusManager>,
    bus: EventBus,
}

impl SettingsApp {
    pub fn new(bus: EventBus) -> Self {
        Self {
            altitude: SurfaceAltitude::Mid,
            current_section: RwLock::new(SettingsSection::Appearance),
            selection_pill_y: RwLock::new(SpringSolver::new(36.0, SpringProfile::Selection)),
            state: RwLock::new(SystemSettingsState::default()),
            dbus: Arc::new(SystemDbusManager::new()),
            bus,
        }
    }

    pub fn select_section(&self, section: SettingsSection) {
        let mut curr = self.current_section.write();
        *curr = section;
        let idx = match section {
            SettingsSection::Wallpaper => 0,
            SettingsSection::Appearance => 1,
            SettingsSection::Displays => 2,
            SettingsSection::Sound => 3,
            SettingsSection::Keyboard => 4,
            SettingsSection::MotionWave => 5,
            SettingsSection::SecurityVault => 6,
            SettingsSection::Updates => 7,
            SettingsSection::About => 8,
        };
        self.selection_pill_y.write().set_target(idx as f32 * 36.0);
        info!("Settings: Selected section -> {:?}", section);
    }

    pub fn set_ota_channel(&self, channel: OTAChannel) {
        let mut s = self.state.write();
        s.active_channel = channel;
        info!("Settings: Switched OTA Channel to: {}", channel);
    }

    pub fn toggle_dark_mode(&self) {
        let mut s = self.state.write();
        s.is_dark_mode = !s.is_dark_mode;
        info!("Settings: Dark Mode toggled -> {}", s.is_dark_mode);
    }

    pub fn set_volume(&self, vol: f32) {
        let mut s = self.state.write();
        s.output_volume = vol.clamp(0.0, 1.0);
        self.dbus.audio.set_volume(s.output_volume);
        self.bus.publish(AEEvent::VolumeChanged { volume: s.output_volume, muted: false });
    }

    pub fn toggle_wifi(&self, enabled: bool) {
        let dbus = self.dbus.clone();
        tokio::spawn(async move {
            dbus.network.set_wifi_enabled(enabled).await;
        });
    }

    pub fn toggle_bluetooth(&self, powered: bool) {
        let dbus = self.dbus.clone();
        tokio::spawn(async move {
            dbus.bluetooth.set_powered(powered).await;
        });
    }

    pub fn update(&self, dt: f32) {
        self.selection_pill_y.write().update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_navigation_and_channel_switching() {
        let bus = EventBus::new();
        let app = SettingsApp::new(bus);

        assert_eq!(*app.current_section.read(), SettingsSection::Appearance);
        app.select_section(SettingsSection::Updates);
        assert_eq!(*app.current_section.read(), SettingsSection::Updates);

        app.set_ota_channel(OTAChannel::UpstreamOne);
        assert_eq!(app.state.read().active_channel, OTAChannel::UpstreamOne);

        app.update(0.016);
    }
}
