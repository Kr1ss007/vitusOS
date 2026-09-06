//! PowerManager -- logind, Idle Timer, Battery, DPMS (Part 34 of spec).
//!
//! Three components:
//! - logind D-Bus signals (lid close, suspend events from system)
//! - PowerManager (compositor-side, reacts to signals + idle timer)
//! - Settings -> Power (user configuration)
//!
//! Locked decisions:
//!     Lid close:    LockScreen only (user configurable)
//!     Idle timeout: LockScreen after 5 minutes (user configurable)
//!     Sleep:        manual only (OrangeBoxMenu or Settings)
//!     Resume:       LockScreen always shown until auth
//!
//! On suspend: LockScreen activated + HEV locked BEFORE suspend completes.
//! No hibernate support (suspend-to-RAM only).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::event_bus::EventBus;
use crate::events::{AEEvent, NotificationPayload};

/// Action to take when the laptop lid closes (Part 34.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LidCloseAction {
    /// LockScreen (default)
    LockScreen = 0,
    /// Suspend to RAM
    Sleep = 1,
    /// Do nothing
    Nothing = 2,
}

impl Default for LidCloseAction {
    fn default() -> Self { Self::LockScreen }
}

pub struct PowerManager {
    bus: EventBus,
    sleep_inhibited: Arc<AtomicBool>,

    // Settings (user configurable via Settings -> Power)
    lid_action: RwLock<LidCloseAction>,
    idle_timeout_min: RwLock<i32>,    // 0 = never
    display_sleep_min: RwLock<i32>,   // 0 = never

    // Idle timer state
    idle_elapsed_s: RwLock<f32>,
    display_elapsed_s: RwLock<f32>,
    display_asleep: RwLock<bool>,
    suspending: RwLock<bool>,

    // Battery state
    last_battery_level: RwLock<f32>,
    battery_low_fired: RwLock<bool>,
    battery_crit_fired: RwLock<bool>,
}

impl PowerManager {
    pub const BATTERY_LOW_THRESHOLD: f32 = 0.20;      // 20%
    pub const BATTERY_CRITICAL_THRESHOLD: f32 = 0.05;  // 5%

    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            sleep_inhibited: Arc::new(AtomicBool::new(false)),
            lid_action: RwLock::new(LidCloseAction::default()),
            idle_timeout_min: RwLock::new(5),
            display_sleep_min: RwLock::new(2),
            idle_elapsed_s: RwLock::new(0.0),
            display_elapsed_s: RwLock::new(0.0),
            display_asleep: RwLock::new(false),
            suspending: RwLock::new(false),
            last_battery_level: RwLock::new(1.0),
            battery_low_fired: RwLock::new(false),
            battery_crit_fired: RwLock::new(false),
        }
    }

    /// Called every compositor frame with dt (Part 34.3).
    pub fn tick(&self, dt: f32) {
        if *self.suspending.read() { return; }

        *self.idle_elapsed_s.write() += dt;
        *self.display_elapsed_s.write() += dt;

        let display_sleep_min = *self.display_sleep_min.read();
        if !*self.display_asleep.read() && display_sleep_min > 0 {
            if *self.display_elapsed_s.read() >= display_sleep_min as f32 * 60.0 {
                *self.display_asleep.write() = true;
                self.bus.publish(AEEvent::DisplaySleep);
                info!("PowerManager: Display sleep (DPMS off) after {}min idle", display_sleep_min);
            }
        }

        let idle_timeout_min = *self.idle_timeout_min.read();
        if idle_timeout_min > 0 {
            if *self.idle_elapsed_s.read() >= idle_timeout_min as f32 * 60.0 {
                *self.idle_elapsed_s.write() = 0.0;
                self.bus.publish(AEEvent::LockScreenActivate);
                info!("PowerManager: LockScreen activated after {}min idle", idle_timeout_min);
            }
        }
    }

    /// Input event resets idle timer -- called from InputRouter (Part 34.3).
    pub fn on_input_event(&self) {
        *self.idle_elapsed_s.write() = 0.0;
        *self.display_elapsed_s.write() = 0.0;

        if *self.display_asleep.read() {
            *self.display_asleep.write() = false;
            self.bus.publish(AEEvent::DisplayWake);
            info!("PowerManager: Display wake (DPMS on) on input");
        }
    }

    /// logind lid close signal handler (Part 34.3).
    /// Called via publish_async -- already on compositor thread.
    pub fn on_lid_closed(&self) {
        self.bus.publish(AEEvent::LidClosed);
        match *self.lid_action.read() {
            LidCloseAction::LockScreen => {
                self.bus.publish(AEEvent::LockScreenActivate);
                info!("PowerManager: Lid closed -> LockScreen");
            }
            LidCloseAction::Sleep => {
                self.bus.publish(AEEvent::SystemSleep);
                info!("PowerManager: Lid closed -> Sleep");
            }
            LidCloseAction::Nothing => {}
        }
    }

    /// logind PrepareForSleep signal (Part 34.3).
    /// On suspend: LockScreen + HEV lock BEFORE suspend completes.
    pub fn on_prepare_for_sleep(&self, suspending: bool) {
        if suspending {
            *self.suspending.write() = true;
            self.bus.publish(AEEvent::LockScreenActivate);
            self.bus.publish(AEEvent::HEVLocked);
            info!("PowerManager: System suspending -- LockScreen + HEV locked");
        } else {
            *self.suspending.write() = false;
            info!("PowerManager: System resuming -- LockScreen active, awaiting auth");
        }
    }

    /// Battery level changed from UPower (Part 34.3).
    /// Level is 0.0-1.0 (not 0-100).
    pub fn on_battery_level_changed(&self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        *self.last_battery_level.write() = level;

        let percentage = level * 100.0;
        let is_charging = false; // UPower provides this separately
        self.bus.publish(AEEvent::BatteryLevelChanged {
            percentage,
            is_charging,
        });

        if !*self.battery_low_fired.read() && level <= Self::BATTERY_LOW_THRESHOLD {
            *self.battery_low_fired.write() = true;
            self.bus.publish_async(AEEvent::NotificationPosted(NotificationPayload {
                title: "Low Battery".to_string(),
                body: "20% remaining. Connect a charger.".to_string(),
                timeout_ms: 8000,
                is_persistent: false,
                ..Default::default()
            }));
            warn!("PowerManager: Battery low (20%)");
        }

        if !*self.battery_crit_fired.read() && level <= Self::BATTERY_CRITICAL_THRESHOLD {
            *self.battery_crit_fired.write() = true;
            self.bus.publish_async(AEEvent::NotificationPosted(NotificationPayload {
                title: "Critical Battery".to_string(),
                body: "5% remaining. Save your work now.".to_string(),
                timeout_ms: -1,
                is_persistent: true,
                ..Default::default()
            }));
            self.bus.publish(AEEvent::BatteryCritical);
            warn!("PowerManager: Battery critical (5%)");
        }

        if level > Self::BATTERY_LOW_THRESHOLD + 0.05 {
            *self.battery_low_fired.write() = false;
            *self.battery_crit_fired.write() = false;
        }
    }

    // -- Settings interface --

    pub fn set_lid_close_action(&self, action: LidCloseAction) {
        *self.lid_action.write() = action;
    }

    pub fn set_idle_timeout_minutes(&self, minutes: i32) {
        *self.idle_timeout_min.write() = minutes;
    }

    pub fn set_display_sleep_minutes(&self, minutes: i32) {
        *self.display_sleep_min.write() = minutes;
    }

    pub fn lid_close_action(&self) -> LidCloseAction { *self.lid_action.read() }
    pub fn idle_timeout_minutes(&self) -> i32 { *self.idle_timeout_min.read() }
    pub fn display_sleep_minutes(&self) -> i32 { *self.display_sleep_min.read() }

    // -- Sleep inhibition --

    pub fn set_sleep_inhibited(&self, inhibited: bool) {
        self.sleep_inhibited.store(inhibited, Ordering::SeqCst);
    }

    pub fn is_sleep_inhibited(&self) -> bool {
        self.sleep_inhibited.load(Ordering::SeqCst)
    }

    // -- Legacy battery update (percentage-based, for backward compat) --

    pub fn update_battery_status(&self, percentage: f32, is_charging: bool) {
        let pct = percentage.clamp(0.0, 100.0);
        self.bus.publish(AEEvent::BatteryLevelChanged {
            percentage: pct,
            is_charging,
        });

        if pct <= 5.0 && !is_charging {
            self.bus.publish(AEEvent::BatteryCritical);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_manager_idle_timeout() {
        let bus = EventBus::new();
        let pm = PowerManager::new(bus.clone());

        // Default: 5 minute idle timeout
        assert_eq!(pm.idle_timeout_minutes(), 5);
        assert_eq!(pm.display_sleep_minutes(), 2);

        // Tick for 5 minutes -> should trigger LockScreen
        pm.tick(5.0 * 60.0 + 0.1);

        // Idle timer was reset after firing
        assert_eq!(*pm.idle_elapsed_s.read(), 0.0);
    }

    #[test]
    fn test_power_manager_display_sleep() {
        let bus = EventBus::new();
        let pm = PowerManager::new(bus.clone());

        // Tick for 2 minutes -> display should sleep
        pm.tick(2.0 * 60.0 + 0.1);
        assert!(*pm.display_asleep.read());

        // Input event wakes display
        pm.on_input_event();
        assert!(!*pm.display_asleep.read());
    }

    #[test]
    fn test_power_manager_lid_close() {
        let bus = EventBus::new();
        let pm = PowerManager::new(bus.clone());

        // Default: LockScreen
        assert_eq!(pm.lid_close_action(), LidCloseAction::LockScreen);

        // Change to Sleep
        pm.set_lid_close_action(LidCloseAction::Sleep);
        assert_eq!(pm.lid_close_action(), LidCloseAction::Sleep);

        // Change to Nothing
        pm.set_lid_close_action(LidCloseAction::Nothing);
        pm.on_lid_closed(); // Should do nothing
    }

    #[test]
    fn test_power_manager_battery_thresholds() {
        let bus = EventBus::new();
        let pm = PowerManager::new(bus.clone());

        // Normal level
        pm.on_battery_level_changed(0.8);
        assert!(!*pm.battery_low_fired.read());

        // Low battery (20%)
        pm.on_battery_level_changed(0.20);
        assert!(*pm.battery_low_fired.read());

        // Critical battery (5%)
        pm.on_battery_level_changed(0.05);
        assert!(*pm.battery_crit_fired.read());

        // Charging resets flags
        pm.on_battery_level_changed(0.30);
        assert!(!*pm.battery_low_fired.read());
        assert!(!*pm.battery_crit_fired.read());
    }

    #[test]
    fn test_power_manager_suspend() {
        let bus = EventBus::new();
        let pm = PowerManager::new(bus.clone());

        pm.on_prepare_for_sleep(true);
        assert!(*pm.suspending.read());

        // While suspending, tick should be no-op
        let idle_before = *pm.idle_elapsed_s.read();
        pm.tick(1.0);
        assert_eq!(*pm.idle_elapsed_s.read(), idle_before);

        pm.on_prepare_for_sleep(false);
        assert!(!*pm.suspending.read());
    }
}
