//! CrashEventHandler — Triage Layer for Compositor, Window, and D-Bus Anomalies (Part 21.7).
//!
//! Classifies runtime anomalies into Recoverable, Degraded, or Fatal actions.

use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::event_bus::EventBus;
use crate::events::AEEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Recoverable,
    Degraded,
    Fatal,
}

pub struct CrashEventHandler {
    bus: EventBus,
}

impl CrashEventHandler {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    /// Triages D-Bus connection dropouts.
    pub fn on_dbus_connection_lost(&self, bus_name: &str) {
        let severity = Severity::Degraded;
        self.dispatch(severity, "DBusBridge", &format!("Connection lost to {}", bus_name));
    }

    /// Triages compositor rendering backend errors.
    pub fn on_compositor_error(&self, detail: &str) {
        let severity = if detail.contains("VK_ERROR_DEVICE_LOST") {
            Severity::Fatal
        } else {
            Severity::Degraded
        };
        self.dispatch(severity, "Compositor", detail);
    }

    /// Triages window state anomalies (e.g. unacknowledged configure event).
    pub fn on_window_state_anomaly(&self, app_id: &str, detail: &str) {
        self.dispatch(
            Severity::Recoverable,
            "WindowManager",
            &format!("{}: {}", app_id, detail),
        );
    }

    fn dispatch(&self, sev: Severity, source: &str, detail: &str) {
        match sev {
            Severity::Recoverable => {
                info!("CrashEventHandler [Recoverable] ({}): {}", source, detail);
            }
            Severity::Degraded => {
                warn!("CrashEventHandler [Degraded] ({}): {}", source, detail);
                self.bus.publish_async(AEEvent::SubsystemHealthChanged {
                    name: source.to_string(),
                    healthy: false,
                });
            }
            Severity::Fatal => {
                error!("CrashEventHandler [Fatal] ({}): {}", source, detail);
                self.bus.publish_async(AEEvent::ShutdownRequested);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_event_handler_triage() {
        let bus = EventBus::new();
        let handler = CrashEventHandler::new(bus);

        handler.on_window_state_anomaly("org.vitusos.filer", "Surface pending configure");
        handler.on_dbus_connection_lost("org.freedesktop.Notifications");
    }
}
