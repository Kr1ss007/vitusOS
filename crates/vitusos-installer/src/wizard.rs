//! Main Setup Assistant State Machine & Spring-Driven Motion Orchestrator.

use crate::account::{AccountProfile, PasswordEvaluator};
use crate::disk::DiskScanner;
use crate::engine::InstallEngine;
use crate::types::{AppearanceMode, InstallTelemetry, PartitionStrategy, TargetDisk, WizardStep};
use crate::ui::{WizardCardLayout, WizardNavigation};
use crate::vault::VaultSetup;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use tokio::sync::mpsc;
use tracing::info;

pub struct SetupWizard {
    pub current_step: WizardStep,
    pub card: WizardCardLayout,
    pub nav: WizardNavigation,
    pub available_disks: Vec<TargetDisk>,
    pub selected_disk_index: usize,
    pub partition_strategy: PartitionStrategy,
    pub account: AccountProfile,
    pub password_input: String,
    pub vault: VaultSetup,
    pub appearance: AppearanceMode,
    pub enable_chime: bool,
    pub telemetry: InstallTelemetry,
    pub is_active: bool,
    bus: EventBus,
    engine: InstallEngine,
    telemetry_rx: Option<mpsc::UnboundedReceiver<InstallTelemetry>>,
}

impl SetupWizard {
    pub fn new(bus: EventBus) -> Self {
        let disks = DiskScanner::scan_disks();
        let total_steps = WizardStep::ALL.len();

        Self {
            current_step: WizardStep::Welcome,
            card: WizardCardLayout::default(),
            nav: WizardNavigation::new(total_steps),
            available_disks: disks,
            selected_disk_index: 0,
            partition_strategy: PartitionStrategy::EraseAndInstall,
            account: AccountProfile::default(),
            password_input: String::new(),
            vault: VaultSetup::default(),
            appearance: AppearanceMode::ObsidianGlass,
            enable_chime: true,
            telemetry: InstallTelemetry {
                phase: "Preparing installation...".to_string(),
                percent: 0.0,
                speed_mb_s: 0.0,
                current_asset: String::new(),
                is_finished: false,
                error_msg: None,
            },
            is_active: true,
            bus,
            engine: InstallEngine::new(),
            telemetry_rx: None,
        }
    }

    /// Activates the setup assistant, smoothly springing into view.
    pub fn activate(&mut self) {
        self.is_active = true;
        self.card.scale_spring.set_target(1.0);
        self.card.opacity_spring.set_target(1.0);
        self.nav.set_active_step(self.current_step as usize);
        info!("SetupWizard: Activated macOS-grade setup assistant.");
    }

    /// Advances forward to the next step with right-to-left spring carousel animation.
    pub fn advance(&mut self) {
        let current_idx = self.current_step as usize;
        if current_idx + 1 < WizardStep::ALL.len() {
            let next_step = WizardStep::ALL[current_idx + 1];
            self.current_step = next_step;
            self.nav.set_active_step(next_step as usize);

            // Forward spring velocity injection (-600px/s)
            self.card.slide_x.set_target(0.0);
            self.card.slide_x.velocity = -600.0;

            if next_step == WizardStep::Installing && self.telemetry_rx.is_none() {
                let (tx, rx) = mpsc::unbounded_channel();
                self.telemetry_rx = Some(rx);
                self.engine.start_install(tx);
            }

            info!("SetupWizard: Advanced to step: {:?}", self.current_step);
        } else if self.current_step == WizardStep::Complete {
            self.complete_and_handoff();
        }
    }

    /// Retreats backward to previous step with left-to-right spring carousel animation.
    pub fn retreat(&mut self) {
        let current_idx = self.current_step as usize;
        if current_idx > 0 && self.current_step != WizardStep::Installing && self.current_step != WizardStep::Complete {
            let prev_step = WizardStep::ALL[current_idx - 1];
            self.current_step = prev_step;
            self.nav.set_active_step(prev_step as usize);

            // Backward spring velocity injection (+600px/s)
            self.card.slide_x.set_target(0.0);
            self.card.slide_x.velocity = 600.0;

            info!("SetupWizard: Returned to step: {:?}", self.current_step);
        }
    }

    /// Completes the installer and smoothly hands off to the live desktop session.
    pub fn complete_and_handoff(&mut self) {
        self.card.scale_spring.set_target(1.06);
        self.card.opacity_spring.set_target(0.0);
        self.is_active = false;
        self.bus.publish(AEEvent::WelcomeScreenCompleted);
        info!("SetupWizard: Setup completed. Handoff to desktop session initiated.");
    }

    /// Updates physics solvers and processes asynchronous telemetry events.
    pub fn update(&mut self, dt: f32) {
        self.card.update(dt);
        self.nav.update(dt);

        let mut should_advance = false;
        // Drain background installation progress
        if let Some(ref mut rx) = self.telemetry_rx {
            while let Ok(msg) = rx.try_recv() {
                if msg.is_finished {
                    should_advance = true;
                }
                self.telemetry = msg;
            }
        }

        if should_advance {
            self.advance();
        }
    }

    /// Computes password strength score for current input.
    pub fn password_strength(&self) -> crate::types::PasswordStrength {
        PasswordEvaluator::evaluate(&self.password_input)
    }
}
