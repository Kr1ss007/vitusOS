# vitusOS (Upstream Color Channel)

vitusOS is an operating system and desktop environment architecture built on the AnimusEngine compositor and Wayland display server. It delivers a fluid, physics-driven user experience with real-time multi-pass Kawase glassmorphism, zero-copy DMA-BUF direct scanout rendering, low-latency PipeWire spatial audio, and unified userspace applications.

This repository hosts the active **Upstream Color** development channel, delivering experimental rolling updates and architecture implementations.

---

## Architectural Overview

vitusOS is designed around deterministic state synchronization, physical motion dynamics, and direct hardware integration:

### 1. AnimusEngine Compositor and Display Server
* **Vulkan 1.3 Direct Scanout Pipeline:** Renders directly into scanout buffers acquired via `wlr_output_attach_render` and imported through `VK_EXT_image_drm_format_modifier`, bypassing intermediate composition buffers for zero-copy latency at 144Hz.
* **AESurfaces Glass System:** Multi-altitude rendering architecture utilizing dual-pass Kawase blur, chromatic aberration, specular lighting, and noise dithering across four canonical altitudes:
  * Low Altitude (8px blur, 94% opacity): Top Panel
  * Mid Altitude (20px blur, 82% opacity): Dock
  * High Altitude (32px blur, 72% opacity): Control Center, Menus, Dialogs
  * Floating Altitude (48px blur, 64% opacity): LockScreen, LoginManager, Shutdown Dialog
* **ae-shell-v1 Wayland Protocol:** Custom extension allowing native client applications to export Global Menu definitions in UTF-8 JSON, update Dock badge counts, and issue attention requests.

### 2. Dual-Bus Event Subsystem
* **EventBus:** Synchronous in-process event broker coupled with a lock-free queue for background worker events, drained on the main frame loop (`drain_async_queue`) to eliminate main-thread stutter.
* **EOBus (Event Outsider Bus):** Dedicated subsystem mediating external Linux environment events (Linux system D-Bus, PAM authentication workers, udev hardware hotplug, and Unix domain sockets) into normalized `AEEvent` streams.

### 3. Physical Motion Dynamics
* **Semi-Implicit Euler Solvers:** 1D and 2D spring solvers executing at high sub-step frequencies, eliminating abrupt animation jumps and supporting kinetic velocity injection.
* **Boundary Resistance:** Configurable dual-axis edge resistance (`enable_edge_resistance_x`, `enable_edge_resistance_y`) preventing window and drawer clipping.
* **Dedicated Settler Queue:** Physicssettler engine (`AnimationEngine::on_settle`) dispatching transition completion callbacks directly without polling or overloading frame tick events.

### 4. Security Architecture
* **Hardware Encryption Vault (HEV):** AES-256-GCM authenticated encryption paired with Argon2id memory-hard key derivation.
* **Credential Memory Scrubbing:** Strict memory zeroization (`zeroize`) applied to passphrases and key materials prior to buffer clearing or memory deallocation.

---

## Native Userspace Suite

* **Filer:** Continuous filesystem daemon providing instant directory loading, thumbnail caching, and unified search integration.
* **Pathfinder:** System-wide search overlay and package manager integrating Ubuntu APT repositories, Flatpak, and Snap with instant package previews.
* **Zen Browser Integration:** Native Mozilla Zen Browser deployment featuring Kawase glass theming, Wayland display backend support, and tab synchronization.
* **Terminow:** High-performance, GPU-accelerated terminal emulator with custom font rendering and transparency.
* **Settings Application:** System configuration hub with display topology, sound sink management, theme configuration, and real-time OTA release channel selection (Upstream Color vs Upstream One).
* **Native Shell Surfaces:**
  * Top Panel (28px height, dynamic active app title, brand mark, Global Menu integration).
  * Global Menu (Application-driven menu bar with dynamic D-Bus routing).
  * Dock (Physics-magnified icon dock with running indicators and attention bounces).
  * Control Center (System tray popover with volume, brightness, Wi-Fi, Bluetooth, and telemetry).
  * LockScreen & LoginManager (Native AESurfaces with optical clock and PAM authentication).
  * CockpitView (Spatial workspace overview with virtual desktop strip and smooth altitude zoom).
  * Shutdown Dialog (Power management interface with automated 60-second countdown).

---

## Repository Structure

```
vitusOS/
|-- assets/
|   |-- cursors/               # macOS-grade compiled Wayland and X11 cursors
|   |-- icons/                 # Scalable SVG icon suite (dock, panel, sidebar, mimetypes)
|   |-- packages/              # Pre-bundled Zen Browser binary packages
|   `-- sounds/                # Canonical uncompressed boot chime (Startup1.wav / 209 KB)
|-- crates/
|   |-- animus-core/           # EventBus, EOBus, StateManager, HardwareTopology, SoundEngine
|   |-- animus-physics/        # Spring solvers, 2D edge resistance, AnimationEngine
|   |-- animus-render/         # Vulkan DMA-BUF context, AESurfaces, typography, AppKit
|   |-- animus-compositor/     # Compositor main, ae-shell-v1, Top Panel, Dock, LockScreen
|   |-- animus-input/          # Evdev and libinput drivers, MotionWave gesture recognizer
|   |-- animus-cache/          # Binary application index and thumbnail cache
|   |-- animus-hev/            # Hardware Encryption Vault (AES-256-GCM / Argon2id)
|   |-- vitusos-native/        # Filer, Pathfinder, Settings, Zen Browser, Package Manager
|   `-- vitusos-installer/     # Bare-metal installation wizard and partitioner
|-- docs/                      # Architecture specifications and technical manuals
`-- Cargo.toml                 # Workspace manifest
```

---

## Building and Testing

### Prerequisites

* Rust toolchain (1.80 or newer)
* Vulkan SDK 1.3
* Linux development headers (libwayland-dev, libxkbcommon-dev, libpipewire-0.3-dev, libdrm-dev, libgbm-dev)

### Compilation

Build the complete workspace:
```bash
cargo build --workspace --release
```

Run the complete test suite across all 9 crates:
```bash
cargo test --workspace
```

### Launching the Compositor

Launch the vitusOS compositor with the default 144Hz pipeline:
```bash
cargo run --bin vitusos-compositor
```

---

## Release Channels & Git Branch Architecture

| Channel | Git Branch | Release ISO Naming Pattern | Description |
| :--- | :--- | :--- | :--- |
| **Upstream Color** | `main` | `vitusOS_upstreamColor_<version>_x86_64_amd64.iso` | Active rolling development channel. Delivers experimental features, latest Vulkan updates, and bleeding-edge compositor builds. |
| **Upstream One** | `upstreamOne` | `vitusOS_upstreamOne_<version>_x86_64_amd64.iso` | Verified, production-grade Stable LTS channel for daily workstation reliability and security. |

---

## Grand Payload ISO Building (8–10 GB OOTB)

vitusOS provides an automated, batteries-included ISO builder that pre-bakes all proprietary and open-source GPU drivers (NVIDIA 550/560, AMD RADV, Intel ANV), full firmware, media codecs, high-fidelity typography, and the AnimusEngine runtime.

To generate a bootable hybrid ISO:

```bash
cd distro/builder

# Build Upstream Color (Rolling development ISO)
sudo ./build_iso.sh --channel upstreamColor --version 0.0.1

# Build Upstream One (Stable production ISO)
sudo ./build_iso.sh --channel upstreamOne --version 1.0.0
```

The resulting hybrid ISO (`vitusOS_<channel>_<version>_x86_64_amd64.iso`) supports direct USB `dd` flashing, UEFI Secure Boot, and legacy BIOS systems.

---

## License

Copyright (c) 2026 vitusOS contributors. All rights reserved.
Distributed under the MIT License.
