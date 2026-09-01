//! vitusOS macOS-grade Setup Assistant & Bare-Metal Installation Wizard.
//!
//! Provides a spatial, Apple-grade setup experience driven by AnimusEngine AESurfaces,
//! SpringSolver physics, military-grade HEV disk encryption, and zero-flicker handoffs.

pub mod account;
pub mod disk;
pub mod engine;
pub mod types;
pub mod ui;
pub mod vault;
pub mod wizard;

pub use account::{AccountProfile, PasswordEvaluator};
pub use disk::DiskScanner;
pub use engine::InstallEngine;
pub use types::{AppearanceMode, DiskTransport, InstallTelemetry, PartitionStrategy, PasswordStrength, TargetDisk, WizardStep};
pub use ui::{WizardCardLayout, WizardNavigation};
pub use vault::VaultSetup;
pub use wizard::SetupWizard;

#[cfg(test)]
mod tests {
    use super::*;
    use animus_core::event_bus::EventBus;

    #[test]
    fn test_wizard_step_lifecycle() {
        let bus = EventBus::new();
        let mut wizard = SetupWizard::new(bus);

        assert_eq!(wizard.current_step, WizardStep::Welcome);
        wizard.activate();
        assert!(wizard.is_active);

        // Step through to Account
        wizard.advance();
        assert_eq!(wizard.current_step, WizardStep::DiskSelect);

        wizard.advance();
        assert_eq!(wizard.current_step, WizardStep::Account);

        // Step backward to DiskSelect
        wizard.retreat();
        assert_eq!(wizard.current_step, WizardStep::DiskSelect);
    }

    #[test]
    fn test_password_strength_and_username_derivation() {
        assert_eq!(PasswordEvaluator::evaluate(""), PasswordStrength::Weak);
        assert_eq!(PasswordEvaluator::evaluate("secret"), PasswordStrength::Weak);
        assert_eq!(PasswordEvaluator::evaluate("Secret1234"), PasswordStrength::Fair);
        assert_eq!(PasswordEvaluator::evaluate("VitusOS!2026Master"), PasswordStrength::Excellent);

        assert_eq!(PasswordEvaluator::derive_username("Alan Turing"), "aturing");
        assert_eq!(PasswordEvaluator::derive_username("Ada Lovelace"), "alovelace");
        assert_eq!(PasswordEvaluator::derive_username("Vitus"), "vitus");
    }

    #[test]
    fn test_recovery_key_format_and_hev() {
        let key = VaultSetup::generate_recovery_key();
        assert!(key.starts_with("VITUS-"));
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), 5);
        for part in parts {
            assert_eq!(part.len(), 5);
        }

        let kdf_result = VaultSetup::test_derive_key("SuperSecretKey2026!");
        assert!(kdf_result.is_ok());
    }

    #[test]
    fn test_disk_scanner_and_formatting() {
        let disks = DiskScanner::scan_disks();
        assert!(!disks.is_empty());
        let disk = &disks[0];
        assert!(!disk.model.is_empty());
        assert!(!disk.formatted_size().is_empty());
    }
}

