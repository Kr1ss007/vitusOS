//! CrashManager Subsystem (Part 21 of specification).
//!
//! Subsystem for fault detection, dependency blast radius isolation, and recovery.

pub mod crash_site;
pub mod event_handler;
pub mod first_responder;
pub mod global_feed;
pub mod handshakes;
pub mod vessels;

use std::sync::Arc;
use tracing::info;

pub use crash_site::{AppCrashRecord, CrashSite};
pub use event_handler::{CrashEventHandler, Severity};
pub use first_responder::FirstResponder;
pub use global_feed::{GlobalFeed, PressureLevel, ResourceSnapshot};
pub use handshakes::{HandshakeResult, Handshakes, SubsystemHealth};
pub use vessels::{Vessel, VesselState, Vessels};

use crate::event_bus::EventBus;

pub struct CrashManager {
    pub first_responder: Arc<FirstResponder>,
    pub global_feed: Arc<GlobalFeed>,
    pub handshakes: Arc<Handshakes>,
    pub event_handler: Arc<CrashEventHandler>,
    pub crash_site: Arc<CrashSite>,
    pub vessels: Arc<Vessels>,
}

impl CrashManager {
    pub fn new(bus: EventBus) -> Self {
        let first_responder = Arc::new(FirstResponder::new(bus.clone()));
        let global_feed = Arc::new(GlobalFeed::new(bus.clone()));
        let handshakes = Arc::new(Handshakes::new(bus.clone()));
        let event_handler = Arc::new(CrashEventHandler::new(bus.clone()));
        let crash_site = Arc::new(CrashSite::new(bus.clone()));
        let vessels = Arc::new(Vessels::new(bus));

        Self {
            first_responder,
            global_feed,
            handshakes,
            event_handler,
            crash_site,
            vessels,
        }
    }

    /// Initializes all fault-detection layers before engine start.
    pub fn initialize(&self) {
        info!("CrashManager: Initializing fault detection and isolation suite...");
        self.first_responder.initialize();
        self.global_feed.start();
        self.handshakes.start();
    }

    /// Stops all background monitoring threads.
    pub fn destroy(&self) {
        self.global_feed.stop();
        self.handshakes.stop();
        self.first_responder.destroy();
        info!("CrashManager: All fault monitors stopped.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_manager_lifecycle() {
        let bus = EventBus::new();
        let cm = CrashManager::new(bus);
        cm.initialize();
        assert_eq!(cm.vessels.state_of("Compositor"), VesselState::Running);
        cm.destroy();
    }
}
