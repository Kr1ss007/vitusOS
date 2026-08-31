//! Stage 0/2 Boot Screen & Crossfade Engine.
//!
//! Renders "victusOS" wordmark with real smooth progress bar beneath it,
//! transitioning cleanly into the setup wizard or desktop space.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootProgressBar {
    pub progress: SpringSolver, // SPRING_SELECTION (400, 28)
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub offset_y: f32,
}

impl Default for BootProgressBar {
    fn default() -> Self {
        Self {
            progress: SpringSolver::new(0.0, SpringProfile::Selection),
            width: 240.0,
            height: 4.0,
            corner_radius: 2.0,
            offset_y: 28.0,
        }
    }
}

pub struct BootCrossfade {
    pub wordmark_text: &'static str,
    pub progress_bar: BootProgressBar,
    pub desktop_scale: SpringSolver,   // SPRING_BOOT (200, 22): 1.02 -> 1.0
    pub screen_opacity: SpringSolver,  // SPRING_BOOT (200, 22): 1.0 -> 0.0
    pub is_complete: bool,
    bus: EventBus,
}

impl BootCrossfade {
    pub fn new(bus: EventBus) -> Self {
        Self {
            wordmark_text: "victusOS",
            progress_bar: BootProgressBar::default(),
            desktop_scale: SpringSolver::new(1.02, SpringProfile::Boot),
            screen_opacity: SpringSolver::new(1.0, SpringProfile::Boot),
            is_complete: false,
            bus,
        }
    }

    /// Sets initialization milestone (0.0 to 1.0).
    pub fn set_progress(&mut self, progress: f32) {
        self.progress_bar.progress.set_target(progress.clamp(0.0, 1.0));
    }

    /// Begins crossfade into the desktop / wizard.
    pub fn begin_fade(&mut self) {
        self.desktop_scale.set_target(1.0);
        self.screen_opacity.set_target(0.0);
    }

    /// Ticks boot animations.
    pub fn update(&mut self, dt: f32) {
        self.progress_bar.progress.update(dt);
        self.desktop_scale.update(dt);
        self.screen_opacity.update(dt);

        if !self.is_complete && self.screen_opacity.value <= 0.005 {
            self.is_complete = true;
            self.bus.publish(AEEvent::BootCrossfadeComplete);
        }
    }
}
