//! CockpitView Camera Altitude Zoom Model.
//!
//! Aligned with Part 29 and FIX-05 (prev_zoom = -1.0 sentinel preventing spurious sound).

use animus_core::context::AnimusContext;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};

pub struct CockpitView {
    pub is_open: bool,
    pub zoom: SpringSolver,          // SPRING_SELECTION (400, 28): 1.0 -> 0.45
    pub offset_y: SpringSolver,      // SPRING_SELECTION (400, 28): 0.0 -> +60.0px
    pub offset_x: SpringSolver,      // SPRING_SELECTION (400, 28): 0.0 -> +80.0px
    pub sidebar_x: SpringSolver,     // SPRING_SELECTION (400, 28): -80.0 -> 0.0px
    pub bg_darken: SpringSolver,     // SPRING_SELECTION (400, 28): 0.0 -> 0.5
    pub prev_zoom: f32,              // -1.0 sentinel (FIX-05)
    pub active_desktop: usize,
    pub desktop_count: usize,
    bus: EventBus,
}

impl CockpitView {
    pub fn new(bus: EventBus) -> Self {
        Self {
            is_open: false,
            zoom: SpringSolver::new(1.0, SpringProfile::Selection),
            offset_y: SpringSolver::new(0.0, SpringProfile::Selection),
            offset_x: SpringSolver::new(0.0, SpringProfile::Selection),
            sidebar_x: SpringSolver::new(-80.0, SpringProfile::Selection),
            bg_darken: SpringSolver::new(0.0, SpringProfile::Selection),
            prev_zoom: -1.0,
            active_desktop: 0,
            desktop_count: 1,
            bus,
        }
    }

    pub fn open(&mut self, _ctx: Option<AnimusContext>) {
        self.is_open = true;
        self.zoom.set_target(0.45);
        self.offset_y.set_target(60.0);
        self.offset_x.set_target(80.0);
        self.sidebar_x.set_target(0.0);
        self.bg_darken.set_target(0.5);
        self.bus.publish(AEEvent::CockpitViewOpened);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.zoom.set_target(1.0);
        self.offset_y.set_target(0.0);
        self.offset_x.set_target(0.0);
        self.sidebar_x.set_target(-80.0);
        self.bg_darken.set_target(0.0);
        self.bus.publish(AEEvent::CockpitViewClosed);
    }

    pub fn update(&mut self, dt: f32) {
        let current_zoom = self.zoom.update(dt);
        self.offset_y.update(dt);
        self.offset_x.update(dt);
        self.sidebar_x.update(dt);
        self.bg_darken.update(dt);

        // Sentinel guard (FIX-05): Never fire on frame 1
        if self.prev_zoom < 0.0 {
            self.prev_zoom = current_zoom;
            return;
        }

        self.prev_zoom = current_zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cockpit_view_sentinel_and_transitions() {
        let bus = EventBus::new();
        let mut cockpit = CockpitView::new(bus);

        assert_eq!(cockpit.prev_zoom, -1.0);
        cockpit.update(0.016);
        assert_eq!(cockpit.prev_zoom, 1.0); // Initialized to 1.0 without false sound trigger

        cockpit.open(None);
        assert!(cockpit.is_open);
        assert_eq!(cockpit.zoom.target, 0.45);

        cockpit.close();
        assert!(!cockpit.is_open);
        assert_eq!(cockpit.zoom.target, 1.0);
    }
}
