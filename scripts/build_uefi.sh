#!/usr/bin/env bash
# ==============================================================================
# vitusOS AnimusBoot Stage 0 UEFI Builder
# Compiles AnimusBoot.efi and creates standard UEFI ESP directory and disk image
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
UEFI_SRC_DIR="${ROOT_DIR}/bootloader/uefi"
TARGET_DIR="${ROOT_DIR}/target"
ESP_DIR="${TARGET_DIR}/esp"
ESP_IMG="${TARGET_DIR}/esp.img"

echo "================================================================================"
echo "                   vitusOS AnimusBoot UEFI Build Pipeline                      "
echo "================================================================================"

# 1. Verify build dependencies
if ! command -v make >/dev/null 2>&1; then
    echo "ERROR: 'make' is not installed." >&2
    exit 1
fi
if ! command -v gcc >/dev/null 2>&1 && ! command -v cc >/dev/null 2>&1; then
    echo "ERROR: C compiler ('gcc' or 'cc') is not installed." >&2
    exit 1
fi
if ! command -v objcopy >/dev/null 2>&1; then
    echo "ERROR: 'objcopy' is not installed." >&2
    exit 1
fi

# 2. Build UEFI Stage 0 bootloader
echo "Building AnimusBoot Stage 0 UEFI binary..."
make -C "${UEFI_SRC_DIR}" clean
make -C "${UEFI_SRC_DIR}"

if [ ! -f "${UEFI_SRC_DIR}/AnimusBoot.efi" ]; then
    echo "ERROR: Failed to produce ${UEFI_SRC_DIR}/AnimusBoot.efi" >&2
    exit 1
fi

echo "AnimusBoot.efi built successfully."

# 3. Assemble ESP Directory Structure (UEFI Specification Standard)
echo "Creating EFI System Partition (ESP) directory at ${ESP_DIR}..."
mkdir -p "${ESP_DIR}/EFI/BOOT"
mkdir -p "${ESP_DIR}/EFI/vitusOS"
mkdir -p "${ESP_DIR}/vitusOS"

# BOOTX64.EFI is the standard x86_64 UEFI fallback boot path
cp -f "${UEFI_SRC_DIR}/AnimusBoot.efi" "${ESP_DIR}/EFI/BOOT/BOOTX64.EFI"
cp -f "${UEFI_SRC_DIR}/AnimusBoot.efi" "${ESP_DIR}/EFI/vitusOS/AnimusBoot.efi"

# Create a sample animus_config.json in ESP if not already present
if [ ! -f "${ESP_DIR}/vitusOS/boot.cfg" ] || ! grep -q "fbcon=nodefer" "${ESP_DIR}/vitusOS/boot.cfg"; then
    cat > "${ESP_DIR}/vitusOS/boot.cfg" << 'EOF'
# vitusOS AnimusBoot Configuration (Zero-TTY, Zero-Flicker, macOS-Grade Boot)
TIMEOUT=0
DEFAULT_ENTRY=vitusos-standard
RESOLUTION=1920x1080
KERNEL_PATH=/vitusOS/vmlinuz
INITRD_PATH=/vitusOS/initrd.img
CMDLINE="quiet splash loglevel=0 logo.nologo vt.global_cursor_default=0 vt.handoff=7 fbcon=map:99 fbcon=nodefer rd.udev.log_level=3 udev.log_level=3 systemd.show_status=0 rd.systemd.show_status=0 systemd.log_level=err nvidia-drm.modeset=1 nvidia-drm.fbdev=1 i915.fastboot=1 i915.modeset=1 console=ttyS0,115200n8 acpi_osi=Linux acpi_backlight=native"
EOF
fi

# 4. Generate 64MB FAT32 ESP Disk Image for QEMU
echo "Generating FAT32 ESP disk image at ${ESP_IMG}..."
rm -f "${ESP_IMG}"
# Create 64MB sparse image
dd if=/dev/zero of="${ESP_IMG}" bs=1M count=64 status=none

# Format as FAT32
if command -v mkfs.vfat >/dev/null 2>&1; then
    mkfs.vfat -F 32 -n "VITUS_ESP" "${ESP_IMG}" >/dev/null
    
    # Populate image with mtools if available
    if command -v mmd >/dev/null 2>&1 && command -v mcopy >/dev/null 2>&1; then
        mmd -i "${ESP_IMG}" ::EFI
        mmd -i "${ESP_IMG}" ::EFI/BOOT
        mmd -i "${ESP_IMG}" ::EFI/vitusOS
        mmd -i "${ESP_IMG}" ::vitusOS
        mcopy -i "${ESP_IMG}" "${ESP_DIR}/EFI/BOOT/BOOTX64.EFI" ::EFI/BOOT/BOOTX64.EFI
        mcopy -i "${ESP_IMG}" "${ESP_DIR}/EFI/vitusOS/AnimusBoot.efi" ::EFI/vitusOS/AnimusBoot.efi
        mcopy -i "${ESP_IMG}" "${ESP_DIR}/vitusOS/boot.cfg" ::vitusOS/boot.cfg
        echo "ESP image formatted and populated successfully (FAT32)."
    else
        echo "WARNING: mtools not found; ${ESP_IMG} formatted but not populated. Using directory ESP."
    fi
else
    echo "WARNING: mkfs.vfat not found; skipping .img creation. Directory ESP at ${ESP_DIR} is ready."
fi

echo "================================================================================"
echo "SUCCESS: AnimusBoot UEFI artifacts prepared:"
echo "  - EFI Binary:    ${ESP_DIR}/EFI/BOOT/BOOTX64.EFI"
echo "  - ESP Directory: ${ESP_DIR}"
echo "  - ESP Disk Image:${ESP_IMG}"
echo "================================================================================"
