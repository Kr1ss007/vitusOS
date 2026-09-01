//! Bare-Metal Boot Handoff (Part 1 & Part 2 of specification).
//!
//! Defines `ANIMUS_GPU_HANDOFF` matching `AnimusHandoff.h` shared by:
//! - Stage 0: AnimusBoot (UEFI EFI App)
//! - Stage 1: Linux Kernel (DRM_SIMPLEDRM=y)
//! - Stage 2: animus-early (initramfs service)
//! - Stage 3: AnimusEngine (userspace display compositor)

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

pub const ANIMUS_HANDOFF_GUID_STR: &str = "e4b8e798-a5f4-4b2c-b9ab-1234567890ab";
pub const EFIVARS_PATH: &str = "/sys/firmware/efi/efivars";

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Unknown = 0,
    Nvidia = 1,
    Amd = 2,
    IntelLegacy = 3, // i915
    IntelArc = 4,    // xe driver (DID 0x5690 - 0x57FF)
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuType {
    Unknown = 0,
    Discrete = 1,
    Integrated = 2,
}

/// Binary EFI Handoff layout matching C11 `ANIMUS_GPU_HANDOFF` (Part 1.1).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimusGpuHandoff {
    pub vendor: GpuVendor,
    pub gpu_type: GpuType,
    pub device_id: u16,
    pub bus_number: u8,
    pub padding: u8,
    pub framebuffer_base: u64,
    pub framebuffer_size: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixels_per_scanline: u32,
    pub pixel_format: u32,
}

impl Default for AnimusGpuHandoff {
    fn default() -> Self {
        Self {
            vendor: GpuVendor::IntelLegacy,
            gpu_type: GpuType::Integrated,
            device_id: 0x46A6, // Intel UHD Graphics
            bus_number: 0,
            padding: 0,
            framebuffer_base: 0x80000000,
            framebuffer_size: 1920 * 1080 * 4,
            horizontal_resolution: 1920,
            vertical_resolution: 1080,
            pixels_per_scanline: 1920,
            pixel_format: 1, // PixelBlueGreenRedReserved8BitPerColor
        }
    }
}

impl AnimusGpuHandoff {
    /// Detects the target Linux DRM kernel driver based on the GPU handoff (Part 2.2).
    pub fn target_driver(&self) -> &'static str {
        match self.vendor {
            GpuVendor::Nvidia => "nvidia_drm",
            GpuVendor::Amd => "amdgpu",
            GpuVendor::IntelArc => "xe",
            GpuVendor::IntelLegacy => "i915",
            GpuVendor::Unknown => "simpledrm",
        }
    }

    /// Kernel boot parameters required for zero-flicker handoff into Stage 3.
    pub fn kernel_cmdline_args(&self) -> &'static str {
        match self.vendor {
            GpuVendor::Nvidia => "nvidia_drm.modeset=1 nvidia_drm.fbdev=1",
            GpuVendor::IntelArc => "i915.force_probe=!* xe.force_probe=* video=efifb:off",
            GpuVendor::IntelLegacy => "i915.enable_guc=3 i915.fastboot=1",
            GpuVendor::Amd => "amdgpu.modeset=1 amdgpu.seamless=1",
            GpuVendor::Unknown => "drm.modeset=1",
        }
    }

    /// Reads `ANIMUS_GPU_HANDOFF` from `/sys/firmware/efi/efivars/AnimusGpuHandoff-...`
    pub fn read_from_efivars() -> Option<Self> {
        let efi_var_filename = format!("AnimusGpuHandoff-{}", ANIMUS_HANDOFF_GUID_STR);
        let path = Path::new(EFIVARS_PATH).join(efi_var_filename);

        if !path.exists() {
            info!("Handoff: No EFI variable found at {:?}. Using default bare-metal topology.", path);
            return None;
        }

        match std::fs::read(&path) {
            Ok(bytes) => {
                // First 4 bytes of efivar are attributes (EFI_VARIABLE_NON_VOLATILE, etc.)
                let data = if bytes.len() >= 4 + std::mem::size_of::<AnimusGpuHandoff>() {
                    &bytes[4..]
                } else if bytes.len() >= std::mem::size_of::<AnimusGpuHandoff>() {
                    &bytes[..]
                } else {
                    warn!("Handoff: EFI variable payload too short ({} bytes)", bytes.len());
                    return None;
                };

                unsafe {
                    let handoff_ptr = data.as_ptr() as *const AnimusGpuHandoff;
                    let handoff = std::ptr::read_unaligned(handoff_ptr);
                    info!(
                        "Handoff: Successfully restored Stage 0 EFI handoff: {:?} {}x{} driver '{}'",
                        handoff.vendor, handoff.horizontal_resolution, handoff.vertical_resolution, handoff.target_driver()
                    );
                    Some(handoff)
                }
            }
            Err(e) => {
                warn!("Handoff: Failed to read EFI variable: {}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_handoff_driver_mapping() {
        let mut h = AnimusGpuHandoff::default();
        assert_eq!(h.target_driver(), "i915");

        h.vendor = GpuVendor::IntelArc;
        assert_eq!(h.target_driver(), "xe");

        h.vendor = GpuVendor::Nvidia;
        assert_eq!(h.target_driver(), "nvidia_drm");

        h.vendor = GpuVendor::Amd;
        assert_eq!(h.target_driver(), "amdgpu");
    }

    #[test]
    fn test_gpu_handoff_binary_size() {
        assert_eq!(std::mem::size_of::<AnimusGpuHandoff>(), 48);
    }
}
