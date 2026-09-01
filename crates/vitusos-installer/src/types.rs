//! Canonical types and data models for the vitusOS macOS-grade Setup Assistant.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WizardStep {
    Welcome = 0,
    DiskSelect = 1,
    Account = 2,
    Vault = 3,
    Theme = 4,
    Installing = 5,
    Complete = 6,
}

impl WizardStep {
    pub const ALL: [WizardStep; 7] = [
        WizardStep::Welcome,
        WizardStep::DiskSelect,
        WizardStep::Account,
        WizardStep::Vault,
        WizardStep::Theme,
        WizardStep::Installing,
        WizardStep::Complete,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to vitusOS",
            Self::DiskSelect => "Select Destination Disk",
            Self::Account => "Create User Account",
            Self::Vault => "Hardware Encryption Vault",
            Self::Theme => "Appearance & Spatial Sound",
            Self::Installing => "Installing vitusOS",
            Self::Complete => "Setup Complete",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            Self::Welcome => "Choose your primary language and region to get started.",
            Self::DiskSelect => "Choose the storage drive where you want to install vitusOS.",
            Self::Account => "Set up your credentials and personal workspace identity.",
            Self::Vault => "Military-grade AES-256-GCM encryption backed by Argon2id & TPM 2.0.",
            Self::Theme => "Customize your desktop visual altitude and acoustics.",
            Self::Installing => "Copying system assets and configuring AnimusEngine runtime.",
            Self::Complete => "Your workspace is ready. Experience the future of desktop OS.",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskTransport {
    Nvme,
    Sata,
    Usb,
    Virtual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionEntry {
    pub name: String,
    pub size_bytes: u64,
    pub filesystem: String,
    pub mount_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDisk {
    pub id: String,
    pub model: String,
    pub path: String,
    pub size_bytes: u64,
    pub transport: DiskTransport,
    pub is_removable: bool,
    pub partitions: Vec<PartitionEntry>,
}

impl TargetDisk {
    pub fn formatted_size(&self) -> String {
        let gb = self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if gb >= 1000.0 {
            format!("{:.1} TB", gb / 1024.0)
        } else {
            format!("{:.0} GB", gb)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionStrategy {
    EraseAndInstall,
    InstallAlongside,
    ManualCustom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordStrength {
    Weak,
    Fair,
    Strong,
    Excellent,
}

impl PasswordStrength {
    pub fn score(&self) -> f32 {
        match self {
            Self::Weak => 0.25,
            Self::Fair => 0.50,
            Self::Strong => 0.75,
            Self::Excellent => 1.00,
        }
    }

    pub fn color_rgba(&self) -> [f32; 4] {
        match self {
            Self::Weak => [1.0, 0.23, 0.19, 1.0],      // Space Red #FF3B30
            Self::Fair => [1.0, 0.80, 0.0, 1.0],       // Space Yellow #FFCC00
            Self::Strong => [0.20, 0.78, 0.35, 1.0],   // Apple Green #34C759
            Self::Excellent => [0.0, 0.48, 1.0, 1.0],  // Space Blue #007AFF
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppearanceMode {
    Dark,
    ObsidianGlass,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallTelemetry {
    pub phase: String,
    pub percent: f32,
    pub speed_mb_s: f32,
    pub current_asset: String,
    pub is_finished: bool,
    pub error_msg: Option<String>,
}
