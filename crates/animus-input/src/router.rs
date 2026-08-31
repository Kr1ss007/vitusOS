//! Global Input Router.

use crate::motion_wave::MotionWave;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct InputRouter {
    pub motion_wave: Arc<RwLock<MotionWave>>,
    bus: EventBus,
}

impl InputRouter {
    pub fn new(bus: EventBus) -> Self {
        let motion_wave = Arc::new(RwLock::new(MotionWave::new(bus.clone())));
        Self { motion_wave, bus }
    }

    pub fn handle_key(&self, keycode: u32, pressed: bool) {
        // Alt-Tab intercept: 133 = Super/Alt, 23 = Tab
        if keycode == 23 && pressed {
            self.bus.publish(AEEvent::CockpitViewOpened);
        }
    }
}
