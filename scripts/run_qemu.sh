#!/usr/bin/env bash
# ==============================================================================
# vitusOS QEMU / KVM Virtual Machine Runner
# Boots vitusOS AnimusBoot Stage 0 UEFI, ESP partition, or Release ISO with KVM
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET_DIR="${ROOT_DIR}/target"

# Default configuration parameters
MODE="uefi"
HEADLESS=false
DRY_RUN=false
TIMEOUT=0
MEMORY="4G"
SMP="4"
ISO_PATH=""
ESP_PATH=""
SERIAL_LOG=""
SERIAL_MODE="stdio"

# Print usage banner
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Options:
  --mode <uefi|iso|headless>   Execution mode:
                                - uefi: Boots AnimusBoot EFI from ESP image/dir (default)
                                - iso:  Boots from CD-ROM ISO image (requires --iso)
                                - headless: Equivalent to --headless flag
  --iso <path>                 Path to ISO file (for --mode iso)
  --esp <path>                 Path to ESP image (.img) or directory (default: target/esp.img)
  --headless                   Run without GUI window (-display none -serial stdio)
  --dry-run                    Print the assembled QEMU command without executing
  --timeout <seconds>          Automatically terminate VM after specified seconds
  --serial-log <path>          Save serial console output to file
  -m, --memory <size>          Memory size (default: 4G)
  -s, --smp <cores>            Number of CPU cores (default: 4)
  -h, --help                   Show this help message and exit

Examples:
  $(basename "$0")                          # Boot AnimusBoot UEFI with KVM GUI
  $(basename "$0") --headless               # Boot in headless mode for CI/terminal
  $(basename "$0") --mode iso --iso vitus.iso  # Boot release ISO
  $(basename "$0") --dry-run                # Inspect QEMU execution arguments
EOF
}

# Parse command-line options
while [[ $# -gt 0 ]]; do
    case "$1" in
        --mode)
            MODE="$2"
            shift 2
            ;;
        --iso)
            ISO_PATH="$2"
            shift 2
            ;;
        --esp)
            ESP_PATH="$2"
            shift 2
            ;;
        --headless)
            echo "WARNING: vitusOS is a production-grade graphical OS. Headless mode is forbidden. Launching in full graphical window mode."
            HEADLESS=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --serial-log)
            SERIAL_LOG="$2"
            shift 2
            ;;
        -m|--memory)
            MEMORY="$2"
            shift 2
            ;;
        -s|--smp)
            SMP="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
done

if [ "$MODE" = "headless" ]; then
    echo "WARNING: vitusOS is a production-grade graphical OS. Mode 'headless' is forbidden. Defaulting to 'uefi' with graphical display."
    HEADLESS=false
    MODE="uefi"
fi

echo "================================================================================"
echo "                      vitusOS KVM/QEMU Execution Harness                       "
echo "================================================================================"

# Verify QEMU binary
QEMU_BIN="$(command -v qemu-system-x86_64 || true)"
if [ -z "$QEMU_BIN" ]; then
    echo "ERROR: qemu-system-x86_64 not found. Run ./scripts/setup_qemu_deps.sh" >&2
    exit 1
fi
echo "Using QEMU Binary: ${QEMU_BIN}"

# Detect KVM Hardware Acceleration
QEMU_ACCEL=()
if [ -c /dev/kvm ] && [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
    echo "Hardware Acceleration: KVM (/dev/kvm) ENABLED [Optimal Performance]"
    QEMU_ACCEL+=("-enable-kvm" "-cpu" "host")
else
    echo "Hardware Acceleration: KVM unavailable or permission denied, using TCG software emulation (-cpu qemu64)"
    QEMU_ACCEL+=("-cpu" "qemu64")
fi

# Locate OVMF UEFI Firmware
OVMF_CODE=""
OVMF_VARS=""
UNIFIED_OVMF=""

# Search common paths
if [ -f "/usr/share/OVMF/OVMF_CODE_4M.fd" ] && [ -f "/usr/share/OVMF/OVMF_VARS_4M.fd" ]; then
    OVMF_CODE="/usr/share/OVMF/OVMF_CODE_4M.fd"
    OVMF_VARS="/usr/share/OVMF/OVMF_VARS_4M.fd"
elif [ -f "/usr/share/OVMF/OVMF_CODE.fd" ] && [ -f "/usr/share/OVMF/OVMF_VARS.fd" ]; then
    OVMF_CODE="/usr/share/OVMF/OVMF_CODE.fd"
    OVMF_VARS="/usr/share/OVMF/OVMF_VARS.fd"
elif [ -f "/usr/share/ovmf/OVMF.fd" ]; then
    UNIFIED_OVMF="/usr/share/ovmf/OVMF.fd"
elif [ -f "/usr/share/qemu/OVMF.fd" ]; then
    UNIFIED_OVMF="/usr/share/qemu/OVMF.fd"
fi

# Setup NVRAM VARS copy so system templates remain untouched
TEMP_VARS=""
cleanup() {
    if [ -n "$TEMP_VARS" ] && [ -f "$TEMP_VARS" ]; then
        rm -f "$TEMP_VARS"
    fi
}
trap cleanup EXIT INT TERM

QEMU_FIRMWARE=()
if [ -n "$OVMF_CODE" ] && [ -n "$OVMF_VARS" ]; then
    TEMP_VARS="$(mktemp /tmp/vitus_ovmf_vars_XXXXXX.fd)"
    cp -f "$OVMF_VARS" "$TEMP_VARS"
    QEMU_FIRMWARE+=(
        "-drive" "if=pflash,format=raw,readonly=on,file=${OVMF_CODE}"
        "-drive" "if=pflash,format=raw,file=${TEMP_VARS}"
    )
    echo "UEFI Firmware: Split OVMF (${OVMF_CODE})"
elif [ -n "$UNIFIED_OVMF" ]; then
    QEMU_FIRMWARE+=("-bios" "${UNIFIED_OVMF}")
    echo "UEFI Firmware: Unified OVMF (${UNIFIED_OVMF})"
else
    echo "ERROR: OVMF UEFI firmware not found on system. Install 'ovmf' package." >&2
    exit 1
fi

# Assemble Base Machine Configuration
QEMU_ARGS=(
    "${QEMU_ACCEL[@]}"
    "-m" "$MEMORY"
    "-smp" "$SMP"
    "${QEMU_FIRMWARE[@]}"
)

# Display and IO Configuration — ALWAYS FULL GRAPHICAL WINDOW (NO HEADLESS)
echo "Display Mode: Full Graphical Window (virtio-vga, virtio-tablet, virtio-keyboard, PipeWire Intel-HDA)"
QEMU_ARGS+=(
    "-vga" "virtio"
    "-device" "virtio-tablet-pci"
    "-device" "virtio-keyboard-pci"
    "-display" "gtk,gl=off"
)
# Intel HDA Audio subsystem with host PipeWire integration
QEMU_ARGS+=(
    "-audiodev" "pipewire,id=snd0"
    "-device" "intel-hda"
    "-device" "hda-duplex,audiodev=snd0"
)

if [ -n "$SERIAL_LOG" ]; then
    QEMU_ARGS+=("-serial" "file:${SERIAL_LOG}")
    echo "Serial Console: Logged to ${SERIAL_LOG}"
else
    QEMU_ARGS+=("-serial" "stdio")
    echo "Serial Console: Interactive (stdio)"
fi

# Storage / Boot Drive Configuration
case "$MODE" in
    uefi)
        # Check ESP path
        if [ -z "$ESP_PATH" ]; then
            if [ -f "${TARGET_DIR}/esp.img" ]; then
                ESP_PATH="${TARGET_DIR}/esp.img"
            elif [ -d "${TARGET_DIR}/esp" ]; then
                ESP_PATH="${TARGET_DIR}/esp"
            else
                echo "ESP artifacts not found. Building UEFI artifacts via scripts/build_uefi.sh..."
                "${SCRIPT_DIR}/build_uefi.sh"
                ESP_PATH="${TARGET_DIR}/esp.img"
            fi
        fi

        if [ -f "$ESP_PATH" ]; then
            echo "Boot Media: FAT32 ESP Disk Image ($ESP_PATH)"
            QEMU_ARGS+=(
                "-drive" "file=${ESP_PATH},format=raw,if=virtio"
            )
        elif [ -d "$ESP_PATH" ]; then
            echo "Boot Media: Virtual FAT ESP Directory ($ESP_PATH)"
            QEMU_ARGS+=(
                "-drive" "file=fat:rw:${ESP_PATH},format=raw"
            )
        else
            echo "ERROR: Specified ESP path '$ESP_PATH' does not exist." >&2
            exit 1
        fi
        ;;
    iso)
        if [ -z "$ISO_PATH" ] || [ ! -f "$ISO_PATH" ]; then
            echo "ERROR: --mode iso requires a valid --iso <path> pointing to an existing ISO file." >&2
            exit 1
        fi
        echo "Boot Media: CD-ROM ISO ($ISO_PATH)"
        QEMU_ARGS+=(
            "-cdrom" "$ISO_PATH"
            "-boot" "d"
        )
        ;;
    *)
        echo "ERROR: Invalid mode '$MODE'. Supported: uefi, iso, headless." >&2
        exit 1
        ;;
esac

echo "================================================================================"
echo "Launching QEMU command:"
echo "${QEMU_BIN} ${QEMU_ARGS[*]}"
echo "================================================================================"

if [ "$DRY_RUN" = true ]; then
    echo "[DRY RUN] Command generated successfully. Exiting without launching VM."
    exit 0
fi

# Execute QEMU (with optional timeout)
if [ "$TIMEOUT" -gt 0 ]; then
    echo "Running with timeout of ${TIMEOUT} seconds..."
    timeout --foreground -k 2 "${TIMEOUT}" "${QEMU_BIN}" "${QEMU_ARGS[@]}" || true
else
    exec "${QEMU_BIN}" "${QEMU_ARGS[@]}"
fi
