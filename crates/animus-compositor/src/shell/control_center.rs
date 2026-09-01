//! Control Center Popover Component on AESurfaces (SurfaceAltitude::High, 32px Kawase Blur).
//!
//! Provides system controls for Display Brightness, Volume, Wi-Fi, Bluetooth, Dark Mode, and Battery.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCenterState {
    pub brightness: f32, // 0.0 -> 1.0
    pub volume: f32,     // 0.0 -> 1.0
    pub is_wifi_enabled: bool,
    pub active_wifi_ssid: Option<String>,
    pub is_bluetooth_enabled: bool,
    pub is_dark_mode: bool,
    pub is_reduced_motion: bool,
    pub battery_percentage: f32,
    pub is_charging: bool,
}

pub struct ControlCenter {
    pub is_open: RwLock<bool>,
    pub altitude: SurfaceAltitude, // High (32px Kawase Blur, 72% Opacity)
    pub reveal_progress: RwLock<SpringSolver>, // SPRING_SELECTION (400, 28): 0.0 -> 1.0
    pub state: RwLock<ControlCenterState>,
    bus: EventBus,
}

impl ControlCenter {
    pub fn new(bus: EventBus) -> Self {
        let state = ControlCenterState {
            brightness: 0.85,
            volume: 0.80,
            is_wifi_enabled: true,
            active_wifi_ssid: Some("vitusOS 5G".to_string()),
            is_bluetooth_enabled: true,
            is_dark_mode: true,
            is_reduced_motion: false,
            battery_percentage: 95.0,
            is_charging: true,
        };

        Self {
            is_open: RwLock::new(false),
            altitude: SurfaceAltitude::High,
            reveal_progress: RwLock::new(SpringSolver::new(0.0, SpringProfile::Selection)),
            state: RwLock::new(state),
            bus,
        }
    }

    pub fn toggle(&self) {
        let mut open = self.is_open.write();
        *open = !*open;
        self.reveal_progress.write().set_target(if *open { 1.0 } else { 0.0 });
        info!("ControlCenter: Toggled -> open={}", *open);
    }

    pub fn set_brightness(&self, val: f32) {
        let b = val.clamp(0.0, 1.0);
        self.state.write().brightness = b;
        self.bus.publish(AEEvent::BrightnessChanged { brightness: b });
    }

    pub fn set_volume(&self, val: f32) {
        let v = val.clamp(0.0, 1.0);
        self.state.write().volume = v;
        self.bus.publish(AEEvent::VolumeChanged { volume: v, muted: false });
    }

    pub fn toggle_dark_mode(&self) {
        let mut s = self.state.write();
        s.is_dark_mode = !s.is_dark_mode;
        info!("ControlCenter: Toggled dark mode -> {}", s.is_dark_mode);
    }

    pub fn toggle_reduced_motion(&self) {
        let mut s = self.state.write();
        s.is_reduced_motion = !s.is_reduced_motion;
        let enabled = s.is_reduced_motion;
        self.bus.publish(AEEvent::ReducedMotionChanged { enabled });
        info!("ControlCenter: Toggled reduced motion -> {}", enabled);
    }

    pub fn toggle_wifi(&self) {
        let mut s = self.state.write();
        s.is_wifi_enabled = !s.is_wifi_enabled;
        let enabled = s.is_wifi_enabled;
        info!("ControlCenter: Toggled Wi-Fi -> {}", enabled);
        let dbus = animus_core::dbus::SystemDbusManager::new();
        tokio::spawn(async move {
            dbus.network.set_wifi_enabled(enabled).await;
        });
    }

    pub fn toggle_bluetooth(&self) {
        let mut s = self.state.write();
        s.is_bluetooth_enabled = !s.is_bluetooth_enabled;
        let enabled = s.is_bluetooth_enabled;
        info!("ControlCenter: Toggled Bluetooth -> {}", enabled);
        let dbus = animus_core::dbus::SystemDbusManager::new();
        tokio::spawn(async move {
            dbus.bluetooth.set_powered(enabled).await;
        });
    }

    pub fn update(&self, dt: f32) {
        self.reveal_progress.write().update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_center_state_and_toggle() {
        let bus = EventBus::new();
        let cc = ControlCenter::new(bus);

        assert!(!*cc.is_open.read());
        cc.toggle();
        assert!(*cc.is_open.read());
        assert_eq!(cc.reveal_progress.read().target, 1.0);

        cc.set_brightness(0.5);
        assert_eq!(cc.state.read().brightness, 0.5);

        cc.toggle_reduced_motion();
        assert!(cc.state.read().is_reduced_motion);
    }
}
