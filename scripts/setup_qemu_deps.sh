#!/usr/bin/env bash
# ==============================================================================
# vitusOS QEMU & KVM Dependency Checker & Environment Validator
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================================================"
echo -e "           vitusOS KVM/QEMU Environment & Dependency Verification               "
echo -e "================================================================================${NC}"

MISSING=0

# 1. Check KVM Acceleration
echo -n "Checking /dev/kvm accessibility... "
if [[ -c /dev/kvm ]]; then
    if [[ -r /dev/kvm && -w /dev/kvm ]]; then
        echo -e "${GREEN}[OK] Read/Write access available${NC}"
    else
        echo -e "${YELLOW}[WARN] Exists, but user lacks RW access. Add user to kvm group:${NC}"
        echo "       sudo usermod -aG kvm \$USER"
    fi
else
    echo -e "${RED}[FAIL] /dev/kvm device node not found. Check BIOS/UEFI virtualization settings (VT-x / AMD-V).${NC}"
    MISSING=$((MISSING + 1))
fi

# 2. Check QEMU binary
echo -n "Checking qemu-system-x86_64... "
if command -v qemu-system-x86_64 &>/dev/null; then
    QEMU_VER=$(qemu-system-x86_64 --version | head -n1)
    echo -e "${GREEN}[OK] ${QEMU_VER}${NC}"
else
    echo -e "${RED}[MISSING] qemu-system-x86_64 not found.${NC}"
    MISSING=$((MISSING + 1))
fi

# 3. Check OVMF Firmware
echo -n "Checking OVMF UEFI firmware... "
OVMF_PATHS=(
    "/usr/share/OVMF/OVMF_CODE_4M.fd"
    "/usr/share/OVMF/OVMF_CODE.fd"
    "/usr/share/ovmf/OVMF.fd"
    "/usr/share/edk2/x64/OVMF_CODE.fd"
    "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"
)

FOUND_OVMF=""
for p in "${OVMF_PATHS[@]}"; do
    if [[ -f "$p" ]]; then
        FOUND_OVMF="$p"
        break
    fi
done

if [[ -n "$FOUND_OVMF" ]]; then
    echo -e "${GREEN}[OK] Found at ${FOUND_OVMF}${NC}"
else
    echo -e "${RED}[MISSING] OVMF UEFI image not found in standard paths.${NC}"
    MISSING=$((MISSING + 1))
fi

# 4. Check GNU-EFI for AnimusBoot compilation
echo -n "Checking gnu-efi headers & libraries... "
if [[ -d "/usr/include/efi" && (-f "/usr/lib/libefi.a" || -f "/usr/lib/x86_64-linux-gnu/libefi.a" || -f "/usr/lib64/libefi.a") ]]; then
    echo -e "${GREEN}[OK] Found gnu-efi${NC}"
else
    echo -e "${YELLOW}[WARN] gnu-efi headers or static libs not in standard paths.${NC}"
fi

# 5. Check Disk formatting utilities
echo -n "Checking disk utilities (mkfs.vfat, mtools, xorriso)... "
UTILS_OK=1
for u in mkfs.vfat mcopy mformat xorriso; do
    if ! command -v "$u" &>/dev/null; then
        UTILS_OK=0
        break
    fi
done

if [[ $UTILS_OK -eq 1 ]]; then
    echo -e "${GREEN}[OK] All utilities found${NC}"
else
    echo -e "${YELLOW}[WARN] Some image tools (mtools/xorriso) missing.${NC}"
fi

echo -e "${BLUE}================================================================================${NC}"
if [[ $MISSING -gt 0 ]]; then
    echo -e "${RED}FAILURE: ${MISSING} required component(s) missing.${NC}"
    echo "Install prerequisites with:"
    echo "  sudo apt-get update && sudo apt-get install -y qemu-system-x86 ovmf gnu-efi mtools libudev-dev"
    exit 1
else
    echo -e "${GREEN}SUCCESS: Environment is fully configured for vitusOS KVM/QEMU testing!${NC}"
    exit 0
fi
