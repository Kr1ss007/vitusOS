//! Welcome Screen & First-Boot Setup Wizard.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WizardStep {
    SecureVault,
    ChooseWallpaper,
    AllSet,
}

pub struct WelcomeScreen {
    pub current_step: WizardStep,
    pub card_scale: SpringSolver, // SPRING_SHEET (420, 30): 0.92 -> 1.0
    pub opacity: SpringSolver,    // SPRING_SELECTION (400, 28)
    pub is_active: bool,
    bus: EventBus,
}

impl WelcomeScreen {
    pub fn new(bus: EventBus) -> Self {
        Self {
            current_step: WizardStep::SecureVault,
            card_scale: SpringSolver::new(0.92, SpringProfile::Sheet),
            opacity: SpringSolver::new(0.0, SpringProfile::Selection),
            is_active: false,
            bus,
        }
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.card_scale.set_target(1.0);
        self.opacity.set_target(1.0);
    }

    pub fn next_step(&mut self) {
        match self.current_step {
            WizardStep::SecureVault => self.current_step = WizardStep::ChooseWallpaper,
            WizardStep::ChooseWallpaper => self.current_step = WizardStep::AllSet,
            WizardStep::AllSet => self.complete(),
        }
    }

    pub fn complete(&mut self) {
        self.card_scale.set_target(0.95);
        self.opacity.set_target(0.0);
        self.is_active = false;
        self.bus.publish(AEEvent::WelcomeScreenCompleted);
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_active || self.opacity.value > 0.01 {
            self.card_scale.update(dt);
            self.opacity.update(dt);
        }
    }
}
