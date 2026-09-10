//! Hardware Block Device Scanner & Partition Topology Detector.

use crate::types::{DiskTransport, PartitionEntry, TargetDisk};
use std::fs;
use std::path::Path;
use tracing::info;

pub struct DiskScanner;

impl DiskScanner {
    /// Scans system block devices and active partition topologies directly from `/sys/block/` on Linux.
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

                    let mut partitions = Vec::new();

                    // Discover real partitions within this block device
                    if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                        for sub in sub_entries.flatten() {
                            let part_name = sub.file_name().to_string_lossy().to_string();
                            if part_name.starts_with(&name) && part_name != name {
                                let part_size_file = sub.path().join("size");
                                let part_sectors = fs::read_to_string(part_size_file)
                                    .unwrap_or_default()
                                    .trim()
                                    .parse::<u64>()
                                    .unwrap_or(0);
                                let part_bytes = part_sectors * 512;

                                if part_bytes > 0 {
                                    let part_dev_path = format!("/dev/{}", part_name);
                                    let mut mount_point = None;
                                    let mut filesystem = "unknown".to_string();

                                    // Check /proc/mounts for real mount point and fs
                                    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
                                        for line in mounts.lines() {
                                            let parts: Vec<&str> = line.split_whitespace().collect();
                                            if parts.len() >= 3 && parts[0] == part_dev_path {
                                                mount_point = Some(parts[1].to_string());
                                                filesystem = parts[2].to_string();
                                                break;
                                            }
                                        }
                                    }

                                    partitions.push(PartitionEntry {
                                        name: part_name,
                                        size_bytes: part_bytes,
                                        filesystem,
                                        mount_point,
                                    });
                                }
                            }
                        }
                    }

                    // Sort partitions by name (e.g. p1, p2, p3)
                    partitions.sort_by(|a, b| a.name.cmp(&b.name));

                    disks.push(TargetDisk {
                        id: name,
                        model,
                        path: dev_path,
                        size_bytes,
                        transport,
                        is_removable,
                        partitions,
                    });
                }
            }
        }

        info!("DiskScanner: Detected {} physical storage devices.", disks.len());
        disks
    }
}
