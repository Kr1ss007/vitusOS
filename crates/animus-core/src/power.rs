use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::event_bus::EventBus;
use crate::events::AEEvent;

/// PowerManager tracks system power state, sleep inhibitors, and battery milestones.
pub struct PowerManager {
    bus: EventBus,
    sleep_inhibited: Arc<AtomicBool>,
}

impl PowerManager {
    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            sleep_inhibited: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Sets or clears sleep inhibition (e.g., during live presentation, media playback, or ISO install).
    pub fn set_sleep_inhibited(&self, inhibited: bool) {
        self.sleep_inhibited.store(inhibited, Ordering::SeqCst);
    }

    pub fn is_sleep_inhibited(&self) -> bool {
        self.sleep_inhibited.load(Ordering::SeqCst)
    }

    /// Simulates/processes battery level telemetry update.
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
