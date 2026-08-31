//! vitusOS Live ISO Installer Wizard.

use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallStep {
    Welcome,
    SelectDisk,
    ConfigureVault,
    CopyingSystem,
    Complete,
}

pub struct InstallerEngine {
    pub current_step: InstallStep,
    pub install_progress: SpringSolver, // SPRING_SELECTION (400, 28)
    pub card_scale: SpringSolver,       // SPRING_SHEET (420, 30)
}

impl Default for InstallerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallerEngine {
    pub fn new() -> Self {
        Self {
            current_step: InstallStep::Welcome,
            install_progress: SpringSolver::new(0.0, SpringProfile::Selection),
            card_scale: SpringSolver::new(1.0, SpringProfile::Sheet),
        }
    }

    pub fn next(&mut self) {
        match self.current_step {
            InstallStep::Welcome => self.current_step = InstallStep::SelectDisk,
            InstallStep::SelectDisk => self.current_step = InstallStep::ConfigureVault,
            InstallStep::ConfigureVault => {
                self.current_step = InstallStep::CopyingSystem;
                self.install_progress.set_target(1.0);
            }
            InstallStep::CopyingSystem => self.current_step = InstallStep::Complete,
            InstallStep::Complete => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.install_progress.update(dt);
        self.card_scale.update(dt);
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting vitusOS Live Installer Engine...");
    let mut installer = InstallerEngine::new();
    installer.next();
    info!("Installer initialized on step: {:?}", installer.current_step);
    Ok(())
}
