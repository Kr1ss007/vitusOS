# vitusOS Codebase Honest Audit

> **Total Rust source**: 8,485 lines across 81 files in 9 crates
> **Spec doc**: 19,358 lines across 47 Parts (Part 0–46)
> **Verdict**: The codebase is approximately **8–12%** of the way to a shippable macOS-grade desktop OS.

---

## The Hard Truth: What Actually Exists vs What Should Exist

### Legend
| Symbol | Meaning |
|--------|---------|
| ✅ Real | Has real logic, real I/O, real data structures, tested |
| ⚠️ Skeleton | Data structures exist, spring values wired, but no real rendering/I/O backend |
| ❌ Missing | Not implemented at all |
| 🎭 Mock | Appears to work in tests but does nothing on real hardware |

---

## Layer 1: Boot Chain & Kernel Integration

| Component | Status | Lines | Notes |
|-----------|--------|-------|-------|
| `AnimusBoot.c` (UEFI loader) | 🎭 Mock | ~200 | Has GOP mode setup and PCI scan code, but **never compiled with a real UEFI toolchain** — uses a hand-written `uefi.h` that won't link against actual firmware. No PE/COFF entry point. Not a real `.efi` binary. |
| `animus-early` (initramfs) | ❌ Missing | 0 | Part 2 specifies a C11 initramfs service for SimpleDRM→real DRM handoff. **Does not exist.** |
| Kernel config / patches | ❌ Missing | 0 | No `CONFIG_DRM_SIMPLEDRM=y`, no kernel command line, no custom modules. |

---

## Layer 2: Compositor (Wayland)

| Component | Status | Lines | Notes |
|-----------|--------|-------|-------|
| wlroots integration | ❌ Missing | 0 | **The compositor has zero wlroots/libwayland code.** No `wl_display`, no `wlr_output`, no `wlr_scene`. The entire compositor is a Rust state machine that prints logs. It cannot display a single pixel on screen. |
| Vulkan rendering pipeline | 🎭 Mock | 136 | `vulkan_context.rs` has correct extension names and DMA-BUF struct definitions, but **zero Vulkan API calls** (`vkCreateInstance`, `vkCreateDevice`, etc. do not exist). It's a HashMap tracking fake buffer IDs. |
| GLSL shaders | ❌ Missing | 0 | Part 5 specifies complete `rounded_rect.vert/frag`, `kawase_blur.frag`, `glyph.vert/frag`. **None exist on disk.** |
| Frame loop (144Hz vsync) | 🎭 Mock | ~30 | The simulation prints "144Hz frame pacing" but there is no actual `wlr_output.frame` callback, no vblank sync, no `WLR_RENDERER`. |
| osf-shell-v1 Wayland protocol | ⚠️ Skeleton | 93 | Has Rust structs for protocol state, but **no `.xml` protocol definition file**, no `wayland-scanner` codegen, no actual Wayland IPC. |

---

## Layer 3: Rendering Engine

| Component | Status | Lines | Notes |
|-----------|--------|-------|-------|
| Glassmorphism / Kawase blur | ❌ Missing | 0 | `SurfaceAltitude` enum exists (53 lines) with blur radii constants, but **no actual blur shader or render pass**. |
| Shadow rendering | ⚠️ Skeleton | 48 | Struct with shadow parameters, no GPU draw calls. |
| Squircle SDF | ✅ Real | 105 | Math is correct ($n=4.4$ superellipse). But it produces `f32` distance values — **nothing consumes it for actual rendering**. |
| Typography / GlyphAtlas | ⚠️ Skeleton | 161+153 | Has `fontdue` glyph rasterization and gamma correction logic. **Real math, but not connected to any rendering pipeline.** Glyphs are rasterized into a `Vec<u8>` that nothing reads. |
| Wallpaper tint sampling | ⚠️ Skeleton | 93 | OKLab luminance math is correct. But reads from a hardcoded `Vec<u8>` of pixel data, not from an actual image file or GPU texture. |
| Color system | ✅ Real | 88 | OKLab color tokens and conversion functions are production-correct. |

---

## Layer 4: Shell Components

| Component | Status | Lines | Notes |
|-----------|--------|-------|-------|
| Panel (top bar) | ⚠️ Skeleton | 43 | Has clock, focused app title, and Wi-Fi/Battery state. **Not rendered anywhere.** |
| Dock | ⚠️ Skeleton | 72 | DockItem structs with magnification springs. **Not rendered. No mouse hover handler. No app launch.** |
| Lock Screen | ⚠️ Skeleton | 112 | Has shake-on-wrong-password spring and PAM auth state machine. **No actual PAM integration, no actual screen rendering.** |
| Control Center | ⚠️ Skeleton | 99 | Toggle states for Wi-Fi/Bluetooth/DND. **No actual NetworkManager/BlueZ D-Bus calls.** |
| Notification Center | ⚠️ Skeleton | 125 | Toast notification stack with spring animations. **No D-Bus `org.freedesktop.Notifications` server.** |
| Shutdown Screen | ⚠️ Skeleton | 94 | Countdown timer and cancel logic. **No actual `systemctl poweroff` execution.** |
| System Screen | ⚠️ Skeleton | 97 | "goodbye" / "i'll see you in a bit" text. **No framebuffer rendering, no actual system call.** |
| Boot Crossfade | ⚠️ Skeleton | 66 | Spring-based opacity fade. **No actual GOP→DRM framebuffer handoff.** |
| CockpitView | ⚠️ Skeleton | 84 | Zoom model with sentinel detection. **No actual window thumbnail rendering.** |
| Welcome Screen | ⚠️ Skeleton | 53 | Slide transitions. **No actual first-boot detection.** |
| Global Menu | ⚠️ Skeleton | 54 | Menu items struct. **No actual app menu extraction.** |
| Login Manager | ⚠️ Skeleton | 73 | User selection and session start. **No actual `logind`/PAM integration.** |

---

## Layer 5: Native Applications

| App | Status | Lines | Notes |
|-----|--------|-------|-------|
| **Filer** | ⚠️ Skeleton+ | 374 | **Best of the native apps.** Has real `std::fs::read_dir`, real `FileOperationDaemon` with background thread `Copy`/`Move`/`Trash`/`Delete`. But: no GUI rendering, no drag-and-drop, no thumbnail generation, no column view rendering. |
| **Pathfinder** | ⚠️ Skeleton | 162 | Has fuzzy search scoring and calculator evaluation. But: **search results come from a hardcoded in-memory `HashMap`**, not from actual system indexing. The `.desktop` scanner only runs on Linux and was just added. No actual spotlight-style overlay rendering. |
| **Terminow** | ⚠️ Skeleton | 219 | Has tab model, command history, and built-in commands. But: **no actual PTY** (`openpty`/`forkpty`), no VTE escape sequence parser, no real shell process (`/bin/bash`). Commands are matched against string literals. |
| **Settings** | ⚠️ Skeleton | 194 | Has 9 sections with state structs and OTA channel enum. But: **no actual system calls** — toggling dark mode changes a `bool` in memory, not `gsettings` or dconf. Volume slider changes a `f32`, doesn't touch PipeWire/PulseAudio. |
| **Zen Browser** | ⚠️ Skeleton | 172 | Has workspace/tab model and `userChrome.css` generation. But: **no actual Gecko embedding**, no `MOZ_ENABLE_WAYLAND`, no process spawning. |
| **Package Manager** | ⚠️ Skeleton | 122 | Has `apt-get install`, `flatpak install`, `snap install` command construction. But: **`Command::new()` calls are behind `#[cfg(target_os = "linux")]`** and have never been tested on real Ubuntu. |
| **App Preview Sheet** | ⚠️ Skeleton | 102 | Spring-animated sheet with format selector. But: **no Gecko webview embed**, no screenshot rendering. |
| **Font Book** | ⚠️ Skeleton | 85 | Font preview state. Minimal. |

---

## Layer 6: System Infrastructure

| Component | Status | Lines | Notes |
|-----------|--------|-------|-------|
| CrashManager / Vessels | ✅ Real | ~600 | Best-implemented subsystem. Real BFS blast radius isolation, respawn rate limiting, health monitoring, global feed classification. **But only manages in-memory state — no actual process supervision (`fork`/`waitpid`).** |
| EventBus | ✅ Real | 112 | Working `crossbeam-channel` pub/sub. Actually used throughout. |
| EOBus (PAM bridge) | ⚠️ Skeleton | 136 | Has `zeroize`-guarded credential structures. **No actual PAM library linkage.** |
| HEV Vault | ✅ Real | 84+64 | Real AES-256-GCM encryption with Argon2id KDF. **Actual cryptography that works.** |
| RegistryManager | ⚠️ Skeleton | 182 | Schema validation with JSON types. **No persistent storage — in-memory only.** |
| StateManager | ⚠️ Skeleton | 98 | Key-value store. **In-memory HashMap, no disk persistence.** |
| SoundEngine | ⚠️ Skeleton | 175 | Boot chime resolution logic with `rodio` fallback detection. **No actual PipeWire integration.** |
| MotionWave | ⚠️ Skeleton | 131 | Gesture recognition state machine. **No `libinput` integration.** |
| AnimusEngine | ⚠️ Skeleton | 91 | Boot sequence and hardware topology. **Reads no actual hardware.** |
| GPU Handoff | ⚠️ Skeleton | 137 | DRM driver mapping table. **No actual ioctl calls.** |

---

## Layer 7: Distro / ISO / Out-of-the-Box

| Component | Status | Notes |
|-----------|--------|-------|
| `build_iso.sh` | ⚠️ Skeleton | Has debootstrap + mksquashfs + xorriso pipeline. **Never been run. Will fail** because the compositor binary doesn't actually exist as a compiled Linux ELF. |
| systemd services | ⚠️ Skeleton | `.service` and `.target` files exist with correct `After=` ordering. **Never tested on real systemd.** |
| udev rules | ⚠️ Skeleton | 4 rule files for DRM/input/audio/vault. **Syntactically valid but never deployed.** |
| Package manifest | ❌ Missing | No actual `packages.manifest` listing all 8-10GB of OOTB dependencies. |
| Installer wizard | ⚠️ Skeleton | Has disk scanning, partition formatting, account creation, HEV vault setup. **All behind `#[cfg(target_os = "linux")]` guards. No actual GUI.** |
| Wallpapers | ❌ Missing | `assets/wallpapers/` is **empty**. |
| System sounds | ⚠️ Partial | 4 audio files exist but 2 are duplicates (`Startup1.wav` = `boot_chime.wav`, `boot_chime.mp3` = `macintosh-g3.mp3`). No system alert sounds, no UI feedback sounds. |
| Cursor theme | ✅ Real | Proper Xcursor theme structure exists. |
| Fonts | ✅ Real | Inter, JetBrains Mono, Panamera, Young Serif bundled. |
| Icons | ⚠️ Partial | 261 sidebar SVGs, 8 panel SVGs, 6 dock SVGs. **But no app-specific icons, no mimetype icons populated, no status icons.** |

---

## The Biggest Problems (In Priority Order)

### 1. 🔴 No Actual Rendering Pipeline
The entire codebase **cannot draw a single pixel on screen**. There are:
- Zero Vulkan API calls
- Zero wlroots/libwayland calls  
- Zero OpenGL/EGL calls
- Zero framebuffer operations
- Zero shader files on disk

This means **everything from the boot animation to the shutdown screen is a Rust struct that prints to `tracing::info!()`**. On real hardware, the user would see a black screen (or whatever the Linux TTY shows).

### 2. 🔴 No Wayland Compositor
The compositor is not a compositor. It's a state machine. A real Wayland compositor needs:
- `wl_display_create()` → `wlr_backend_autocreate()` → `wlr_renderer_autocreate()`
- `wlr_scene` for surface management
- `wlr_xdg_shell` for window management
- `wlr_seat` for input
- Frame callbacks for vsync

None of this exists. The `animus-compositor` crate would need to be rewritten in C or use `smithay` (Rust Wayland compositor library).

### 3. 🔴 No Process Management
Native apps are Rust structs instantiated in-process. On a real OS:
- Filer needs to be a standalone process with a Wayland client surface
- Terminow needs `openpty()`/`forkpty()` to spawn a real shell
- Settings needs D-Bus calls to NetworkManager, BlueZ, PipeWire, logind
- Zen Browser needs to spawn a real `zen-browser` process

### 4. 🟡 No System Integration
- No actual PipeWire audio
- No actual NetworkManager Wi-Fi
- No actual BlueZ Bluetooth
- No actual logind session management
- No actual PolicyKit elevation
- No actual dconf/gsettings persistence

### 5. 🟡 No OOTB Package Manifest
The spec says 8-10GB ISO with everything pre-installed. There's no manifest of what "everything" includes — no list of codecs, drivers, firmware, development tools, productivity apps, etc.

---

## What's Actually Good and Real

| Component | Why It's Legit |
|-----------|---------------|
| Spring physics engine | Mathematically correct damped harmonic oscillator with 8 named profiles |
| HEV encryption vault | Real AES-256-GCM + Argon2id, not toy crypto |
| CrashManager blast radius | Genuine BFS isolation algorithm with rate limiting |
| EventBus | Real `crossbeam-channel` pub/sub, actually wired throughout |
| Squircle SDF | Correct $G_2$ continuous curvature math |
| OKLab color system | Real perceptual color space conversions |
| GlyphAtlas rasterization | Real `fontdue` glyph rasterization with gamma correction |
| Filer filesystem ops | Real `std::fs` operations on actual disk |
| Icon/Font/Cursor assets | Real files that would ship in a real OS |
| Installer disk scanning | Real `/sys/block/` enumeration (on Linux) |

---

## Realistic Effort Estimate

| Milestone | Estimated Effort | Description |
|-----------|-----------------|-------------|
| **Bootable ISO that shows a wallpaper** | ~3-4 months (2 devs) | Requires real wlroots/smithay compositor, Vulkan pipeline, and ISO builder |
| **Lock screen + login that works** | +1-2 months | PAM integration, session management |
| **Desktop with working Filer + Terminow** | +2-3 months | PTY handling, drag-and-drop, thumbnail generation |
| **Full native app suite** | +3-4 months | Settings with real D-Bus, Zen Browser integration, Pathfinder with real indexing |
| **macOS-level polish** | +6-12 months | Animations at 144Hz, zero flicker, sound design, accessibility, multi-monitor |
| **OOTB "just works"** | +3-6 months | Driver/codec/firmware packaging, hardware testing, OTA updates |

**Total realistic estimate to macOS-grade**: **18-30 months** with a small dedicated team.

---

## Summary

The codebase has **excellent architecture and design** — the spec is thorough, the data structures are well-organized, the spring physics are mathematically sound, and the crash isolation system is genuinely clever. But it's an **architecture without an engine**. It's like having a complete car blueprint, a working ignition key (HEV crypto), a beautifully designed dashboard (shell components), and a working horn (EventBus) — but no engine block, no transmission, no wheels.

The 54 passing tests verify that the Rust structs' internal state machines work correctly. But they don't verify that anything can actually be seen, heard, or interacted with by a human user.
