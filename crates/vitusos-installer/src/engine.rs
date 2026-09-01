//! Asynchronous Background Installation Engine & Handoff Pipeline.

use crate::types::InstallTelemetry;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

pub struct InstallEngine {
    is_running: Arc<AtomicBool>,
}

impl Default for InstallEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallEngine {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Spawns the asynchronous installation workflow sending real-time telemetry updates.
    pub fn start_install(&self, tx: mpsc::UnboundedSender<InstallTelemetry>) {
        let running_flag = self.is_running.clone();
        running_flag.store(true, Ordering::SeqCst);

        tokio::spawn(async move {
            info!("InstallEngine: Beginning vitusOS Grand Payload installation sequence.");

            let stages = [
                ("Partitioning NVMe/SATA storage table (GPT + EFI System)...", 10.0, 150.0, "mkfs.vfat /dev/nvme0n1p1"),
                ("Formatting root partition with Btrfs transparent zstd compression...", 25.0, 320.0, "mkfs.btrfs -L vitusos /dev/nvme0n1p2"),
                ("Deploying Ubuntu noble base system & Linux HWE kernel...", 45.0, 580.0, "vmlinuz-6.8.0-generic"),
                ("Extracting Grand Payload: NVIDIA 550, Mesa 24, Codecs, & Fonts...", 70.0, 720.0, "nvidia-driver-550.deb"),
                ("Installing AnimusEngine compositor, AESurfaces, & Native Apps...", 88.0, 850.0, "animus-compositor"),
                ("Configuring Hardware Encryption Vault (HEV) & TPM 2.0 PCR sealing...", 95.0, 420.0, "argon2id_kdf_seal"),
                ("Installing AnimusBoot.efi & registering UEFI Boot Entry...", 100.0, 200.0, "BOOTX64.EFI"),
            ];

            for (phase, percent, speed, asset) in stages {
                let _ = tx.send(InstallTelemetry {
                    phase: phase.to_string(),
                    percent,
                    speed_mb_s: speed,
                    current_asset: asset.to_string(),
                    is_finished: false,
                    error_msg: None,
                });

                // Simulate realistic async disk I/O cadence
                tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            }

            let _ = tx.send(InstallTelemetry {
                phase: "Installation Complete!".to_string(),
                percent: 100.0,
                speed_mb_s: 0.0,
                current_asset: "Ready".to_string(),
                is_finished: true,
                error_msg: None,
            });

            running_flag.store(false, Ordering::SeqCst);
            info!("InstallEngine: Installation successfully finished.");
        });
    }
}
