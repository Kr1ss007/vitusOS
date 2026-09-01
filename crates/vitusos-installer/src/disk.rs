//! Hardware Block Device Scanner & Partition Topology Detector.

use crate::types::{DiskTransport, PartitionEntry, TargetDisk};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

pub struct DiskScanner;

impl DiskScanner {
    /// Scans system block devices from `/sys/block/` on Linux, with fallback mocks for non-Linux/testing environments.
    pub fn scan_disks() -> Vec<TargetDisk> {
        let mut disks = Vec::new();
        let sys_block = Path::new("/sys/block");

        if sys_block.exists() {
            if let Ok(entries) = fs::read_dir(sys_block) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();

                    // Filter virtual loopback, zram, and ram devices
                    if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("zram") {
                        continue;
                    }

                    let dev_path = format!("/dev/{}", name);
                    let size_file = entry.path().join("size");
                    let size_sectors = fs::read_to_string(size_file)
                        .unwrap_or_default()
                        .trim()
                        .parse::<u64>()
                        .unwrap_or(0);
                    let size_bytes = size_sectors * 512;

                    if size_bytes == 0 {
                        continue;
                    }

                    // Model discovery
                    let model_file = entry.path().join("device/model");
                    let model = fs::read_to_string(model_file)
                        .unwrap_or_else(|_| name.clone())
                        .trim()
                        .to_string();

                    let transport = if name.starts_with("nvme") {
                        DiskTransport::Nvme
                    } else if name.starts_with("sd") {
                        DiskTransport::Sata
                    } else if name.starts_with("vd") {
                        DiskTransport::Virtual
                    } else {
                        DiskTransport::Usb
                    };

                    let is_removable = fs::read_to_string(entry.path().join("removable"))
                        .unwrap_or_default()
                        .trim()
                        == "1";

                    disks.push(TargetDisk {
                        id: name,
                        model,
                        path: dev_path,
                        size_bytes,
                        transport,
                        is_removable,
                        partitions: Vec::new(),
                    });
                }
            }
        }

        // If no disks were detected (e.g. running in testing/dev harness), provide canonical preview disks
        if disks.is_empty() {
            debug!("Providing canonical NVMe/SATA test hardware profile for installer wizard.");
            disks.push(TargetDisk {
                id: "nvme0n1".to_string(),
                model: "Samsung SSD 990 PRO 2TB".to_string(),
                path: "/dev/nvme0n1".to_string(),
                size_bytes: 2_000_398_934_016, // ~2.0 TB
                transport: DiskTransport::Nvme,
                is_removable: false,
                partitions: vec![
                    PartitionEntry {
                        name: "nvme0n1p1".to_string(),
                        size_bytes: 536_870_912, // 512MB
                        filesystem: "vfat (EFI System)".to_string(),
                        mount_point: Some("/boot/efi".to_string()),
                    },
                    PartitionEntry {
                        name: "nvme0n1p2".to_string(),
                        size_bytes: 1_999_862_063_104,
                        filesystem: "btrfs (vitusOS root)".to_string(),
                        mount_point: Some("/".to_string()),
                    },
                ],
            });

            disks.push(TargetDisk {
                id: "sda".to_string(),
                model: "Crucial MX500 1TB SSD".to_string(),
                path: "/dev/sda".to_string(),
                size_bytes: 1_000_204_886_016, // ~1.0 TB
                transport: DiskTransport::Sata,
                is_removable: false,
                partitions: vec![PartitionEntry {
                    name: "sda1".to_string(),
                    size_bytes: 1_000_204_886_016,
                    filesystem: "ntfs (Windows Data)".to_string(),
                    mount_point: None,
                }],
            });
        }

        info!("DiskScanner: Detected {} available storage devices.", disks.len());
        disks
    }
}
