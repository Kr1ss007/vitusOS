//! Spatial Window Geometry & AnimusEngine AESurfaces UI Layout.

use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::appkit::{AEButton, ButtonVariant};
use animus_render::{GlassProperties, SurfaceAltitude};
use serde::{Deserialize, Serialize};

/// Canonical Apple-grade Floating Card Dimensions (820 × 580 px).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardCardLayout {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub altitude: SurfaceAltitude,
    pub scale_spring: SpringSolver, // SPRING_SHEET (420, 30)
    pub opacity_spring: SpringSolver, // SPRING_SELECTION (400, 28)
    pub slide_x: SpringSolver,      // SPRING_DESKTOP_SWITCH (350, 26)
}

impl Default for WizardCardLayout {
    fn default() -> Self {
        Self {
            width: 820.0,
            height: 580.0,
            corner_radius: 24.0,
            altitude: SurfaceAltitude::High,
            scale_spring: SpringSolver::new(0.94, SpringProfile::Sheet),
            opacity_spring: SpringSolver::new(0.0, SpringProfile::Selection),
            slide_x: SpringSolver::new(0.0, SpringProfile::DesktopSwitch),
        }
    }
}

impl WizardCardLayout {
    pub fn glass_properties(&self) -> GlassProperties {
        self.altitude.glass_properties()
    }

    pub fn update(&mut self, dt: f32) {
        self.scale_spring.update(dt);
        self.opacity_spring.update(dt);
        self.slide_x.update(dt);
    }
}

/// Navigation Action Controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardNavigation {
    pub back_button: AEButton,
    pub continue_button: AEButton,
    pub dots_scale: Vec<SpringSolver>, // 1 spring per step
    pub active_step_index: usize,
}

impl WizardNavigation {
    pub fn new(total_steps: usize) -> Self {
        let mut back_button = AEButton::new("Back", ButtonVariant::Secondary);
        back_button.width = 110.0;
        back_button.height = 36.0;

        let mut continue_button = AEButton::new("Continue", ButtonVariant::Primary);
        continue_button.width = 140.0;
        continue_button.height = 36.0;

        let dots_scale = (0..total_steps)
            .map(|i| {
                let target = if i == 0 { 1.25 } else { 1.0 };
                SpringSolver::new(target, SpringProfile::Hover)
            })
            .collect();

        Self {
            back_button,
            continue_button,
            dots_scale,
            active_step_index: 0,
        }
    }

    pub fn set_active_step(&mut self, index: usize) {
        self.active_step_index = index;
        for (i, dot) in self.dots_scale.iter_mut().enumerate() {
            dot.set_target(if i == index { 1.30 } else { 1.0 });
        }

        // Adjust continue button label on confirmation steps
        if index == 4 {
            self.continue_button.label = "Start Install".to_string();
        } else if index >= 5 {
            self.continue_button.label = "Finish".to_string();
        } else {
            self.continue_button.label = "Continue".to_string();
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.back_button.update(dt);
        self.continue_button.update(dt);
        for dot in &mut self.dots_scale {
            dot.update(dt);
        }
    }
}
