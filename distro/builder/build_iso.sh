#!/usr/bin/env bash
# ==============================================================================
# vitusOS ISO Image Builder
# Channels: upstreamColor (Rolling/Dev) | upstreamOne (Stable LTS)
# Naming Convention: vitusOS_<Channel>_<Version>_x86_64_amd64.iso
# ==============================================================================

set -euo pipefail

CHANNEL="upstreamColor"
VERSION="0.0.1"
ARCH="x86_64_amd64"
DISTRO_NAME="vitusOS"
UBUNTU_RELEASE="noble"
UBUNTU_MIRROR="http://archive.ubuntu.com/ubuntu/"
OUTPUT_DIR="$(pwd)/out"
WORK_DIR="$(pwd)/build_work"

# Print Banner
echo "================================================================================"
echo "                   vitusOS Grand Payload ISO Builder                            "
echo "================================================================================"

# Parse Command Line Arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --channel)
            CHANNEL="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --arch)
            ARCH="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: sudo ./build_iso.sh [OPTIONS]"
            echo "Options:"
            echo "  --channel <upstreamColor|upstreamOne>  Release channel (default: upstreamColor)"
            echo "  --version <x.y.z>                      Version tag (default: 0.0.1)"
            echo "  --arch <architecture>                  Arch identifier (default: x86_64_amd64)"
            echo "  --output-dir <path>                    Output directory for final ISO"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate Channel
if [[ "$CHANNEL" != "upstreamColor" && "$CHANNEL" != "upstreamOne" ]]; then
    echo "ERROR: Invalid channel '$CHANNEL'. Must be 'upstreamColor' or 'upstreamOne'."
    exit 1
fi

ISO_FILENAME="${DISTRO_NAME}_${CHANNEL}_${VERSION}_${ARCH}.iso"
ISO_PATH="${OUTPUT_DIR}/${ISO_FILENAME}"
CHROOT_DIR="${WORK_DIR}/chroot"
LIVE_IMAGE_DIR="${WORK_DIR}/image"

echo "Channel:          ${CHANNEL}"
echo "Version:          ${VERSION}"
echo "Output ISO:       ${ISO_PATH}"
echo "Target Base:      Ubuntu ${UBUNTU_RELEASE} (24.04 LTS)"
echo "================================================================================"

# Verify Root Privileges
if [[ $EUID -ne 0 ]]; then
   echo "ERROR: This script must be run as root (sudo ./build_iso.sh)." 
   exit 1
fi

# Create Directories
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${CHROOT_DIR}"
mkdir -p "${LIVE_IMAGE_DIR}/casper"
mkdir -p "${LIVE_IMAGE_DIR}/boot/grub"
mkdir -p "${LIVE_IMAGE_DIR}/EFI/BOOT"

# Step 1: Bootstrap Base System
echo "[1/8] Bootstrapping Ubuntu ${UBUNTU_RELEASE} base..."
if [ ! -f "${CHROOT_DIR}/bin/bash" ]; then
    debootstrap --arch=amd64 "${UBUNTU_RELEASE}" "${CHROOT_DIR}" "${UBUNTU_MIRROR}"
fi

# Step 2: Configure Mounts
echo "[2/8] Setting up chroot mounts..."
mount --bind /dev "${CHROOT_DIR}/dev"
mount --bind /dev/pts "${CHROOT_DIR}/dev/pts"
mount -t proc proc "${CHROOT_DIR}/proc"
mount -t sysfs sys "${CHROOT_DIR}/sys"
mount -t tmpfs tmpfs "${CHROOT_DIR}/tmp"

cleanup() {
    echo "Cleaning up mounts..."
    umount -lf "${CHROOT_DIR}/tmp" 2>/dev/null || true
    umount -lf "${CHROOT_DIR}/sys" 2>/dev/null || true
    umount -lf "${CHROOT_DIR}/proc" 2>/dev/null || true
    umount -lf "${CHROOT_DIR}/dev/pts" 2>/dev/null || true
    umount -lf "${CHROOT_DIR}/dev" 2>/dev/null || true
}
trap cleanup EXIT

# Step 3: Configure APT Repositories
echo "[3/8] Configuring full universe/multiverse repositories..."
cat << 'EOF' > "${CHROOT_DIR}/etc/apt/sources.list"
deb http://archive.ubuntu.com/ubuntu/ noble main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu/ noble-updates main restricted universe multiverse
deb http://security.ubuntu.com/ubuntu noble-security main restricted universe multiverse
EOF

# Step 4: Install Packages & Drivers in Chroot
echo "[4/8] Installing Grand Payload dependencies (NVIDIA, Mesa, PipeWire, Codecs, Fonts)..."
cp ../packages/packages.list "${CHROOT_DIR}/tmp/packages.list"
chroot "${CHROOT_DIR}" /bin/bash -c "
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    grep -v '^#' /tmp/packages.list | grep -v '^$' | xargs apt-get install -y --no-install-recommends
    flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo || true
    systemctl enable systemd-networkd systemctl enable NetworkManager || true
"

# Step 5: Install vitusOS Engine & Desktop Session
echo "[5/8] Installing AnimusEngine compositor, native shell, and configuration files..."
cp ../systemd/animus-compositor.service "${CHROOT_DIR}/etc/systemd/system/animus-compositor.service"
cp ../wayland-sessions/vitusos.desktop "${CHROOT_DIR}/usr/share/wayland-sessions/vitusos.desktop"
cp ../udev/99-animus-drm.rules "${CHROOT_DIR}/etc/udev/rules.d/99-animus-drm.rules"
cp ../udev/99-animus-input.rules "${CHROOT_DIR}/etc/udev/rules.d/99-animus-input.rules"

# Configure Live User & Autologin
chroot "${CHROOT_DIR}" /bin/bash -c "
    useradd -m -s /bin/bash -G sudo,video,render,input,audio,plugdev vitus
    echo 'vitus:vitus' | chpasswd
    echo 'vitus ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/vitus
    systemctl enable animus-compositor.service
"

# Step 6: Extract Kernel & Initrd for Live Boot
echo "[6/8] Extracting kernel and initrd..."
cp "${CHROOT_DIR}"/boot/vmlinuz-* "${LIVE_IMAGE_DIR}/casper/vmlinuz"
cp "${CHROOT_DIR}"/boot/initrd.img-* "${LIVE_IMAGE_DIR}/casper/initrd"

# Step 7: Create SquashFS Image
echo "[7/8] Generating filesystem.squashfs (zstd level 19)..."
rm -f "${LIVE_IMAGE_DIR}/casper/filesystem.squashfs"
mksquashfs "${CHROOT_DIR}" "${LIVE_IMAGE_DIR}/casper/filesystem.squashfs" \
    -comp zstd -Xcompression-level 19 \
    -e proc sys dev tmp var/cache/apt

# Step 8: Build Bootable Hybrid ISO
echo "[8/8] Generating hybrid ISO: ${ISO_FILENAME}..."
cat << EOF > "${LIVE_IMAGE_DIR}/boot/grub/grub.cfg"
set default="0"
set timeout=5

menuentry "vitusOS (${CHANNEL}) [Live Session]" {
    linux /casper/vmlinuz boot=casper quiet splash nvidia-drm.modeset=1 console=tty1 ---
    initrd /casper/initrd
}

menuentry "vitusOS (${CHANNEL}) [Safe Graphics / Compatibility]" {
    linux /casper/vmlinuz boot=casper nomodeset ---
    initrd /casper/initrd
}
EOF

grub-mkstandalone \
    --format=x86_64-efi \
    --output="${LIVE_IMAGE_DIR}/EFI/BOOT/BOOTX64.EFI" \
    --locales="" \
    --fonts="" \
    "boot/grub/grub.cfg=${LIVE_IMAGE_DIR}/boot/grub/grub.cfg"

xorriso -as mkisofs \
    -r -V "${DISTRO_NAME}_${CHANNEL}" \
    -J -l -b boot/grub/i386-pc/eltorito.img \
    -c boot.catalog \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --grub2-boot-info \
    -eltorito-alt-boot \
    -e EFI/BOOT/BOOTX64.EFI \
    -no-emul-boot -isohybrid-gpt-basdat \
    -o "${ISO_PATH}" \
    "${LIVE_IMAGE_DIR}"

# Generate SHA256 Checksum
cd "${OUTPUT_DIR}"
sha256sum "${ISO_FILENAME}" > "${ISO_FILENAME}.sha256"

echo "================================================================================"
echo " SUCCESS: ${ISO_PATH}"
echo " SHA256:  $(cat ${ISO_FILENAME}.sha256)"
echo "================================================================================"
