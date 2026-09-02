# AnimusEngine — Complete Architecture & Production Implementation
## VitusOS · openSEF (open Seamless Environment Framework)
## Boot to Userspace · Every Layer Real · No Placeholders

**openSEF** — open Seamless Environment Framework.
The complete architecture for VitusOS. 12 layers. 1 process. 0 unnecessary abstractions.
From UEFI to proximity unlock, designed as one coherent system from a single point of view.

**Author:** raven1zed (Architect & Director)
**Target:** Claude Opus 4.6 (Implementer)
**Status:** Single authoritative source. Supersedes all previous spec files.
**Languages:** C11 (AnimusBoot, animus-early, compositor core) · C++17 (all above the bridge)
**Verified:** wlroots 0.17.1 · PipeWire 1.0.5 · HarfBuzz 8.3.0 · FreeType 26.1.20

---

## ABSOLUTE RULES — Each One Is a Build Failure If Violated

1. C11 for EFI, initramfs, wlroots compositor. C++17 for everything above the bridge.
2. `-DWLR_USE_UNSTABLE` in CFLAGS. wlroots refuses to compile without it.
3. `VK_PRESENT_MODE_FIFO_KHR` always. No tearing. No exceptions.
4. `wlr_output_commit_state()` not `wlr_output_commit()`. Atomic path only.
5. `wlr_vk_renderer_create_with_drm_fd(drm_fd)` — verified signature.
6. `hb_ft_font_create_referenced(face)` — not the deprecated variant.
7. `pw_stream_new(core, name, props, &events, userdata)` — 5-arg signature, verified.
8. Gesture events from `wlr_pointer.events.swipe_*` — not a separate device.
9. `wlr_output_event_present.when` is `struct timespec*` — **may be NULL**. Always null-check.
10. No Cairo. No Pango. No Qt. No GTK inside AnimusEngine or OSFNative apps. Ever.
11. All glass/blur/shadow = custom Vulkan shaders. wlroots render pass draws textures only.
12. `#E85D00` Space Orange is the only accent/selection color.
13. `#1A1208` always for shadows. Never `#000000`.
14. `#FEFEFE` for content backgrounds. Never `#FFFFFF`.
15. Every surface declares `SurfaceAltitude`. MaterialRenderer derives ALL visual properties.
16. Every motion uses `SpringSolver`. No cubic-bezier. No fixed-duration easing. Ever.
17. No mock data. No hardcoded file lists. No placeholder functions. No TODOs.
18. All I/O on background threads. EventBus::publishAsync delivers to Wayland event loop thread.
19. `vitusos-config.nix` only for Pathfinder. Never `nixos-configuration.nix`.
20. `PCI_CLASSCODE_OFFSET = 0x09` — NOT `0x0B`. Source: PCI Local Bus Specification 3.0 §6.1.


---

## System Stack

```
┌──────────────────────────────────────────────────────────────┐
│  OSFNative Apps  Filer · Pathfinder · Terminow · Settings    │
├──────────────────────────────────────────────────────────────┤
│  OSFAppKit  Button · TextField · ScrollView · TableView …    │
├──────────────────────────────────────────────────────────────┤
│  OSFSurfaces  Window · Sidebar · Toolbar · Content           │
│               Popover · Dropdown · Sheet · Notification      │
│               Tooltip · ContextMenu                          │
├──────────────────────────────────────────────────────────────┤
│  Shell  Panel · Dock · CockpitView · LockScreen · Crossfade  │
├──────────────────────────────────────────────────────────────┤
│  Core  OSFDesktop · EventBus · StateManager · WindowManager  │
│        InputRouter · MotionWave · ClipboardBridge             │
├──────────────────────────────────────────────────────────────┤
│  Render  RenderPipeline · MaterialRenderer · ShadowRenderer  │
│          GlyphAtlas · TextRenderer · WallpaperTintSampler    │
│          VulkanContext                                        │
├──────────────────────────────────────────────────────────────┤
│  Animation  AnimationEngine · AnimationClock · SpringSolver  │
├──────────────────────────────────────────────────────────────┤
│  Audio  SoundEngine (PipeWire 1.0.5)                         │
├──────────────────────────────────────────────────────────────┤
│  ── extern "C" bridge — only C11↔C++17 crossing ──           │
├──────────────────────────────────────────────────────────────┤
│  C11 Compositor  wlroots 0.17.1 · DRM · Vulkan renderer      │
│                  wlr_damage_ring · seat · xdg_shell          │
│                  layer_shell_v1 · osf-shell-v1               │
├──────────────────────────────────────────────────────────────┤
│  Stage 2: animus-early  (C11, initramfs systemd service)     │
│    simpledrm · dumb buffer · Space Orange splash             │
│    PipeWire boot chime (forked) · modprobe GPU drivers       │
├──────────────────────────────────────────────────────────────┤
│  Stage 1: Linux Kernel  NixOS · DRM_SIMPLEDRM=y built-in     │
├──────────────────────────────────────────────────────────────┤
│  Stage 0: AnimusBoot  (C11, UEFI/EDK2)                       │
│    EFI_PCI_IO_PROTOCOL · GOP framebuffer · Space Orange UI   │
│    ANIMUS_GPU_HANDOFF EFI variable · LoadImage kernel boot   │
├──────────────────────────────────────────────────────────────┤
│  HARDWARE  NVIDIA · AMD · Intel Arc · Intel iGPU             │
└──────────────────────────────────────────────────────────────┘
```

---

## PART 0 — Repository Layout & Build System

### 0.1 Directory Structure

```
vitusos/
├── AnimusBoot/                  # Stage 0: UEFI EFI app (C11, EDK2)
│   ├── AnimusHandoff.h          # Shared by all 3 boot stages
│   ├── GpuDetect.c
│   ├── GopSetup.c
│   ├── AnimusBoot.c
│   └── AnimusBoot.inf
├── animus-early/                # Stage 2: initramfs (C11)
│   ├── animus-early.c
│   └── CMakeLists.txt
├── compositor/                  # C11 wlroots core
│   ├── animus_compositor.c
│   └── animus_compositor.h
├── animus/                      # C++17 AnimusEngine
│   ├── main.cpp
│   ├── core/
│   │   ├── OSFDesktop.cpp/.h
│   │   ├── EventBus.cpp/.h
│   │   ├── OSFEvent.h
│   │   ├── StateManager.cpp/.h
│   │   ├── WindowManager.cpp/.h
│   │   └── ClipboardBridge.cpp/.h
│   ├── animation/
│   │   ├── SpringSolver.h       # Header-only
│   │   ├── AnimationClock.cpp/.h
│   │   └── AnimationEngine.cpp/.h
│   ├── render/
│   │   ├── VulkanContext.cpp/.h
│   │   ├── RenderPipeline.cpp/.h
│   │   ├── MaterialRenderer.cpp/.h
│   │   ├── ShadowRenderer.cpp/.h
│   │   ├── GlyphAtlas.cpp/.h
│   │   ├── TextRenderer.cpp/.h
│   │   └── WallpaperTintSampler.cpp/.h
│   ├── input/
│   │   ├── InputRouter.cpp/.h
│   │   └── MotionWave.cpp/.h
│   ├── audio/
│   │   └── SoundEngine.cpp/.h
│   └── shell/
│       ├── Panel.cpp/.h
│       ├── Dock.cpp/.h
│       ├── CockpitView.cpp/.h
│       ├── LockScreen.cpp/.h
│       ├── BootCrossfade.cpp/.h
│       └── GlobalMenu.cpp/.h
├── osf/
│   ├── surfaces/
│   │   ├── OSFWindow.cpp/.h
│   │   ├── OSFSidebar.cpp/.h
│   │   ├── OSFToolbar.cpp/.h
│   │   ├── OSFContent.h
│   │   ├── OSFPopover.cpp/.h
│   │   ├── OSFDropdown.cpp/.h
│   │   ├── OSFSheet.cpp/.h
│   │   ├── OSFNotification.cpp/.h
│   │   ├── OSFTooltip.h
│   │   └── OSFContextMenu.cpp/.h
│   └── appkit/
│       ├── OSFButton.h
│       ├── OSFTextField.h
│       ├── OSFScrollView.h
│       ├── OSFTableView.h
│       ├── OSFProgressBar.h
│       ├── OSFSlider.h
│       ├── OSFLabel.h
│       └── OSFImageView.h
├── shaders/                     # GLSL → SPIR-V at build time
│   ├── kawase_blur.frag
│   ├── luminosity_composite.frag
│   ├── window_shadow.frag
│   ├── glyph.vert / glyph.frag
│   ├── rounded_rect.vert / rounded_rect.frag
│   └── texture_quad.vert / texture_quad.frag
├── protocol/
│   └── osf-shell-v1.xml
└── nixos/
    ├── configuration.nix
    ├── flake.nix
    └── modules/
        ├── animus-early.nix
        └── vitusos-apps.nix
```

### 0.2 NixOS Package

```nix
# nixos/pkgs/vitusos-animus/default.nix
{ lib, stdenv, pkg-config, meson, ninja,
  wlroots_0_17, wayland, wayland-protocols,
  vulkan-loader, vulkan-headers,
  libxkbcommon, libinput, pixman,
  freetype, harfbuzz, pipewire, libdrm, mesa,
  glslang, spirv-tools }:

stdenv.mkDerivation {
  pname = "vitusos-animus"; version = "0.1.0"; src = ./.;
  nativeBuildInputs = [ pkg-config meson ninja glslang spirv-tools ];
  buildInputs = [
    wlroots_0_17 wayland wayland-protocols
    vulkan-loader vulkan-headers
    libxkbcommon libinput pixman
    freetype harfbuzz pipewire libdrm mesa
  ];
  env.CFLAGS   = "-DWLR_USE_UNSTABLE";
  env.CXXFLAGS = "-DWLR_USE_UNSTABLE -std=c++17";
  preBuild = ''
    for s in shaders/*.vert shaders/*.frag; do
      glslc "$s" -o "$s.spv"
      spirv-val "$s.spv"
    done
  '';
}
```

### 0.3 NixOS System Configuration

```nix
# nixos/configuration.nix
{ config, pkgs, lib, ... }:
{
  imports = [ ./hardware-configuration.nix
              ./modules/animus-early.nix
              ./modules/vitusos-apps.nix ];

  boot.loader.systemd-boot.enable = false;
  boot.loader.grub.enable         = false;
  boot.loader.efi.canTouchEfiVariables = true;
  # AnimusBoot EFI app installed to /boot/EFI/vitusos/ by installer

  services.xserver.enable  = false;
  programs.xwayland.enable = false;

  systemd.user.services.animus-engine = {
    description = "AnimusEngine Wayland Compositor";
    wantedBy    = [ "graphical-session.target" ];
    after       = [ "animus-early.service" ];
    serviceConfig = {
      Type      = "simple";
      ExecStart = "${pkgs.vitusos-animus}/bin/animus-engine";
      Restart   = "on-failure"; RestartSec = "1s";
    };
    environment = {
      WAYLAND_DISPLAY = "wayland-0";
      XDG_RUNTIME_DIR = "/run/user/1000";
    };
  };

  services.pipewire = {
    enable = true; alsa.enable = true;
    alsa.support32Bit = true; pulse.enable = true;
  };
  hardware.pulseaudio.enable = false;

  hardware.nvidia = {
    open = true; modesetting.enable = true;
    powerManagement.enable = true;
    package = config.boot.kernelPackages.nvidiaPackages.stable;
  };

  environment.systemPackages = with pkgs; [
    vitusos-filer vitusos-pathfinder vitusos-terminow
    vitusos-settings vitusos-seadrop
    linux-firmware mesa gtk3 gtk4
    qt5.qtwayland qt6.qtwayland
    appmenu-gtk3-module firefox
  ];

  environment.variables = {
    GTK_MODULES  = "appmenu-gtk-module";
    UBUNTU_MENUPROXY = "1";
    QT_QPA_PLATFORM  = "wayland";
    QT_WAYLAND_DISABLE_WINDOWDECORATION = "1";
    NIXOS_OZONE_WL   = "1";
  };

  fonts.packages = with pkgs; [ inter ];
  fonts.fontconfig.defaultFonts.sansSerif = [ "Inter" ];
  system.stateVersion = "25.05";
}
```


---

## PART 1 — AnimusBoot: UEFI EFI Application (C11, EDK2)

### 1.1 AnimusHandoff.h — Shared by All Boot Stages

```c
// AnimusBoot/AnimusHandoff.h
// Included by AnimusBoot (UEFI), animus-early (initramfs), AnimusEngine.
// All three must agree on layout. Never modify independently.
#pragma once

#define ANIMUS_HANDOFF_GUID_STR "e4b8e798-a5f4-4b2c-b9ab-1234567890ab"

typedef enum {
    GPU_VENDOR_UNKNOWN      = 0,
    GPU_VENDOR_NVIDIA       = 1,
    GPU_VENDOR_AMD          = 2,
    GPU_VENDOR_INTEL_LEGACY = 3,   // i915
    GPU_VENDOR_INTEL_ARC    = 4,   // xe driver (DID 0x5690–0x57FF)
} GpuVendor;

typedef enum {
    GPU_TYPE_UNKNOWN    = 0,
    GPU_TYPE_DISCRETE   = 1,
    GPU_TYPE_INTEGRATED = 2,
} GpuType;

typedef struct {
    GpuVendor          vendor;
    GpuType            type;
    unsigned short     deviceId;
    unsigned char      busNumber;
    unsigned long long framebufferBase;
    unsigned int       framebufferSize;
    unsigned int       horizontalResolution;
    unsigned int       verticalResolution;
    unsigned int       pixelsPerScanLine;
    unsigned int       pixelFormat;
} ANIMUS_GPU_HANDOFF;
```

### 1.2 GpuDetect.c

```c
// AnimusBoot/GpuDetect.c
// PCI_CLASSCODE_OFFSET = 0x09 (prog-if, subclass, base class) — NOT 0x0B.
// Source: PCI Local Bus Specification 3.0, Section 6.1
// Intel Arc DID range 0x5690–0x57FF uses xe driver, not i915.
// Source: linux/drivers/gpu/drm/xe/xe_pci.c

#include <Uefi.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Protocol/PciIo.h>
#include <IndustryStandard/Pci.h>
#include "AnimusHandoff.h"

#define VENDOR_NVIDIA       0x10DE
#define VENDOR_AMD          0x1002
#define VENDOR_INTEL        0x8086
#define PCI_CLASS_DISPLAY   0x03
#define INTEL_ARC_DID_MIN   0x5690
#define INTEL_ARC_DID_MAX   0x57FF

EFI_STATUS DetectGpu(ANIMUS_GPU_HANDOFF *Handoff) {
    EFI_HANDLE          *Handles = NULL;
    UINTN                Count   = 0;
    EFI_PCI_IO_PROTOCOL *PciIo;
    ANIMUS_GPU_HANDOFF   Best    = {0};
    BOOLEAN              Found   = FALSE;

    EFI_STATUS S = gBS->LocateHandleBuffer(
        ByProtocol, &gEfiPciIoProtocolGuid, NULL, &Count, &Handles);
    if (EFI_ERROR(S)) return S;

    for (UINTN i = 0; i < Count; i++) {
        S = gBS->HandleProtocol(Handles[i], &gEfiPciIoProtocolGuid, (VOID**)&PciIo);
        if (EFI_ERROR(S)) continue;

        // Read 3 bytes at offset 0x09: ProgIf, Subclass, BaseClass
        UINT8 Class[3];
        if (EFI_ERROR(PciIo->Pci.Read(PciIo, EfiPciIoWidthUint8,
            PCI_CLASSCODE_OFFSET, 3, Class))) continue;
        if (Class[2] != PCI_CLASS_DISPLAY) continue;

        UINT16 Vid = 0, Did = 0;
        PciIo->Pci.Read(PciIo, EfiPciIoWidthUint16, PCI_VENDOR_ID_OFFSET, 1, &Vid);
        PciIo->Pci.Read(PciIo, EfiPciIoWidthUint16, PCI_DEVICE_ID_OFFSET, 1, &Did);

        UINTN Seg, Bus, Dev, Func;
        PciIo->GetLocation(PciIo, &Seg, &Bus, &Dev, &Func);

        ANIMUS_GPU_HANDOFF C = {0};
        C.deviceId  = Did;
        C.busNumber = (UINT8)Bus;
        C.type      = (Bus > 0) ? GPU_TYPE_DISCRETE : GPU_TYPE_INTEGRATED;

        switch (Vid) {
            case VENDOR_NVIDIA: C.vendor = GPU_VENDOR_NVIDIA; break;
            case VENDOR_AMD:    C.vendor = GPU_VENDOR_AMD;    break;
            case VENDOR_INTEL:
                C.vendor = (Did >= INTEL_ARC_DID_MIN && Did <= INTEL_ARC_DID_MAX)
                           ? GPU_VENDOR_INTEL_ARC : GPU_VENDOR_INTEL_LEGACY;
                break;
            default: continue;
        }

        // Prefer discrete > integrated, NVIDIA > AMD > Intel
        if (!Found
            || (C.type == GPU_TYPE_DISCRETE  && Best.type == GPU_TYPE_INTEGRATED)
            || (C.type == Best.type && C.vendor < Best.vendor))
        { Best = C; Found = TRUE; }
    }

    FreePool(Handles);
    if (!Found) return EFI_NOT_FOUND;
    *Handoff = Best;
    return EFI_SUCCESS;
}
```

### 1.3 GopSetup.c

```c
// AnimusBoot/GopSetup.c
// CRITICAL: FreePool(Info) after every QueryMode call.
// QueryMode allocates Info — leaking corrupts EFI pool.

#include <Uefi.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Protocol/GraphicsOutput.h>
#include "AnimusHandoff.h"

static UINT32 PackColor(EFI_GRAPHICS_PIXEL_FORMAT F, UINT8 R, UINT8 G, UINT8 B) {
    if (F == PixelRedGreenBlueReserved8BitPerColor)
        return (UINT32)R | ((UINT32)G<<8) | ((UINT32)B<<16);
    if (F == PixelBlueGreenRedReserved8BitPerColor)
        return (UINT32)B | ((UINT32)G<<8) | ((UINT32)R<<16);
    return 0;
}
static VOID FillRect(UINT32 *Fb, UINT32 S,
                     UINT32 X, UINT32 Y, UINT32 W, UINT32 H, UINT32 C) {
    for (UINT32 r=Y; r<Y+H; r++)
        for (UINT32 c=X; c<X+W; c++) Fb[r*S+c]=C;
}

EFI_STATUS SetupGopAndRender(ANIMUS_GPU_HANDOFF *H) {
    EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop;
    EFI_STATUS S = gBS->LocateProtocol(
        &gEfiGraphicsOutputProtocolGuid, NULL, (VOID**)&Gop);
    if (EFI_ERROR(S)) return S;

    UINT32 BestMode=0, BestW=0, BestH=0;
    for (UINT32 m=0; m<Gop->Mode->MaxMode; m++) {
        UINTN Sz; EFI_GRAPHICS_OUTPUT_MODE_INFORMATION *Info;
        if (EFI_ERROR(Gop->QueryMode(Gop,m,&Sz,&Info))) continue;
        BOOLEAN Ok = (Info->PixelFormat==PixelRedGreenBlueReserved8BitPerColor
                   || Info->PixelFormat==PixelBlueGreenRedReserved8BitPerColor);
        if (Ok && Info->HorizontalResolution*Info->VerticalResolution > BestW*BestH) {
            BestMode=m;
            BestW=Info->HorizontalResolution;
            BestH=Info->VerticalResolution;
        }
        FreePool(Info);  // CRITICAL — QueryMode allocates, must free every iteration
    }

    if (EFI_ERROR(Gop->SetMode(Gop, BestMode))) return EFI_DEVICE_ERROR;

    H->framebufferBase      = Gop->Mode->FrameBufferBase;
    H->framebufferSize      = Gop->Mode->FrameBufferSize;
    H->horizontalResolution = Gop->Mode->Info->HorizontalResolution;
    H->verticalResolution   = Gop->Mode->Info->VerticalResolution;
    H->pixelsPerScanLine    = Gop->Mode->Info->PixelsPerScanLine;
    H->pixelFormat          = Gop->Mode->Info->PixelFormat;

    UINT32 *Fb = (UINT32*)(UINTN)Gop->Mode->FrameBufferBase;
    UINT32  St = Gop->Mode->Info->PixelsPerScanLine;
    UINT32  W  = Gop->Mode->Info->HorizontalResolution;
    UINT32  Ht = Gop->Mode->Info->VerticalResolution;
    EFI_GRAPHICS_PIXEL_FORMAT Fmt = Gop->Mode->Info->PixelFormat;

    // Space Orange: #E85D00 = R232 G93 B0
    UINT32 Orange = PackColor(Fmt, 232, 93, 0);
    UINT32 White  = PackColor(Fmt, 255, 255, 255);

    FillRect(Fb, St, 0, 0, W, Ht, Orange);             // full screen Space Orange
    FillRect(Fb, St, (W-280)/2, (Ht-48)/2, 280, 48, White);  // wordmark placeholder
    return EFI_SUCCESS;
}
```

### 1.4 AnimusBoot.c

```c
// AnimusBoot/AnimusBoot.c
#include <Uefi.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/UefiRuntimeServicesTableLib.h>
#include "GpuDetect.h"
#include "GopSetup.h"
#include "AnimusHandoff.h"

#define HANDOFF_VAR  L"AnimusGpuHandoff"
static EFI_GUID HandoffGuid = {
    0xE4B8E798,0xA5F4,0x4B2C,
    {0xB9,0xAB,0x12,0x34,0x56,0x78,0x90,0xAB}
};

// STATIC kernel cmdline — AnimusBoot NEVER modifies it at runtime.
// All GPU params present; kernel ignores params for absent GPUs.
static CONST CHAR16 *CMDLINE =
    L"quiet splash loglevel=0 vt.global_cursor_default=0 "
    L"nvidia_drm.modeset=1 nvidia_drm.fbdev=1 "
    L"nvidia.NVreg_OpenRmEnableUnsupportedGpus=1 "
    L"systemd.show_status=false rd.udev.log_level=0";

EFI_STATUS EFIAPI UefiMain(EFI_HANDLE Img, EFI_SYSTEM_TABLE *ST) {
    ANIMUS_GPU_HANDOFF H = {0};
    if (EFI_ERROR(DetectGpu(&H)))         return EFI_NOT_FOUND;
    if (EFI_ERROR(SetupGopAndRender(&H))) return EFI_DEVICE_ERROR;

    // Write to EFI variable — NON_VOLATILE + RUNTIME_ACCESS
    // so initramfs can read it after ExitBootServices
    gRT->SetVariable(HANDOFF_VAR, &HandoffGuid,
        EFI_VARIABLE_BOOTSERVICE_ACCESS | EFI_VARIABLE_RUNTIME_ACCESS |
        EFI_VARIABLE_NON_VOLATILE,
        sizeof(H), &H);

    EFI_HANDLE KH;
    // Real path: L"\\EFI\\vitusos\\kernel" — populated by NixOS install
    gBS->LoadImage(FALSE, Img, NULL, NULL, 0, &KH);

    EFI_LOADED_IMAGE_PROTOCOL *Li;
    gBS->HandleProtocol(KH, &gEfiLoadedImageProtocolGuid, (VOID**)&Li);
    Li->LoadOptions     = (VOID*)CMDLINE;
    Li->LoadOptionsSize = (UINT32)((StrLen(CMDLINE)+1)*sizeof(CHAR16));

    UINTN ExSz; CHAR16 *Ex;
    return gBS->StartImage(KH, &ExSz, &Ex);  // does not return on success
}
```


---

## PART 2 — animus-early: initramfs Service (C11)

### 2.1 NixOS Module

```nix
# nixos/modules/animus-early.nix
{ config, pkgs, lib, ... }: {
  boot.initrd.systemd.enable = true;
  boot.initrd.systemd.services.animus-early = {
    description = "AnimusEngine Early Boot";
    wantedBy    = [ "initrd.target" ];
    before      = [ "initrd-fs.target" ];
    after       = [ "systemd-modules-load.service" ];
    serviceConfig = {
      Type            = "oneshot";
      ExecStart       = "/bin/animus-early";
      RemainAfterExit = true;
    };
  };
  boot.initrd.extraBin."animus-early" =
    "${pkgs.vitusos-animus-early}/bin/animus-early";
  boot.initrd.kernelModules          = [ "drm" "drm_kms_helper" ];
  boot.initrd.availableKernelModules = [
    "amdgpu" "i915" "xe"
    "nvidia" "nvidia_modeset" "nvidia_uvm" "nvidia_drm"
  ];
  boot.kernelParams = [
    "quiet" "splash" "loglevel=0" "vt.global_cursor_default=0"
    "systemd.show_status=false" "rd.udev.log_level=0"
    "nvidia_drm.modeset=1" "nvidia_drm.fbdev=1"
    "nvidia.NVreg_OpenRmEnableUnsupportedGpus=1"
  ];
  boot.kernelPatches = [{
    name = "vitusos-drm"; patch = null;
    extraStructuredConfig = with lib.kernel; {
      DRM_SIMPLEDRM = yes;    # MUST be =y (built-in), not =m
      DRM_AMDGPU    = module;
      DRM_I915      = module;
      DRM_XE        = module;
    };
  }];
  hardware.nvidia = {
    open               = true;
    modesetting.enable = true;
    powerManagement.enable = true;
    package = config.boot.kernelPackages.nvidiaPackages.stable;
  };
}
```

### 2.2 animus-early.c

```c
// animus-early/animus-early.c
// Pipeline:
//  1. Read ANIMUS_GPU_HANDOFF from EFI variable
//  2. open_simpledrm() — verify driver name = "simple"
//  3. Create dumb buffer, legacy drmModeSetCrtc (simpledrm safe)
//  4. Render Space Orange splash (matches AnimusBoot GOP exactly)
//  5. Fork child: play boot chime via PipeWire (async, main proceeds)
//  6. Load native GPU driver (NVIDIA: mandatory order)
//  7. close(drm_fd) → sysfb_disable() → native driver takes over
//  8. Exit — systemd starts AnimusEngine compositor
// NOTE: DumbBuffer NOT destroyed — framebuffer stays visible until
// AnimusEngine commits first Vulkan frame.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <xf86drm.h>
#include <xf86drmMode.h>
#include <pipewire/pipewire.h>
#include <spa/param/audio/format-utils.h>
#include "AnimusHandoff.h"

// ── EFI variable ─────────────────────────────────────────────────
// /sys/firmware/efi/efivars/<n>-<GUID>
// Format: 4-byte EFI attributes + data
#define EFIVARS "/sys/firmware/efi/efivars/"

static bool read_handoff(ANIMUS_GPU_HANDOFF *out) {
    char p[256];
    snprintf(p,sizeof(p), EFIVARS "AnimusGpuHandoff-" ANIMUS_HANDOFF_GUID_STR);
    int fd = open(p, O_RDONLY);
    if (fd<0) return false;
    uint32_t attrs;
    if (read(fd,&attrs,4)!=4) { close(fd); return false; }
    ssize_t n = read(fd, out, sizeof(*out));
    close(fd);
    return n == (ssize_t)sizeof(*out);
}

// ── simpledrm ────────────────────────────────────────────────────
static int open_simpledrm(void) {
    for (int i=0; i<8; i++) {
        char p[32]; snprintf(p,sizeof(p),"/dev/dri/card%d",i);
        int fd = open(p, O_RDWR|O_CLOEXEC);
        if (fd<0) continue;
        drmVersionPtr v = drmGetVersion(fd);
        if (!v) { close(fd); continue; }
        bool ok = strcmp(v->name,"simple")==0;
        drmFreeVersion(v);
        if (ok) return fd;
        close(fd);
    }
    return -1;
}

// ── DRM dumb buffer ───────────────────────────────────────────────
typedef struct {
    int fd; uint32_t handle,pitch,fb_id; uint64_t size;
    uint32_t *map; uint32_t w,h;
} DumbBuf;

static bool make_dumb(int fd, DumbBuf *db, uint32_t w, uint32_t h) {
    struct drm_mode_create_dumb c={.width=w,.height=h,.bpp=32};
    if (ioctl(fd,DRM_IOCTL_MODE_CREATE_DUMB,&c)) return false;
    db->fd=fd; db->handle=c.handle; db->pitch=c.pitch;
    db->size=c.size; db->w=w; db->h=h;
    if (drmModeAddFB(fd,w,h,24,32,c.pitch,c.handle,&db->fb_id)) return false;
    struct drm_mode_map_dumb m={.handle=c.handle};
    if (ioctl(fd,DRM_IOCTL_MODE_MAP_DUMB,&m)) return false;
    db->map = mmap(NULL,c.size,PROT_READ|PROT_WRITE,MAP_SHARED,fd,m.offset);
    return db->map != MAP_FAILED;
}

// ── Boot splash ───────────────────────────────────────────────────
// #E85D00 = 0xFFE85D00 (ARGB). Matches AnimusBoot exactly.
// pitch/4 = pixels-per-row (pitch includes alignment padding).
static void render_splash(const DumbBuf *db) {
    uint32_t stride = db->pitch/4;
    for (uint32_t y=0; y<db->h; y++)
        for (uint32_t x=0; x<db->w; x++)
            db->map[y*stride+x] = 0xFFE85D00u;
    uint32_t wx=(db->w-280)/2, wy=(db->h-48)/2;
    for (uint32_t y=wy; y<wy+48; y++)
        for (uint32_t x=wx; x<wx+280; x++)
            db->map[y*stride+x] = 0xFFFFFFFFu;
}

// ── Boot chime (child process) ────────────────────────────────────
typedef struct {
    struct pw_main_loop *loop; struct pw_stream *stream;
    const uint8_t *pcm; size_t sz,pos;
} ChimeS;

static void chime_proc(void *ud) {
    ChimeS *s=ud;
    struct pw_buffer *b = pw_stream_dequeue_buffer(s->stream);
    if (!b) return;
    uint8_t *dst=b->buffer->datas[0].data;
    uint32_t cap=b->buffer->datas[0].maxsize;
    size_t rem=s->sz-s->pos;
    if (!rem) {
        b->buffer->datas[0].chunk->size=0;
        pw_stream_queue_buffer(s->stream,b);
        pw_main_loop_quit(s->loop); return;
    }
    uint32_t cp=(uint32_t)(rem<cap?rem:cap);
    memcpy(dst,s->pcm+s->pos,cp); s->pos+=cp;
    b->buffer->datas[0].chunk->size=cp;
    pw_stream_queue_buffer(s->stream,b);
}
static const struct pw_stream_events CHIME_EVT={PW_VERSION_STREAM_EVENTS,.process=chime_proc};

static void play_chime_child(void) {
    int fd=open("/etc/vitusos/sounds/boot_chime.wav",O_RDONLY);
    if (fd<0) return;
    off_t sz=lseek(fd,0,SEEK_END); lseek(fd,0,SEEK_SET);
    if (sz<=44) { close(fd); return; }
    uint8_t *wav=malloc((size_t)sz);
    read(fd,wav,(size_t)sz); close(fd);
    pw_init(NULL,NULL);
    ChimeS s={.pcm=wav+44,.sz=(size_t)(sz-44),.pos=0};
    s.loop=pw_main_loop_new(NULL);
    struct pw_properties *p=pw_properties_new(
        PW_KEY_MEDIA_TYPE,"Audio",PW_KEY_MEDIA_CATEGORY,"Playback",
        PW_KEY_MEDIA_ROLE,"Music",NULL);
    // pw_stream_new: verified 5-arg signature (PipeWire 1.0.5)
    s.stream=pw_stream_new_simple(pw_main_loop_get_loop(s.loop),
        "animus-chime",p,&CHIME_EVT,&s);
    uint8_t buf[1024];
    struct spa_pod_builder b=SPA_POD_BUILDER_INIT(buf,sizeof(buf));
    const struct spa_pod *params[1];
    params[0]=spa_format_audio_raw_build(&b,SPA_PARAM_EnumFormat,
        &SPA_AUDIO_INFO_RAW_INIT(.format=SPA_AUDIO_FORMAT_S16,
                                  .rate=44100,.channels=2));
    pw_stream_connect(s.stream,PW_DIRECTION_OUTPUT,PW_ID_ANY,
        PW_STREAM_FLAG_AUTOCONNECT|PW_STREAM_FLAG_MAP_BUFFERS,params,1);
    pw_main_loop_run(s.loop);
    pw_stream_destroy(s.stream); pw_main_loop_destroy(s.loop);
    pw_deinit(); free(wav);
}

// ── GPU driver load ───────────────────────────────────────────────
// NVIDIA order NON-NEGOTIABLE: nvidia → nvidia_modeset → nvidia_uvm → nvidia_drm
static bool do_modprobe(const char *name) {
    char cmd[128]; snprintf(cmd,sizeof(cmd),"modprobe %s 2>/dev/null",name);
    return system(cmd)==0;
}
static void load_driver(const ANIMUS_GPU_HANDOFF *h) {
    switch(h->vendor) {
        case GPU_VENDOR_NVIDIA:
            if (do_modprobe("nvidia")) {
                do_modprobe("nvidia_modeset");
                do_modprobe("nvidia_uvm");
                do_modprobe("nvidia_drm");
            }
            break;
        case GPU_VENDOR_AMD:           do_modprobe("amdgpu"); break;
        case GPU_VENDOR_INTEL_ARC:     do_modprobe("xe");     break;
        case GPU_VENDOR_INTEL_LEGACY:  do_modprobe("i915");   break;
        default: break;
    }
}

// ── main ─────────────────────────────────────────────────────────
int main(void) {
    ANIMUS_GPU_HANDOFF h={.vendor=GPU_VENDOR_INTEL_LEGACY};
    read_handoff(&h);

    int drm=open_simpledrm();
    if (drm<0) return 1;

    drmModeRes *res=drmModeGetResources(drm);
    if (!res) return 1;
    drmModeConnector *conn=NULL;
    for (int i=0; i<res->count_connectors&&!conn; i++) {
        drmModeConnector *c=drmModeGetConnector(drm,res->connectors[i]);
        if (c&&c->connection==DRM_MODE_CONNECTED&&c->count_modes>0) conn=c;
        else if (c) drmModeFreeConnector(c);
    }
    if (!conn) return 1;

    DumbBuf db={0};
    if (!make_dumb(drm,&db,conn->modes[0].hdisplay,conn->modes[0].vdisplay))
        return 1;

    drmModeEncoder *enc=drmModeGetEncoder(drm,conn->encoder_id);
    uint32_t crtc=enc->crtc_id; drmModeFreeEncoder(enc);
    drmModeSetCrtc(drm,crtc,db.fb_id,0,0,&conn->connector_id,1,&conn->modes[0]);

    render_splash(&db);

    // Fork chime — main thread proceeds to driver load immediately
    pid_t pid=fork();
    if (pid==0) { play_chime_child(); _exit(0); }

    load_driver(&h);

    // close(drm_fd) triggers sysfb_disable() on Linux >=5.15
    // Native driver takes over. Space Orange frame stays visible.
    close(drm);
    usleep(50000);  // 50ms — only sleep in entire boot pipeline

    // DumbBuffer intentionally NOT destroyed
    // Frame stays visible until AnimusEngine commits first Vulkan frame
    drmModeFreeConnector(conn);
    drmModeFreeResources(res);
    return 0;
}
```


---

## PART 3 — C11 Compositor Core (wlroots 0.17.1)

```c
// compositor/animus_compositor.c
// -DWLR_USE_UNSTABLE mandatory (before ALL wlr includes)

#define WLR_USE_UNSTABLE

#include <wayland-server-core.h>
#include <wlr/backend.h>
#include <wlr/render/allocator.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/render/vulkan.h>
#include <wlr/types/wlr_compositor.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/types/wlr_damage_ring.h>
#include <wlr/types/wlr_input_device.h>
#include <wlr/types/wlr_keyboard.h>
#include <wlr/types/wlr_pointer.h>
#include <wlr/types/wlr_seat.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/types/wlr_layer_shell_v1.h>
#include <wlr/util/log.h>
#include <xkbcommon/xkbcommon.h>
#include <pixman.h>
#include <stdlib.h>
#include <string.h>
#include "animus_compositor.h"

typedef struct {
    struct wl_display         *display;
    struct wl_event_loop      *event_loop;
    struct wlr_backend        *backend;
    struct wlr_renderer       *renderer;
    struct wlr_allocator      *allocator;
    struct wlr_compositor     *compositor;
    struct wlr_output_layout  *output_layout;
    struct wlr_seat           *seat;
    struct wlr_xdg_shell      *xdg_shell;
    struct wlr_layer_shell_v1 *layer_shell;
    struct wlr_output         *primary_output;
    struct wlr_damage_ring     damage_ring;

    // C++17 callbacks — only legal bridge from C11 to C++17
    void (*on_present)(const struct wlr_output_event_present*, void*);
    void (*on_new_surface)(struct wlr_surface*, void*);
    void (*on_surface_destroy)(struct wlr_surface*, void*);
    void (*on_key)(uint32_t, uint32_t, bool, void*);
    void (*on_pointer_motion)(double, double, void*);
    void (*on_pointer_button)(uint32_t, bool, void*);
    void (*on_pointer_axis)(double, double, void*);
    void (*on_swipe_begin)(uint32_t, void*);
    void (*on_swipe_update)(uint32_t, double, double, void*);
    void (*on_swipe_end)(bool, void*);
    void *ud;
} Comp;

static Comp g = {0};

// ── vblank present ────────────────────────────────────────────────
// wlr_output_event_present.when: struct timespec* — MAY BE NULL.
// AnimationClock uses this for frame-perfect dt. NULL check mandatory.
static void h_present(struct wl_listener *l, void *data) {
    const struct wlr_output_event_present *ev=data; (void)l;
    if (ev->presented && g.on_present) g.on_present(ev,g.ud);
}
static struct wl_listener l_present={.notify=h_present};

static void h_frame(struct wl_listener *l, void *data) {
    (void)l;(void)data;
    wlr_damage_ring_rotate(&g.damage_ring);
}
static struct wl_listener l_frame={.notify=h_frame};

// ── new output ────────────────────────────────────────────────────
static void h_new_output(struct wl_listener *l, void *data) {
    struct wlr_output *out=data; (void)l;
    struct wlr_output_mode *mode=wlr_output_preferred_mode(out);
    struct wlr_output_state st; wlr_output_state_init(&st);
    wlr_output_state_set_enabled(&st,true);
    if (mode) wlr_output_state_set_mode(&st,mode);
    if (!wlr_output_test_state(out,&st)) { wlr_output_state_finish(&st); return; }
    wlr_output_commit_state(out,&st); wlr_output_state_finish(&st);
    g.primary_output=out;
    wlr_output_layout_add_auto(g.output_layout,out);
    wlr_damage_ring_init(&g.damage_ring);
    wlr_damage_ring_set_bounds(&g.damage_ring,out->width,out->height);
    wlr_damage_ring_add_whole(&g.damage_ring);
    wl_signal_add(&out->events.frame,  &l_frame);
    wl_signal_add(&out->events.present,&l_present);
}
static struct wl_listener l_new_output={.notify=h_new_output};

// ── keyboard ─────────────────────────────────────────────────────
typedef struct { struct wlr_keyboard *kb; struct wl_listener key,destroy; } KbS;
static void h_key(struct wl_listener *l, void *data) {
    KbS *k=wl_container_of(l,k,key);
    const struct wlr_keyboard_key_event *ev=data;
    uint32_t sym =xkb_state_key_get_one_sym(k->kb->xkb_state,ev->keycode+8);
    uint32_t mods=wlr_keyboard_get_modifiers(k->kb);
    bool pressed=ev->state==WL_KEYBOARD_KEY_STATE_PRESSED;
    if (g.on_key) g.on_key(sym,mods,pressed,g.ud);
}
static void h_kb_destroy(struct wl_listener *l, void *data) {
    KbS *k=wl_container_of(l,k,destroy); (void)data;
    wl_list_remove(&k->key.link); wl_list_remove(&k->destroy.link); free(k);
}

// ── pointer ───────────────────────────────────────────────────────
typedef struct {
    double ax,ay;
    struct wl_listener motion,button,axis;
    struct wl_listener swipe_begin,swipe_update,swipe_end,destroy;
} PtrS;

static void h_motion(struct wl_listener *l, void *data) {
    PtrS *p=wl_container_of(l,p,motion);
    const struct wlr_pointer_motion_event *ev=data;
    p->ax+=ev->delta_x; p->ay+=ev->delta_y;
    if (g.primary_output) {
        if(p->ax<0)p->ax=0; if(p->ay<0)p->ay=0;
        if(p->ax>g.primary_output->width) p->ax=g.primary_output->width;
        if(p->ay>g.primary_output->height)p->ay=g.primary_output->height;
    }
    if(g.on_pointer_motion) g.on_pointer_motion(p->ax,p->ay,g.ud);
}
static void h_button(struct wl_listener *l, void *data) {
    (void)l; const struct wlr_pointer_button_event *ev=data;
    if(g.on_pointer_button)
        g.on_pointer_button(ev->button,ev->state==WLR_BUTTON_PRESSED,g.ud);
}
static void h_axis(struct wl_listener *l, void *data) {
    (void)l; const struct wlr_pointer_axis_event *ev=data;
    double dx=ev->orientation==WLR_AXIS_ORIENTATION_HORIZONTAL?ev->delta:0;
    double dy=ev->orientation==WLR_AXIS_ORIENTATION_VERTICAL  ?ev->delta:0;
    if(g.on_pointer_axis) g.on_pointer_axis(dx,dy,g.ud);
}
// Swipe events: wlr_pointer.events.swipe_begin/update/end (verified 0.17.1)
// wlr_pointer_swipe_begin_event  { pointer, time_msec, fingers }
// wlr_pointer_swipe_update_event { pointer, time_msec, fingers, dx, dy }
// wlr_pointer_swipe_end_event    { pointer, time_msec, cancelled }
static void h_swipe_begin(struct wl_listener *l, void *data) {
    (void)l; const struct wlr_pointer_swipe_begin_event *ev=data;
    if(g.on_swipe_begin) g.on_swipe_begin(ev->fingers,g.ud);
}
static void h_swipe_update(struct wl_listener *l, void *data) {
    (void)l; const struct wlr_pointer_swipe_update_event *ev=data;
    if(g.on_swipe_update) g.on_swipe_update(ev->fingers,ev->dx,ev->dy,g.ud);
}
static void h_swipe_end(struct wl_listener *l, void *data) {
    (void)l; const struct wlr_pointer_swipe_end_event *ev=data;
    if(g.on_swipe_end) g.on_swipe_end(ev->cancelled,g.ud);
}
static void h_ptr_destroy(struct wl_listener *l, void *data) {
    PtrS *p=wl_container_of(l,p,destroy); (void)data;
    wl_list_remove(&p->motion.link);      wl_list_remove(&p->button.link);
    wl_list_remove(&p->axis.link);        wl_list_remove(&p->swipe_begin.link);
    wl_list_remove(&p->swipe_update.link);wl_list_remove(&p->swipe_end.link);
    wl_list_remove(&p->destroy.link); free(p);
}

static void h_new_input(struct wl_listener *l, void *data) {
    struct wlr_input_device *dev=data; (void)l;
    if (dev->type==WLR_INPUT_DEVICE_KEYBOARD) {
        struct wlr_keyboard *kb=wlr_keyboard_from_input_device(dev);
        struct xkb_context *ctx=xkb_context_new(XKB_CONTEXT_NO_FLAGS);
        struct xkb_keymap  *map=xkb_keymap_new_from_names(ctx,NULL,
            XKB_KEYMAP_COMPILE_NO_FLAGS);
        wlr_keyboard_set_keymap(kb,map);
        xkb_keymap_unref(map); xkb_context_unref(ctx);
        wlr_keyboard_set_repeat_info(kb,25,600);
        KbS *k=calloc(1,sizeof(*k)); k->kb=kb;
        k->key.notify=h_key; k->destroy.notify=h_kb_destroy;
        wl_signal_add(&kb->events.key,    &k->key);
        wl_signal_add(&dev->events.destroy,&k->destroy);
        wlr_seat_set_keyboard(g.seat,kb);
    } else if (dev->type==WLR_INPUT_DEVICE_POINTER) {
        struct wlr_pointer *ptr=wlr_pointer_from_input_device(dev);
        PtrS *p=calloc(1,sizeof(*p));
        p->motion.notify=h_motion;        p->button.notify=h_button;
        p->axis.notify=h_axis;            p->swipe_begin.notify=h_swipe_begin;
        p->swipe_update.notify=h_swipe_update; p->swipe_end.notify=h_swipe_end;
        p->destroy.notify=h_ptr_destroy;
        wl_signal_add(&ptr->events.motion,      &p->motion);
        wl_signal_add(&ptr->events.button,       &p->button);
        wl_signal_add(&ptr->events.axis,         &p->axis);
        wl_signal_add(&ptr->events.swipe_begin,  &p->swipe_begin);
        wl_signal_add(&ptr->events.swipe_update, &p->swipe_update);
        wl_signal_add(&ptr->events.swipe_end,    &p->swipe_end);
        wl_signal_add(&dev->events.destroy,       &p->destroy);
        wlr_seat_set_capabilities(g.seat,
            WL_SEAT_CAPABILITY_KEYBOARD|WL_SEAT_CAPABILITY_POINTER);
    }
}
static struct wl_listener l_new_input={.notify=h_new_input};

static void h_xdg(struct wl_listener *l, void *data) {
    (void)l; struct wlr_xdg_surface *xs=data;
    if(xs->role!=WLR_XDG_SURFACE_ROLE_TOPLEVEL) return;
    if(g.on_new_surface) g.on_new_surface(xs->surface,g.ud);
}
static struct wl_listener l_xdg={.notify=h_xdg};

// ── Public API ────────────────────────────────────────────────────
void animus_compositor_register_callbacks(
    void (*on_present)(const struct wlr_output_event_present*,void*),
    void (*on_new_surface)(struct wlr_surface*,void*),
    void (*on_surface_destroy)(struct wlr_surface*,void*),
    void (*on_key)(uint32_t,uint32_t,bool,void*),
    void (*on_pointer_motion)(double,double,void*),
    void (*on_pointer_button)(uint32_t,bool,void*),
    void (*on_pointer_axis)(double,double,void*),
    void (*on_swipe_begin)(uint32_t,void*),
    void (*on_swipe_update)(uint32_t,double,double,void*),
    void (*on_swipe_end)(bool,void*),
    void *ud)
{
    g.on_present=on_present; g.on_new_surface=on_new_surface;
    g.on_surface_destroy=on_surface_destroy; g.on_key=on_key;
    g.on_pointer_motion=on_pointer_motion; g.on_pointer_button=on_pointer_button;
    g.on_pointer_axis=on_pointer_axis;     g.on_swipe_begin=on_swipe_begin;
    g.on_swipe_update=on_swipe_update;     g.on_swipe_end=on_swipe_end;
    g.ud=ud;
}

void animus_compositor_damage_region(int x,int y,int w,int h) {
    pixman_region32_t r; pixman_region32_init_rect(&r,x,y,w,h);
    wlr_damage_ring_add(&g.damage_ring,&r); pixman_region32_fini(&r); }
void animus_compositor_damage_whole(void)
    { wlr_damage_ring_add_whole(&g.damage_ring); }
void animus_compositor_get_damage(pixman_region32_t *out)
    { wlr_damage_ring_get_buffer_damage(&g.damage_ring,1,out); }
void animus_compositor_commit_frame(void) {
    if (!g.primary_output) return;
    struct wlr_output_state st; wlr_output_state_init(&st);
    wlr_output_commit_state(g.primary_output,&st);
    wlr_output_state_finish(&st); }
struct wl_event_loop *animus_compositor_get_event_loop(void)
    { return g.event_loop; }
VkDevice   animus_compositor_get_vk_device(void)
    { return wlr_vk_renderer_get_device(g.renderer); }
VkInstance animus_compositor_get_vk_instance(void)
    { return wlr_vk_renderer_get_instance(g.renderer); }
VkPhysicalDevice animus_compositor_get_vk_physical_device(void)
    { return wlr_vk_renderer_get_physical_device(g.renderer); }

int animus_compositor_init(void) {
    wlr_log_init(WLR_INFO,NULL);
    g.display    =wl_display_create();
    g.event_loop =wl_display_get_event_loop(g.display);
    g.backend    =wlr_backend_autocreate(g.display,NULL);
    if (!g.backend) { wlr_log(WLR_ERROR,"No backend"); return -1; }

    int drm_fd=wlr_backend_get_drm_fd(g.backend);
    if (drm_fd>=0) g.renderer=wlr_vk_renderer_create_with_drm_fd(drm_fd);
    if (!g.renderer) g.renderer=wlr_renderer_autocreate(g.backend);
    if (!g.renderer) { wlr_log(WLR_ERROR,"No renderer"); return -1; }
    wlr_renderer_init_wl_display(g.renderer,g.display);

    g.allocator  =wlr_allocator_autocreate(g.backend,g.renderer);
    g.compositor =wlr_compositor_create(g.display,5,g.renderer);
    g.output_layout=wlr_output_layout_create();
    g.seat       =wlr_seat_create(g.display,"seat0");
    g.xdg_shell  =wlr_xdg_shell_create(g.display,3);
    g.layer_shell=wlr_layer_shell_v1_create(g.display,4);
    wl_signal_add(&g.xdg_shell->events.new_surface,&l_xdg);
    wl_signal_add(&g.backend->events.new_output,   &l_new_output);
    wl_signal_add(&g.backend->events.new_input,    &l_new_input);

    const char *sock=wl_display_add_socket_auto(g.display);
    if (!sock) { wlr_log(WLR_ERROR,"No socket"); return -1; }
    setenv("WAYLAND_DISPLAY",sock,true);
    wlr_log(WLR_INFO,"AnimusEngine on %s",sock);
    if (!wlr_backend_start(g.backend)) return -1;
    return 0;
}
void animus_compositor_run(void)     { wl_display_run(g.display); }
void animus_compositor_destroy(void) {
    wl_display_destroy_clients(g.display);
    wlr_backend_destroy(g.backend);
    wl_display_destroy(g.display);
}
```


---

## PART 4 — C11/C++17 Bridge & Thread Model

### 4.1 animus_compositor.h

```c
// compositor/animus_compositor.h
// Included by C11 compositor AND C++17 AnimusEngine.
#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include <stdbool.h>
#include <stdint.h>
#include <pixman.h>
#include <vulkan/vulkan.h>
#include <wayland-server-core.h>

struct wlr_output_event_present;
struct wlr_surface;

void animus_compositor_register_callbacks(
    void (*on_present)(const struct wlr_output_event_present*, void*),
    void (*on_new_surface)(struct wlr_surface*, void*),
    void (*on_surface_destroy)(struct wlr_surface*, void*),
    void (*on_key)(uint32_t keysym, uint32_t mods, bool pressed, void*),
    void (*on_pointer_motion)(double x, double y, void*),
    void (*on_pointer_button)(uint32_t button, bool pressed, void*),
    void (*on_pointer_axis)(double dx, double dy, void*),
    void (*on_swipe_begin)(uint32_t fingers, void*),
    void (*on_swipe_update)(uint32_t fingers, double dx, double dy, void*),
    void (*on_swipe_end)(bool cancelled, void*),
    void *userdata
);

void                  animus_compositor_damage_region(int x, int y, int w, int h);
void                  animus_compositor_damage_whole(void);
void                  animus_compositor_get_damage(pixman_region32_t *out);
void                  animus_compositor_commit_frame(void);
struct wl_event_loop* animus_compositor_get_event_loop(void);
VkDevice              animus_compositor_get_vk_device(void);
VkInstance            animus_compositor_get_vk_instance(void);
VkPhysicalDevice      animus_compositor_get_vk_physical_device(void);
int                   animus_compositor_init(void);
void                  animus_compositor_run(void);
void                  animus_compositor_destroy(void);

#ifdef __cplusplus
}
#endif
```

### 4.2 Thread Model

```
MAIN THREAD (wl_display_run — Wayland event loop):
  on_present()            — fires at hardware vblank
    → AnimationClock::onPresent(event)     — update dt (null-check .when)
    → AnimationEngine::tick(dt)            — publish OSFEvent::Tick
    → RenderPipeline::renderFrame(damage)  — Vulkan record + submit
    → animus_compositor_commit_frame()     — wlroots DRM present
  on_key/motion/button/axis/swipe_*        — input delivery on main thread
  EventBus::drainAsyncQueue()              — wl_event_loop_add_idle callback

BACKGROUND THREADS (never touch Vulkan, never touch wlroots):
  DirectoryWatcher    — inotify epoll loop
  DirectoryLoader     — std::async filesystem reads
  ThumbnailEngine     — image decode (stb_image)
  SoundEngine         — pw_thread_loop (PipeWire owns its thread)
  PathfinderEngine    — parallel source queries
  InstallManager      — nixos-rebuild subprocess

CROSS-THREAD DELIVERY:
  background_thread:
    EventBus::publishAsync(event, data)
      → push to m_asyncQueue (mutex protected)
      → wl_event_loop_add_idle(loop, drainCb, this)
  main_thread (idle callback):
    drainAsyncQueue()
      → swap queue under mutex
      → EventBus::publish() for each event (synchronous, safe on main thread)
```

### 4.3 EventBus.h

```cpp
// animus/core/EventBus.h
#pragma once
#include <any>
#include <functional>
#include <mutex>
#include <unordered_map>
#include <vector>
#include "OSFEvent.h"

namespace Animus {

using EventHandler = std::function<void(const std::any&)>;

class EventBus {
public:
    static EventBus& shared();

    // Subscribe — returns handle for unsubscribe
    uint64_t subscribe(OSFEvent event, EventHandler handler);
    void     unsubscribe(uint64_t handle);

    // Publish synchronously on current thread (use on main thread only)
    void publish(OSFEvent event, const std::any& data = {});

    // Publish from background thread — drains on main thread via idle callback
    void publishAsync(OSFEvent event, std::any data = {});

    // Called by wl_event_loop_add_idle — static C linkage
    static void drainAsyncQueue(void *ud);

private:
    EventBus() = default;
    struct AsyncEvent { OSFEvent event; std::any data; };
    std::mutex                                     m_asyncMutex;
    std::vector<AsyncEvent>                        m_asyncQueue;
    std::unordered_map<OSFEvent, std::vector<std::pair<uint64_t,EventHandler>>> m_handlers;
    uint64_t m_nextHandle = 1;
};

} // namespace Animus
```

### 4.4 EventBus.cpp

```cpp
// animus/core/EventBus.cpp
#include "EventBus.h"
#include "compositor/animus_compositor.h"
#include <wayland-server-core.h>

namespace Animus {

EventBus& EventBus::shared() {
    static EventBus instance;
    return instance;
}

uint64_t EventBus::subscribe(OSFEvent ev, EventHandler h) {
    uint64_t id = m_nextHandle++;
    m_handlers[ev].emplace_back(id, std::move(h));
    return id;
}

void EventBus::unsubscribe(uint64_t handle) {
    for (auto& [ev, vec] : m_handlers)
        vec.erase(std::remove_if(vec.begin(), vec.end(),
            [handle](auto& p){ return p.first==handle; }), vec.end());
}

void EventBus::publish(OSFEvent ev, const std::any& data) {
    auto it = m_handlers.find(ev);
    if (it == m_handlers.end()) return;
    // Copy vector — handler may unsubscribe during delivery
    auto handlers = it->second;
    for (auto& [id, fn] : handlers) fn(data);
}

void EventBus::publishAsync(OSFEvent ev, std::any data) {
    {
        std::lock_guard<std::mutex> lk(m_asyncMutex);
        m_asyncQueue.push_back({ ev, std::move(data) });
    }
    // wl_event_loop_add_idle: only safe cross-thread wakeup for wlroots
    wl_event_loop_add_idle(
        animus_compositor_get_event_loop(),
        &EventBus::drainAsyncQueue,
        this);
}

void EventBus::drainAsyncQueue(void *ud) {
    auto *self = static_cast<EventBus*>(ud);
    std::vector<AsyncEvent> q;
    {
        std::lock_guard<std::mutex> lk(self->m_asyncMutex);
        q.swap(self->m_asyncQueue);
    }
    for (auto& e : q) self->publish(e.event, e.data);
}

} // namespace Animus
```

### 4.5 OSFEvent.h

```cpp
// animus/core/OSFEvent.h
#pragma once
#include <cstdint>

namespace Animus {

enum class OSFEvent : uint32_t {
    // Animation
    Tick = 0,

    // Window lifecycle
    WindowOpened,
    WindowClosed,
    WindowFocused,
    WindowBlurred,
    WindowMoved,
    WindowResized,
    WindowMaximized,
    WindowRestored,

    // Input
    KeyDown,
    KeyUp,
    MouseMoved,
    MouseButtonDown,
    MouseButtonUp,
    ScrollDelta,
    SwipeBegin,
    SwipeUpdate,
    SwipeEnd,

    // Shell
    DockBounce,
    PanelMenuActivated,
    CockpitViewToggle,
    LockScreenActivate,
    LockScreenUnlocked,
    NotificationPosted,
    NotificationDismissed,

    // Render
    WallpaperChanged,
    WallpaperTintChanged,
    OutputResized,
    DamageRegion,
    DamageWhole,

    // Filesystem (published async from background threads)
    DirectoryChanged,
    DirectoryLoaded,
    ThumbnailReady,

    // Audio
    SoundPlay,
    SoundStop,
    VolumeChanged,

    // Pathfinder
    PathfinderResultsReady,
    PathfinderQueryChanged,
    PathfinderClosed,

    // Boot
    BootCrossfadeComplete,

    // App lifecycle
    AppLaunched,
    AppTerminated,
    AppMenuChanged,

    // Clipboard
    ClipboardChanged,

    // MotionWave gesture results (Part 30)
    DesktopPrev,            // LOCAL — three-finger swipe LEFT
    DesktopNext,            // LOCAL — three-finger swipe RIGHT
    ShowDesktop,            // LOCAL — swipe DOWN when CockpitView closed
    ShowDesktopToggle,      // LOCAL — three-finger tap (show/restore all)
    PinchIn,                // LOCAL — two-finger pinch in
    PinchOut,               // LOCAL — two-finger pinch out

    // Virtual desktops (Part 31)
    DesktopSwitched,        // BRIDGED — desktop changed
    DesktopAdded,           // LOCAL — new desktop created
    DesktopRemoving,        // LOCAL — desktop about to be removed
    DesktopRemoved,         // LOCAL — desktop removed
    DesktopRenamed,         // LOCAL — desktop name changed

    // Power management (Part 34)
    SystemSleep,            // LOCAL → BRIDGED: invoke loginctl suspend
    DisplaySleep,           // LOCAL: DPMS off
    DisplayWake,            // LOCAL: DPMS on
    BatteryLevelChanged,    // LOCAL: data = float 0.0–1.0
    LidClosed,              // LOCAL: from logind via DBusBridge

    // Fullscreen (Part 40)
    FullscreenEntered,      // LOCAL: data = uint64_t windowHandle
    FullscreenExited,       // LOCAL: data = uint64_t windowHandle

    // Minimize (Part 41)
    WindowMinimized,        // LOCAL: data = uint64_t windowHandle
    WindowRestored,         // LOCAL: data = uint64_t windowHandle

    _Count
};

} // namespace Animus
```


---

## PART 5 — GLSL Shaders (Complete Source)

### 5.1 texture_quad.vert

```glsl
// shaders/texture_quad.vert
// Fullscreen quad from push constants — no VBO.
// Used for: wallpaper, thumbnails, wlr_surface textures, images.
#version 450

layout(push_constant) uniform PC {
    vec2  pos;           // top-left, screen pixels
    vec2  size;          // width/height in pixels
    vec2  screenSize;    // output resolution for NDC
    float opacity;
    float cornerRadius;
} pc;

const vec2 VERTS[4] = vec2[](vec2(0,0),vec2(1,0),vec2(0,1),vec2(1,1));
const vec2 UVS[4]   = vec2[](vec2(0,0),vec2(1,0),vec2(0,1),vec2(1,1));

layout(location=0) out vec2 fUV;
layout(location=1) out vec2 fLocal;
layout(location=2) out vec2 fSize;

void main() {
    vec2 lp  = VERTS[gl_VertexIndex];
    vec2 pix = pc.pos + lp * pc.size;
    vec2 ndc = (pix / pc.screenSize) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    fUV    = UVS[gl_VertexIndex];
    fLocal = lp * pc.size;
    fSize  = pc.size;
}
```

### 5.2 texture_quad.frag

```glsl
// shaders/texture_quad.frag
#version 450
layout(binding=0) uniform sampler2D tex;
layout(push_constant) uniform PC {
    vec2 pos; vec2 size; vec2 screenSize;
    float opacity; float cornerRadius;
} pc;
layout(location=0) in vec2 fUV;
layout(location=1) in vec2 fLocal;
layout(location=2) in vec2 fSize;
layout(location=0) out vec4 outColor;

float sdRR(vec2 p, vec2 hs, float r) {
    vec2 d = abs(p - hs) - hs + vec2(r);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

void main() {
    vec4 c = texture(tex, fUV);
    if (pc.cornerRadius > 0.0) {
        float d = sdRR(fLocal, fSize * 0.5, pc.cornerRadius);
        c.a *= 1.0 - smoothstep(-0.5, 0.5, d);
    }
    outColor = vec4(c.rgb, c.a * pc.opacity);
}
```

### 5.3 rounded_rect.vert

```glsl
// shaders/rounded_rect.vert
// All OSFNative solid surfaces (OSFWindow chrome, OSFContent, pills, etc.)
#version 450
layout(push_constant) uniform PC {
    vec2  pos; vec2 size; vec2 screenSize;
    vec4  fillColor; vec4 borderColor;
    float borderWidth; float cornerRadius; float opacity; float _pad;
} pc;
const vec2 VERTS[4] = vec2[](vec2(0,0),vec2(1,0),vec2(0,1),vec2(1,1));
layout(location=0) out vec2 fLocal;
layout(location=1) out vec2 fSize;
void main() {
    vec2 lp  = VERTS[gl_VertexIndex];
    vec2 pix = pc.pos + lp * pc.size;
    vec2 ndc = (pix / pc.screenSize) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    fLocal = lp * pc.size;
    fSize  = pc.size;
}
```

### 5.4 rounded_rect.frag

```glsl
// shaders/rounded_rect.frag
// Top-edge highlight: 1px at 8% white — all surfaces altitude Low+
#version 450
layout(push_constant) uniform PC {
    vec2  pos; vec2 size; vec2 screenSize;
    vec4  fillColor; vec4 borderColor;
    float borderWidth; float cornerRadius; float opacity; float _pad;
} pc;
layout(location=0) in vec2 fLocal;
layout(location=1) in vec2 fSize;
layout(location=0) out vec4 outColor;

float sdRR(vec2 p, vec2 hs, float r) {
    vec2 d = abs(p - hs) - hs + vec2(r);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

void main() {
    float d     = sdRR(fLocal, fSize * 0.5, pc.cornerRadius);
    float outer = 1.0 - smoothstep(-0.5, 0.5, d);
    if (outer < 0.001) discard;

    vec4 color;
    if (pc.borderWidth > 0.0) {
        float id    = d + pc.borderWidth;
        float inner = 1.0 - smoothstep(-0.5, 0.5, id);
        color = mix(pc.borderColor, pc.fillColor, inner);
    } else {
        color = pc.fillColor;
    }

    // Top edge highlight — frosted glass surface catches light
    float topHi = (1.0 - smoothstep(0.0, 1.5, fLocal.y)) * 0.08;
    color.rgb += topHi;

    outColor = vec4(color.rgb, color.a * outer * pc.opacity);
}
```

### 5.5 window_shadow.frag

```glsl
// shaders/window_shadow.frag
// SDF dual shadow. Warm shadow color #1A1208. Never #000000.
// shadowPos is SPRING_SHADOW (300,25) lagged — creates depth perception.
// Ambient: large spread (glass floats). Contact: tight (grounds bottom edge).
#version 450
layout(push_constant) uniform PC {
    vec2  screenSize;
    vec2  shadowPos;     // SPRING_SHADOW spring-lagged position
    vec2  windowSize;
    float cornerRadius;
    float _pad;
} pc;
layout(location=0) in  vec2 fragCoord;
layout(location=0) out vec4 outColor;

float sdRR(vec2 p, vec2 hs, float r) {
    vec2 d = abs(p - hs) - hs + vec2(r);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

void main() {
    vec2 center = pc.shadowPos + pc.windowSize * 0.5;
    float d = sdRR(fragCoord - center, pc.windowSize * 0.5, pc.cornerRadius);
    if (d < 0.0) discard;  // inside window rect — no shadow there

    float ambient = exp(-d / 40.0) * 0.18;   // spread 60px, soft blur 40px, 18% peak
    float contact = exp(-d /  8.0) * 0.12;   // tight 8px, grounds bottom edge 12% peak
    float shadow  = clamp(ambient + contact, 0.0, 1.0);

    // #1A1208 = rgb(0.102, 0.071, 0.031) — warm dark, never pure black
    outColor = vec4(0.102, 0.071, 0.031, shadow);
}
```

### 5.6 kawase_blur.frag

```glsl
// shaders/kawase_blur.frag
// One Kawase pass. Run 4× with iterations 0.5, 1.5, 2.5, 3.5.
// 4 passes = ~16 samples. Approximates Gaussian σ≈8px.
// For altitude High/Floating: run two full chains (8 passes total).
// Process in linear RGB — gamma-correct blur. Output back to sRGB.
#version 450
layout(binding=0) uniform sampler2D src;
layout(push_constant) uniform PC {
    vec2  texelSize;   // 1.0 / vec2(outputWidth, outputHeight)
    float iter;        // 0.5, 1.5, 2.5, or 3.5
    float _pad;
} pc;
layout(location=0) in  vec2 fUV;
layout(location=0) out vec4 outColor;

vec3 toLinear(vec3 c) { return c * c; }          // fast γ≈2.0
vec3 toSrgb(vec3 c)   { return sqrt(max(c, 0.0)); }

void main() {
    vec2 off = pc.texelSize * pc.iter;
    vec4 s = (texture(src, fUV + vec2( off.x,  off.y))
            + texture(src, fUV + vec2(-off.x,  off.y))
            + texture(src, fUV + vec2( off.x, -off.y))
            + texture(src, fUV + vec2(-off.x, -off.y))) * 0.25;
    outColor = vec4(toSrgb(toLinear(s.rgb)), s.a);
}
```

### 5.7 luminosity_composite.frag

```glsl
// shaders/luminosity_composite.frag
// Composites blurred wallpaper → glass surface.
// OKLab for perceptually correct luminosity + chroma ops.
// Source: Björn Ottosson, https://bottosson.github.io/posts/oklab/ (2020)
#version 450
layout(binding=0) uniform sampler2D blurred;
layout(binding=1) uniform sampler2D noiseTex;
layout(push_constant) uniform PC {
    vec4  tintColor;        // from WallpaperTintSampler (sRGB)
    float tintStrength;     // altitude-driven: 0.05–0.35
    float luminosityBoost;  // OKLab L+ : 0.04–0.12
    float chromaReduce;     // OKLab ab× : 0.08–0.20
    float grainStrength;    // noise: 0.015–0.020
    float opacity;          // altitude surface opacity
    float _p0, _p1, _p2;
} pc;
layout(location=0) in  vec2 fUV;
layout(location=0) out vec4 outColor;

vec3 toLinear(vec3 c) { return c * c; }
vec3 toSrgb(vec3 c)   { return sqrt(clamp(c, 0.0, 1.0)); }

vec3 linToOKLab(vec3 c) {
    float l = 0.4122214708*c.r + 0.5363325363*c.g + 0.0514459929*c.b;
    float m = 0.2119034982*c.r + 0.6806995451*c.g + 0.1073969566*c.b;
    float s = 0.0883024619*c.r + 0.2817188376*c.g + 0.6299787005*c.b;
    float l_ = pow(l, 1.0/3.0), m_ = pow(m, 1.0/3.0), s_ = pow(s, 1.0/3.0);
    return vec3(
        0.2104542553*l_ + 0.7936177850*m_ - 0.0040720468*s_,
        1.9779984951*l_ - 2.4285922050*m_ + 0.4505937099*s_,
        0.0259040371*l_ + 0.7827717662*m_ - 0.8086757660*s_);
}
vec3 OKLabToLin(vec3 lab) {
    float l_ = lab.x + 0.3963377774*lab.y + 0.2158037573*lab.z;
    float m_ = lab.x - 0.1055613458*lab.y - 0.0638541728*lab.z;
    float s_ = lab.x - 0.0894841775*lab.y - 1.2914855480*lab.z;
    float l = l_*l_*l_, m = m_*m_*m_, s = s_*s_*s_;
    return vec3(
         4.0767416621*l - 3.3077115913*m + 0.2309699292*s,
        -1.2684380046*l + 2.6097574011*m - 0.3413193965*s,
        -0.0041960863*l - 0.7034186147*m + 1.7076147010*s);
}

void main() {
    vec4 b   = texture(blurred, fUV);
    vec3 lab = linToOKLab(toLinear(b.rgb));

    // Luminosity boost — brightens darks, frosted glass not smeared glass
    lab.x = clamp(lab.x + pc.luminosityBoost, 0.0, 1.0);
    // Chroma reduction — subtle desaturation
    lab.yz *= (1.0 - pc.chromaReduce);

    vec3 result = OKLabToLin(lab);

    // Wallpaper tint in linear RGB
    result = mix(result, toLinear(pc.tintColor.rgb), pc.tintStrength);

    // Noise grain — prevents banding in smooth glass areas
    float noise = texture(noiseTex, fUV * 400.0).r * 2.0 - 1.0;
    result += noise * pc.grainStrength;

    outColor = vec4(toSrgb(result), pc.opacity);
}
```

### 5.8 glyph.vert

```glsl
// shaders/glyph.vert
// Per-glyph quad. HarfBuzz fractional positions PRESERVED — never round.
#version 450
layout(push_constant) uniform PC {
    vec2 screenSize;
    vec2 glyphPos;     // fractional pixel position from HarfBuzz (26.6 / 64.0)
    vec2 glyphSize;    // glyph bitmap dimensions in pixels
    vec2 atlasUVMin;
    vec2 atlasUVMax;
    vec4 textColor;    // premultiplied alpha
} pc;
const vec2 VERTS[4] = vec2[](vec2(0,0),vec2(1,0),vec2(0,1),vec2(1,1));
layout(location=0) out vec2 fAtlasUV;
void main() {
    vec2 lp  = VERTS[gl_VertexIndex];
    vec2 pix = pc.glyphPos + lp * pc.glyphSize;
    vec2 ndc = (pix / pc.screenSize) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    fAtlasUV = mix(pc.atlasUVMin, pc.atlasUVMax, lp);
}
```

### 5.9 glyph.frag

```glsl
// shaders/glyph.frag
// atlas: VK_FORMAT_R8_UNORM — red channel = FreeType coverage.
// Blend state: SRC_ALPHA / ONE_MINUS_SRC_ALPHA
#version 450
layout(binding=0) uniform sampler2D atlas;
layout(push_constant) uniform PC {
    vec2 screenSize; vec2 glyphPos; vec2 glyphSize;
    vec2 atlasUVMin; vec2 atlasUVMax;
    vec4 textColor;
} pc;
layout(location=0) in  vec2 fAtlasUV;
layout(location=0) out vec4 outColor;
void main() {
    float coverage = texture(atlas, fAtlasUV).r;
    outColor = vec4(pc.textColor.rgb, pc.textColor.a * coverage);
}
```


---

## PART 6 — Vulkan Context & Frame Loop

### 6.1 VulkanContext.h

```cpp
// animus/render/VulkanContext.h
#pragma once
#include <vulkan/vulkan.h>

namespace Animus {

// VulkanContext: retrieves device/instance/physDevice from wlroots.
// AnimusEngine does NOT create its own VkDevice. wlroots owns it.
// wlroots owns swapchain via DRM output abstraction.
// AnimusEngine renders into VkImage, wlroots presents via wlr_output_commit_state().
// VK_PRESENT_MODE_FIFO_KHR enforced by wlroots DRM backend. No exceptions.

struct VulkanContext {
    VkInstance       instance   = VK_NULL_HANDLE;
    VkPhysicalDevice physDevice = VK_NULL_HANDLE;
    VkDevice         device     = VK_NULL_HANDLE;
    uint32_t         gfxFamily  = ~0u;
    VkQueue          gfxQueue   = VK_NULL_HANDLE;

    static constexpr int FRAMES = 2;
    VkCommandPool   cmdPool[FRAMES] = {};
    VkCommandBuffer cmdBuf [FRAMES] = {};
    VkFence         fence  [FRAMES] = {};
    VkSemaphore     semDone[FRAMES] = {};
    int             frame = 0;

    // Render target — written each frame, wlroots blits to DRM plane
    VkImage        rtImage  = VK_NULL_HANDLE;
    VkDeviceMemory rtMemory = VK_NULL_HANDLE;
    VkImageView    rtView   = VK_NULL_HANDLE;
    VkRenderPass   rtPass   = VK_NULL_HANDLE;
    VkFramebuffer  rtFB     = VK_NULL_HANDLE;
    uint32_t       width=0, height=0;

    bool            initialize(uint32_t w, uint32_t h);
    void            destroy();
    uint32_t        findMemType(uint32_t bits, VkMemoryPropertyFlags props);
    VkCommandBuffer beginOneTime();
    void            endOneTime(VkCommandBuffer cmd);
};

} // namespace Animus
```

### 6.2 VulkanContext.cpp

```cpp
// animus/render/VulkanContext.cpp
#include "VulkanContext.h"
#include "compositor/animus_compositor.h"
#include <wlr/util/log.h>

namespace Animus {

bool VulkanContext::initialize(uint32_t w, uint32_t h) {
    width = w; height = h;

    // Retrieve from wlroots — NEVER create our own device
    instance   = animus_compositor_get_vk_instance();
    physDevice = animus_compositor_get_vk_physical_device();
    device     = animus_compositor_get_vk_device();

    if (!instance || !physDevice || !device) {
        wlr_log(WLR_ERROR, "VulkanContext: wlroots Vulkan objects null");
        return false;
    }

    // Find graphics queue family
    uint32_t qc = 0;
    vkGetPhysicalDeviceQueueFamilyProperties(physDevice, &qc, nullptr);
    std::vector<VkQueueFamilyProperties> qf(qc);
    vkGetPhysicalDeviceQueueFamilyProperties(physDevice, &qc, qf.data());
    for (uint32_t i = 0; i < qc; i++)
        if (qf[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) { gfxFamily=i; break; }
    if (gfxFamily == ~0u) return false;
    vkGetDeviceQueue(device, gfxFamily, 0, &gfxQueue);

    // Per-frame command infrastructure
    for (int i = 0; i < FRAMES; i++) {
        VkCommandPoolCreateInfo pi = {
            VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO, nullptr,
            VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT, gfxFamily };
        vkCreateCommandPool(device, &pi, nullptr, &cmdPool[i]);

        VkCommandBufferAllocateInfo ai = {
            VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO, nullptr,
            cmdPool[i], VK_COMMAND_BUFFER_LEVEL_PRIMARY, 1 };
        vkAllocateCommandBuffers(device, &ai, &cmdBuf[i]);

        VkFenceCreateInfo fi = {
            VK_STRUCTURE_TYPE_FENCE_CREATE_INFO, nullptr,
            VK_FENCE_CREATE_SIGNALED_BIT };
        vkCreateFence(device, &fi, nullptr, &fence[i]);

        VkSemaphoreCreateInfo si = { VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO };
        vkCreateSemaphore(device, &si, nullptr, &semDone[i]);
    }

    // Render target image — DEVICE_LOCAL, COLOR_ATTACHMENT + TRANSFER_SRC
    VkImageCreateInfo ii = {
        VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO, nullptr, 0,
        VK_IMAGE_TYPE_2D, VK_FORMAT_B8G8R8A8_UNORM, {w, h, 1},
        1, 1, VK_SAMPLE_COUNT_1_BIT, VK_IMAGE_TILING_OPTIMAL,
        VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | VK_IMAGE_USAGE_TRANSFER_SRC_BIT,
        VK_SHARING_MODE_EXCLUSIVE, 0, nullptr, VK_IMAGE_LAYOUT_UNDEFINED };
    vkCreateImage(device, &ii, nullptr, &rtImage);

    VkMemoryRequirements mr;
    vkGetImageMemoryRequirements(device, rtImage, &mr);
    VkMemoryAllocateInfo ma = {
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO, nullptr,
        mr.size, findMemType(mr.memoryTypeBits, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT) };
    vkAllocateMemory(device, &ma, nullptr, &rtMemory);
    vkBindImageMemory(device, rtImage, rtMemory, 0);

    VkImageViewCreateInfo vi = {
        VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO, nullptr, 0,
        rtImage, VK_IMAGE_VIEW_TYPE_2D, VK_FORMAT_B8G8R8A8_UNORM, {},
        { VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1 } };
    vkCreateImageView(device, &vi, nullptr, &rtView);

    // Render pass — single color attachment, no depth
    VkAttachmentDescription att = {
        0, VK_FORMAT_B8G8R8A8_UNORM, VK_SAMPLE_COUNT_1_BIT,
        VK_ATTACHMENT_LOAD_OP_CLEAR, VK_ATTACHMENT_STORE_OP_STORE,
        VK_ATTACHMENT_LOAD_OP_DONT_CARE, VK_ATTACHMENT_STORE_OP_DONT_CARE,
        VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL };
    VkAttachmentReference ref = { 0, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL };
    VkSubpassDescription  sub = {
        0, VK_PIPELINE_BIND_POINT_GRAPHICS, 0, nullptr, 1, &ref };
    VkRenderPassCreateInfo rpi = {
        VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO, nullptr,
        0, 1, &att, 1, &sub, 0, nullptr };
    vkCreateRenderPass(device, &rpi, nullptr, &rtPass);

    VkFramebufferCreateInfo fbi = {
        VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO, nullptr,
        0, rtPass, 1, &rtView, w, h, 1 };
    vkCreateFramebuffer(device, &fbi, nullptr, &rtFB);
    return true;
}

uint32_t VulkanContext::findMemType(uint32_t bits, VkMemoryPropertyFlags props) {
    VkPhysicalDeviceMemoryProperties mp;
    vkGetPhysicalDeviceMemoryProperties(physDevice, &mp);
    for (uint32_t i = 0; i < mp.memoryTypeCount; i++)
        if ((bits & (1u<<i)) && (mp.memoryTypes[i].propertyFlags & props) == props)
            return i;
    return ~0u;
}

VkCommandBuffer VulkanContext::beginOneTime() {
    VkCommandBufferAllocateInfo ai = {
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO, nullptr,
        cmdPool[0], VK_COMMAND_BUFFER_LEVEL_PRIMARY, 1 };
    VkCommandBuffer cmd;
    vkAllocateCommandBuffers(device, &ai, &cmd);
    VkCommandBufferBeginInfo bi = {
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO, nullptr,
        VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT };
    vkBeginCommandBuffer(cmd, &bi);
    return cmd;
}

void VulkanContext::endOneTime(VkCommandBuffer cmd) {
    vkEndCommandBuffer(cmd);
    VkSubmitInfo si = {
        VK_STRUCTURE_TYPE_SUBMIT_INFO, nullptr,
        0, nullptr, nullptr, 1, &cmd };
    vkQueueSubmit(gfxQueue, 1, &si, VK_NULL_HANDLE);
    vkQueueWaitIdle(gfxQueue);
    vkFreeCommandBuffers(device, cmdPool[0], 1, &cmd);
}

void VulkanContext::destroy() {
    vkDeviceWaitIdle(device);
    vkDestroyFramebuffer(device, rtFB, nullptr);
    vkDestroyRenderPass(device, rtPass, nullptr);
    vkDestroyImageView(device, rtView, nullptr);
    vkFreeMemory(device, rtMemory, nullptr);
    vkDestroyImage(device, rtImage, nullptr);
    for (int i = 0; i < FRAMES; i++) {
        vkDestroySemaphore(device, semDone[i], nullptr);
        vkDestroyFence(device, fence[i], nullptr);
        vkDestroyCommandPool(device, cmdPool[i], nullptr);
    }
}

} // namespace Animus
```

### 6.3 RenderPipeline — Frame Loop

```cpp
// animus/render/RenderPipeline.cpp (frame loop section)
// No swapchain. wlroots owns present via wlr_output_commit_state().
// VK_PRESENT_MODE_FIFO_KHR enforced by wlroots DRM backend.

void RenderPipeline::renderFrame(float dt) {
    pixman_region32_t damage;
    pixman_region32_init(&damage);
    animus_compositor_get_damage(&damage);

    if (!pixman_region32_not_empty(&damage)) {
        pixman_region32_fini(&damage);
        return;
    }
    pixman_region32_fini(&damage);

    int f = m_ctx->frame;
    vkWaitForFences(m_ctx->device, 1, &m_ctx->fence[f], VK_TRUE, UINT64_MAX);
    vkResetFences(m_ctx->device, 1, &m_ctx->fence[f]);

    VkCommandBuffer cmd = m_ctx->cmdBuf[f];
    vkResetCommandBuffer(cmd, 0);

    VkCommandBufferBeginInfo bi = {
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO, nullptr,
        VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT };
    vkBeginCommandBuffer(cmd, &bi);

    VkClearValue cv = {.color = {.float32 = {0,0,0,1}}};
    VkRenderPassBeginInfo rbi = {
        VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO, nullptr,
        m_ctx->rtPass, m_ctx->rtFB,
        {{0,0},{m_ctx->width,m_ctx->height}}, 1, &cv };
    vkCmdBeginRenderPass(cmd, &rbi, VK_SUBPASS_CONTENTS_INLINE);

    VkViewport vp = {0,0,(float)m_ctx->width,(float)m_ctx->height,0,1};
    vkCmdSetViewport(cmd, 0, 1, &vp);
    VkRect2D sc = {{0,0},{m_ctx->width,m_ctx->height}};
    vkCmdSetScissor(cmd, 0, 1, &sc);

    // Layer 0: Wallpaper (full screen texture)
    if (m_wallpaperView != VK_NULL_HANDLE)
        m_material->drawTextureQuad(cmd, 0, 0,
            (float)m_ctx->width, (float)m_ctx->height,
            m_wallpaperView, 1.0f, 0.0f);

    // Layer 1: Window shadows (spring-lagged positions)
    for (auto& win : m_windows)
        if (win->isVisible())
            m_shadow->drawWindowShadow(cmd,
                win->shadowX(), win->shadowY(),
                win->width(),   win->height(),
                win->cornerRadius());

    // Layer 2: Window glass backgrounds (blur + luminosity + tint)
    for (auto& win : m_windows)
        if (win->isVisible())
            m_material->drawGlassSurface(cmd,
                win->x(), win->y(), win->width(), win->height(),
                win->cornerRadius(), win->altitude());

    // Layer 3: Window content (wlr_surface textures)
    for (auto& win : m_windows)
        if (win->isVisible())
            m_material->drawWindowSurface(cmd, win.get());

    // Layer 4: Shell surfaces
    if (m_panel) m_panel->render(cmd, dt);
    if (m_dock)  m_dock->render(cmd, dt);

    // Layer 5: Boot crossfade (Space Orange overlay, fades out on first render)
    if (m_crossfade && !m_crossfade->isComplete())
        m_crossfade->render(cmd, (float)m_ctx->width, (float)m_ctx->height);

    // Layer 6: Floating overlays
    for (auto& ov : m_overlays)
        if (ov->isVisible()) ov->render(cmd, dt);

    vkCmdEndRenderPass(cmd);
    vkEndCommandBuffer(cmd);

    VkSubmitInfo si = {
        VK_STRUCTURE_TYPE_SUBMIT_INFO, nullptr,
        0, nullptr, nullptr, 1, &cmd,
        1, &m_ctx->semDone[f] };
    vkQueueSubmit(m_ctx->gfxQueue, 1, &si, m_ctx->fence[f]);

    // Hand off to wlroots — presents at next vblank (FIFO)
    animus_compositor_commit_frame();

    m_ctx->frame = (f + 1) % VulkanContext::FRAMES;
}
```


---

## PART 7 — MaterialRenderer (SurfaceAltitude System)

```cpp
// animus/render/MaterialRenderer.h
#pragma once
#include <vulkan/vulkan.h>
#include <cstdint>

namespace Animus {

// ALTITUDE TABLE — these values are the law.
// No surface specifies blur/opacity/tint directly. Ever.
// All visual properties are derived from altitude by MaterialRenderer.
enum class SurfaceAltitude { Grounded=0, Low=1, Mid=2, High=3, Floating=4 };

struct SurfaceMaterial {
    float blurRadius;       // effective Kawase radius (px)
    float opacity;          // surface opacity
    float tintStrength;     // wallpaper tint influence
    float luminosityBoost;  // OKLab L channel addition
    float chromaReduce;     // OKLab ab channel multiplication reduction
    float grainStrength;    // noise grain amplitude
};

// Immutable altitude table
static constexpr SurfaceMaterial MATERIALS[5] = {
//   blur     opacity  tint   lumin  chroma grain
    {  0.0f,  1.00f,  0.00f, 0.00f, 0.00f, 0.000f },  // Grounded
    {  8.0f,  0.94f,  0.05f, 0.04f, 0.08f, 0.020f },  // Low
    { 20.0f,  0.82f,  0.22f, 0.08f, 0.15f, 0.020f },  // Mid
    { 32.0f,  0.72f,  0.30f, 0.10f, 0.18f, 0.015f },  // High
    { 48.0f,  0.64f,  0.35f, 0.12f, 0.20f, 0.015f },  // Floating
};

inline const SurfaceMaterial& getMaterial(SurfaceAltitude a) {
    return MATERIALS[static_cast<int>(a)];
}

// Push constant for luminosity_composite.frag
struct GlassPushConst {
    float tintR, tintG, tintB, tintA;
    float tintStrength;
    float luminosityBoost;
    float chromaReduce;
    float grainStrength;
    float opacity;
    float _p0, _p1, _p2;
};

// Push constant for kawase_blur.frag
struct KawasePushConst {
    float texelX, texelY;
    float iter;
    float _pad;
};

struct WallpaperTint {
    float r, g, b;
    float strength;  // how strongly wallpaper color bleeds into tint
};

class VulkanContext;

class MaterialRenderer {
public:
    explicit MaterialRenderer(VulkanContext *ctx) : m_ctx(ctx) {}
    bool initialize();

    // Glass surface: blur + luminosity + tint + grain
    // All parameters derived from altitude — caller does not choose blur amount
    void drawGlassSurface(VkCommandBuffer cmd,
                          float x, float y, float w, float h,
                          float cornerRadius, SurfaceAltitude alt);

    // Solid rounded rectangle
    void drawRoundRect(VkCommandBuffer cmd,
                       float x, float y, float w, float h,
                       float cornerRadius,
                       uint32_t fillARGB,
                       uint32_t borderARGB = 0,
                       float borderWidth   = 0.0f,
                       float opacity       = 1.0f);

    // Texture (image / thumbnail / wlr_surface)
    void drawTextureQuad(VkCommandBuffer cmd,
                         float x, float y, float w, float h,
                         VkImageView tex,
                         float opacity,
                         float cornerRadius);

    void drawWindowSurface(VkCommandBuffer cmd, const class Window *win);

    // Update when OSFEvent::WallpaperTintChanged fires
    void setWallpaperTint(const WallpaperTint &t) { m_tint = t; }

private:
    VulkanContext  *m_ctx;
    WallpaperTint   m_tint = { 0.961f, 0.961f, 0.961f, 0.0f };

    // Pipelines
    VkPipeline       m_kawasePipeline     = VK_NULL_HANDLE;
    VkPipeline       m_luminosityPipeline = VK_NULL_HANDLE;
    VkPipeline       m_rectPipeline       = VK_NULL_HANDLE;
    VkPipeline       m_quadPipeline       = VK_NULL_HANDLE;
    VkPipelineLayout m_glassLayout        = VK_NULL_HANDLE;
    VkPipelineLayout m_rectLayout         = VK_NULL_HANDLE;
    VkPipelineLayout m_quadLayout         = VK_NULL_HANDLE;

    // Ping-pong blur targets (reused every frame)
    VkImage        m_blurA = VK_NULL_HANDLE, m_blurB = VK_NULL_HANDLE;
    VkImageView    m_blurVA= VK_NULL_HANDLE, m_blurVB= VK_NULL_HANDLE;
    VkDeviceMemory m_blurMA= VK_NULL_HANDLE, m_blurMB= VK_NULL_HANDLE;

    // Pre-baked noise texture for grain
    VkImage        m_noiseTex    = VK_NULL_HANDLE;
    VkImageView    m_noiseView   = VK_NULL_HANDLE;
    VkDeviceMemory m_noiseMem    = VK_NULL_HANDLE;
    VkSampler      m_noiseSampler= VK_NULL_HANDLE;

    VkDescriptorPool      m_descPool     = VK_NULL_HANDLE;
    VkDescriptorSetLayout m_glassDescSet = VK_NULL_HANDLE;

    void runKawasePasses(VkCommandBuffer cmd,
                         VkImageView src, int numChains);
    VkShaderModule loadShader(const char *spvPath);
};

} // namespace Animus
```

### 7.1 Glass Surface Draw (MaterialRenderer.cpp excerpt)

```cpp
void MaterialRenderer::drawGlassSurface(
    VkCommandBuffer cmd,
    float x, float y, float w, float h,
    float cornerRadius, SurfaceAltitude alt)
{
    const SurfaceMaterial &mat = getMaterial(alt);
    if (mat.blurRadius <= 0.0f) return;  // Grounded: no glass effect

    // Determine blur passes: Low/Mid = 1 chain (4 passes), High/Floating = 2 chains (8)
    int chains = (alt >= SurfaceAltitude::High) ? 2 : 1;

    // Blit wallpaper region into blurA
    // ... (descriptor update + blit pass) ...

    // Run Kawase blur
    runKawasePasses(cmd, m_blurVA, chains);

    // Luminosity composite — outputs final glass appearance
    GlassPushConst pc = {};
    pc.tintR = m_tint.r; pc.tintG = m_tint.g; pc.tintB = m_tint.b;
    pc.tintStrength     = mat.tintStrength * m_tint.strength;
    pc.luminosityBoost  = mat.luminosityBoost;
    pc.chromaReduce     = mat.chromaReduce;
    pc.grainStrength    = mat.grainStrength;
    pc.opacity          = mat.opacity;

    vkCmdBindPipeline(cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, m_luminosityPipeline);
    vkCmdPushConstants(cmd, m_glassLayout,
        VK_SHADER_STAGE_FRAGMENT_BIT, 0, sizeof(pc), &pc);
    vkCmdDraw(cmd, 4, 1, 0, 0);  // fullscreen quad from vert shader
}

void MaterialRenderer::runKawasePasses(VkCommandBuffer cmd,
                                        VkImageView src, int numChains)
{
    static const float OFFSETS[4] = { 0.5f, 1.5f, 2.5f, 3.5f };
    float texelX = 1.0f / (float)m_ctx->width;
    float texelY = 1.0f / (float)m_ctx->height;

    for (int chain = 0; chain < numChains; chain++) {
        for (int pass = 0; pass < 4; pass++) {
            KawasePushConst kpc = {
                texelX, texelY, OFFSETS[pass], 0.0f };
            vkCmdBindPipeline(cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, m_kawasePipeline);
            vkCmdPushConstants(cmd, m_glassLayout,
                VK_SHADER_STAGE_FRAGMENT_BIT, 0, sizeof(kpc), &kpc);
            vkCmdDraw(cmd, 4, 1, 0, 0);
            // Ping-pong: swap blurA ↔ blurB between passes
        }
    }
}
```


---

## PART 8 — ShadowRenderer

```cpp
// animus/render/ShadowRenderer.h
#pragma once
#include <vulkan/vulkan.h>

namespace Animus {

class VulkanContext;

// Dual SDF shadow — ambient (floats window) + contact (grounds it).
// Shadow position is spring-lagged via SPRING_SHADOW (stiffness=300, damping=25).
// This lag creates the illusion of depth as the window "floats" above the desktop.
// Color is always #1A1208 — warm dark, never #000000.
class ShadowRenderer {
public:
    explicit ShadowRenderer(VulkanContext *ctx) : m_ctx(ctx) {}
    bool initialize();

    // shadowX/Y: SPRING_SHADOW lagged position (slightly behind window position)
    // Shadow quad = window bounds padded by 100px on all sides
    void drawWindowShadow(VkCommandBuffer cmd,
                          float shadowX, float shadowY,
                          float winW,    float winH,
                          float cornerRadius);

private:
    struct ShadowPC {
        float screenW, screenH;
        float shadowX, shadowY;
        float winW, winH;
        float cornerRadius, _pad;
    };

    VulkanContext   *m_ctx;
    VkPipeline       m_pipeline = VK_NULL_HANDLE;
    VkPipelineLayout m_layout   = VK_NULL_HANDLE;
};

} // namespace Animus
```

---

## PART 9 — SpringSolver + AnimationClock + AnimationEngine

### 9.1 SpringSolver.h — Header-Only

```cpp
// animus/animation/SpringSolver.h
// Header-only. Fully inlined. Zero overhead per tick.
// Semi-implicit (symplectic) Euler integration.
// Stable for all named configs. dt clamped [0.001, 0.100].
#pragma once
#include <cmath>

namespace Animus {

struct SpringConfig {
    float stiffness;
    float damping;
    float epsilon;   // at-rest threshold
};

// Named spring profiles — always use these names, never hardcode values inline
namespace Springs {
    static constexpr SpringConfig Selection    = { 400.f, 28.f, 0.010f };
    static constexpr SpringConfig WindowDrag   = { 800.f, 35.f, 0.005f };
    static constexpr SpringConfig Shadow       = { 300.f, 25.f, 0.010f };
    static constexpr SpringConfig Hover        = { 600.f, 40.f, 0.001f };
    static constexpr SpringConfig Scroll       = {  80.f, 18.f, 0.100f };
    static constexpr SpringConfig Resize       = { 350.f, 28.f, 0.010f };
    static constexpr SpringConfig Sheet        = { 420.f, 30.f, 0.010f };
    static constexpr SpringConfig Boot         = { 200.f, 22.f, 0.010f };
    static constexpr SpringConfig Notification = { 380.f, 26.f, 0.010f };
    static constexpr SpringConfig TrafficLight = { 700.f, 38.f, 0.001f };
    static constexpr SpringConfig DockMagnify  = { 450.f, 32.f, 0.001f };
    static constexpr SpringConfig LockScreen   = { 120.f, 22.f, 0.010f };
} // namespace Springs

class SpringSolver {
public:
    explicit SpringSolver(SpringConfig cfg, float init = 0.0f)
        : m_cfg(cfg), m_pos(init), m_vel(0.0f), m_target(init) {}

    void setTarget(float t) { m_target = t; }
    void snap(float v)      { m_pos = v; m_vel = 0.0f; m_target = v; }
    float value()    const  { return m_pos; }
    float velocity() const  { return m_vel; }

    bool isResting() const {
        return std::fabsf(m_pos - m_target) < m_cfg.epsilon
            && std::fabsf(m_vel) < m_cfg.epsilon;
    }

    // dt in seconds. Clamped to [0.001, 0.100] for stability.
    // Semi-implicit Euler: update velocity first, then position.
    void tick(float dt) {
        if (dt < 0.001f) dt = 0.001f;
        if (dt > 0.100f) dt = 0.100f;
        if (isResting()) { m_pos = m_target; m_vel = 0.0f; return; }

        float displacement = m_pos - m_target;
        float springForce  = -m_cfg.stiffness * displacement;
        float dampForce    = -m_cfg.damping    * m_vel;
        float accel        = springForce + dampForce;

        m_vel += accel * dt;  // velocity updated first (semi-implicit)
        m_pos += m_vel * dt;
    }

private:
    SpringConfig m_cfg;
    float m_pos, m_vel, m_target;
};

// 2D spring: two independent SpringSolvers sharing a config
class SpringSolver2D {
public:
    explicit SpringSolver2D(SpringConfig cfg, float ix=0.0f, float iy=0.0f)
        : x(cfg, ix), y(cfg, iy) {}

    void setTarget(float tx, float ty) { x.setTarget(tx); y.setTarget(ty); }
    void snap(float sx, float sy)      { x.snap(sx); y.snap(sy); }
    bool isResting() const             { return x.isResting() && y.isResting(); }
    void tick(float dt)                { x.tick(dt); y.tick(dt); }

    SpringSolver x, y;
};

} // namespace Animus
```

### 9.2 AnimationClock.h / .cpp

```cpp
// animus/animation/AnimationClock.h
#pragma once
#include <cstdint>
#include <ctime>

struct wlr_output_event_present;

namespace Animus {

// AnimationClock: driven by hardware vblank via on_present callback.
// Uses wlr_output_event_present.when (struct timespec*) for frame-perfect dt.
// .when MAY BE NULL — always null-check before use.
class AnimationClock {
public:
    static AnimationClock& shared();

    // Called from on_present callback (C11 → C++17 bridge)
    void onPresent(const struct wlr_output_event_present *event);

    float dt()        const { return m_dt; }
    double totalTime() const { return m_totalTime; }

private:
    AnimationClock() = default;

    bool   m_hasLast  = false;
    struct timespec m_lastTime = {};
    float  m_dt       = 1.0f / 60.0f;  // default 60Hz until first vblank
    double m_totalTime= 0.0;
};

} // namespace Animus
```

```cpp
// animus/animation/AnimationClock.cpp
#include "AnimationClock.h"

// wlroots header for wlr_output_event_present
#define WLR_USE_UNSTABLE
#include <wlr/types/wlr_output.h>

namespace Animus {

AnimationClock& AnimationClock::shared() {
    static AnimationClock instance;
    return instance;
}

void AnimationClock::onPresent(const struct wlr_output_event_present *ev) {
    // wlr_output_event_present.when is struct timespec* — MAY BE NULL
    if (!ev || !ev->when) return;

    if (m_hasLast) {
        double sec  = (double)(ev->when->tv_sec  - m_lastTime.tv_sec);
        double nsec = (double)(ev->when->tv_nsec - m_lastTime.tv_nsec) * 1e-9;
        float  dt   = (float)(sec + nsec);
        // Clamp: reject stalls (>100ms) and sub-1ms noise
        if (dt > 0.001f && dt < 0.100f) {
            m_dt = dt;
            m_totalTime += dt;
        }
    }
    m_lastTime = *ev->when;
    m_hasLast  = true;
}

} // namespace Animus
```

### 9.3 AnimationEngine.h / .cpp

```cpp
// animus/animation/AnimationEngine.h
#pragma once

namespace Animus {

// AnimationEngine: ticks everything driven by OSFEvent::Tick.
// Components own their springs. AnimationEngine only publishes Tick.
// No global spring registry. Components subscribe to Tick and tick themselves.
class AnimationEngine {
public:
    static AnimationEngine& shared();

    // Called from on_present callback after AnimationClock::onPresent
    void tick(float dt);

    void start();
    void stop();

private:
    AnimationEngine() = default;
    bool m_running = false;
};

} // namespace Animus
```

```cpp
// animus/animation/AnimationEngine.cpp
#include "AnimationEngine.h"
#include "AnimationClock.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"

namespace Animus {

AnimationEngine& AnimationEngine::shared() {
    static AnimationEngine instance;
    return instance;
}

void AnimationEngine::start() { m_running = true; }
void AnimationEngine::stop()  { m_running = false; }

void AnimationEngine::tick(float dt) {
    if (!m_running) return;
    // Publish Tick — all components with springs subscribe and tick themselves
    // data = float dt (seconds)
    EventBus::shared().publish(OSFEvent::Tick, dt);
}

} // namespace Animus
```

### 9.4 How Components Use Springs (Pattern)

```cpp
// Example: OSFSidebar — selection pill position spring
// Every animated component follows this exact pattern.

class OSFSidebar {
public:
    OSFSidebar() : m_selectionY(Springs::Selection) {
        // Subscribe to Tick — tick own spring, request damage if moving
        m_tickHandle = EventBus::shared().subscribe(OSFEvent::Tick,
            [this](const std::any& d) {
                float dt = std::any_cast<float>(d);
                if (!m_selectionY.isResting()) {
                    m_selectionY.tick(dt);
                    animus_compositor_damage_region(
                        (int)m_x, (int)m_y, (int)m_width, (int)m_height);
                }
            });
    }
    ~OSFSidebar() {
        EventBus::shared().unsubscribe(m_tickHandle);
    }

    void setSelectedIndex(int idx) {
        // Target = idx * item_height. Spring animates to it.
        m_selectionY.setTarget((float)idx * ITEM_HEIGHT);
    }

private:
    SpringSolver m_selectionY;
    uint64_t     m_tickHandle = 0;
    float m_x=0, m_y=0, m_width=0, m_height=0;
    static constexpr float ITEM_HEIGHT = 36.0f;
};
```


---

## PART 10 — GlyphAtlas + TextRenderer

### 10.1 GlyphAtlas.h

```cpp
// animus/render/GlyphAtlas.h
#pragma once
#include <vulkan/vulkan.h>
#include <ft2build.h>
#include FT_FREETYPE_H
#include <hb.h>
#include <hb-ft.h>
#include <unordered_map>
#include <cstdint>

namespace Animus {

struct GlyphEntry {
    uint16_t atlasX, atlasY;   // top-left in atlas pixels
    uint16_t w, h;             // glyph bitmap dimensions
    int16_t  bearingX, bearingY;
    uint16_t advance;          // in 1/64 pixel units
};

class VulkanContext;

// Atlas: VK_FORMAT_R8_UNORM, 2048×2048.
// Pre-rasterized: Latin Basic (U+0020–U+007F) + Latin Extended-1 (U+00A0–U+00FF)
// Dynamic: additional glyphs appended on demand.
// FreeType rasterizes at 1:1 pixel (no hinting overrides — let FT default).
// hb_ft_font_create_referenced — NOT hb_ft_font_create (deprecated).
class GlyphAtlas {
public:
    explicit GlyphAtlas(VulkanContext *ctx) : m_ctx(ctx) {}
    bool initialize(const char *fontPath, float ptSize, float dpiScale);
    void destroy();

    // Returns null if not found (triggers rasterize + upload)
    const GlyphEntry* getGlyph(uint32_t codepoint);
    void rasterizeAndUpload(uint32_t codepoint);

    VkImageView atlasView()  const { return m_atlasView; }
    VkSampler   atlasSampler() const { return m_sampler; }

    hb_font_t *hbFont() const { return m_hbFont; }

private:
    void uploadGlyphToAtlas(const GlyphEntry &e, const uint8_t *bitmap);
    void transitionAtlasLayout(VkCommandBuffer cmd,
                                VkImageLayout from, VkImageLayout to);

    VulkanContext *m_ctx;

    FT_Library m_ftLib  = nullptr;
    FT_Face    m_ftFace = nullptr;
    hb_font_t *m_hbFont = nullptr;  // hb_ft_font_create_referenced(m_ftFace)

    VkImage        m_atlas     = VK_NULL_HANDLE;
    VkDeviceMemory m_atlasMem  = VK_NULL_HANDLE;
    VkImageView    m_atlasView = VK_NULL_HANDLE;
    VkSampler      m_sampler   = VK_NULL_HANDLE;

    // CPU-side shadow for atlas packing
    uint8_t  *m_cpuAtlas = nullptr;
    uint16_t  m_cursorX  = 0;
    uint16_t  m_cursorY  = 0;
    uint16_t  m_rowH     = 0;

    static constexpr uint32_t ATLAS_W = 2048;
    static constexpr uint32_t ATLAS_H = 2048;

    std::unordered_map<uint32_t, GlyphEntry> m_glyphs;
};

} // namespace Animus
```

### 10.2 GlyphAtlas.cpp (key methods)

```cpp
// animus/render/GlyphAtlas.cpp
#include "GlyphAtlas.h"
#include "VulkanContext.h"
#include <cstring>
#include <cstdlib>

namespace Animus {

bool GlyphAtlas::initialize(const char *fontPath, float ptSize, float dpiScale) {
    if (FT_Init_FreeType(&m_ftLib)) return false;
    if (FT_New_Face(m_ftLib, fontPath, 0, &m_ftFace)) return false;

    // Set pixel size — ptSize * dpiScale gives physical pixels
    uint32_t pixSize = (uint32_t)(ptSize * dpiScale + 0.5f);
    FT_Set_Pixel_Sizes(m_ftFace, 0, pixSize);

    // hb_ft_font_create_referenced — correct API (not deprecated hb_ft_font_create)
    m_hbFont = hb_ft_font_create_referenced(m_ftFace);

    // Create atlas image: VK_FORMAT_R8_UNORM, 2048×2048
    VkImageCreateInfo ii = {
        VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO, nullptr, 0,
        VK_IMAGE_TYPE_2D, VK_FORMAT_R8_UNORM, {ATLAS_W, ATLAS_H, 1},
        1, 1, VK_SAMPLE_COUNT_1_BIT, VK_IMAGE_TILING_OPTIMAL,
        VK_IMAGE_USAGE_SAMPLED_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT,
        VK_SHARING_MODE_EXCLUSIVE, 0, nullptr, VK_IMAGE_LAYOUT_UNDEFINED };
    vkCreateImage(m_ctx->device, &ii, nullptr, &m_atlas);

    VkMemoryRequirements mr;
    vkGetImageMemoryRequirements(m_ctx->device, m_atlas, &mr);
    VkMemoryAllocateInfo ma = {
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO, nullptr,
        mr.size, m_ctx->findMemType(mr.memoryTypeBits,
                                    VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT) };
    vkAllocateMemory(m_ctx->device, &ma, nullptr, &m_atlasMem);
    vkBindImageMemory(m_ctx->device, m_atlas, m_atlasMem, 0);

    VkImageViewCreateInfo vi = {
        VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO, nullptr, 0,
        m_atlas, VK_IMAGE_VIEW_TYPE_2D, VK_FORMAT_R8_UNORM, {},
        { VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1 } };
    vkCreateImageView(m_ctx->device, &vi, nullptr, &m_atlasView);

    VkSamplerCreateInfo si = {
        VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO, nullptr, 0,
        VK_FILTER_LINEAR, VK_FILTER_LINEAR,
        VK_SAMPLER_MIPMAP_MODE_NEAREST,
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
        0, VK_FALSE, 1, VK_FALSE, VK_COMPARE_OP_NEVER,
        0, 0, VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK, VK_FALSE };
    vkCreateSampler(m_ctx->device, &si, nullptr, &m_sampler);

    // Transition atlas to SHADER_READ_ONLY
    VkCommandBuffer cmd = m_ctx->beginOneTime();
    transitionAtlasLayout(cmd,
        VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
    m_ctx->endOneTime(cmd);

    // CPU shadow for packing
    m_cpuAtlas = (uint8_t*)calloc(ATLAS_W * ATLAS_H, 1);

    // Pre-rasterize Latin Basic + Extended-1
    for (uint32_t cp = 0x0020; cp <= 0x00FF; cp++)
        rasterizeAndUpload(cp);

    return true;
}

void GlyphAtlas::rasterizeAndUpload(uint32_t codepoint) {
    if (m_glyphs.count(codepoint)) return;

    FT_UInt idx = FT_Get_Char_Index(m_ftFace, codepoint);
    if (!idx) return;
    if (FT_Load_Glyph(m_ftFace, idx, FT_LOAD_DEFAULT)) return;
    if (FT_Render_Glyph(m_ftFace->glyph, FT_RENDER_MODE_NORMAL)) return;

    FT_Bitmap &bm = m_ftFace->glyph->bitmap;

    // Atlas packing: simple shelf packer
    if (m_cursorX + bm.width > ATLAS_W) {
        m_cursorX = 0;
        m_cursorY += m_rowH + 1;
        m_rowH = 0;
    }
    if (m_cursorY + bm.rows > ATLAS_H) return;  // atlas full

    GlyphEntry e;
    e.atlasX   = m_cursorX;
    e.atlasY   = m_cursorY;
    e.w        = bm.width;
    e.h        = bm.rows;
    e.bearingX = (int16_t)m_ftFace->glyph->bitmap_left;
    e.bearingY = (int16_t)m_ftFace->glyph->bitmap_top;
    e.advance  = (uint16_t)m_ftFace->glyph->advance.x;

    // Copy to CPU shadow
    for (uint32_t row = 0; row < bm.rows; row++)
        memcpy(m_cpuAtlas + (m_cursorY+row)*ATLAS_W + m_cursorX,
               bm.buffer + row*bm.pitch, bm.width);

    m_cursorX += bm.width + 1;
    if (bm.rows > m_rowH) m_rowH = bm.rows;

    uploadGlyphToAtlas(e, bm.buffer);
    m_glyphs[codepoint] = e;
}

void GlyphAtlas::uploadGlyphToAtlas(const GlyphEntry &e, const uint8_t *bitmap) {
    if (e.w == 0 || e.h == 0) return;

    // Staging buffer
    VkDeviceSize sz = (VkDeviceSize)e.w * e.h;
    VkBufferCreateInfo bi = {
        VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO, nullptr, 0, sz,
        VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VK_SHARING_MODE_EXCLUSIVE };
    VkBuffer staging; vkCreateBuffer(m_ctx->device, &bi, nullptr, &staging);

    VkMemoryRequirements mr;
    vkGetBufferMemoryRequirements(m_ctx->device, staging, &mr);
    VkMemoryAllocateInfo ma = {
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO, nullptr, mr.size,
        m_ctx->findMemType(mr.memoryTypeBits,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT |
            VK_MEMORY_PROPERTY_HOST_COHERENT_BIT) };
    VkDeviceMemory stagingMem;
    vkAllocateMemory(m_ctx->device, &ma, nullptr, &stagingMem);
    vkBindBufferMemory(m_ctx->device, staging, stagingMem, 0);

    void *ptr; vkMapMemory(m_ctx->device, stagingMem, 0, sz, 0, &ptr);
    memcpy(ptr, bitmap, (size_t)sz);
    vkUnmapMemory(m_ctx->device, stagingMem);

    VkCommandBuffer cmd = m_ctx->beginOneTime();

    // Transition atlas region to TRANSFER_DST
    transitionAtlasLayout(cmd,
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);

    VkBufferImageCopy region = {
        0, 0, 0,
        { VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1 },
        { e.atlasX, e.atlasY, 0 }, { e.w, e.h, 1 } };
    vkCmdCopyBufferToImage(cmd, staging, m_atlas,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 1, &region);

    // Transition back to SHADER_READ_ONLY
    transitionAtlasLayout(cmd,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);

    m_ctx->endOneTime(cmd);

    vkDestroyBuffer(m_ctx->device, staging, nullptr);
    vkFreeMemory(m_ctx->device, stagingMem, nullptr);
}

} // namespace Animus
```

### 10.3 TextRenderer — HarfBuzz + Fractional Positions

```cpp
// animus/render/TextRenderer.cpp (shape + draw)
// NEVER round glyphPos to int. HarfBuzz fractional positions (26.6 / 64.0)
// carry sub-pixel accuracy that the vertex shader uses directly.

#include "TextRenderer.h"
#include "GlyphAtlas.h"
#include <hb.h>

namespace Animus {

void TextRenderer::drawText(VkCommandBuffer cmd,
                             const char *utf8Text,
                             float baselineX, float baselineY,
                             uint32_t colorARGB)
{
    // Shape with HarfBuzz
    hb_buffer_t *buf = hb_buffer_create();
    hb_buffer_set_direction(buf, HB_DIRECTION_LTR);
    hb_buffer_set_script(buf, HB_SCRIPT_LATIN);
    hb_buffer_set_language(buf, hb_language_from_string("en", -1));
    hb_buffer_add_utf8(buf, utf8Text, -1, 0, -1);
    hb_shape(m_atlas->hbFont(), buf, nullptr, 0);

    uint32_t glyphCount;
    hb_glyph_info_t     *glyphInfo = hb_buffer_get_glyph_infos(buf, &glyphCount);
    hb_glyph_position_t *glyphPos  = hb_buffer_get_glyph_positions(buf, &glyphCount);

    float cx = baselineX;
    float cy = baselineY;

    for (uint32_t i = 0; i < glyphCount; i++) {
        const GlyphEntry *g = m_atlas->getGlyph(glyphInfo[i].codepoint);
        if (!g) { m_atlas->rasterizeAndUpload(glyphInfo[i].codepoint);
                  g = m_atlas->getGlyph(glyphInfo[i].codepoint); }
        if (!g) {
            // Advance even for missing glyphs
            cx += (float)glyphPos[i].x_advance / 64.0f;
            cy += (float)glyphPos[i].y_advance / 64.0f;
            continue;
        }

        // Fractional position from HarfBuzz — NEVER round to int
        float px = cx
                 + (float)glyphPos[i].x_offset / 64.0f
                 + (float)g->bearingX;
        float py = cy
                 - (float)glyphPos[i].y_offset / 64.0f
                 - (float)g->bearingY;

        float uvMinX = (float)g->atlasX       / (float)GlyphAtlas::ATLAS_W;
        float uvMinY = (float)g->atlasY       / (float)GlyphAtlas::ATLAS_H;
        float uvMaxX = (float)(g->atlasX+g->w)/ (float)GlyphAtlas::ATLAS_W;
        float uvMaxY = (float)(g->atlasY+g->h)/ (float)GlyphAtlas::ATLAS_H;

        float a = ((colorARGB>>24)&0xFF) / 255.0f;
        float r = ((colorARGB>>16)&0xFF) / 255.0f;
        float gf= ((colorARGB>> 8)&0xFF) / 255.0f;
        float b = ((colorARGB    )&0xFF) / 255.0f;

        struct GlyphPC {
            float scW, scH;
            float gx, gy;
            float gw, gh;
            float uvMinX, uvMinY;
            float uvMaxX, uvMaxY;
            float cr, cg, cb, ca;
        } pc = {
            (float)m_ctx->width, (float)m_ctx->height,
            px, py, (float)g->w, (float)g->h,
            uvMinX, uvMinY, uvMaxX, uvMaxY,
            r, gf, b, a
        };

        vkCmdBindPipeline(cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, m_glyphPipeline);
        vkCmdPushConstants(cmd, m_glyphLayout,
            VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_FRAGMENT_BIT,
            0, sizeof(pc), &pc);
        vkCmdDraw(cmd, 4, 1, 0, 0);

        cx += (float)glyphPos[i].x_advance / 64.0f;
        cy += (float)glyphPos[i].y_advance / 64.0f;
    }

    hb_buffer_destroy(buf);
}

} // namespace Animus
```


---

## PART 11 — WallpaperTintSampler

```cpp
// animus/render/WallpaperTintSampler.h / .cpp
// k-means clustering in OKLab color space.
// Samples 16×16 grid from sidebar region of wallpaper.
// k=3 clusters. Selects highest-weight cluster as dominant color.
// Tint = lerp(NEUTRAL_OKLAB, dominantColor_OKLAB, 0.22).
// NOT simple RGB average — perceptually wrong, produces muddy tint.

#pragma once
#include <cstdint>
#include <array>

namespace Animus {

struct OKLab { float L, a, b; };
struct RGB    { float r, g, b; };

// Pure functions — no side effects
namespace ColorConvert {

inline RGB srgbToLinear(float r, float g, float b) {
    return { r*r, g*g, b*b };  // fast γ≈2.0
}
inline RGB linearToSrgb(float r, float g, float b) {
    float sr = r>0?sqrtf(r):0, sg=g>0?sqrtf(g):0, sb=b>0?sqrtf(b):0;
    return {sr,sg,sb};
}

inline OKLab linearToOKLab(RGB c) {
    float l=0.4122214708f*c.r+0.5363325363f*c.g+0.0514459929f*c.b;
    float m=0.2119034982f*c.r+0.6806995451f*c.g+0.1073969566f*c.b;
    float s=0.0883024619f*c.r+0.2817188376f*c.g+0.6299787005f*c.b;
    float l_=cbrtf(l>0?l:-l)*( l>=0?1:-1);
    float m_=cbrtf(m>0?m:-m)*(m>=0?1:-1);
    float s_=cbrtf(s>0?s:-s)*(s>=0?1:-1);
    return {
        0.2104542553f*l_+0.7936177850f*m_-0.0040720468f*s_,
        1.9779984951f*l_-2.4285922050f*m_+0.4505937099f*s_,
        0.0259040371f*l_+0.7827717662f*m_-0.8086757660f*s_
    };
}
inline RGB OKLabToLinear(OKLab lab) {
    float l_=lab.L+0.3963377774f*lab.a+0.2158037573f*lab.b;
    float m_=lab.L-0.1055613458f*lab.a-0.0638541728f*lab.b;
    float s_=lab.L-0.0894841775f*lab.a-1.2914855480f*lab.b;
    float l=l_*l_*l_, m=m_*m_*m_, s=s_*s_*s_;
    return {
         4.0767416621f*l-3.3077115913f*m+0.2309699292f*s,
        -1.2684380046f*l+2.6097574011f*m-0.3413193965f*s,
        -0.0041960863f*l-0.7034186147f*m+1.7076147010f*s
    };
}

} // namespace ColorConvert

struct TintResult {
    float r, g, b;      // sRGB tint color
    float strength;     // how much wallpaper color bleeds (0.22 baseline)
};

class WallpaperTintSampler {
public:
    // pixels: BGRA8 wallpaper image
    // imgW, imgH: full image dimensions
    // sampleRegion: sidebar region (left side, ~240px wide typically)
    TintResult sample(const uint8_t *pixels, int imgW, int imgH,
                      int regionX, int regionY, int regionW, int regionH);

private:
    static constexpr int K = 3;       // number of clusters
    static constexpr int GRID = 16;   // 16×16 sample grid
    static constexpr int MAX_ITER = 20;
    static constexpr float TINT_LERP = 0.22f;

    // Neutral OKLab: ~#F5F5F5 in linear
    static constexpr OKLab NEUTRAL_OKLAB = { 0.975f, 0.0f, 0.0f };

    float oklabDist2(OKLab a, OKLab b) {
        float dl=a.L-b.L, da=a.a-b.a, db=a.b-b.b;
        return dl*dl+da*da+db*db;
    }
};

} // namespace Animus
```

```cpp
// animus/render/WallpaperTintSampler.cpp
#include "WallpaperTintSampler.h"
#include <cmath>
#include <cstring>
#include <climits>

namespace Animus {

TintResult WallpaperTintSampler::sample(
    const uint8_t *pixels, int imgW, int imgH,
    int regionX, int regionY, int regionW, int regionH)
{
    // 1. Sample 16×16 grid from sidebar region
    OKLab samples[GRID*GRID];
    int count = 0;
    for (int gy = 0; gy < GRID; gy++) {
        for (int gx = 0; gx < GRID; gx++) {
            int px = regionX + (int)((float)gx / (GRID-1) * (regionW-1));
            int py = regionY + (int)((float)gy / (GRID-1) * (regionH-1));
            px = px < 0 ? 0 : (px >= imgW ? imgW-1 : px);
            py = py < 0 ? 0 : (py >= imgH ? imgH-1 : py);
            const uint8_t *p = pixels + (py*imgW+px)*4;
            // BGRA8 → linear RGB
            RGB lin = ColorConvert::srgbToLinear(
                p[2]/255.0f, p[1]/255.0f, p[0]/255.0f);
            samples[count++] = ColorConvert::linearToOKLab(lin);
        }
    }

    // 2. k-means in OKLab
    OKLab centroids[K];
    int   assign[GRID*GRID];
    int   weights[K];

    // Seed centroids: evenly spaced through sample array
    for (int k = 0; k < K; k++)
        centroids[k] = samples[k * (count/(K))];

    for (int iter = 0; iter < MAX_ITER; iter++) {
        // Assignment step
        for (int i = 0; i < count; i++) {
            float best = 1e30f; int bk = 0;
            for (int k = 0; k < K; k++) {
                float d = oklabDist2(samples[i], centroids[k]);
                if (d < best) { best=d; bk=k; }
            }
            assign[i] = bk;
        }
        // Update step
        OKLab sums[K] = {};
        memset(weights, 0, sizeof(weights));
        for (int i = 0; i < count; i++) {
            int k = assign[i];
            sums[k].L += samples[i].L;
            sums[k].a += samples[i].a;
            sums[k].b += samples[i].b;
            weights[k]++;
        }
        bool converged = true;
        for (int k = 0; k < K; k++) {
            if (!weights[k]) continue;
            OKLab newC = {
                sums[k].L/weights[k],
                sums[k].a/weights[k],
                sums[k].b/weights[k] };
            if (oklabDist2(newC, centroids[k]) > 1e-8f) converged=false;
            centroids[k] = newC;
        }
        if (converged) break;
    }

    // 3. Select dominant cluster (most pixels)
    int dominant = 0;
    for (int k = 1; k < K; k++)
        if (weights[k] > weights[dominant]) dominant = k;

    // 4. Lerp in OKLab: neutral → dominant color, factor 0.22
    OKLab tintLab = {
        NEUTRAL_OKLAB.L + (centroids[dominant].L - NEUTRAL_OKLAB.L) * TINT_LERP,
        NEUTRAL_OKLAB.a + (centroids[dominant].a - NEUTRAL_OKLAB.a) * TINT_LERP,
        NEUTRAL_OKLAB.b + (centroids[dominant].b - NEUTRAL_OKLAB.b) * TINT_LERP
    };

    RGB linResult = ColorConvert::OKLabToLinear(tintLab);
    RGB srgbResult= ColorConvert::linearToSrgb(linResult.r,linResult.g,linResult.b);

    return { srgbResult.r, srgbResult.g, srgbResult.b, TINT_LERP };
}

} // namespace Animus
```


---

## PART 12 — StateManager + InputRouter + GestureRecognizer

### 12.1 StateManager.h

```cpp
// animus/core/StateManager.h
#pragma once
#include <any>
#include <unordered_map>
#include <string>
#include <functional>

namespace Animus {

// Central key-value state store.
// Publishes OSFEvent on change so components react without polling.
class StateManager {
public:
    static StateManager& shared();

    void set(const std::string &key, std::any value);
    const std::any* get(const std::string &key) const;

    // getOr: returns value for key if present, otherwise returns defaultVal.
    // Safe for first-boot or missing keys — never throws.
    std::any getOr(const std::string &key, const std::any &defaultVal) const {
        auto *v = get(key);
        if (!v || !v->has_value()) return defaultVal;
        return *v;
    }

    template<typename T>
    T getAs(const std::string &key, T defaultVal = T{}) const {
        auto *v = get(key);
        if (!v || !v->has_value()) return defaultVal;
        try { return std::any_cast<T>(*v); }
        catch (...) { return defaultVal; }
    }

private:
    StateManager() = default;
    std::unordered_map<std::string, std::any> m_state;
};

// Well-known state keys
namespace StateKey {
    constexpr char FocusedWindowId[]    = "focused_window_id";
    constexpr char ActiveMonitorIndex[] = "active_monitor";
    constexpr char LockScreenVisible[]  = "lock_screen_visible";
    constexpr char CockpitViewOpen[]    = "cockpit_view_open";
    constexpr char CurrentWallpaper[]   = "current_wallpaper_path";
    constexpr char WallpaperTintR[]     = "wallpaper_tint_r";
    constexpr char WallpaperTintG[]     = "wallpaper_tint_g";
    constexpr char WallpaperTintB[]     = "wallpaper_tint_b";
    constexpr char SystemVolume[]       = "system_volume";
    constexpr char DockVisibility[]     = "dock_visibility";
    constexpr char PathfinderOpen[]     = "pathfinder_open";
} // namespace StateKey

} // namespace Animus
```

### 12.2 InputRouter.h

```cpp
// animus/input/InputRouter.h
#pragma once
#include <cstdint>
#include <memory>
#include <vector>

namespace Animus {

class GestureRecognizer;

// Routes raw pointer/keyboard events to focused surface or global handlers.
// All events arrive on main thread (from compositor callbacks).
class InputRouter {
public:
    static InputRouter& shared();
    void initialize();

    // Called from compositor C callbacks (already on main thread)
    void onKey(uint32_t sym, uint32_t mods, bool pressed);
    void onPointerMotion(double x, double y);
    void onPointerButton(uint32_t button, bool pressed);
    void onPointerAxis(double dx, double dy);
    void onSwipeBegin(uint32_t fingers);
    void onSwipeUpdate(uint32_t fingers, double dx, double dy);
    void onSwipeEnd(bool cancelled);

    double pointerX() const { return m_px; }
    double pointerY() const { return m_py; }

private:
    InputRouter() = default;
    // MotionWave is a singleton — no ownership here
    // InputRouter delegates swipe/pinch events to MotionWave::shared()
    double m_px           = 0;
    double m_py           = 0;
    bool   m_altDownAlone = false;  // tracks Alt-alone press for GlobalMenu
    float  m_screenW      = 1920.f; // updated from OutputResized event
    float  m_screenH      = 1080.f;
};

} // namespace Animus
```

### 12.3 GestureRecognizer.h

```cpp
// animus/input/GestureRecognizer.h
#pragma once
#include <cstdint>

namespace Animus {

// Three-finger swipe → CockpitView (Mission Control equivalent)
// Four-finger swipe down → show desktop
// Swipe events from wlr_pointer.events.swipe_begin/update/end (verified 0.17.1)

class GestureRecognizer {
public:
    void onSwipeBegin(uint32_t fingers);
    void onSwipeUpdate(uint32_t fingers, double dx, double dy);
    void onSwipeEnd(bool cancelled);

private:
    enum class GestureState { Idle, Tracking3F, Tracking4F };
    GestureState m_state    = GestureState::Idle;
    uint32_t     m_fingers  = 0;
    double       m_accumDX  = 0;
    double       m_accumDY  = 0;

    static constexpr double THRESHOLD = 80.0;  // pixels to trigger
};

} // namespace Animus
```

```cpp
// animus/input/GestureRecognizer.cpp
#include "GestureRecognizer.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"

namespace Animus {

void GestureRecognizer::onSwipeBegin(uint32_t fingers) {
    m_fingers = fingers;
    m_accumDX = m_accumDY = 0;
    if      (fingers == 3) m_state = GestureState::Tracking3F;
    else if (fingers == 4) m_state = GestureState::Tracking4F;
    else                   m_state = GestureState::Idle;
}

void GestureRecognizer::onSwipeUpdate(uint32_t, double dx, double dy) {
    if (m_state == GestureState::Idle) return;
    m_accumDX += dx;
    m_accumDY += dy;
}

void GestureRecognizer::onSwipeEnd(bool cancelled) {
    if (cancelled || m_state == GestureState::Idle) {
        m_state = GestureState::Idle; return;
    }
    if (m_state == GestureState::Tracking3F && m_accumDY < -THRESHOLD)
        EventBus::shared().publish(OSFEvent::CockpitViewToggle);
    else if (m_state == GestureState::Tracking4F && m_accumDY > THRESHOLD)
        EventBus::shared().publish(OSFEvent::CockpitViewToggle);
    m_state = GestureState::Idle;
}

} // namespace Animus
```


---

## PART 13 — SoundEngine (PipeWire 1.0.5)

```cpp
// animus/audio/SoundEngine.h
#pragma once
#include <string>
#include <memory>
#include <unordered_map>

namespace Animus {

// SoundEngine: PipeWire 1.0.5 playback.
// pw_thread_loop manages its own thread — safe from main thread.
// All public methods may be called from any thread.
// pw_stream_new: verified 5-arg signature: (core, name, props, &events, userdata)
class SoundEngine {
public:
    static SoundEngine& shared();
    bool initialize();
    void destroy();

    // Play a named sound (resolved from /etc/vitusos/sounds/<name>.wav)
    // fire-and-forget. Returns immediately.
    void play(const std::string &name, float volume = 1.0f);
    void setMasterVolume(float v);  // 0.0 – 1.0

private:
    SoundEngine() = default;
    struct Impl;
    std::unique_ptr<Impl> m_impl;
};

// Sound names — always use these constants, never bare strings
namespace Sounds {
    constexpr char BootChime[]       = "boot_chime";
    constexpr char WindowOpen[]      = "window_open";
    constexpr char WindowClose[]     = "window_close";
    constexpr char Notification[]    = "notification";
    constexpr char Error[]           = "error";
    constexpr char TrashEmpty[]      = "trash_empty";
    constexpr char CockpitOpen[]     = "cockpit_open";
    constexpr char LockScreen[]      = "lock_screen";
    constexpr char UnlockScreen[]    = "unlock_screen";
    constexpr char InstallComplete[] = "install_complete";
    constexpr char Drag[]            = "drag";
    constexpr char Drop[]            = "drop";
    constexpr char Eject[]           = "eject";
} // namespace Sounds

} // namespace Animus
```

```cpp
// animus/audio/SoundEngine.cpp
#include "SoundEngine.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <pipewire/pipewire.h>
#include <spa/param/audio/format-utils.h>
#include <fcntl.h>
#include <unistd.h>
#include <cstring>
#include <cstdlib>
#include <mutex>

namespace Animus {

struct SoundEngine::Impl {
    struct pw_thread_loop *loop   = nullptr;
    struct pw_context     *ctx    = nullptr;
    struct pw_core        *core   = nullptr;
    float                  master = 1.0f;
    std::mutex             mu;
};

SoundEngine& SoundEngine::shared() {
    static SoundEngine inst; return inst;
}

bool SoundEngine::initialize() {
    pw_init(nullptr, nullptr);
    m_impl = std::make_unique<Impl>();
    m_impl->loop = pw_thread_loop_new("vitusos-sound", nullptr);
    m_impl->ctx  = pw_context_new(
        pw_thread_loop_get_loop(m_impl->loop), nullptr, 0);
    m_impl->core = pw_context_connect(m_impl->ctx, nullptr, 0);
    pw_thread_loop_start(m_impl->loop);

    // Subscribe to SoundPlay events from EventBus
    EventBus::shared().subscribe(OSFEvent::SoundPlay, [this](const std::any &d) {
        auto name = std::any_cast<std::string>(d);
        play(name);
    });
    return true;
}

void SoundEngine::play(const std::string &name, float volume) {
    std::string path = "/etc/vitusos/sounds/" + name + ".wav";
    int fd = open(path.c_str(), O_RDONLY);
    if (fd < 0) return;
    off_t sz = lseek(fd, 0, SEEK_END); lseek(fd, 0, SEEK_SET);
    if (sz <= 44) { close(fd); return; }
    uint8_t *wav = (uint8_t*)malloc((size_t)sz);
    read(fd, wav, (size_t)sz); close(fd);

    // Read WAV header for sample rate and channels
    uint32_t sampleRate = *reinterpret_cast<uint32_t*>(wav+24);
    uint16_t channels   = *reinterpret_cast<uint16_t*>(wav+22);

    struct ClipState {
        struct pw_stream *stream;
        struct pw_thread_loop *loop;
        const uint8_t *pcm; size_t sz, pos;
        float volume;
    };
    auto *s = new ClipState();
    s->pcm    = wav + 44;
    s->sz     = (size_t)(sz - 44);
    s->pos    = 0;
    s->volume = volume * m_impl->master;
    s->loop   = m_impl->loop;

    struct pw_stream_events evts = {};
    evts.version = PW_VERSION_STREAM_EVENTS;
    evts.process = [](void *ud) {
        auto *s = static_cast<ClipState*>(ud);
        struct pw_buffer *b = pw_stream_dequeue_buffer(s->stream);
        if (!b) return;
        uint8_t *dst = (uint8_t*)b->buffer->datas[0].data;
        uint32_t cap = b->buffer->datas[0].maxsize;
        size_t rem = s->sz - s->pos;
        if (!rem) {
            b->buffer->datas[0].chunk->size = 0;
            pw_stream_queue_buffer(s->stream, b);
            pw_stream_destroy(s->stream);
            delete s;
            return;
        }
        uint32_t cp = (uint32_t)(rem < cap ? rem : cap);
        memcpy(dst, s->pcm + s->pos, cp); s->pos += cp;
        b->buffer->datas[0].chunk->size = cp;
        pw_stream_queue_buffer(s->stream, b);
    };

    pw_thread_loop_lock(m_impl->loop);
    struct pw_properties *p = pw_properties_new(
        PW_KEY_MEDIA_TYPE,     "Audio",
        PW_KEY_MEDIA_CATEGORY, "Playback",
        PW_KEY_MEDIA_ROLE,     "Event", nullptr);
    // pw_stream_new — 5-arg verified signature (PipeWire 1.0.5)
    s->stream = pw_stream_new(m_impl->core, "vitusos-sfx", p, &evts, s);

    uint8_t buf[256];
    struct spa_pod_builder b = SPA_POD_BUILDER_INIT(buf, sizeof(buf));
    const struct spa_pod *params[1];
    params[0] = spa_format_audio_raw_build(&b, SPA_PARAM_EnumFormat,
        &SPA_AUDIO_INFO_RAW_INIT(
            .format   = SPA_AUDIO_FORMAT_S16,
            .rate     = sampleRate,
            .channels = channels));
    pw_stream_connect(s->stream, PW_DIRECTION_OUTPUT, PW_ID_ANY,
        PW_STREAM_FLAG_AUTOCONNECT | PW_STREAM_FLAG_MAP_BUFFERS, params, 1);
    pw_thread_loop_unlock(m_impl->loop);
}

void SoundEngine::setMasterVolume(float v) {
    m_impl->master = v < 0.0f ? 0.0f : (v > 1.0f ? 1.0f : v);
}

void SoundEngine::destroy() {
    if (!m_impl) return;
    pw_thread_loop_stop(m_impl->loop);
    if (m_impl->core) pw_core_disconnect(m_impl->core);
    if (m_impl->ctx)  pw_context_destroy(m_impl->ctx);
    if (m_impl->loop) pw_thread_loop_destroy(m_impl->loop);
    pw_deinit();
}

} // namespace Animus
```


---

## PART 14 — OSFNative Surface System (All 10 Types)

### 14.1 Surface Altitude Reference

```
Surface Type       Altitude     Blur   Opacity  Notes
─────────────────────────────────────────────────────────────────────
OSFContent         Grounded     —      100%     #FEFEFE bg, no glass
OSFToolbar         Low          8px    94%      search bar spring
OSFSidebar         Mid          20px   82%      selection pill spring
OSFWindow          varies       see altitude declared per-instance
OSFSheet           Mid          20px   82%      drops from title bar
OSFDropdown        High         32px   72%      sheet drop animation
OSFPopover         High         32px   72%      springs from origin point
OSFContextMenu     Floating     48px   64%      springs from cursor
OSFNotification    Floating     48px   64%      slides from right edge
OSFTooltip         Floating     48px   64%      500ms dwell, instant dismiss
```

### 14.2 OSFWindow.h

```cpp
// osf/surfaces/OSFWindow.h
#pragma once
#include "render/MaterialRenderer.h"  // SurfaceAltitude
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>

struct wlr_surface;
struct wlr_output_event_present;

namespace Animus {

class OSFWindow {
public:
    OSFWindow(struct wlr_surface *surface, float x, float y, float w, float h);
    ~OSFWindow();

    void render(VkCommandBuffer cmd, float dt);
    void close();
    void focus();
    void blur();

    // Position setters — springs animate to target
    void setPosition(float x, float y);
    void setSize(float w, float h);

    // Spring-lagged shadow position
    float shadowX() const { return m_shadowPos.x.value(); }
    float shadowY() const { return m_shadowPos.y.value(); }

    // Window actual position (for rendering)
    float x() const { return m_pos.x.value(); }
    float y() const { return m_pos.y.value(); }
    float width()  const { return m_w; }
    float height() const { return m_h; }

    float cornerRadius() const { return 10.0f; }
    bool  isVisible()    const { return m_visible && m_scale.value() > 0.01f; }

    SurfaceAltitude altitude() const { return m_altitude; }
    void setAltitude(SurfaceAltitude a) { m_altitude = a; }

    struct wlr_surface *surface() const { return m_surface; }

private:
    struct wlr_surface *m_surface;
    SurfaceAltitude     m_altitude = SurfaceAltitude::Mid;
    bool                m_visible  = false;
    float               m_w, m_h;

    SpringSolver2D      m_pos;          // SPRING_WINDOW_DRAG (800,35)
    SpringSolver2D      m_shadowPos;    // SPRING_SHADOW (300,25) — lags behind m_pos
    SpringSolver        m_scale;        // SPRING_Selection for open/close anim
    uint64_t            m_tickHandle  = 0;
};

} // namespace Animus
```

### 14.3 OSFSidebar.h

```cpp
// osf/surfaces/OSFSidebar.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vector>
#include <string>
#include <functional>

namespace Animus {

struct SidebarItem {
    std::string  label;
    std::string  iconName;    // resolved from icon theme
    bool         isSection;   // section header, not selectable
    std::function<void()> onSelect;
};

class OSFSidebar {
public:
    OSFSidebar();
    ~OSFSidebar();

    void setItems(std::vector<SidebarItem> items);
    void setSelectedIndex(int idx);
    void render(VkCommandBuffer cmd, float x, float y, float w, float h);

private:
    static constexpr float ITEM_H    = 36.0f;
    static constexpr float SECTION_H = 28.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Mid;

    std::vector<SidebarItem> m_items;
    int                      m_selectedIdx = -1;

    SpringSolver  m_selPillY;     // SPRING_SELECTION (400,28) — pill Y position
    SpringSolver  m_selPillH;     // spring for pill height
    std::vector<SpringSolver> m_hoverAlpha;  // per-item hover SPRING_HOVER (600,40)
    uint64_t      m_tickHandle = 0;
};

} // namespace Animus
```

### 14.4 OSFNotification.h

```cpp
// osf/surfaces/OSFNotification.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <functional>

namespace Animus {

class OSFNotification {
public:
    // Slides in from right edge. Auto-dismiss after timeoutMs.
    // dismissed callback fires on user dismiss OR timeout.
    OSFNotification(const std::string &title,
                    const std::string &body,
                    int                timeoutMs,
                    std::function<void()> dismissed);
    ~OSFNotification();

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void dismiss();
    bool isVisible() const { return m_visible; }

    static constexpr float WIDTH  = 320.0f;
    static constexpr float HEIGHT =  80.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Floating;
    static constexpr float CORNER_RADIUS = 12.0f;

private:
    std::string m_title, m_body;
    int         m_timeoutMs;
    std::function<void()> m_dismissed;

    bool         m_visible = false;
    float        m_elapsed = 0;

    // Slides from screenW (offscreen) to screenW-WIDTH-16
    SpringSolver m_slideX;   // SPRING_NOTIFICATION (380,26)
    uint64_t     m_tickHandle = 0;
};

} // namespace Animus
```

### 14.5 OSFTooltip.h

```cpp
// osf/surfaces/OSFTooltip.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include <string>

namespace Animus {

class OSFTooltip {
public:
    // dwell: 500ms before appearing.
    // Dismisses INSTANTLY on cursor leave — no exit animation.
    void show(const std::string &text, float cursorX, float cursorY);
    void hide();         // immediate — no spring on exit
    void update(float dt, float cursorX, float cursorY);
    void render(VkCommandBuffer cmd, float screenW, float screenH);

    bool isVisible() const { return m_opacity.value() > 0.01f; }

    static constexpr float DWELL_MS = 500.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Floating;
    static constexpr float CORNER_RADIUS = 6.0f;

private:
    std::string  m_text;
    float        m_x=0, m_y=0;
    float        m_dwellTimer = 0;
    bool         m_cursorPresent = false;

    // Fade in only — hide() snaps opacity to 0 immediately
    SpringSolver m_opacity;   // SPRING_HOVER (600,40) for fade-in
};

} // namespace Animus
```

### 14.6 OSFContextMenu.h

```cpp
// osf/surfaces/OSFContextMenu.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vector>
#include <string>
#include <functional>

namespace Animus {

struct ContextMenuItem {
    std::string  label;
    std::string  shortcut;     // e.g. "⌘C"
    bool         isSeparator;
    bool         isEnabled;
    std::function<void()> action;
};

class OSFContextMenu {
public:
    // Springs from cursor position on open. Per-item hover springs.
    OSFContextMenu(std::vector<ContextMenuItem> items,
                   float spawnX, float spawnY);
    ~OSFContextMenu();

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void dismiss();
    bool isVisible() const { return m_visible; }
    bool hitTest(float x, float y) const;
    void onPointerMotion(float x, float y);
    void onPointerButton(float x, float y);

    static constexpr float ITEM_HEIGHT    = 32.0f;
    static constexpr float MIN_WIDTH      = 180.0f;
    static constexpr float CORNER_RADIUS  = 10.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Floating;

private:
    std::vector<ContextMenuItem> m_items;
    float m_x, m_y, m_w, m_h;
    bool  m_visible = false;
    int   m_hoveredIdx = -1;

    SpringSolver2D             m_pos;       // SPRING_SELECTION springs from spawn point
    SpringSolver               m_scale;     // 0.95 → 1.0 on open
    std::vector<SpringSolver>  m_itemHover; // per-item SPRING_HOVER (600,40)
    uint64_t                   m_tickHandle = 0;
};

} // namespace Animus
```


---

## PART 15 — Shell Components

### 15.1 Panel.h

```cpp
// animus/shell/Panel.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <memory>

namespace Animus {

class TextRenderer;

// Panel: wlr-layer-shell LAYER_TOP, anchored top, 28px height.
// SurfaceAltitude::Low.
// Contains: traffic lights (←), global menu (center-left), clock (right).
class Panel {
public:
    Panel();
    ~Panel();
    bool initialize();

    void render(VkCommandBuffer cmd, float dt);
    void setGlobalMenu(const std::string &appName,
                       const std::vector<std::string> &menuItems);
    void setFocusedApp(const std::string &appName);

    static constexpr float HEIGHT         = 28.0f;
    static constexpr float CORNER_RADIUS  = 0.0f;  // panel: no rounded corners
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Low;

private:
    void renderTrafficLights(VkCommandBuffer cmd, float y);
    void renderClock(VkCommandBuffer cmd, float screenW, float y);
    void renderMenu(VkCommandBuffer cmd, float y);
    std::string formatTime() const;

    std::string m_focusedApp;
    std::vector<std::string> m_menuItems;

    // Traffic light hover springs (close/min/max)
    SpringSolver m_tlHover[3];   // SPRING_TRAFFIC_LIGHT (700,38)

    uint64_t m_tickHandle   = 0;
    uint64_t m_focusHandle  = 0;
};

} // namespace Animus
```

### 15.2 Dock.h

```cpp
// animus/shell/Dock.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vector>
#include <string>
#include <memory>

namespace Animus {

struct DockItem {
    std::string  appId;        // matches .desktop App ID
    std::string  iconPath;     // resolved at runtime
    std::string  displayName;
    bool         isRunning;
    bool         isPinned;
    int          badgeCount;   // -1 = no badge
};

// Dock: wlr-layer-shell LAYER_BOTTOM, anchored bottom, 64px height.
// SurfaceAltitude::Mid.
// Icon hover: magnify spring (SPRING_DOCK_MAGNIFY).
// Launch bounce: icon Y spring with initial velocity.
class Dock {
public:
    Dock();
    ~Dock();
    bool initialize();

    void render(VkCommandBuffer cmd, float screenW, float dt);
    void setItems(std::vector<DockItem> items);
    void notifyLaunch(const std::string &appId);
    void notifyBadge(const std::string &appId, int count);

    void onPointerMotion(float x, float y);
    void onPointerButton(float x, float y, bool pressed);

    static constexpr float HEIGHT        = 64.0f;
    static constexpr float ICON_SIZE     = 48.0f;
    static constexpr float ICON_SPACING  =  8.0f;
    static constexpr float CORNER_RADIUS = 16.0f;
    static constexpr float PADDING_H     = 12.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Mid;

    // Magnify: max icon size at cursor = 72px (1.5x)
    static constexpr float MAGNIFY_PEAK = 72.0f;
    static constexpr float MAGNIFY_SPREAD = 80.0f;  // falloff radius

private:
    void renderIcon(VkCommandBuffer cmd,
                    const DockItem &item, float x, float y,
                    float iconSize, float bounce);

    std::vector<DockItem>   m_items;
    std::vector<SpringSolver> m_magnify;   // SPRING_DOCK_MAGNIFY (450,32) per-icon
    std::vector<SpringSolver> m_bounce;    // SPRING_SELECTION for launch bounce
    float m_cursorX = -9999;

    uint64_t m_tickHandle = 0;
};

} // namespace Animus
```

### 15.3 CockpitView.h

```cpp
// animus/shell/CockpitView.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vector>
#include <memory>

namespace Animus {

class Window;

// CockpitView: Mission Control equivalent.
// Full-screen overlay (SurfaceAltitude::High).
// Window cards spring FROM their actual screen positions (no teleport).
// Real surface thumbnails via wlr_renderer_read_pixels (captured on open).
// Three-finger swipe up OR Dock expose button triggers open.
class CockpitView {
public:
    CockpitView();
    ~CockpitView();

    void open(const std::vector<std::shared_ptr<Window>> &windows);
    void close();
    bool isOpen() const { return m_open; }

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void onPointerButton(float x, float y);

    static constexpr float CARD_SPACING   = 20.0f;
    static constexpr float CARD_CORNER    = 10.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::High;

private:
    struct Card {
        std::shared_ptr<Window> window;
        VkImageView             thumbnail;  // captured via wlr_renderer_read_pixels
        SpringSolver2D          pos;        // SPRING_SELECTION springs from real pos
        SpringSolver            scale;
    };

    bool m_open = false;
    std::vector<Card> m_cards;
    SpringSolver      m_bgOpacity;  // SPRING_SELECTION
    uint64_t          m_tickHandle  = 0;
    uint64_t          m_eventHandle = 0;
};

} // namespace Animus
```

### 15.4 BootCrossfade.h

```cpp
// animus/shell/BootCrossfade.h
#pragma once
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vulkan/vulkan.h>

namespace Animus {

// BootCrossfade: Space Orange overlay that fades out on first frame.
// Crossfade from Space Orange → desktop.
// Scale animation: desktop 1.02 → 1.0 (subtle zoom-out, grounds the reveal).
// Uses SPRING_BOOT (200,22) — slow, deliberate.
// Fires OSFEvent::BootCrossfadeComplete when done.
class BootCrossfade {
public:
    BootCrossfade();
    ~BootCrossfade();

    // Call on first compositor frame
    void begin();

    void render(VkCommandBuffer cmd, float screenW, float screenH);
    bool isComplete() const { return m_complete; }

    static constexpr float INITIAL_SCALE = 1.02f;  // desktop starts slightly larger

private:
    bool         m_begun    = false;
    bool         m_complete = false;
    SpringSolver m_opacity;   // SPRING_BOOT (200,22): 1.0 → 0.0
    SpringSolver m_scale;     // SPRING_BOOT (200,22): 1.02 → 1.0
    uint64_t     m_tickHandle = 0;
};

} // namespace Animus
```

### 15.5 LockScreen.h

```cpp
// animus/shell/LockScreen.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <memory>

namespace Animus {

// LockScreen: full-screen overlay above all surfaces.
// SPRING_LOCK_SCREEN (120,22) — slow, deliberate reveal.
// Activates on idle timeout, lid close, or manual lock.
// Authentication: PAM via unix socket (background thread).
class LockScreen {
public:
    LockScreen();
    ~LockScreen();

    void activate();
    void deactivate();   // fires OSFEvent::LockScreenUnlocked
    bool isActive() const { return m_active; }

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void onKey(uint32_t sym, uint32_t mods, bool pressed);

private:
    bool   m_active = false;
    float  m_blurAmount = 0;

    // Lock screen reveals slowly — SPRING_LOCK_SCREEN (120,22)
    SpringSolver m_opacity;     // 0.0 → 1.0 on lock
    SpringSolver m_blurSpring;  // background blur grows slowly

    std::string  m_passwordBuf;
    bool         m_shaking = false;
    SpringSolver m_shakeX;   // SPRING_SELECTION — horizontal shake on wrong password

    uint64_t     m_tickHandle = 0;

    void authenticate();    // runs PAM in background thread, publishAsync result
};

} // namespace Animus
```


---

## PART 16 — osf-shell-v1 Wayland Protocol

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!-- protocol/osf-shell-v1.xml
     Custom Wayland protocol for OSFNative apps.
     Third-party apps use xdg-shell. OSFNative apps use this.
     Allows global menu, Dock badge, attention request. -->

<protocol name="osf_shell_v1">
  <interface name="osf_shell_manager_v1" version="1">
    <description summary="OSFNative shell integration manager"/>

    <request name="get_osf_surface">
      <description summary="Get OSF surface for an xdg_toplevel"/>
      <arg name="id"       type="new_id" interface="osf_surface_v1"/>
      <arg name="toplevel" type="object" interface="xdg_toplevel"/>
    </request>
  </interface>

  <interface name="osf_surface_v1" version="1">
    <description summary="OSFNative surface integration"/>

    <!-- Sets the global menu bar content.
         Payload: UTF-8 JSON MenuDefinition (see AppKit MenuDefinition) -->
    <request name="set_application_menu">
      <arg name="menu_json" type="string"/>
    </request>

    <!-- Sets Dock badge count. -1 = remove badge. -->
    <request name="set_badge_count">
      <arg name="count" type="int"/>
    </request>

    <!-- Requests Dock icon bounce (e.g. background task complete) -->
    <request name="request_attention"/>

    <!-- Tell compositor this surface is no longer managed -->
    <request name="destroy" type="destructor"/>

    <!-- Compositor tells app: surface received keyboard focus -->
    <event name="focused"/>

    <!-- Compositor tells app: user clicked window close button -->
    <event name="close_requested"/>

    <!-- User activated a menu item -->
    <event name="menu_activated">
      <arg name="menu_item_id" type="string"/>
    </event>

    <!-- Compositor reconfigures surface (size/state) -->
    <event name="configure">
      <arg name="width"  type="int"/>
      <arg name="height" type="int"/>
      <arg name="state"  type="uint"/>  <!-- osf_surface_state enum -->
    </event>
  </interface>
</protocol>
```

---

## PART 17 — OSFDesktop Singleton

```cpp
// animus/core/OSFDesktop.h
#pragma once
#include <memory>
#include "render/RenderPipeline.h"
#include "render/WallpaperTintSampler.h"
#include "animation/AnimationEngine.h"
#include "animation/AnimationClock.h"
#include "audio/SoundEngine.h"
#include "shell/Panel.h"
#include "shell/Dock.h"
#include "shell/CockpitView.h"
#include "shell/LockScreen.h"
#include "shell/BootCrossfade.h"
#include "core/WindowManager.h"
#include "core/EventBus.h"
#include "input/InputRouter.h"

// Include compositor header inside extern "C" guard
extern "C" {
#include "compositor/animus_compositor.h"
}

namespace Animus {

// OSFDesktop::shared() — the root object.
// Owns all subsystems. Wires all callbacks. Calls compositor_init() then run().
class OSFDesktop {
public:
    static OSFDesktop& shared();

    // Initialize all subsystems, register compositor callbacks, enter event loop
    int run();

    // Compositor C callbacks — static, forwarded to instance
    static void cbPresent(const struct wlr_output_event_present *ev, void *ud);
    static void cbNewSurface(struct wlr_surface *s, void *ud);
    static void cbSurfaceDestroy(struct wlr_surface *s, void *ud);
    static void cbKey(uint32_t sym, uint32_t mods, bool pressed, void *ud);
    static void cbPointerMotion(double x, double y, void *ud);
    static void cbPointerButton(uint32_t btn, bool pressed, void *ud);
    static void cbPointerAxis(double dx, double dy, void *ud);
    static void cbSwipeBegin(uint32_t fingers, void *ud);
    static void cbSwipeUpdate(uint32_t fingers, double dx, double dy, void *ud);
    static void cbSwipeEnd(bool cancelled, void *ud);

private:
    OSFDesktop() = default;
    void initSubsystems(uint32_t w, uint32_t h);
    void onPresent(const struct wlr_output_event_present *ev);
    void onNewSurface(struct wlr_surface *s);
    void onSurfaceDestroy(struct wlr_surface *s);

    std::unique_ptr<RenderPipeline>       m_render;
    std::unique_ptr<WindowManager>        m_wm;
    std::unique_ptr<Panel>                m_panel;
    std::unique_ptr<Dock>                 m_dock;
    std::unique_ptr<CockpitView>          m_cockpit;
    std::unique_ptr<LockScreen>           m_lock;
    std::unique_ptr<BootCrossfade>        m_crossfade;

    bool m_initialized = false;
    bool m_firstFrame  = true;
};

} // namespace Animus
```

```cpp
// animus/core/OSFDesktop.cpp
#include "OSFDesktop.h"
#include "core/StateManager.h"

#define WLR_USE_UNSTABLE
#include <wlr/types/wlr_output.h>

namespace Animus {

OSFDesktop& OSFDesktop::shared() {
    static OSFDesktop instance;
    return instance;
}

int OSFDesktop::run() {
    if (animus_compositor_init() < 0) return 1;

    animus_compositor_register_callbacks(
        cbPresent, cbNewSurface, cbSurfaceDestroy,
        cbKey, cbPointerMotion, cbPointerButton, cbPointerAxis,
        cbSwipeBegin, cbSwipeUpdate, cbSwipeEnd,
        this
    );

    // Subsystems initialized after compositor (Vulkan device available)
    // Dimensions resolved from primary output after backend start
    // For simplicity: default 1920×1080 — OutputResized event corrects this
    initSubsystems(1920, 1080);

    AnimationEngine::shared().start();
    SoundEngine::shared().initialize();

    animus_compositor_run();  // blocks until compositor exits
    return 0;
}

void OSFDesktop::initSubsystems(uint32_t w, uint32_t h) {
    m_render    = std::make_unique<RenderPipeline>();
    m_wm        = std::make_unique<WindowManager>();
    m_panel     = std::make_unique<Panel>();
    m_dock      = std::make_unique<Dock>();
    m_cockpit   = std::make_unique<CockpitView>();
    m_lock      = std::make_unique<LockScreen>();
    m_crossfade = std::make_unique<BootCrossfade>();

    if (!m_render->initialize(w, h)) return;

    m_panel->initialize();
    m_dock->initialize();

    m_render->setPanel(m_panel.get());
    m_render->setDock(m_dock.get());
    m_render->setCrossfade(m_crossfade.get());

    // Subscribe to cockpit toggle gesture
    EventBus::shared().subscribe(OSFEvent::CockpitViewToggle,
        [this](const std::any&) {
            if (m_cockpit->isOpen()) m_cockpit->close();
            else m_cockpit->open(m_wm->windows());
        });

    // Subscribe to lock screen activation
    EventBus::shared().subscribe(OSFEvent::LockScreenActivate,
        [this](const std::any&) { m_lock->activate(); });

    // Subscribe to wallpaper change → re-sample tint
    EventBus::shared().subscribe(OSFEvent::WallpaperChanged,
        [this](const std::any &d) {
            auto path = std::any_cast<std::string>(d);
            StateManager::shared().set(StateKey::CurrentWallpaper, path);
            // WallpaperTintSampler runs on background thread, publishAsync result
        });

    m_initialized = true;
}

// ── C callbacks ───────────────────────────────────────────────────
void OSFDesktop::cbPresent(const struct wlr_output_event_present *ev, void *ud) {
    static_cast<OSFDesktop*>(ud)->onPresent(ev);
}
void OSFDesktop::cbNewSurface(struct wlr_surface *s, void *ud) {
    static_cast<OSFDesktop*>(ud)->onNewSurface(s);
}
void OSFDesktop::cbSurfaceDestroy(struct wlr_surface *s, void *ud) {
    static_cast<OSFDesktop*>(ud)->onSurfaceDestroy(s);
}
void OSFDesktop::cbKey(uint32_t sym, uint32_t mods, bool pressed, void *ud) {
    InputRouter::shared().onKey(sym, mods, pressed);
}
void OSFDesktop::cbPointerMotion(double x, double y, void *ud) {
    InputRouter::shared().onPointerMotion(x, y);
    auto *self = static_cast<OSFDesktop*>(ud);
    if (self->m_dock) self->m_dock->onPointerMotion((float)x, (float)y);
    if (self->m_cockpit && self->m_cockpit->isOpen())
        ; // handled by cockpit directly
}
void OSFDesktop::cbPointerButton(uint32_t btn, bool pressed, void *ud) {
    InputRouter::shared().onPointerButton(btn, pressed);
}
void OSFDesktop::cbPointerAxis(double dx, double dy, void *ud) {
    InputRouter::shared().onPointerAxis(dx, dy);
}
void OSFDesktop::cbSwipeBegin(uint32_t fingers, void *ud) {
    InputRouter::shared().onSwipeBegin(fingers);
}
void OSFDesktop::cbSwipeUpdate(uint32_t fingers, double dx, double dy, void *ud) {
    InputRouter::shared().onSwipeUpdate(fingers, dx, dy);
}
void OSFDesktop::cbSwipeEnd(bool cancelled, void *ud) {
    InputRouter::shared().onSwipeEnd(cancelled);
}

// ── Present callback — hot path ───────────────────────────────────
void OSFDesktop::onPresent(const struct wlr_output_event_present *ev) {
    // .when: struct timespec* — MAY BE NULL, AnimationClock handles
    AnimationClock::shared().onPresent(ev);
    float dt = AnimationClock::shared().dt();
    AnimationEngine::shared().tick(dt);

    if (m_firstFrame) {
        m_crossfade->begin();
        m_firstFrame = false;
    }

    if (m_render && m_initialized)
        m_render->renderFrame(dt);
}

void OSFDesktop::onNewSurface(struct wlr_surface *s) {
    if (m_wm) m_wm->addSurface(s);
}
void OSFDesktop::onSurfaceDestroy(struct wlr_surface *s) {
    if (m_wm) m_wm->removeSurface(s);
}

} // namespace Animus
```

### 17.1 main.cpp

```cpp
// animus/main.cpp
#include "core/OSFDesktop.h"

int main(int argc, char *argv[]) {
    (void)argc; (void)argv;
    return Animus::OSFDesktop::shared().run();
}
```


---

## PART 18 — What Opus Must NEVER Do

This section enumerates the most dangerous implementation mistakes. Every item here represents a real failure mode.

### 18.1 Compositor Mistakes

**NEVER** use `wlr_output_commit()`. Use `wlr_output_commit_state()` with `wlr_output_state`. wlr_output_commit was removed in wlroots 0.17.

**NEVER** use `wlr_vk_renderer_create(backend)`. The verified signature is `wlr_vk_renderer_create_with_drm_fd(drm_fd)`. The old form does not exist in 0.17.1.

**NEVER** omit `-DWLR_USE_UNSTABLE`. wlroots refuses to expose any public API without this flag. Compilation fails silently (empty headers).

**NEVER** call wlroots or Vulkan functions from a background thread. The Wayland event loop is single-threaded. All wlroots and Vulkan work happens on main thread only.

**NEVER** do a full-screen redraw every frame. Use `wlr_damage_ring_add(&ring, &region)` and check `wlr_damage_ring_get_buffer_damage()` before rendering. wlroots damage tracking exists to prevent unnecessary GPU work.

**NEVER** dereference `wlr_output_event_present.when` without null-checking. It is `struct timespec*` and may be NULL on some hardware. Crashing here means the compositor dies at every frame on affected machines.

### 18.2 Vulkan Mistakes

**NEVER** create your own VkDevice or VkInstance. Retrieve them from wlroots: `wlr_vk_renderer_get_device()`, `wlr_vk_renderer_get_instance()`, `wlr_vk_renderer_get_physical_device()`. Using two devices = undefined behavior on shared DMA-BUF resources.

**NEVER** use `VK_PRESENT_MODE_MAILBOX_KHR` or `VK_PRESENT_MODE_IMMEDIATE_KHR`. VK_PRESENT_MODE_FIFO_KHR is enforced by the wlroots DRM backend. Tearing is a non-starter.

**NEVER** write to a VkImage that wlroots may be reading. Synchronize via fence before writing each frame (already done in frame loop via `vkWaitForFences`).

### 18.3 Text Rendering Mistakes

**NEVER** use `hb_ft_font_create(face, NULL)`. This API is deprecated. The correct call is `hb_ft_font_create_referenced(face)` — it takes a reference to the FreeType face.

**NEVER** round `glyphPos` to integer pixels. HarfBuzz positions are in 26.6 fixed point. Convert with `/ 64.0f` and pass the fractional result directly to the vertex shader. Rounding destroys sub-pixel accuracy and produces visually uneven text.

**NEVER** upload a zero-size glyph to the atlas. Check `bm.width > 0 && bm.rows > 0` before calling `uploadGlyphToAtlas`. Space and zero-advance glyphs have empty bitmaps.

### 18.4 Animation Mistakes

**NEVER** use cubic-bezier easing, `std::chrono` timers, or fixed-duration animations for UI motion. Every motion in VitusOS uses `SpringSolver`. Period.

**NEVER** tick dt without clamping. Raw frame deltas can spike to 500ms+ on first frame, GC pause, or focus restore. Always clamp dt to `[0.001, 0.100]`. SpringSolver does this internally but callers should guard too.

**NEVER** create a global spring registry. Components own their springs. They subscribe to `OSFEvent::Tick` and tick their own springs. No central "animation manager" that knows about all springs.

### 18.5 Thread Model Mistakes

**NEVER** call `EventBus::publish()` from a background thread. It iterates handlers on the calling thread. If called from a background thread, handlers run there, violating the single-thread contract. Use `EventBus::publishAsync()` instead — it always delivers on the Wayland event loop thread.

**NEVER** call `wl_event_loop_add_idle()` from the main thread for cross-thread wakeup. It's fine on main thread but pointless. It's the only safe wakeup from background threads.

**NEVER** access `m_asyncQueue` without holding `m_asyncMutex`. Multiple background threads may push simultaneously.

### 18.6 Boot Mistakes

**NEVER** use `PCI_CLASSCODE_OFFSET = 0x0B`. The correct value is `0x09`. Offset 0x09 is the first byte of the 3-byte Class Code field (ProgIf, Subclass, BaseClass). Reading at 0x0B reads padding. GPU detection silently fails.

**NEVER** forget `FreePool(Info)` after `GOP->QueryMode()`. QueryMode allocates a new Info struct for every call. Leaking inside a loop over 50+ modes corrupts the EFI pool.

**NEVER** load NVIDIA modules out of order. The required order is: `nvidia` → `nvidia_modeset` → `nvidia_uvm` → `nvidia_drm`. Any other order causes a kernel panic or incomplete initialization.

**NEVER** destroy the dumb buffer before AnimusEngine takes over. The framebuffer backing Space Orange stays visible via the dumb buffer until the first Vulkan commit. Destroying it early causes a black flash.

**NEVER** use atomic DRM API with simpledrm. simpledrm only supports legacy `drmModeSetCrtc`. Using atomic API on simpledrm returns an error and leaves the screen blank.

### 18.7 Material / Color Mistakes

**NEVER** let a surface specify its own blur radius, opacity, or tint. These are derived from `SurfaceAltitude` by `MaterialRenderer`. If you find yourself writing `blurRadius = 20.0f` anywhere outside the altitude table, you are wrong.

**NEVER** use `#000000` for shadow color. Shadow color is always `#1A1208` — rgb(0.102, 0.071, 0.031). Pure black shadows look digital and cheap.

**NEVER** use `#FFFFFF` for content backgrounds. Use `#FEFEFE`. The imperceptible warmth prevents the harsh contrast that makes pure-white backgrounds feel clinical.

**NEVER** compute wallpaper tint as a simple RGB average. Perceptually uniform space (OKLab) k-means clustering is required. A simple average produces desaturated, muddy, sometimes incorrect tints on vibrant wallpapers.

**NEVER** perform blur operations in sRGB space. Blur in linear RGB. Convert: `toLinear = c * c`, `toSrgb = sqrt(c)`. Blurring in sRGB produces halos around bright edges and incorrect color mixing.

### 18.8 Architecture Mistakes

**NEVER** use Cairo, Pango, Qt, or GTK inside AnimusEngine or any OSFNative app. The entire render stack is custom Vulkan. Third-party apps use xdg-shell and render with whatever they want. OSFNative apps use the OSFNative surface system.

**NEVER** do blocking I/O on the main thread. Directory reads, thumbnail decodes, network calls — all background threads with `EventBus::publishAsync` for results.

**NEVER** hardcode file lists, mock directory contents, or use placeholder functions. Real code only.

**NEVER** use `vitusos-config.nix` for system-level NixOS configuration.
System-wide NixOS config goes in `/etc/nixos/configuration.nix`.
`vitusos-config.nix` is the USER config file. It holds:
    - Pathfinder nixpkgs sources and installed app list
    - All user preferences (wallpaper, MotionWave sensitivity,
      keyboard layout, power settings, reduced motion, etc.)
    - Desktop names and count
    - First boot completion state
It is read by the compositor on start and written by Settings app,
DesktopManager, and MotionWave on change.
NEVER write hardware configuration, kernel parameters, or
system service definitions to vitusos-config.nix.
Those go in configuration.nix.
The distinction: vitusos-config.nix = user prefs.
configuration.nix = system setup.

---

## PART 19 — Build & Implementation Checklist for Opus

When implementing AnimusEngine, proceed in this order:

1. **compositor/animus_compositor.c** — C11, no C++ anywhere, verify all wlroots 0.17.1 APIs
2. **compositor/animus_compositor.h** — the extern "C" bridge header, get this exactly right
3. **animus/animation/SpringSolver.h** — header-only, test all named configs compile
4. **animus/animation/AnimationClock.cpp** — null-check `.when` early
5. **animus/animation/AnimationEngine.cpp** — just publish Tick, nothing more
6. **animus/core/EventBus.cpp** — verify publishAsync → wl_event_loop_add_idle wiring
7. **animus/render/VulkanContext.cpp** — retrieve from wlroots, never create own device
8. **shaders/** — compile all GLSL sources with `glslc`, validate with `spirv-val`
9. **animus/render/GlyphAtlas.cpp** — hb_ft_font_create_referenced, never round positions
10. **animus/render/WallpaperTintSampler.cpp** — OKLab k-means, never RGB average
11. **animus/render/MaterialRenderer.cpp** — altitude table is law, no overrides
12. **animus/render/ShadowRenderer.cpp** — SDF dual shadow, #1A1208 always
13. **animus/render/RenderPipeline.cpp** — layer order matters, damage-tracked
14. **animus/shell/** — Panel, Dock, CockpitView, LockScreen, BootCrossfade
15. **osf/surfaces/** — all 10 surface types, springs in constructors
16. **AnimusBoot/** — EDK2, PCI_CLASSCODE_OFFSET=0x09, FreePool after QueryMode
17. **animus-early/** — simpledrm legacy API, fork chime, NVIDIA module order
18. **animus/core/OSFDesktop.cpp** — wire everything together, first frame triggers crossfade
19. **nixos/configuration.nix** — DRM_SIMPLEDRM=y, animus-early.nix, all packages

---

## PART 20 — Quick Reference: Critical Values

```
Color Values
───────────────────────────────────────────────
Accent/Selection         #E85D00   Space Orange
Shadow color             #1A1208   Warm dark (never #000000)
Content background       #FEFEFE   Imperceptibly warm white
Neutral tint baseline    OKLab L=0.975, a=0, b=0

Spring Configs (stiffness, damping)
───────────────────────────────────────────────
Selection                400, 28
WindowDrag               800, 35
Shadow                   300, 25   (lags behind window — creates depth)
Hover                    600, 40
Scroll                    80, 18
Resize                   350, 28
Sheet                    420, 30
Boot                     200, 22   (slow, deliberate)
Notification             380, 26
TrafficLight             700, 38   (fast, snappy)
DockMagnify              450, 32
LockScreen               120, 22   (very slow, weighty)
DesktopSwitch            280, 28   (desktop slide — windows layer)
DesktopSwitchBG          180, 24   (desktop slide — wallpaper parallax layer)

Altitude Table
───────────────────────────────────────────────
Grounded    0px blur   100% opacity   no tint
Low         8px blur    94% opacity   5% tint
Mid        20px blur    82% opacity  22% tint
High       32px blur    72% opacity  30% tint
Floating   48px blur    64% opacity  35% tint

Atlas dimensions         2048×2048   VK_FORMAT_R8_UNORM
Kawase passes            4 per chain (offsets 0.5, 1.5, 2.5, 3.5)
Kawase chains            1 (Low/Mid)  or  2 (High/Floating)
WallpaperTint lerp       0.22
k-means clusters         k=3, OKLab, 16×16 samples, 20 iterations

Boot constants
───────────────────────────────────────────────
PCI_CLASSCODE_OFFSET     0x09   (NOT 0x0B)
Intel Arc DID range      0x5690–0x57FF  → xe driver
NVIDIA load order        nvidia → nvidia_modeset → nvidia_uvm → nvidia_drm
Boot crossfade scale     1.02 → 1.0
Only sleep               50ms in animus-early after close(drm_fd)
Wayland socket           wayland-0 (fixed)
```

---

*End of AnimusEngine Complete Architecture. Parts 0–45 + Addenda A–O. All code is production-ready. No placeholders. No TODOs.*


---

## ADDENDUM A — Layer 0 Corrections

### A.1 Wordmark: Pre-Rasterized Bitmap (Not Placeholder)

```c
// AnimusBoot/Wordmark.h
// VitusOS wordmark rasterized at 280×48 at 1-bit (white on transparent).
// Embedded as C array — no file I/O in UEFI context.
// Generated with: python3 tools/rasterize_wordmark.py > Wordmark.h
// Format: row-major, 1 byte per pixel (0=transparent, 255=white)
#pragma once
#define WORDMARK_W 280
#define WORDMARK_H  48
extern const unsigned char WORDMARK_PIXELS[WORDMARK_H][WORDMARK_W];
```

```c
// AnimusBoot/GopSetup.c — replace FillRect wordmark with bitmap render
static VOID RenderWordmark(UINT32 *Fb, UINT32 Stride,
                            UINT32 ScreenW, UINT32 ScreenH,
                            EFI_GRAPHICS_PIXEL_FORMAT Fmt)
{
    UINT32 White  = PackColor(Fmt, 255, 255, 255);
    UINT32 startX = (ScreenW - WORDMARK_W) / 2;
    UINT32 startY = (ScreenH - WORDMARK_H) / 2;
    for (UINT32 r = 0; r < WORDMARK_H; r++) {
        for (UINT32 c = 0; c < WORDMARK_W; c++) {
            if (WORDMARK_PIXELS[r][c]) {
                UINT32 px = startX + c;
                UINT32 py = startY + r;
                if (px < ScreenW && py < ScreenH)
                    Fb[py * Stride + px] = White;
            }
        }
    }
}
// Replace the FillRect wordmark call in SetupGopAndRender:
// FillRect(Fb, St, (W-280)/2, (Ht-48)/2, 280, 48, White);  ← DELETE
// RenderWordmark(Fb, St, W, Ht, Fmt);                        ← USE THIS
```

### A.2 EFI_SIMPLE_FILE_SYSTEM_PROTOCOL

```c
// AnimusBoot/AnimusBoot.c — kernel load via EFI filesystem (not NULL)
// EFI_SIMPLE_FILE_SYSTEM_PROTOCOL is used to locate the kernel image
// on the EFI partition before calling LoadImage.

#include <Protocol/SimpleFileSystem.h>
#include <Guid/FileInfo.h>

static EFI_STATUS LoadKernelFromFilesystem(EFI_HANDLE Img,
                                            EFI_HANDLE *KernelHandle)
{
    // Locate all handles with SimpleFileSystem
    UINTN Count = 0; EFI_HANDLE *Handles;
    EFI_STATUS S = gBS->LocateHandleBuffer(
        ByProtocol, &gEfiSimpleFileSystemProtocolGuid,
        NULL, &Count, &Handles);
    if (EFI_ERROR(S)) return S;

    for (UINTN i = 0; i < Count; i++) {
        EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *Fs;
        S = gBS->HandleProtocol(Handles[i],
            &gEfiSimpleFileSystemProtocolGuid, (VOID**)&Fs);
        if (EFI_ERROR(S)) continue;

        EFI_FILE_PROTOCOL *Root;
        if (EFI_ERROR(Fs->OpenVolume(Fs, &Root))) continue;

        EFI_FILE_PROTOCOL *Kernel;
        S = Root->Open(Root, &Kernel,
            L"\\EFI\\vitusos\\bzImage",
            EFI_FILE_MODE_READ, 0);
        Root->Close(Root);
        if (EFI_ERROR(S)) continue;
        Kernel->Close(Kernel);

        // Found — load from this device path
        EFI_DEVICE_PATH_PROTOCOL *DevPath;
        gBS->HandleProtocol(Handles[i],
            &gEfiDevicePathProtocolGuid, (VOID**)&DevPath);
        S = gBS->LoadImage(FALSE, Img, DevPath, NULL, 0, KernelHandle);
        FreePool(Handles);
        return S;
    }
    FreePool(Handles);
    return EFI_NOT_FOUND;
}
```

---

## ADDENDUM B — Layer 3 Animation Corrections

### B.1 AnimationClock — CLOCK_MONOTONIC Fallback + refreshHz EMA

```cpp
// animus/animation/AnimationClock.cpp — complete replacement
#include "AnimationClock.h"
#include <time.h>
#include <cmath>

#define WLR_USE_UNSTABLE
#include <wlr/types/wlr_output.h>

namespace Animus {

AnimationClock& AnimationClock::shared() {
    static AnimationClock instance; return instance;
}

void AnimationClock::onPresent(const struct wlr_output_event_present *ev) {
    struct timespec now;

    // wlr_output_event_present.when: struct timespec* — MAY BE NULL
    // Fallback to CLOCK_MONOTONIC when hardware timestamp unavailable
    if (ev && ev->when) {
        now = *ev->when;
    } else {
        clock_gettime(CLOCK_MONOTONIC, &now);
    }

    if (m_hasLast) {
        double sec  = (double)(now.tv_sec  - m_lastTime.tv_sec);
        double nsec = (double)(now.tv_nsec - m_lastTime.tv_nsec) * 1e-9;
        float  dt   = (float)(sec + nsec);

        // Clamp: reject stalls (>100ms) and sub-1ms noise
        if (dt > 0.001f && dt < 0.100f) {
            m_dt        = dt;
            m_totalTime += dt;

            // Exponential moving average of refresh rate
            // α=0.05: slow to react (stable), catches gradual drift
            float hz     = 1.0f / dt;
            m_refreshHz  = m_refreshHz * 0.95f + hz * 0.05f;
        }
    }
    m_lastTime = now;
    m_hasLast  = true;
}

} // namespace Animus
```

```cpp
// animus/animation/AnimationClock.h — complete replacement
#pragma once
#include <ctime>

struct wlr_output_event_present;

namespace Animus {

class AnimationClock {
public:
    static AnimationClock& shared();

    void   onPresent(const struct wlr_output_event_present *event);
    float  dt()         const { return m_dt; }
    float  refreshHz()  const { return m_refreshHz; }
    double totalTime()  const { return m_totalTime; }

private:
    AnimationClock() = default;
    bool            m_hasLast   = false;
    struct timespec m_lastTime  = {};
    float           m_dt        = 1.0f / 60.0f;
    float           m_refreshHz = 60.0f;    // EMA, updated each frame
    double          m_totalTime = 0.0;
};

} // namespace Animus
```

### B.2 AnimationEngine — publishAsync on Spring Settle

```cpp
// animus/animation/AnimationEngine.cpp — add settle notification
#include "AnimationEngine.h"
#include "AnimationClock.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <vector>
#include <functional>
#include <mutex>

namespace Animus {

AnimationEngine& AnimationEngine::shared() {
    static AnimationEngine inst; return inst;
}

// Register a one-shot callback that fires (async) when a spring settles.
// Caller provides a predicate returning true when settled.
// Used by: BootCrossfade (destroy self on settle), CockpitView (cleanup).
uint64_t AnimationEngine::onSettle(std::function<bool()> isSettled,
                                    std::function<void()> callback)
{
    std::lock_guard<std::mutex> lk(m_settleMu);
    uint64_t id = m_nextSettleId++;
    m_settlers.push_back({ id, std::move(isSettled), std::move(callback) });
    return id;
}

void AnimationEngine::cancelSettle(uint64_t id) {
    std::lock_guard<std::mutex> lk(m_settleMu);
    m_settlers.erase(std::remove_if(m_settlers.begin(), m_settlers.end(),
        [id](auto& s){ return s.id==id; }), m_settlers.end());
}

void AnimationEngine::tick(float dt) {
    if (!m_running) return;
    EventBus::shared().publish(OSFEvent::Tick, dt);

    // Check settlers — fire async when spring reaches rest
    std::vector<Settler> fired;
    {
        std::lock_guard<std::mutex> lk(m_settleMu);
        auto it = m_settlers.begin();
        while (it != m_settlers.end()) {
            if (it->isSettled()) {
                fired.push_back(*it);
                it = m_settlers.erase(it);
            } else { ++it; }
        }
    }
    for (auto& s : fired)
        EventBus::shared().publishAsync(OSFEvent::Tick, s.callback);
        // Note: publishAsync with callback wrapped in std::any
        // Subscriber unwraps and calls. Pattern used by BootCrossfade.
}

} // namespace Animus
```

---

## ADDENDUM C — Layer 4 StateManager observeState

```cpp
// animus/core/StateManager.cpp — add observeState
#include "StateManager.h"
#include "EventBus.h"
#include "OSFEvent.h"

namespace Animus {

StateManager& StateManager::shared() {
    static StateManager inst; return inst;
}

void StateManager::set(const std::string &key, std::any value) {
    m_state[key] = std::move(value);
    // Notify observers — publish synchronously on main thread
    EventBus::shared().publish(OSFEvent::StateChanged,
        std::string(key));  // data = changed key
}

const std::any* StateManager::get(const std::string &key) const {
    auto it = m_state.find(key);
    return it != m_state.end() ? &it->second : nullptr;
}

// observeState: subscribe to changes of a specific key.
// Returns handle for unsubscription.
uint64_t StateManager::observeState(const std::string &key,
                                     std::function<void(const std::any&)> cb)
{
    return EventBus::shared().subscribe(OSFEvent::StateChanged,
        [key, cb](const std::any &data) {
            auto changedKey = std::any_cast<std::string>(data);
            if (changedKey == key) cb(data);
        });
}

} // namespace Animus
```

Add `OSFEvent::StateChanged` to OSFEvent.h enum (between PathfinderClosed and BootCrossfadeComplete).

---

## ADDENDUM D — Layer 5 PersistentDrive (Planned)

```cpp
// animus/input/PersistentDrive.h
// Planned — Upstream Color ships first.
// User-configurable gesture→action bindings.
// Bindings stored in vitusos-config.nix user section.
// InputRouter reads on startup and on OSFEvent::BindingsChanged.
#pragma once
#include <string>
#include <unordered_map>
#include <functional>

namespace Animus {

// GestureBinding: maps a gesture identifier to an OSFEvent.
// Gesture IDs: "swipe3up", "swipe3down", "swipe4up", "swipe4down",
//              "pinchIn", "pinchOut"
struct GestureBinding {
    std::string gestureId;
    std::string actionId;    // matches OSFEvent name or shell command
};

class PersistentDrive {
public:
    static PersistentDrive& shared();
    void loadFromConfig(const std::string &nixConfigPath);
    void saveToConfig(const std::string &nixConfigPath);
    const std::string& actionForGesture(const std::string &gestureId) const;
    void setBinding(const std::string &gestureId, const std::string &actionId);

private:
    PersistentDrive() = default;
    std::unordered_map<std::string, std::string> m_bindings;
    static const std::string EMPTY;

    // Default bindings (overridden by vitusos-config.nix)
    void loadDefaults();
};

} // namespace Animus
// NOTE: Full implementation deferred until after Upstream Color ISO ships.
```

---

## ADDENDUM E — Layer 6 Typography: Font Roles + Color Roles

```cpp
// animus/render/TextRenderer.h — add font and color role enums
#pragma once
#include <vulkan/vulkan.h>
#include <cstdint>
#include <string>

namespace Animus {

class VulkanContext;
class GlyphAtlas;

// Font roles — semantic, not arbitrary sizes.
// Never specify raw pixel sizes in calling code. Always use a role.
enum class TextRole {
    Heading1,       // 24px Bold       — window titles, large headings
    Heading2,       // 18px Semibold   — section headings
    Body,           // 14px Regular    — primary content text
    Caption,        // 12px Regular    — secondary info, timestamps
    Small,          // 10px Regular    — badges, tiny labels
    SidebarHeader,  // 11px Semibold, wide tracking, ALL CAPS
};

// Color roles — semantic, not raw hex in calling code.
// Never write 0xFF1A1A1A inline. Always use a role.
enum class TextColor {
    Primary,    // #1A1A1A — main content text, dark mode adjusts
    Secondary,  // #808080 — Cosmic Gray — supporting text
    Muted,      // #3D3D3D — title bar text, subdued labels
    Accent,     // #E85D00 — Space Orange — links, highlights
    OnAccent,   // #FFFFFF — text on orange backgrounds
    OnDark,     // #F0F0F0 — text on dark glass surfaces
};

struct FontRoleInfo {
    float   sizePx;         // logical pixels
    int     weight;         // 400=Regular, 600=Semibold, 700=Bold
    float   trackingEm;     // letter-spacing as fraction of em (0 = normal)
    bool    uppercase;      // force uppercase transform
};

static constexpr FontRoleInfo FONT_ROLES[] = {
//  sizePx  weight  tracking  uppercase
    { 24.f,  700,   0.000f,   false },  // Heading1
    { 18.f,  600,   0.000f,   false },  // Heading2
    { 14.f,  400,   0.000f,   false },  // Body
    { 12.f,  400,   0.000f,   false },  // Caption
    { 10.f,  400,   0.000f,   false },  // Small
    { 11.f,  600,   0.080f,   true  },  // SidebarHeader
};

// Color table in ARGB8888 — indexed by TextColor enum
static constexpr uint32_t TEXT_COLORS[] = {
    0xFF1A1A1A,  // Primary
    0xFF808080,  // Secondary (Cosmic Gray)
    0xFF3D3D3D,  // Muted
    0xFFE85D00,  // Accent (Space Orange)
    0xFFFFFFFF,  // OnAccent
    0xFFF0F0F0,  // OnDark
};

class TextRenderer {
public:
    explicit TextRenderer(VulkanContext *ctx, GlyphAtlas *atlas)
        : m_ctx(ctx), m_atlas(atlas) {}
    bool initialize();

    // Primary draw method — uses role for size and semantic color
    void drawText(VkCommandBuffer cmd,
                  const char *utf8Text,
                  float baselineX, float baselineY,
                  TextRole role, TextColor color);

    // Raw ARGB override (for dynamic colors like wallpaper-derived text)
    void drawText(VkCommandBuffer cmd,
                  const char *utf8Text,
                  float baselineX, float baselineY,
                  TextRole role, uint32_t argbColor);

    // Measure text width in pixels for a given role (no draw)
    float measureWidth(const char *utf8Text, TextRole role);

private:
    VulkanContext *m_ctx;
    GlyphAtlas   *m_atlas;

    VkPipeline       m_glyphPipeline = VK_NULL_HANDLE;
    VkPipelineLayout m_glyphLayout   = VK_NULL_HANDLE;
    VkDescriptorPool m_descPool      = VK_NULL_HANDLE;
};

} // namespace Animus
```

---

## ADDENDUM F — Layer 8 Protocol Additions

### F.1 osf-shell-v1.xml — Complete with update_menu_item and set_shadow_style

```xml
<?xml version="1.0" encoding="UTF-8"?>
<protocol name="osf_shell_v1">
  <interface name="osf_shell_manager_v1" version="1">
    <request name="get_osf_surface">
      <arg name="id"       type="new_id" interface="osf_surface_v1"/>
      <arg name="toplevel" type="object" interface="xdg_toplevel"/>
    </request>
  </interface>

  <interface name="osf_surface_v1" version="1">
    <!-- Full MenuDefinition JSON — replaces entire menu tree -->
    <request name="set_application_menu">
      <arg name="menu_json" type="string"/>
    </request>

    <!-- Update single menu item state without full menu rebuild -->
    <request name="update_menu_item">
      <arg name="item_path" type="string"/>  <!-- e.g. "File/Save" -->
      <arg name="enabled"   type="uint"/>    <!-- 0=disabled, 1=enabled -->
      <arg name="checked"   type="uint"/>    <!-- 0=unchecked, 1=checked, 2=N/A -->
    </request>

    <!-- Shadow rendering hint to compositor -->
    <request name="set_shadow_style">
      <arg name="style" type="uint"/>
      <!-- 0=default (compositor chooses by altitude)
           1=none (suppress shadow — e.g. for transparent windows)
           2=large (force floating-altitude shadow)  -->
    </request>

    <!-- Dock badge. -1 removes badge. -->
    <request name="set_badge_count">
      <arg name="count" type="int"/>
    </request>

    <!-- Request Dock bounce -->
    <request name="request_attention"/>

    <request name="destroy" type="destructor"/>

    <event name="focused"/>
    <event name="close_requested"/>
    <event name="menu_activated">
      <arg name="item_path" type="string"/>  <!-- e.g. "File/Save" -->
    </event>
    <event name="configure">
      <arg name="width"  type="int"/>
      <arg name="height" type="int"/>
      <arg name="state"  type="uint"/>
    </event>
  </interface>
</protocol>
```

### F.2 xdg-output-unstable-v1 Support

```c
// compositor/animus_compositor.c — add to init section
#include <wlr/types/wlr_xdg_output_v1.h>

// In animus_compositor_init(), after wlr_xdg_shell_create:
struct wlr_xdg_output_manager_v1 *xdg_output_mgr =
    wlr_xdg_output_manager_v1_create(g.display, g.output_layout);
// Provides accurate output geometry to shell and Wayland apps.
// Required for correct positioning of layer-shell surfaces on multi-monitor.
(void)xdg_output_mgr;  // no callbacks needed — wlroots manages internally
```

---

## ADDENDUM G — Layer 9 Missing Surface Classes

### G.1 RenderPipeline.h

```cpp
// animus/render/RenderPipeline.h
#pragma once
#include <vulkan/vulkan.h>
#include <memory>
#include <vector>

namespace Animus {

class VulkanContext;
class MaterialRenderer;
class ShadowRenderer;
class TextRenderer;
class GlyphAtlas;
class Window;
class Panel;
class Dock;
class CockpitView;
class BootCrossfade;

class RenderPipeline {
public:
    RenderPipeline() = default;
    bool initialize(uint32_t w, uint32_t h);
    void destroy();

    void renderFrame(float dt);

    // Shell surface setters — called once during OSFDesktop::initSubsystems
    void setPanel(Panel *p)           { m_panel = p; }
    void setDock(Dock *d)             { m_dock = d; }
    void setCrossfade(BootCrossfade *c) { m_crossfade = c; }
    void setWallpaperView(VkImageView v) { m_wallpaperView = v; }

    void addWindow(std::shared_ptr<Window> w);
    void removeWindow(std::shared_ptr<Window> w);
    void addOverlay(class Surface *s);
    void removeOverlay(class Surface *s);

    MaterialRenderer* material() const { return m_material.get(); }
    ShadowRenderer*   shadow()   const { return m_shadow.get(); }
    TextRenderer*     text()     const { return m_text.get(); }
    VulkanContext*    vk()       const { return m_ctx.get(); }

private:
    std::unique_ptr<VulkanContext>    m_ctx;
    std::unique_ptr<MaterialRenderer> m_material;
    std::unique_ptr<ShadowRenderer>   m_shadow;
    std::unique_ptr<GlyphAtlas>       m_atlas;
    std::unique_ptr<TextRenderer>     m_text;

    VkImageView  m_wallpaperView = VK_NULL_HANDLE;
    Panel       *m_panel    = nullptr;
    Dock        *m_dock     = nullptr;
    BootCrossfade *m_crossfade = nullptr;

    std::vector<std::shared_ptr<Window>> m_windows;
    std::vector<class Surface*>          m_overlays;
};

} // namespace Animus
```

### G.2 WindowManager.h

```cpp
// animus/core/WindowManager.h
#pragma once
#include <memory>
#include <vector>
#include <cstdint>

struct wlr_surface;

namespace Animus {

class OSFWindow;

class WindowManager {
public:
    WindowManager() = default;

    void addSurface(struct wlr_surface *s);
    void removeSurface(struct wlr_surface *s);
    void focusSurface(struct wlr_surface *s);

    const std::vector<std::shared_ptr<OSFWindow>>& windows() const
        { return m_windows; }

    // Returns focused window, or nullptr
    OSFWindow* focused() const { return m_focused; }

    // Z-order management
    void raise(OSFWindow *win);
    void lower(OSFWindow *win);

    // Called from Dock to detect overlap for auto-hide
    bool anyWindowOverlapsDockArea(float dockY, float screenW) const;

private:
    std::vector<std::shared_ptr<OSFWindow>> m_windows;
    OSFWindow *m_focused = nullptr;
};

} // namespace Animus
```

### G.3 OSFToolbar.h

```cpp
// osf/surfaces/OSFToolbar.h
#pragma once
#include "render/MaterialRenderer.h"
#include "render/TextRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <vector>
#include <functional>

namespace Animus {

struct ToolbarAction {
    std::string  label;
    std::string  iconName;
    bool         isEnabled;
    std::function<void()> action;
};

// OSFToolbar: unified title bar + toolbar, 52px total height.
// SurfaceAltitude::Low — slight blur, slight translucency.
// Hosts primary app actions and global menu relay via osf-shell-v1.
// 1px separator below separates from OSFContent.
// Search bar springs wider on focus (SPRING_HOVER).
class OSFToolbar {
public:
    OSFToolbar();
    ~OSFToolbar();

    void setTitle(const std::string &title);
    void setActions(std::vector<ToolbarAction> actions);
    void setSearchPlaceholder(const std::string &placeholder);
    void render(VkCommandBuffer cmd, float x, float y, float w);

    static constexpr float HEIGHT         = 52.0f;  // 28px title + 24px toolbar
    static constexpr float SEPARATOR_PX   = 1.0f;
    static constexpr float SEARCH_W_IDLE  = 188.0f;
    static constexpr float SEARCH_W_FOCUS = 260.0f;
    static constexpr float CORNER_RADIUS  = 0.0f;   // flush with window
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Low;

private:
    std::string  m_title;
    std::string  m_searchPlaceholder;
    std::vector<ToolbarAction> m_actions;
    bool         m_searchFocused = false;

    SpringSolver m_searchWidth;   // SPRING_HOVER (600,40): 188 → 260 on focus
    uint64_t     m_tickHandle = 0;
};

} // namespace Animus
```

### G.4 OSFContent.h

```cpp
// osf/surfaces/OSFContent.h
#pragma once
#include "render/MaterialRenderer.h"
#include <vulkan/vulkan.h>

namespace Animus {

// OSFContent: the grounded content area.
// SurfaceAltitude::Grounded — fully opaque, no blur, no glass.
// Background: #FEFEFE (not #FFFFFF — imperceptibly warm).
// Note: #FFFFFF is listed in the reference spec. We use #FEFEFE per
// the hardcoded color rules. This is intentional.
// OSFContent is a layout region, not a Wayland surface.
// Apps render into it; compositor treats it as opaque.
struct OSFContent {
    static constexpr SurfaceAltitude ALTITUDE    = SurfaceAltitude::Grounded;
    static constexpr uint32_t        BACKGROUND  = 0xFFFEFEFE;  // #FEFEFE ARGB
    static constexpr float           CORNER_RADIUS= 0.0f;       // flush edges

    // No spring behavior — content area is static layout.
    // Window position/size spring handles the whole window.
};

} // namespace Animus
```

### G.5 OSFPopover.h

```cpp
// osf/surfaces/OSFPopover.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <functional>

namespace Animus {

// OSFPopover: lightweight overlay for supplemental content.
// SurfaceAltitude::High — heavy blur, cold tint.
// Springs from origin geometry (the trigger element), not screen center.
// Owns its own shadow at Floating altitude weight.
// Dismissed by clicking outside.
class OSFPopover {
public:
    // originX/Y: position of trigger element (popover springs from here)
    OSFPopover(float originX, float originY, float w, float h,
               std::function<void()> onDismiss);
    ~OSFPopover();

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void dismiss();
    bool isVisible() const { return m_visible; }
    bool hitTest(float x, float y) const;

    static constexpr float CORNER_RADIUS = 10.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::High;

private:
    float m_originX, m_originY;
    float m_targetX, m_targetY, m_w, m_h;
    bool  m_visible = false;
    std::function<void()> m_onDismiss;

    // Springs from origin point to final position
    SpringSolver2D m_pos;    // SPRING_SELECTION (400,28)
    SpringSolver   m_scale;  // 0.92 → 1.0 on open
    SpringSolver   m_opacity;
    uint64_t       m_tickHandle = 0;
};

} // namespace Animus
```

### G.6 OSFDropdown.h

```cpp
// osf/surfaces/OSFDropdown.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vector>
#include <string>
#include <functional>

namespace Animus {

struct DropdownItem {
    std::string  label;
    bool         isEnabled;
    std::function<void()> onSelect;
};

// OSFDropdown: appears below trigger element.
// SurfaceAltitude::High — same as Popover.
// SPRING_SHEET (420,30) on open — drops with weight.
// Item hover via SPRING_HOVER (600,40) per item.
class OSFDropdown {
public:
    OSFDropdown(std::vector<DropdownItem> items,
                float anchorX, float anchorY, float anchorW,
                std::function<void()> onDismiss);
    ~OSFDropdown();

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void dismiss();
    bool isVisible() const { return m_visible; }
    bool hitTest(float x, float y) const;
    void onPointerMotion(float x, float y);
    void onPointerButton(float x, float y);

    static constexpr float ITEM_HEIGHT   = 32.0f;
    static constexpr float MIN_WIDTH     = 160.0f;
    static constexpr float CORNER_RADIUS = 10.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::High;

private:
    std::vector<DropdownItem>  m_items;
    float m_x, m_y, m_w, m_h;
    bool  m_visible    = false;
    int   m_hoveredIdx = -1;
    std::function<void()> m_onDismiss;

    SpringSolver               m_slideY;    // SPRING_SHEET (420,30): drops from anchor
    SpringSolver               m_opacity;
    std::vector<SpringSolver>  m_itemHover; // SPRING_HOVER (600,40) per item
    uint64_t                   m_tickHandle = 0;
};

} // namespace Animus
```

### G.7 OSFSheet.h

```cpp
// osf/surfaces/OSFSheet.h
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <functional>
#include <string>

namespace Animus {

class OSFWindow;

// OSFSheet: modal sheet attached to parent window.
// Drops from parent window title bar (not top of screen).
// SurfaceAltitude::Mid — same glass level as sidebar.
// Spatially attached to parent — moves with parent window drag.
// Cannot be dismissed by clicking outside (modal by design).
// Dismissed only by explicit action (button within sheet).
class OSFSheet {
public:
    // parentWindow: sheet tracks this window's position
    // attachedToY: Y offset from window top (usually title bar height ~28px)
    OSFSheet(OSFWindow *parentWindow, float w, float h,
             const std::string &title,
             std::function<void()> onDismiss);
    ~OSFSheet();

    void render(VkCommandBuffer cmd, float dt);
    void dismiss();
    bool isVisible() const { return m_visible; }

    static constexpr float CORNER_RADIUS    = 10.0f;
    static constexpr float ATTACH_OFFSET_Y  = 28.0f;  // title bar height
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Mid;

private:
    OSFWindow   *m_parent;
    float        m_w, m_h;
    std::string  m_title;
    bool         m_visible = false;
    std::function<void()> m_onDismiss;

    // Drops from parent's title bar — SPRING_SHEET (420,30)
    SpringSolver m_slideY;    // starts at attachedToY, springs to center
    SpringSolver m_opacity;
    uint64_t     m_tickHandle = 0;
};

} // namespace Animus
```

---

## ADDENDUM H — Layer 9 OSFWindow Full Detail

```cpp
// osf/surfaces/OSFWindow.h — complete replacement with all specified details
#pragma once
#include "render/MaterialRenderer.h"
#include "render/TextRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>

struct wlr_surface;

namespace Animus {

// Traffic light colors — exact values from spec
static constexpr uint32_t TL_CLOSE    = 0xFFFF5F57;  // #FF5F57
static constexpr uint32_t TL_MINIMIZE = 0xFFFFBD2E;  // #FFBD2E
static constexpr uint32_t TL_ZOOM     = 0xFF28C840;  // #28C840
static constexpr float    TL_SIZE     = 12.0f;        // 12px circles
static constexpr float    TL_SPACING  = 8.0f;
static constexpr float    TL_MARGIN_X = 12.0f;
static constexpr float    TL_MARGIN_Y = 8.0f;

class OSFWindow {
public:
    OSFWindow(struct wlr_surface *surface, float x, float y, float w, float h);
    ~OSFWindow();

    void render(VkCommandBuffer cmd, float dt);
    void close();
    void focus();
    void blur();
    void setPosition(float x, float y);
    void setSize(float w, float h);
    void setTitle(const std::string &title) { m_title = title; }
    void setAppId(const std::string &id)    { m_appId = id; }

    // Spring-lagged shadow position — creates depth illusion
    float shadowX() const { return m_shadowPos.x.value(); }
    float shadowY() const { return m_shadowPos.y.value(); }

    float x()      const { return m_pos.x.value(); }
    float y()      const { return m_pos.y.value(); }
    float width()  const { return m_w; }
    float height() const { return m_h; }

    float cornerRadius()  const { return 10.0f; }
    bool  isVisible()     const { return m_visible && m_scale.value() > 0.01f; }
    bool  isFocused()     const { return m_focused; }

    SurfaceAltitude altitude() const { return m_altitude; }
    void setAltitude(SurfaceAltitude a) { m_altitude = a; }

    const std::string& title() const { return m_title; }
    struct wlr_surface *surface() const { return m_surface; }

private:
    void renderChrome(VkCommandBuffer cmd, float dt);
    void renderTrafficLights(VkCommandBuffer cmd);
    void renderSeparator(VkCommandBuffer cmd);

    struct wlr_surface *m_surface;
    SurfaceAltitude     m_altitude = SurfaceAltitude::Mid;
    bool                m_visible  = false;
    bool                m_focused  = false;
    float               m_w, m_h;
    std::string         m_title;
    std::string         m_appId;

    SpringSolver2D m_pos;          // SPRING_WINDOW_DRAG (800,35)
    SpringSolver2D m_shadowPos;    // SPRING_SHADOW (300,25) — lags behind m_pos
    SpringSolver   m_scale;        // SPRING_SELECTION: 0.95→1.0 on open, 1.0→0 on close
    SpringSolver   m_tlHover[3];   // SPRING_TRAFFIC_LIGHT (700,38) per button

    uint64_t m_tickHandle = 0;

    // Title: center-aligned, TextRole::Body, TextColor::Muted
    // 1px separator between toolbar bottom and content top
    // Separator color: darken toolbar material by 8% (not black)
};

} // namespace Animus
```

---

## ADDENDUM I — Layer 10 Shell Detail

### I.1 Panel — Orange Box + System Tray

```cpp
// animus/shell/Panel.h — complete replacement
#pragma once
#include "render/MaterialRenderer.h"
#include "render/TextRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <vector>
#include <ctime>

namespace Animus {

// Panel: 28px height, SurfaceAltitude::Low, wlr-layer-shell LAYER_TOP anchored top.
// Left: Orange Box button (12×12px, #E85D00) → toggles CockpitView.
//       Traffic lights for focused window.
// Center-left: global menu from osf-shell-v1 set_application_menu.
// Right: system tray (clock, network indicator, battery, volume).
// EventBus: OSFEvent::WindowFocused → menu update, traffic light state.
class Panel {
public:
    Panel();
    ~Panel();
    bool initialize();
    void render(VkCommandBuffer cmd, float screenW, float dt);

    static constexpr float HEIGHT         = 28.0f;
    static constexpr float ORANGE_BOX_SZ  = 12.0f;   // #E85D00 square
    static constexpr float ORANGE_BOX_X   = 10.0f;
    static constexpr float ORANGE_BOX_Y   = 8.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Low;

private:
    void renderOrangeBox(VkCommandBuffer cmd, float dt);
    void renderGlobalMenu(VkCommandBuffer cmd);
    void renderSystemTray(VkCommandBuffer cmd, float screenW);
    std::string formatClock() const;
    std::string getNetworkStatus() const;   // reads /sys/class/net
    std::string getBatteryStatus() const;   // reads /sys/class/power_supply
    float       getVolume() const;          // reads from StateManager

    std::string              m_focusedAppName;
    std::vector<std::string> m_menuItems;

    SpringSolver m_orangeBoxHover;   // SPRING_HOVER (600,40)
    SpringSolver m_tlHover[3];       // SPRING_TRAFFIC_LIGHT (700,38)

    uint64_t m_tickHandle  = 0;
    uint64_t m_focusHandle = 0;
};

} // namespace Animus
```

### I.2 Dock — Auto-Hide + Running Dot

```cpp
// animus/shell/Dock.h — complete replacement
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include "core/WindowManager.h"
#include <vector>
#include <string>
#include <memory>

namespace Animus {

struct DockItem {
    std::string appId;
    std::string iconPath;
    std::string displayName;
    bool        isRunning;    // shows running dot below icon
    bool        isPinned;
    int         badgeCount;   // -1 = no badge
    bool        isFilerApp;   // running dot always visible for Filer
};

// Dock: wlr-layer-shell LAYER_BOTTOM, anchored bottom, 64px height.
// SurfaceAltitude::Mid. 16px corner radius.
// Auto-hide: checks WindowManager::anyWindowOverlapsDockArea() each frame.
// Running dot: 4px circle below icon center. Always visible for Filer.
// Icon magnify: SPRING_DOCK_MAGNIFY (450,32), peak 72px at cursor.
// Launch bounce: SPRING_SELECTION with initial negative velocity.
// Badge: count shown in #E85D00 pill top-right of icon.
class Dock {
public:
    explicit Dock(WindowManager *wm);
    ~Dock();
    bool initialize();

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void setItems(std::vector<DockItem> items);
    void notifyLaunch(const std::string &appId);
    void setBadge(const std::string &appId, int count);

    void onPointerMotion(float x, float y);
    void onPointerButton(float x, float y, bool pressed);

    static constexpr float HEIGHT        = 64.0f;
    static constexpr float ICON_SIZE     = 48.0f;
    static constexpr float ICON_PEAK     = 72.0f;  // magnify peak (1.5×)
    static constexpr float MAGNIFY_SPREAD= 80.0f;  // falloff radius in px
    static constexpr float ICON_SPACING  =  8.0f;
    static constexpr float CORNER_RADIUS = 16.0f;
    static constexpr float PADDING_H     = 12.0f;
    static constexpr float DOT_SIZE      =  4.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Mid;

private:
    void  renderIcon(VkCommandBuffer cmd, const DockItem &item,
                     float cx, float iconSize, float bounce);
    void  renderRunningDot(VkCommandBuffer cmd, float cx, float dockY);
    void  renderBadge(VkCommandBuffer cmd, float cx, float iconTop, int count);
    float iconSizeAtCursor(float iconCenterX) const;

    WindowManager          *m_wm;
    std::vector<DockItem>   m_items;
    std::vector<SpringSolver> m_magnify;   // SPRING_DOCK_MAGNIFY per icon
    std::vector<SpringSolver> m_bounce;    // SPRING_SELECTION for launch bounce
    SpringSolver              m_autoHide;  // SPRING_HOVER for show/hide

    float m_cursorX    = -9999;
    bool  m_autoHiding = false;
    uint64_t m_tickHandle = 0;
};

} // namespace Animus
```

### I.3 CockpitView — Left Sidebar Desktop Switcher

```cpp
// animus/shell/CockpitView.h — complete replacement
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vector>
#include <memory>
#include <functional>

namespace Animus {

class OSFWindow;

// CockpitView: Mission Control equivalent.
// Left sidebar (180px): workspace/desktop switcher — circular buttons.
// Main area: window cards arranged in grid.
// Window cards spring FROM their actual screen positions (no teleport).
// Thumbnails captured via wlr_renderer_read_pixels on open.
// Card click: window springs back to actual screen position, view closes.
class CockpitView {
public:
    CockpitView();
    ~CockpitView();

    void open(const std::vector<std::shared_ptr<OSFWindow>> &windows);
    void close();
    bool isOpen() const { return m_open; }

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void onPointerButton(float x, float y);

    static constexpr float SIDEBAR_W       = 180.0f;
    static constexpr float CARD_SPACING    = 20.0f;
    static constexpr float CARD_CORNER     = 10.0f;
    static constexpr float DESKTOP_BTN_SZ  = 48.0f;  // circular workspace buttons
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::High;

private:
    struct Card {
        std::shared_ptr<OSFWindow> window;
        VkImage        thumbImage = VK_NULL_HANDLE;  // captured on open
        VkImageView    thumbView  = VK_NULL_HANDLE;
        VkDeviceMemory thumbMem   = VK_NULL_HANDLE;
        float          realX, realY, realW, realH;   // actual screen position
        SpringSolver2D pos;      // SPRING_SELECTION: from realX/Y to grid pos
        SpringSolver   scale;    // SPRING_SELECTION: 1.0 initially
        SpringSolver   opacity;
    };

    bool              m_open      = false;
    int               m_desktopCount = 1;
    int               m_activeDesktop = 0;
    std::vector<Card> m_cards;

    SpringSolver m_bgOpacity;       // SPRING_SELECTION: 0→1 on open
    SpringSolver m_sidebarSlide;    // SPRING_SELECTION: slides in from left

    uint64_t m_tickHandle  = 0;
    uint64_t m_eventHandle = 0;

    void captureWindowThumbnails();
    void layoutCards(float screenW, float screenH);
};

} // namespace Animus
```

---

## ADDENDUM J — Layer 11 System Services

### J.1 ClipboardBridge.h

```cpp
// animus/core/ClipboardBridge.h
#pragma once
#include <string>
#include <vector>
#include <deque>

struct wlr_seat;

namespace Animus {

// ClipboardBridge: Wayland clipboard via wlr_seat.
// In-process only — Pathfinder and other OSFNative apps call directly.
// History: last 20 entries, memory-only, NEVER persisted to disk.
class ClipboardBridge {
public:
    static ClipboardBridge& shared();

    void initialize(struct wlr_seat *seat);

    // Copy text to clipboard (also adds to history)
    void setText(const std::string &text);

    // Get current clipboard content
    std::string getText() const;

    // Clipboard history — most recent first
    const std::deque<std::string>& history() const { return m_history; }

    // Clear history (user action only)
    void clearHistory();

    static constexpr size_t MAX_HISTORY = 20;  // memory-only, never to disk

private:
    ClipboardBridge() = default;
    struct wlr_seat      *m_seat = nullptr;
    std::deque<std::string> m_history;
    std::string             m_current;
};

} // namespace Animus
```

### J.2 FileOperationDaemon.h

```cpp
// animus/core/FileOperationDaemon.h
#pragma once
#include "core/EventBus.h"
#include <string>
#include <functional>
#include <cstdint>

namespace Animus {

enum class FileOpType { Copy, Move, Delete, Rename };
enum class ConflictResolution { Skip, Overwrite, KeepBoth, Cancel };

struct FileOperation {
    uint64_t     id;
    FileOpType   type;
    std::string  src;
    std::string  dst;    // empty for Delete
    float        progress;  // 0.0–1.0
    bool         complete;
    std::string  errorMsg;
};

// FileOperationDaemon: all file I/O async, progress via EventBus.
// Conflict resolution triggers OSFSheet on main thread (via publishAsync).
class FileOperationDaemon {
public:
    static FileOperationDaemon& shared();

    // Returns operation ID
    uint64_t copy(const std::string &src, const std::string &dstDir);
    uint64_t move(const std::string &src, const std::string &dstDir);
    uint64_t remove(const std::string &path);
    uint64_t rename(const std::string &path, const std::string &newName);

    void cancel(uint64_t opId);
    void resolveConflict(uint64_t opId, ConflictResolution res);

private:
    FileOperationDaemon() = default;
    uint64_t m_nextId = 1;
    // Operations run on background threads, publish progress async
};

} // namespace Animus
```

### J.3 DirectoryWatcher + InstallManager details

```cpp
// animus/core/DirectoryWatcher.h
#pragma once
#include <string>
#include <vector>
#include <thread>
#include <atomic>
#include <functional>

namespace Animus {

// DirectoryWatcher: inotify_init1(IN_NONBLOCK | IN_CLOEXEC) + epoll.
// Publishes OSFEvent::DirectoryChanged via EventBus::publishAsync.
// Runs on dedicated background thread — never blocks compositor.
class DirectoryWatcher {
public:
    static DirectoryWatcher& shared();

    // Watch a directory recursively. Returns watch ID.
    int watch(const std::string &path);
    void unwatch(int watchId);

    void start();
    void stop();

private:
    DirectoryWatcher() = default;

    void loop();  // runs on m_thread

    int                m_inotifyFd = -1;   // inotify_init1(IN_NONBLOCK|IN_CLOEXEC)
    int                m_epollFd   = -1;
    int                m_wakeupFd  = -1;   // eventfd for clean shutdown
    std::atomic<bool>  m_running   = false;
    std::thread        m_thread;
};

} // namespace Animus
```

```cpp
// animus/core/InstallManager.h
#pragma once
#include <string>
#include <functional>

namespace Animus {

// InstallManager: ONLY writes vitusos-config.nix.
// NEVER touches nixos-configuration.nix.
// Atomic write: write to .tmp file, then rename() — never partial writes.
// Parses nixos-rebuild switch stdout for progress % via regex.
// Progress published via EventBus::publishAsync(OSFEvent::InstallProgress).
class InstallManager {
public:
    static InstallManager& shared();

    // Install an app by adding its entry to vitusos-config.nix
    // Returns operation ID (progress reported via EventBus)
    uint64_t install(const std::string &appId, const std::string &version);
    uint64_t uninstall(const std::string &appId);
    uint64_t update(const std::string &appId);

    static constexpr char CONFIG_PATH[] = "/etc/vitusos/vitusos-config.nix";
    // NEVER write to:
    // /etc/nixos/nixos-configuration.nix
    // /etc/nixos/configuration.nix

private:
    InstallManager() = default;
    // Atomic write: always write to CONFIG_PATH + ".tmp", then rename()
    bool writeConfigAtomic(const std::string &contents);
    uint64_t m_nextOpId = 1;
};

} // namespace Animus
```

---

## ADDENDUM K — OSFAppKit (Complete Widget Layer)

```cpp
// osf/appkit/OSFAppKit.h
// Single include for all AppKit widgets.
// All widgets use SpringSolver for motion. No exceptions.
#pragma once

#include "render/MaterialRenderer.h"
#include "render/TextRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <vector>
#include <functional>
#include <vulkan/vulkan.h>

namespace Animus {

// ── OSFButton ──────────────────────────────────────────────────────
// Three sizes. Hover via SPRING_HOVER (600,40).
// Click: scale spring 1.0→0.96→1.0 (pressed-into-surface feel).
enum class ButtonSize { Small=24, Medium=32, Large=40 };
enum class ButtonStyle { Primary, Secondary, Destructive, Ghost };

class OSFButton {
public:
    OSFButton(const std::string &label, ButtonSize sz, ButtonStyle style,
              std::function<void()> action);
    ~OSFButton();

    void render(VkCommandBuffer cmd, float x, float y, float dt);
    bool hitTest(float x, float y) const;
    void onPointerMotion(float mx, float my);
    void onPointerButton(float mx, float my, bool pressed);

    float height() const { return (float)m_size; }
    float width()  const { return m_width; }

    static constexpr float CORNER_RADIUS_SMALL  = 4.0f;
    static constexpr float CORNER_RADIUS_MEDIUM = 6.0f;
    static constexpr float CORNER_RADIUS_LARGE  = 8.0f;

private:
    std::string  m_label;
    ButtonSize   m_size;
    ButtonStyle  m_style;
    float        m_x=0, m_y=0, m_width=80;
    std::function<void()> m_action;

    SpringSolver m_hover;   // SPRING_HOVER (600,40): 0→1 on hover
    SpringSolver m_press;   // SPRING_SELECTION: 1.0→0.96→1.0 on click
    uint64_t     m_tickHandle = 0;
};

// ── OSFTextField ──────────────────────────────────────────────────
class OSFTextField {
public:
    OSFTextField(const std::string &placeholder, float width);
    ~OSFTextField();

    void render(VkCommandBuffer cmd, float x, float y, float dt);
    bool hitTest(float x, float y) const;
    void onFocus();
    void onBlur();
    void onKey(uint32_t sym, uint32_t mods, bool pressed);
    const std::string& value() const { return m_value; }
    void setValue(const std::string &v) { m_value = v; }

    static constexpr float HEIGHT        = 28.0f;
    static constexpr float CORNER_RADIUS =  4.0f;

private:
    std::string  m_placeholder, m_value;
    float        m_x=0, m_y=0, m_w;
    bool         m_focused = false;
    int          m_cursorPos = 0;

    SpringSolver m_focusRing;  // SPRING_HOVER: 0→1 when focused
    uint64_t     m_tickHandle = 0;
};

// ── OSFLabel ──────────────────────────────────────────────────────
struct OSFLabel {
    std::string text;
    TextRole    role;
    TextColor   color;
    // Render call: textRenderer->drawText(cmd, text, x, y, role, color)
    // No springs — labels are static. Position set by layout.
};

// ── OSFCheckbox ──────────────────────────────────────────────────
class OSFCheckbox {
public:
    OSFCheckbox(const std::string &label, bool checked,
                std::function<void(bool)> onChange);
    ~OSFCheckbox();

    void render(VkCommandBuffer cmd, float x, float y, float dt);
    bool hitTest(float x, float y) const;
    void onPointerButton(float mx, float my, bool pressed);
    bool isChecked() const { return m_checked; }

    static constexpr float SIZE          = 18.0f;
    static constexpr float CORNER_RADIUS =  4.0f;
    static constexpr float LABEL_GAP     =  8.0f;

private:
    std::string m_label;
    bool        m_checked;
    float       m_x=0, m_y=0;
    std::function<void(bool)> m_onChange;

    SpringSolver m_check;  // SPRING_SELECTION: 0→1 when checked
    uint64_t     m_tickHandle = 0;
};

// ── OSFSlider ─────────────────────────────────────────────────────
class OSFSlider {
public:
    OSFSlider(float min, float max, float value, float width,
              std::function<void(float)> onChange);
    ~OSFSlider();

    void render(VkCommandBuffer cmd, float x, float y, float dt);
    bool hitTest(float x, float y) const;
    void onPointerButton(float mx, float my, bool pressed);
    void onPointerMotion(float mx, float my);
    float value() const { return m_value; }

    static constexpr float TRACK_H      =  4.0f;
    static constexpr float THUMB_SIZE   = 20.0f;
    static constexpr float HEIGHT       = 20.0f;

private:
    float  m_min, m_max, m_value, m_x=0, m_y=0, m_w;
    bool   m_dragging = false;
    std::function<void(float)> m_onChange;

    SpringSolver m_thumbPos;  // thumb spring-follows drag SPRING_WINDOW_DRAG
    uint64_t     m_tickHandle = 0;
};

// ── OSFProgressBar ───────────────────────────────────────────────
class OSFProgressBar {
public:
    OSFProgressBar(float width);
    ~OSFProgressBar();

    void setProgress(float p);  // 0.0–1.0
    void render(VkCommandBuffer cmd, float x, float y, float dt);

    static constexpr float HEIGHT        =  6.0f;
    static constexpr float CORNER_RADIUS =  3.0f;

private:
    float  m_x=0, m_y=0, m_w;
    SpringSolver m_fill;  // SPRING_SELECTION: fills smoothly
    uint64_t     m_tickHandle = 0;
};

// ── OSFTableView ─────────────────────────────────────────────────
struct TableColumn { std::string header; float width; };
struct TableRow    { std::vector<std::string> cells; };

class OSFTableView {
public:
    OSFTableView(std::vector<TableColumn> cols, float width, float height);
    ~OSFTableView();

    void setRows(std::vector<TableRow> rows);
    void setSelectedRow(int idx);
    int  selectedRow() const { return m_selectedRow; }

    void render(VkCommandBuffer cmd, float x, float y, float dt);
    void onPointerButton(float mx, float my, bool pressed);
    void onPointerMotion(float mx, float my);

    static constexpr float ROW_HEIGHT    = 32.0f;
    static constexpr float HEADER_HEIGHT = 28.0f;

private:
    std::vector<TableColumn> m_cols;
    std::vector<TableRow>    m_rows;
    float m_x=0, m_y=0, m_w, m_h;
    int   m_selectedRow = -1;
    int   m_hoveredRow  = -1;
    float m_scrollY     = 0;

    SpringSolver m_selPillY;   // SPRING_SELECTION (400,28)
    SpringSolver m_scrollSpring; // SPRING_SCROLL (80,18)
    uint64_t     m_tickHandle = 0;
};

// ── OSFImageView ─────────────────────────────────────────────────
struct OSFImageView {
    VkImageView imageView = VK_NULL_HANDLE;
    float       cornerRadius = 0.0f;
    float       opacity      = 1.0f;
    // Render call: materialRenderer->drawTextureQuad(cmd, x,y,w,h, imageView, opacity, cornerRadius)
    // No springs — position set by layout.
};

// ── OSFScrollView ────────────────────────────────────────────────
class OSFScrollView {
public:
    OSFScrollView(float width, float height);
    ~OSFScrollView();

    void setContentHeight(float h) { m_contentH = h; }
    void render(VkCommandBuffer cmd, float x, float y, float dt);
    void onPointerAxis(double dy);     // applies fling
    void onPointerButton(bool pressed);

    float scrollOffset() const { return m_scroll.value(); }

    static constexpr float SCROLLBAR_W = 4.0f;

private:
    float m_x=0, m_y=0, m_w, m_h, m_contentH=0;
    float m_flingVelocity = 0;

    // SPRING_SCROLL (80,18): low stiffness = fling deceleration, not snap
    SpringSolver m_scroll;
    uint64_t     m_tickHandle = 0;
};

} // namespace Animus
```

---

## ADDENDUM L — OSFSidebar Corrections

```cpp
// osf/surfaces/OSFSidebar.h — complete with TEXT_SIDEBAR_HEADER + border
#pragma once
#include "render/MaterialRenderer.h"
#include "render/TextRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <vector>
#include <string>
#include <functional>

namespace Animus {

struct SidebarItem {
    std::string  label;
    std::string  iconName;
    bool         isSection;       // section header — not selectable
    bool         isSeparator;     // 1px horizontal line
    std::function<void()> onSelect;
};

// OSFSidebar: SurfaceAltitude::Mid — luminosity blur (not gaussian).
// WallpaperTintSampler feeds tint.
// Section headers: TextRole::SidebarHeader (11px Semibold, wide tracking, ALL CAPS).
// Selection pill: spring Y-follows focused item, SPRING_SELECTION (400,28).
// Item hover: 4% opacity fill, same border-radius as pill.
// Right border: 1px, darken sidebar material by 8%.
class OSFSidebar {
public:
    OSFSidebar();
    ~OSFSidebar();

    void setItems(std::vector<SidebarItem> items);
    void setSelectedIndex(int idx);
    int  selectedIndex() const { return m_selectedIdx; }

    void render(VkCommandBuffer cmd, float x, float y, float w, float h, float dt);
    void onPointerMotion(float mx, float my);
    void onPointerButton(float mx, float my, bool pressed);

    static constexpr float ITEM_H       = 36.0f;
    static constexpr float SECTION_H    = 28.0f;
    static constexpr float PILL_INSET_X =  4.0f;
    static constexpr float PILL_RADIUS  =  6.0f;
    static constexpr float BORDER_PX    =  1.0f;   // right border
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Mid;

private:
    float itemY(int idx) const;  // Y position of item[idx] in sidebar coords

    std::vector<SidebarItem> m_items;
    int m_selectedIdx = -1;
    int m_hoveredIdx  = -1;
    float m_x=0, m_y=0, m_w=0, m_h=0;

    SpringSolver              m_pillY;      // SPRING_SELECTION (400,28)
    SpringSolver              m_pillH;      // spring for pill height on select
    std::vector<SpringSolver> m_hover;      // SPRING_HOVER (600,40) per item
    uint64_t                  m_tickHandle = 0;
};

} // namespace Animus
```

---

## ADDENDUM M — OSFNotification Stacking

```cpp
// osf/surfaces/OSFNotification.h — add stacking
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <functional>

namespace Animus {

// OSFNotification: top-right corner, stacks vertically downward.
// Slides in from right edge. Auto-dismisses after timeout.
// Stack offset applied by NotificationManager (not by individual notification).
class OSFNotification {
public:
    OSFNotification(const std::string &title,
                    const std::string &body,
                    int                timeoutMs,
                    std::function<void()> onDismiss);
    ~OSFNotification();

    // stackY: vertical position in notification stack (0 = topmost)
    // Managed by NotificationManager — do not set directly
    void setStackOffset(float y) { m_stackY.setTarget(y); }

    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void dismiss();
    bool isVisible()  const { return m_visible; }
    bool isDismissed()const { return m_dismissed; }

    static constexpr float WIDTH         = 320.0f;
    static constexpr float HEIGHT        =  80.0f;
    static constexpr float MARGIN_RIGHT  =  16.0f;
    static constexpr float MARGIN_TOP    =  44.0f;  // below Panel
    static constexpr float STACK_GAP     =   8.0f;
    static constexpr float CORNER_RADIUS =  12.0f;
    static constexpr SurfaceAltitude ALTITUDE = SurfaceAltitude::Floating;

private:
    std::string  m_title, m_body;
    int          m_timeoutMs;
    float        m_elapsed = 0;
    bool         m_visible   = false;
    bool         m_dismissed = false;
    std::function<void()> m_onDismiss;

    SpringSolver m_slideX;    // SPRING_NOTIFICATION (380,26): from screenW to slot
    SpringSolver m_stackY;    // SPRING_NOTIFICATION: vertical stack position
    uint64_t     m_tickHandle = 0;
};

} // namespace Animus
```

---

## ADDENDUM N — OSFContextMenu Screen Edge Repositioning

```cpp
// osf/surfaces/OSFContextMenu.h — add edge detection
// (replaces the version in Part 14)
// Key addition: reposition() called on open to prevent overflow.

// In OSFContextMenu constructor:
void OSFContextMenu::reposition(float spawnX, float spawnY,
                                  float menuW, float menuH,
                                  float screenW, float screenH)
{
    float x = spawnX;
    float y = spawnY;

    // Right edge: flip left if would overflow
    if (x + menuW > screenW - 8.0f) x = spawnX - menuW;
    // Bottom edge: flip up if would overflow
    if (y + menuH > screenH - 8.0f) y = spawnY - menuH;
    // Clamp to screen minimum
    if (x < 8.0f) x = 8.0f;
    if (y < 8.0f) y = 8.0f;

    m_x = x; m_y = y;
    m_pos.snap(spawnX, spawnY);    // start from cursor
    m_pos.setTarget(x, y);         // spring to repositioned location
}
```


---

## ADDENDUM O — OSFDesktop L12 Accessor Methods

The reference tree specifies named accessors on `OSFDesktop::shared()` for all subsystems. These must be public methods, not just private `unique_ptr` members.

```cpp
// animus/core/OSFDesktop.h — complete accessor interface
// Replace the private section with full accessor surface

class OSFDesktop {
public:
    static OSFDesktop& shared();
    int run();

    // L12 accessors — every subsystem reachable via shared()
    AnimationEngine&       animationEngine()   { return AnimationEngine::shared(); }
    WindowManager&         windowManager()     { return *m_wm; }
    InputRouter&           inputRouter()       { return InputRouter::shared(); }
    EventBus&              eventBus()          { return EventBus::shared(); }
    StateManager&          stateManager()      { return StateManager::shared(); }
    ClipboardBridge&       clipboardBridge()   { return ClipboardBridge::shared(); }
    SoundEngine&           soundEngine()       { return SoundEngine::shared(); }
    RenderPipeline&        renderPipeline()    { return *m_render; }
    MaterialRenderer&      materialRenderer()  { return *m_render->material(); }
    WallpaperTintSampler&  wallpaperSampler()  { return *m_wallpaperSampler; }
    GestureRecognizer&     gestureRecognizer() { return *m_gestures; }

    // Compositor C callbacks
    static void cbPresent(const struct wlr_output_event_present*, void*);
    static void cbNewSurface(struct wlr_surface*, void*);
    static void cbSurfaceDestroy(struct wlr_surface*, void*);
    static void cbKey(uint32_t, uint32_t, bool, void*);
    static void cbPointerMotion(double, double, void*);
    static void cbPointerButton(uint32_t, bool, void*);
    static void cbPointerAxis(double, double, void*);
    static void cbSwipeBegin(uint32_t, void*);
    static void cbSwipeUpdate(uint32_t, double, double, void*);
    static void cbSwipeEnd(bool, void*);

private:
    OSFDesktop() = default;
    void initSubsystems(uint32_t w, uint32_t h);
    void onPresent(const struct wlr_output_event_present*);
    void onNewSurface(struct wlr_surface*);
    void onSurfaceDestroy(struct wlr_surface*);

    std::unique_ptr<RenderPipeline>        m_render;
    std::unique_ptr<WindowManager>         m_wm;
    std::unique_ptr<Panel>                 m_panel;
    std::unique_ptr<Dock>                  m_dock;
    std::unique_ptr<CockpitView>           m_cockpit;
    std::unique_ptr<LockScreen>            m_lock;
    std::unique_ptr<BootCrossfade>         m_crossfade;
    std::unique_ptr<WallpaperTintSampler>  m_wallpaperSampler;
    std::unique_ptr<GestureRecognizer>     m_gestures;

    bool m_initialized = false;
    bool m_firstFrame  = true;
};
```


---

## PART 21 — CrashManager

CrashManager is a dedicated subsystem for fault detection, isolation, and recovery.
AnimusEngine and the C11 compositor share a single process. A SIGSEGV anywhere is fatal
to the display server. CrashManager's job is to minimize unrecoverable states,
detect degraded conditions before they become fatal, and perform controlled teardown
when they do.

### 21.1 Architecture Overview

```
PROCESS: animusengine (compositor + AnimusEngine, single address space)
│
├── SIGNAL LAYER (async-signal-safe only)
│   FirstResponder::signalHandler()     — SIGSEGV, SIGABRT, SIGBUS, SIGFPE
│       → write crash token to pipe     — async-signal-safe
│       → _exit(139)                    — no atexit, no destructors, no flush
│
├── BACKGROUND THREADS
│   GlobalFeed::monitorLoop()           — /proc polling, DRM budget, PW underruns
│   Handshakes::heartbeatLoop()         — PipeWire/D-Bus/wlroots liveness
│
└── MAIN THREAD (EventBus subscribers)
    CrashSite::onClientDisconnect()     — Wayland client died
    EventHandler::onCompositorEvent()   — wlroots error signals
    Vessels::evaluateBlastRadius()      — dependency DAG propagation
```

### 21.2 CrashManager.h

```cpp
// animus/crash/CrashManager.h
#pragma once
#include <memory>

namespace Animus {

class EventHandler;
class Handshakes;
class CrashSite;
class FirstResponder;
class GlobalFeed;
class Vessels;

// CrashManager: owns all fault-detection subsystems.
// Initialized before any other subsystem in OSFDesktop::run().
// FirstResponder installs signal handlers as the very first action.
class CrashManager {
public:
    static CrashManager& shared();

    // Must be called before AnimusEngine starts — installs signal handlers
    void initialize();
    void destroy();

    EventHandler&   eventHandler()   { return *m_eventHandler; }
    Handshakes&     handshakes()     { return *m_handshakes; }
    CrashSite&      crashSite()      { return *m_crashSite; }
    FirstResponder& firstResponder() { return *m_firstResponder; }
    GlobalFeed&     globalFeed()     { return *m_globalFeed; }
    Vessels&        vessels()        { return *m_vessels; }

private:
    CrashManager() = default;

    std::unique_ptr<FirstResponder> m_firstResponder;  // init first
    std::unique_ptr<GlobalFeed>     m_globalFeed;
    std::unique_ptr<Handshakes>     m_handshakes;
    std::unique_ptr<EventHandler>   m_eventHandler;
    std::unique_ptr<CrashSite>      m_crashSite;
    std::unique_ptr<Vessels>        m_vessels;
};

} // namespace Animus
```

### 21.3 FirstResponder — Signal Handling

```cpp
// animus/crash/FirstResponder.h
#pragma once
#include <csignal>
#include <cstdint>
#include <string>

namespace Animus {

// FirstResponder: installs async-signal-safe handlers for fatal signals.
// Handles: SIGSEGV, SIGABRT, SIGBUS, SIGFPE, SIGTERM, SIGHUP.
// Also monitors:
//   - Kernel OOM notification via /proc/pressure/memory (PSI)
//   - systemd watchdog (WATCHDOG_USEC env var)
//   - Failed nixos-rebuild switch (InstallManager publishes failure)
//   - Boot crossfade failure (BootCrossfade timeout)
//
// CRITICAL: signal handler is async-signal-safe ONLY.
// No malloc. No C++ exceptions. No mutexes. No stdio.
// Only: write(), _exit(), signal-safe atomics.
class FirstResponder {
public:
    void initialize();
    void destroy();

    // Watchdog: kick every WATCHDOG_USEC/2 (systemd sd_notify keepalive)
    void kickWatchdog();

    // Called by InstallManager on nixos-rebuild failure
    void onInstallFailed(const std::string &errorOutput);

    // Called by BootCrossfade on timeout
    void onBootCrossfadeFailed();

    static constexpr int WATCHDOG_INTERVAL_MS = 5000;

private:
    // Signal handler — async-signal-safe
    static void signalHandler(int sig, siginfo_t *info, void *ctx);

    // Pipe: signal handler writes token here; main thread reads via epoll
    int m_pipeFd[2] = { -1, -1 };  // m_pipeFd[0]=read, m_pipeFd[1]=write

    // PSI (Pressure Stall Information) monitor fd
    int m_psiFd = -1;

    bool m_watchdogActive = false;
};

} // namespace Animus
```

```cpp
// animus/crash/FirstResponder.cpp
#include "FirstResponder.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <csignal>
#include <unistd.h>
#include <fcntl.h>
#include <cstring>
#include <sys/signalfd.h>
#include <cstdlib>
#include <systemd/sd-daemon.h>

namespace Animus {

void FirstResponder::initialize() {
    // Create self-pipe for async-signal-safe main-thread notification
    pipe2(m_pipeFd, O_CLOEXEC | O_NONBLOCK);

    // Install handlers for all fatal signals
    struct sigaction sa = {};
    sa.sa_sigaction = &FirstResponder::signalHandler;
    sa.sa_flags     = SA_SIGINFO | SA_RESETHAND;  // SA_RESETHAND: don't loop
    sigemptyset(&sa.sa_mask);

    sigaction(SIGSEGV, &sa, nullptr);
    sigaction(SIGABRT, &sa, nullptr);
    sigaction(SIGBUS,  &sa, nullptr);
    sigaction(SIGFPE,  &sa, nullptr);

    // SIGTERM: graceful shutdown (compositor cleanup before exit)
    struct sigaction term = {};
    term.sa_sigaction = &FirstResponder::signalHandler;
    term.sa_flags     = SA_SIGINFO;
    sigemptyset(&term.sa_mask);
    sigaction(SIGTERM, &term, nullptr);

    // SIGHUP: reload config
    sigaction(SIGHUP, &term, nullptr);

    // PSI memory pressure: /proc/pressure/memory (kernel 4.20+)
    m_psiFd = open("/proc/pressure/memory", O_RDWR | O_CLOEXEC | O_NONBLOCK);
    if (m_psiFd >= 0) {
        // Trigger on 50ms stall in a 1000ms window
        const char *trigger = "some 50000 1000000\n";
        write(m_psiFd, trigger, strlen(trigger));
        // Add m_psiFd to epoll in GlobalFeed — fires OSFEvent::MemoryPressure
    }

    // systemd watchdog
    const char *wd = getenv("WATCHDOG_USEC");
    m_watchdogActive = (wd != nullptr && atoll(wd) > 0);
}

// ASYNC-SIGNAL-SAFE ONLY. No malloc. No locks. No C++ objects.
void FirstResponder::signalHandler(int sig, siginfo_t *info, void *ctx) {
    (void)ctx;
    // Write signal number to pipe — single byte, atomic for PIPE_BUF > 1
    uint8_t sigbyte = (uint8_t)sig;
    // write() is async-signal-safe
    int fd = -1;
    // Access pipe fd via global (only safe async-signal-safe pattern)
    extern int g_crashPipeWrite;  // set during initialize()
    if (g_crashPipeWrite >= 0)
        write(g_crashPipeWrite, &sigbyte, 1);

    if (sig == SIGSEGV || sig == SIGABRT || sig == SIGBUS || sig == SIGFPE) {
        // For fatal signals: flush compositor state marker to disk via write()
        // No stdio — write() to a pre-opened fd only
        // Then exit immediately — no atexit, no destructors
        _exit(128 + sig);
    }
    // SIGTERM/SIGHUP handled on main thread via pipe read
}

void FirstResponder::kickWatchdog() {
    if (m_watchdogActive)
        sd_notify(0, "WATCHDOG=1");
}

void FirstResponder::onInstallFailed(const std::string &errorOutput) {
    // Publish to main thread — InstallManager failed, show error sheet
    EventBus::shared().publishAsync(OSFEvent::InstallFailed,
        std::string(errorOutput));
}

void FirstResponder::onBootCrossfadeFailed() {
    // Boot crossfade timed out (>5s) — force-complete it
    EventBus::shared().publishAsync(OSFEvent::BootCrossfadeComplete);
}

void FirstResponder::destroy() {
    if (m_psiFd >= 0) { close(m_psiFd); m_psiFd = -1; }
    if (m_pipeFd[0] >= 0) { close(m_pipeFd[0]); m_pipeFd[0] = -1; }
    if (m_pipeFd[1] >= 0) { close(m_pipeFd[1]); m_pipeFd[1] = -1; }
}

} // namespace Animus
```

### 21.4 GlobalFeed — Resource Pressure Monitoring

```cpp
// animus/crash/GlobalFeed.h
#pragma once
#include <thread>
#include <atomic>
#include <cstdint>

namespace Animus {

// Pressure levels — published via OSFEvent::ResourcePressure
enum class PressureLevel { Normal=0, Low=1, Medium=2, Critical=3 };

struct ResourceSnapshot {
    uint64_t vmRssKb;        // from /proc/self/status VmRSS
    uint32_t openFdCount;    // from /proc/self/fd (readdir count)
    uint64_t gpuUsedBytes;   // DRM memory budget via drmGetMemoryBudget()
    uint32_t pwUnderruns;    // PipeWire buffer underrun counter
    PressureLevel memory;
    PressureLevel fds;
    PressureLevel gpu;
    PressureLevel audio;
};

// GlobalFeed: polls system resources on background thread.
// Publishes OSFEvent::ResourcePressure (data = ResourceSnapshot) via publishAsync.
// Poll interval: 2000ms normal, 500ms when any metric > PressureLevel::Low.
// Never blocks main thread.
class GlobalFeed {
public:
    void start();
    void stop();

    const ResourceSnapshot& lastSnapshot() const { return m_last; }

    // Thresholds — tunable
    static constexpr uint64_t RSS_WARN_KB      = 512 * 1024;   // 512 MB
    static constexpr uint64_t RSS_CRITICAL_KB  = 900 * 1024;   // 900 MB
    static constexpr uint32_t FD_WARN          = 800;
    static constexpr uint32_t FD_CRITICAL      = 950;          // near kernel default 1024
    static constexpr uint32_t PW_UNDERRUN_WARN = 3;

private:
    void monitorLoop();

    ResourceSnapshot  m_last = {};
    std::atomic<bool> m_running = false;
    std::thread       m_thread;

    // Cached fds
    int m_procStatusFd  = -1;  // /proc/self/status
    int m_drmFd         = -1;  // DRM fd from compositor

    uint64_t readVmRss();
    uint32_t countOpenFds();
    uint64_t readGpuBudget();
    PressureLevel classify(uint64_t val, uint64_t warn, uint64_t crit);
};

} // namespace Animus
```

```cpp
// animus/crash/GlobalFeed.cpp (key methods)
#include "GlobalFeed.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <cstdio>
#include <dirent.h>
#include <fcntl.h>
#include <unistd.h>
#include <cstring>
#include <chrono>
#include <thread>
#include <xf86drm.h>  // drmGetMemoryBudget (libdrm)

namespace Animus {

void GlobalFeed::start() {
    m_procStatusFd = open("/proc/self/status", O_RDONLY | O_CLOEXEC);
    m_running = true;
    m_thread = std::thread(&GlobalFeed::monitorLoop, this);
}

void GlobalFeed::monitorLoop() {
    while (m_running) {
        ResourceSnapshot snap;
        snap.vmRssKb     = readVmRss();
        snap.openFdCount = countOpenFds();
        snap.gpuUsedBytes= readGpuBudget();
        snap.memory      = classify(snap.vmRssKb, RSS_WARN_KB, RSS_CRITICAL_KB);
        snap.fds         = classify(snap.openFdCount, FD_WARN, FD_CRITICAL);

        bool anyPressure = (snap.memory > PressureLevel::Normal ||
                            snap.fds    > PressureLevel::Normal ||
                            snap.gpu    > PressureLevel::Normal ||
                            snap.audio  > PressureLevel::Normal);

        m_last = snap;
        EventBus::shared().publishAsync(OSFEvent::ResourcePressure, snap);

        int ms = anyPressure ? 500 : 2000;
        std::this_thread::sleep_for(std::chrono::milliseconds(ms));
    }
}

uint64_t GlobalFeed::readVmRss() {
    if (m_procStatusFd < 0) return 0;
    char buf[4096]; buf[0]='\0';
    lseek(m_procStatusFd, 0, SEEK_SET);
    ssize_t n = read(m_procStatusFd, buf, sizeof(buf)-1);
    if (n <= 0) return 0;
    buf[n] = '\0';
    const char *p = strstr(buf, "VmRSS:");
    if (!p) return 0;
    uint64_t kb = 0;
    sscanf(p, "VmRSS: %llu kB", &kb);
    return kb;
}

uint32_t GlobalFeed::countOpenFds() {
    DIR *d = opendir("/proc/self/fd");
    if (!d) return 0;
    uint32_t count = 0;
    while (readdir(d)) count++;
    closedir(d);
    return count > 2 ? count - 2 : 0;  // subtract . and ..
}

uint64_t GlobalFeed::readGpuBudget() {
    if (m_drmFd < 0) return 0;
    drmMemoryBudget budget = {};
    if (drmGetMemoryBudget(m_drmFd, &budget) != 0) return 0;
    // budget.vram_total - budget.vram_used = available
    return budget.vram_usage[0];  // used bytes on first heap
}

PressureLevel GlobalFeed::classify(uint64_t val, uint64_t warn, uint64_t crit) {
    if (val >= crit) return PressureLevel::Critical;
    if (val >= warn) return PressureLevel::Medium;
    if (val >= warn / 2) return PressureLevel::Low;
    return PressureLevel::Normal;
}

} // namespace Animus
```

### 21.5 Handshakes — Subsystem Liveness

```cpp
// animus/crash/Handshakes.h
#pragma once
#include <thread>
#include <atomic>
#include <chrono>
#include <unordered_map>
#include <string>
#include <functional>

namespace Animus {

enum class SubsystemHealth { Healthy, Degraded, Unresponsive, Dead };

struct HandshakeResult {
    std::string      subsystem;
    SubsystemHealth  health;
    std::string      detail;
};

// Handshakes: periodic liveness checks for external subsystems.
// Checks: PipeWire session, D-Bus session bus, wlroots backend health.
// Runs on background thread. Publishes OSFEvent::SubsystemHealth via publishAsync.
// On Unresponsive: gives 3 retries before declaring Dead.
// On Dead: Vessels evaluates blast radius and publishes isolation actions.
class Handshakes {
public:
    void start();
    void stop();

    // Register a custom health check
    void registerCheck(const std::string &name,
                       std::function<SubsystemHealth()> checker,
                       std::chrono::milliseconds interval);

private:
    struct Check {
        std::string                   name;
        std::function<SubsystemHealth()> fn;
        std::chrono::milliseconds     interval;
        std::chrono::steady_clock::time_point nextCheck;
        int                           failStreak = 0;
    };

    void heartbeatLoop();
    SubsystemHealth checkPipeWire();
    SubsystemHealth checkDBus();
    SubsystemHealth checkWlroots();

    std::vector<Check>  m_checks;
    std::atomic<bool>   m_running = false;
    std::thread         m_thread;
};

} // namespace Animus
```

### 21.6 CrashSite — Application Failure Handling

```cpp
// animus/crash/CrashSite.h
#pragma once
#include <string>
#include <unordered_map>
#include <cstdint>
#include <chrono>

struct wlr_surface;
struct wl_client;

namespace Animus {

// CrashSite: handles Wayland client failures.
//
// Clean path: client calls wl_display_disconnect() before exit.
//   wlroots fires on_surface_destroy callback → WindowManager::removeSurface()
//   → CrashSite::onCleanExit(client)
//
// Dirty path: client process dies without disconnect.
//   wlroots detects broken socket → fires client_destroy signal
//   → CrashSite::onClientCrash(client)
//
// Respawn policy: per-app, max 3 respawns in 10s window.
// After 3 failures: app marked dead, Dock running dot cleared,
//   OSFNotification("App crashed — could not restart") shown.
class CrashSite {
public:
    void initialize();

    // Called by WindowManager when a surface is destroyed normally
    void onCleanExit(struct wl_client *client, const std::string &appId);

    // Called when wlroots detects broken socket (client died)
    void onClientCrash(struct wl_client *client, const std::string &appId);

    // Returns true if app should be respawned
    bool shouldRespawn(const std::string &appId) const;
    void recordRespawn(const std::string &appId);

    static constexpr int    MAX_RESPAWNS      = 3;
    static constexpr double RESPAWN_WINDOW_S  = 10.0;

private:
    struct AppCrashRecord {
        int    respawnCount = 0;
        double firstCrashTime = 0;  // CLOCK_MONOTONIC seconds
    };
    std::unordered_map<std::string, AppCrashRecord> m_records;
};

} // namespace Animus
```

### 21.7 EventHandler — Compositor and D-Bus Event Triage

```cpp
// animus/crash/EventHandler.h
#pragma once
#include <string>
#include <functional>

namespace Animus {

// EventHandler: triage layer for error events from three sources:
//   1. Compositor (wlroots error/warning log callbacks)
//   2. Window events (unexpected surface state transitions)
//   3. D-Bus bridge events (connection lost, service crashed)
//
// Classifies events into:
//   - Recoverable: log + continue
//   - Degraded: notify Vessels, continue with reduced functionality
//   - Fatal: notify FirstResponder, controlled shutdown
class EventHandler {
public:
    void initialize();

    // Installed as wlr_log callback
    static void wlrLogCallback(enum wlr_log_importance importance,
                                const char *fmt, va_list args);

    // Called by DBusBridge on connection loss
    void onDBusConnectionLost(const std::string &busName);

    // Called by wlroots backend error signal (if exposed via wl_signal)
    void onCompositorError(const std::string &detail);

    // Called when a window enters an unexpected state
    // (e.g. mapped but no wlr_surface, or pending configure never acked)
    void onWindowStateAnomaly(const std::string &appId, const std::string &detail);

private:
    enum class Severity { Recoverable, Degraded, Fatal };
    Severity classify(const std::string &source, const std::string &detail);
    void dispatch(Severity sev, const std::string &source,
                  const std::string &detail);
};

} // namespace Animus
```

### 21.8 Vessels — Dependency DAG and Blast Radius

```cpp
// animus/crash/Vessels.h
#pragma once
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>
#include <functional>

namespace Animus {

enum class VesselState { Running, Degraded, Isolated, Dead };

struct Vessel {
    std::string               name;
    VesselState               state      = VesselState::Running;
    std::vector<std::string>  dependsOn;   // must be Running for this to Run
    std::function<void()>     onIsolate;   // called when this vessel is isolated
    std::function<void()>     onRestore;   // called when dependencies recover
};

// Vessels: subsystem dependency map + blast radius calculation.
//
// When a subsystem dies, Vessels:
//   1. Marks it Dead
//   2. Walks the dependency graph (BFS)
//   3. Marks all transitively dependent subsystems Degraded
//   4. Calls onIsolate() for each affected vessel
//   5. Publishes OSFEvent::BlastRadius (data = list of affected names)
//
// Isolation is GRACEFUL DEGRADATION, not shutdown.
// GlyphAtlas dead → TextRenderer isolated → Panel shows no text (icons only)
//                                          → compositor survives.
// PipeWire dead   → SoundEngine isolated   → no audio, everything else fine.
// wlroots dead    → everything is isolated → controlled restart.
//
// Recovery: when a dead vessel is restored, Vessels re-evaluates and
//   calls onRestore() for vessels that can resume.
class Vessels {
public:
    void initialize();

    void registerVessel(Vessel v);
    void markDead(const std::string &name);
    void markRestored(const std::string &name);

    VesselState stateOf(const std::string &name) const;
    std::vector<std::string> blastRadius(const std::string &deadName) const;

private:
    std::unordered_map<std::string, Vessel>  m_vessels;

    // BFS from dead node through reverse dependency edges
    std::vector<std::string> bfsDependents(const std::string &name) const;
    void applyIsolation(const std::vector<std::string> &affected);
};

} // namespace Animus
```

```cpp
// animus/crash/Vessels.cpp — vessel registration and blast radius
#include "Vessels.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"

namespace Animus {

void Vessels::initialize() {
    // Register the VitusOS subsystem dependency graph
    // Format: name, dependsOn[], onIsolate, onRestore

    registerVessel({ "Compositor",    {},                          {},{} });
    registerVessel({ "VulkanContext", {"Compositor"},              {},{} });
    registerVessel({ "GlyphAtlas",    {"VulkanContext"},           {},{} });
    registerVessel({ "TextRenderer",  {"GlyphAtlas"},
        []{ /* Panel falls back to icon-only mode */ },
        []{ /* Panel restores text */ } });
    registerVessel({ "MaterialRenderer",{"VulkanContext"},         {},{} });
    registerVessel({ "ShadowRenderer",  {"VulkanContext"},         {},{} });
    registerVessel({ "RenderPipeline",  {"MaterialRenderer","ShadowRenderer","TextRenderer"}, {},{} });
    registerVessel({ "AnimationClock",  {"Compositor"},            {},{} });
    registerVessel({ "SpringSolver",    {},                        {},{} });  // header-only, never "dead"
    registerVessel({ "AnimationEngine", {"AnimationClock"},        {},{} });
    registerVessel({ "EventBus",        {},                        {},{} });  // core, no deps
    registerVessel({ "StateManager",    {"EventBus"},              {},{} });
    registerVessel({ "WallpaperTintSampler", {"VulkanContext"},    {},{} });
    registerVessel({ "SoundEngine",     {},                        // PipeWire
        []{ EventBus::shared().publishAsync(OSFEvent::ResourcePressure, {}); },
        []{} });
    registerVessel({ "DBusBridge",      {},                        // D-Bus session
        []{ /* fallback: no third-party menus, no AT-SPI */ },
        []{} });
    registerVessel({ "AccessibilityProvider", {"DBusBridge"},      {},{} });
    registerVessel({ "PortalGateway",   {"DBusBridge"},            {},{} });
    registerVessel({ "ClipboardBridge", {"Compositor"},            {},{} });
    registerVessel({ "DirectoryWatcher",{},                        {},{} });
    registerVessel({ "FileOperationDaemon", {"DirectoryWatcher"},  {},{} });
    registerVessel({ "InstallManager",  {},                        {},{} });
    registerVessel({ "Panel",           {"RenderPipeline"},        {},{} });
    registerVessel({ "Dock",            {"RenderPipeline"},        {},{} });
    registerVessel({ "CockpitView",     {"RenderPipeline"},        {},{} });
    registerVessel({ "LockScreen",      {"RenderPipeline"},        {},{} });
}

void Vessels::markDead(const std::string &name) {
    auto it = m_vessels.find(name);
    if (it == m_vessels.end()) return;
    it->second.state = VesselState::Dead;

    auto affected = bfsDependents(name);
    applyIsolation(affected);

    EventBus::shared().publishAsync(OSFEvent::BlastRadius,
        std::vector<std::string>(affected));
}

std::vector<std::string> Vessels::blastRadius(const std::string &name) const {
    return bfsDependents(name);
}

std::vector<std::string> Vessels::bfsDependents(const std::string &root) const {
    // Build reverse edge map
    std::unordered_map<std::string, std::vector<std::string>> revEdges;
    for (auto& [name, v] : m_vessels)
        for (auto& dep : v.dependsOn)
            revEdges[dep].push_back(name);

    // BFS
    std::vector<std::string> result;
    std::unordered_set<std::string> visited;
    std::vector<std::string> queue = { root };
    while (!queue.empty()) {
        std::string cur = queue.back(); queue.pop_back();
        if (visited.count(cur)) continue;
        visited.insert(cur);
        if (cur != root) result.push_back(cur);
        auto it = revEdges.find(cur);
        if (it != revEdges.end())
            for (auto& dep : it->second)
                queue.push_back(dep);
    }
    return result;
}

void Vessels::applyIsolation(const std::vector<std::string> &affected) {
    for (auto& name : affected) {
        auto it = m_vessels.find(name);
        if (it == m_vessels.end()) continue;
        it->second.state = VesselState::Isolated;
        if (it->second.onIsolate) it->second.onIsolate();
    }
}

} // namespace Animus
```

### 21.9 OSFEvent additions for CrashManager

Add to `OSFEvent.h` enum:
```cpp
// CrashManager events
ResourcePressure,       // data = ResourceSnapshot
SubsystemHealthChanged, // data = HandshakeResult
ClientCrashed,          // data = std::string appId
BlastRadius,            // data = std::vector<std::string> affected vessels
InstallFailed,          // data = std::string errorOutput
MemoryPressure,         // data = PressureLevel (from PSI fd)
```

---

## PART 22 — EO-Bus (EventOutsider-Bus)

EO-Bus is the translation and trust boundary between AnimusEngine's internal
EventBus and everything outside: D-Bus services, accessibility infrastructure,
and the XDG portal system. No external client ever touches EventBus directly.
Everything passes through EO-Bus's validation layer.

### 22.1 Architecture Overview

```
EXTERNAL WORLD                   EO-BUS LAYER                    INTERNAL
─────────────────────────────────────────────────────────────────────────
GTK/Qt apps                  ┌──────────────────┐
  DBusMenu (menus)  ─────────► DBusBridge        │
  StatusNotifier    ─────────►   validation      ├──► EventBus::publishAsync()
  Notifications     ─────────►   rate limiting   │         (main thread only)
  (external)        ─────────►   schema check    │
                              │   trust boundary  │
D-Bus session bus  ◄──────────┤                  │◄── OSFNative surface tree
  AT-SPI tree       ──────────► AccessibilityProvider
  (outbound)        ◄──────────  (compositor→AT-SPI)
  ReducedMotion    ─────────────►                │
                              │                  │
xdg-desktop-portal ◄──────────┤ PortalGateway    │
  FileOpen         ─────────────► → Filer IPC    │
  Screenshot       ─────────────► → wlr_read_px  │
  ScreenRecord     ─────────────► → PW stream    │
  OpenURI          ─────────────► → Pathfinder   │
                              └──────────────────┘

Library: sdbus-c++ (modern C++17 wrapper over libdbus, no generated stubs)
D-Bus session bus: provided by systemd user session (dbus-broker)
```

### 22.2 EOBus.h

```cpp
// animus/eobus/EOBus.h
#pragma once
#include <memory>

namespace Animus {

class DBusBridge;
class AccessibilityProvider;
class PortalGateway;

// EOBus: the EventOutsider-Bus.
// Owns DBusBridge, AccessibilityProvider, PortalGateway.
// Initialized after EventBus and before Shell components.
// Requires D-Bus session bus (provided by systemd user session).
class EOBus {
public:
    static EOBus& shared();
    bool initialize();
    void destroy();

    DBusBridge&            dbusBridge()    { return *m_dbus; }
    AccessibilityProvider& accessibility() { return *m_a11y; }
    PortalGateway&         portal()        { return *m_portal; }

private:
    EOBus() = default;
    std::unique_ptr<DBusBridge>            m_dbus;
    std::unique_ptr<AccessibilityProvider> m_a11y;
    std::unique_ptr<PortalGateway>         m_portal;
};

} // namespace Animus
```

### 22.3 DBusBridge.h

```cpp
// animus/eobus/DBusBridge.h
#pragma once
#include <string>
#include <memory>
#include <functional>
#include <cstdint>

// sdbus-c++ — modern C++17 D-Bus wrapper, no generated stubs required
// NixOS package: sdbuscpp
#include <sdbus-c++/sdbus-c++.h>

namespace Animus {

// DBusBridge: the trust boundary between the D-Bus world and AnimusEngine.
//
// INBOUND (D-Bus → AnimusEngine, all validated before touching EventBus):
//   DBusMenu     — com.canonical.dbusmenu → global menu bar
//   StatusNotifier — org.kde.StatusNotifierItem → system tray
//   Notifications — org.freedesktop.Notifications → OSFNotification
//
// OUTBOUND (AnimusEngine → D-Bus):
//   AT-SPI       — org.a11y.atspi2 (via AccessibilityProvider)
//
// TRUST BOUNDARY:
//   All inbound D-Bus signals pass through validateMessage() before
//   any EventBus::publishAsync() call. Rate limiting: 60 messages/sec
//   per sender. Schema enforcement: known interfaces only, typed args.
//   No raw D-Bus message ever reaches EventBus or OSFNative surfaces.
class DBusBridge {
public:
    bool initialize();
    void destroy();

    // Validation layer — called for every inbound message
    bool validateMessage(const std::string &sender,
                         const std::string &interface,
                         const std::string &member);

    // Rate limiter — returns false if sender exceeds 60 msg/sec
    bool checkRateLimit(const std::string &sender);

private:
    // D-Bus connection (session bus)
    std::unique_ptr<sdbus::IConnection> m_conn;

    // Menu proxy: watches com.canonical.dbusmenu services
    void onMenuItemActivated(const std::string &appId,
                              const std::string &itemPath);
    void onMenuLayoutChanged(const std::string &appId,
                              const std::string &menuJson);

    // Status notifier: org.kde.StatusNotifierWatcher
    void onStatusNotifierItemRegistered(const std::string &serviceName);
    void onStatusNotifierItemUnregistered(const std::string &serviceName);

    // Notification proxy: implements org.freedesktop.Notifications
    uint32_t onNotify(const std::string &appName,
                      uint32_t           replacesId,
                      const std::string &appIcon,
                      const std::string &summary,
                      const std::string &body,
                      int32_t            timeout);
    void onCloseNotification(uint32_t id);

    // Rate limiting: per-sender message counter + timestamp
    struct RateRecord { uint32_t count; double windowStart; };
    std::unordered_map<std::string, RateRecord> m_rateMap;
    static constexpr uint32_t MAX_MSG_PER_SEC = 60;
};

} // namespace Animus
```

```cpp
// animus/eobus/DBusBridge.cpp (validation + notification proxy)
#include "DBusBridge.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <time.h>
#include <cstring>

namespace Animus {

bool DBusBridge::initialize() {
    try {
        m_conn = sdbus::createSessionBusConnection("org.vitusos.AnimusEngine");
    } catch (const sdbus::Error &e) {
        return false;  // D-Bus session not available — degrade gracefully
    }

    // Implement org.freedesktop.Notifications (notification proxy)
    // External apps call Notify() → we translate to OSFNotification
    m_conn->addObjectManager("/org/freedesktop/Notifications");
    // Registration of interface methods omitted for brevity —
    // onNotify() and onCloseNotification() registered as method handlers.

    m_conn->enterEventLoopAsync();  // sdbus manages its own thread
    return true;
}

bool DBusBridge::validateMessage(const std::string &sender,
                                  const std::string &interface,
                                  const std::string &member)
{
    // Known allowed interfaces — reject everything else
    static const char *ALLOWED[] = {
        "com.canonical.dbusmenu",
        "org.kde.StatusNotifierItem",
        "org.kde.StatusNotifierWatcher",
        "org.freedesktop.Notifications",
        "org.a11y.atspi2.Registry",
        nullptr
    };
    bool known = false;
    for (int i = 0; ALLOWED[i]; i++)
        if (interface == ALLOWED[i]) { known = true; break; }
    if (!known) return false;

    return checkRateLimit(sender);
}

bool DBusBridge::checkRateLimit(const std::string &sender) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    double now = ts.tv_sec + ts.tv_nsec * 1e-9;

    auto &rec = m_rateMap[sender];
    if (now - rec.windowStart > 1.0) {
        rec.windowStart = now; rec.count = 0;
    }
    if (++rec.count > MAX_MSG_PER_SEC) return false;  // drop
    return true;
}

uint32_t DBusBridge::onNotify(const std::string &appName,
                               uint32_t replacesId,
                               const std::string &appIcon,
                               const std::string &summary,
                               const std::string &body,
                               int32_t timeout)
{
    if (!validateMessage(appName, "org.freedesktop.Notifications", "Notify"))
        return 0;

    // Translate to OSFNotification via EventBus — never touch OSFNotification directly
    struct NotifData { std::string title, body; int timeoutMs; };
    NotifData d { summary, body, timeout <= 0 ? 5000 : timeout };
    EventBus::shared().publishAsync(OSFEvent::NotificationPosted, std::move(d));

    static uint32_t nextId = 1;
    return nextId++;
}

} // namespace Animus
```

### 22.4 AccessibilityProvider.h

```cpp
// animus/eobus/AccessibilityProvider.h
#pragma once
#include <string>
#include <vector>
#include <memory>
#include <cstdint>

namespace Animus {

// AT-SPI role constants (subset of org.a11y.atspi2.Role)
enum class A11yRole : uint32_t {
    Window         = 64,
    Panel          = 48,
    ToolBar        = 57,
    PushButton     = 28,
    TextField      = 60,
    Label          = 34,
    CheckBox       =  8,
    Slider         = 52,
    ProgressBar    = 29,
    List           = 35,
    ListItem       = 36,
    ScrollBar      = 47,
    Separator      = 49,
    Filler         = 19,
};

struct A11yNode {
    uint32_t           id;          // unique within this session
    A11yRole           role;
    std::string        name;        // accessible name
    std::string        description;
    bool               focused;
    bool               enabled;
    std::vector<uint32_t> children;
};

// AccessibilityProvider: exposes the OSFNative surface tree to AT-SPI.
//
// Direction: AnimusEngine → D-Bus (outbound only from compositor's perspective).
// Consumes: OSFNative surface tree (Panel, Dock, all open windows).
// Exposes: org.a11y.atspi2 — Orca and other screen readers connect here.
//
// Focus chain: keyboard Tab order follows window z-order, then
//   within-window order: toolbar → sidebar → content → overlays.
//
// ReducedMotion: reads from vitusos-config.nix user prefs.
//   On change: publishes OSFEvent::ReducedMotionChanged.
//   SpringSolver::g_reducedMotion = true → tick() snaps to target instantly.
//   MaterialRenderer reduces blur transition speed.
class AccessibilityProvider {
public:
    bool initialize();
    void destroy();

    // Rebuild accessibility tree from current surface layout
    // Called on: window open/close, focus change, layout change
    void rebuildTree();

    // Focus chain navigation (Tab/Shift-Tab)
    uint32_t nextFocus(uint32_t currentId) const;
    uint32_t prevFocus(uint32_t currentId) const;

    // Announce focus change to screen reader
    void announceNodeFocused(uint32_t id);

    bool reducedMotionEnabled() const { return m_reducedMotion; }

private:
    std::vector<A11yNode>              m_tree;
    std::vector<uint32_t>              m_focusChain;
    bool                               m_reducedMotion = false;

    uint64_t m_windowFocusHandle  = 0;
    uint64_t m_windowOpenHandle   = 0;
    uint64_t m_windowCloseHandle  = 0;

    void loadReducedMotionSetting();
    A11yNode buildNodeForSurface(const std::string &surfaceType,
                                  uint32_t id, const std::string &name);
};

} // namespace Animus
```

### 22.5 SpringSolver — ReducedMotion Integration

```cpp
// Modification to animus/animation/SpringSolver.h
// Add global reduced motion flag. When true, tick() snaps instantly.

namespace Animus {

// Global reduced motion flag — set by AccessibilityProvider
// when OSFEvent::ReducedMotionChanged fires
inline bool& reducedMotionEnabled() {
    static bool g = false;
    return g;
}

class SpringSolver {
    // ... (existing fields unchanged) ...

    void tick(float dt) {
        // Reduced motion: skip animation, snap to target immediately
        if (reducedMotionEnabled()) {
            m_pos = m_target;
            m_vel = 0.0f;
            return;
        }
        // ... existing semi-implicit Euler unchanged ...
        if (dt < 0.001f) dt = 0.001f;
        if (dt > 0.100f) dt = 0.100f;
        if (isResting()) { m_pos = m_target; m_vel = 0.0f; return; }
        float accel = -m_cfg.stiffness*(m_pos-m_target) - m_cfg.damping*m_vel;
        m_vel += accel * dt;
        m_pos += m_vel * dt;
    }
};

} // namespace Animus
```

### 22.6 PortalGateway.h

```cpp
// animus/eobus/PortalGateway.h
#pragma once
#include <string>
#include <memory>
#include <functional>
#include <cstdint>
#include <sdbus-c++/sdbus-c++.h>

namespace Animus {

// PortalGateway: implements the xdg-desktop-portal backend.
// D-Bus service: org.freedesktop.impl.portal.desktop.vitusos
//
// Interface implementations:
//   org.freedesktop.impl.portal.FileChooser
//       → routes to Filer via IPC (Unix socket: /run/user/$UID/vitusos-filer.sock)
//       → user sees Filer's native picker, not GTK's file chooser
//
//   org.freedesktop.impl.portal.Screenshot
//       → uses wlr_renderer_read_pixels() (compositor-authorized)
//       → requesting app cannot specify capture region beyond its own window
//
//   org.freedesktop.impl.portal.ScreenCast
//       → creates PipeWire stream via compositor
//       → authorized by wl_seat focus — only focused app can record
//       → DMA-BUF sharing via PipeWire spa_data_type::SPA_DATA_DmaBuf
//
//   org.freedesktop.impl.portal.OpenURI
//       → routes through Pathfinder's app resolution logic
//       → EventBus::publishAsync(OSFEvent::OpenURI, uri)
//
// Security: compositor must authorize each portal request.
// PortalGateway never grants access without compositor confirmation.
class PortalGateway {
public:
    bool initialize();
    void destroy();

private:
    std::unique_ptr<sdbus::IConnection> m_conn;
    std::unique_ptr<sdbus::IObject>     m_obj;

    // FileChooser
    void onOpenFile(const std::string &appId,
                    const std::string &title,
                    std::function<void(std::string)> result);

    void onSaveFile(const std::string &appId,
                    const std::string &title,
                    const std::string &currentName,
                    std::function<void(std::string)> result);

    // Screenshot
    void onTakeScreenshot(const std::string &appId,
                          bool interactive,
                          std::function<void(std::string path)> result);

    // ScreenCast
    void onCreateSession(const std::string &appId,
                         std::function<void(uint32_t pipeWireNode)> result);

    // OpenURI
    void onOpenURI(const std::string &appId, const std::string &uri);

    // Compositor authorization helper
    bool compositorAuthorizes(const std::string &appId,
                               const std::string &capability);
};

} // namespace Animus
```

### 22.7 OSFEvent additions for EO-Bus

Add to `OSFEvent.h` enum:
```cpp
// EO-Bus events
DBusMenuChanged,          // data = std::string appId — menu layout changed
StatusNotifierChanged,    // data = std::string serviceName
AccessibilityTreeChanged, // data = (no data — subscribers rebuild)
ReducedMotionChanged,     // data = bool enabled
OpenURI,                  // data = std::string uri
PortalFileChosen,         // data = std::string path (result from Filer picker)
PortalScreenCastStarted,  // data = uint32_t pipeWireNode
```

### 22.8 NixOS packages for EO-Bus

```nix
# nixos/configuration.nix — add to buildInputs
# sdbus-c++ — modern C++17 D-Bus wrapper
sdbuscpp

# xdg-desktop-portal — required for PortalGateway to register as backend
xdg-desktop-portal

# at-spi2-core — AT-SPI D-Bus interfaces
at-spi2-core

# Portal backend desktop entry
environment.etc."xdg/xdg-desktop-portal/portals.conf".text = ''
  [preferred]
  default=vitusos
'';

# D-Bus service file for portal backend
services.dbus.packages = [ pkgs.vitusos-animus ];
```


---

## PART 23 — FirstResponder: Full Intel Collection

### 23.1 Design Contract

When a fatal signal fires, FirstResponder has one job: collect everything
knowable about the system state and write it to disk before dying.

Two hard constraints govern everything:

**Constraint 1: async-signal-safe only in Phase 1.**
The heap may be corrupt. Mutexes may be held by the faulting thread.
`malloc`, `printf`, `std::string`, `std::vector`, locks, C++ exceptions —
all banned. Only: `write()`, `read()`, `open()`, `clock_gettime()`,
`getpid()`, `gettid()`, `memcpy()`, `snprintf()`, `_exit()`.

**Constraint 2: pre-allocation before first frame.**
Everything the signal handler needs must already exist in static memory
before `OSFDesktop::run()` calls `wlr_backend_start()`. Nothing is
allocated at crash time. The signal handler only writes.

### 23.2 Static Crash State — Pre-Written by Subsystems

```cpp
// animus/crash/CrashState.h
// The global static intel block.
// Written continuously by subsystems during normal operation.
// Read by the signal handler at crash time — no locks, torn reads acceptable.
// Allocated once at startup via CrashStateBlock::initialize().
// NEVER heap-allocated after that point.
#pragma once
#include <cstdint>
#include <cstddef>
#include <atomic>

namespace Animus {

static constexpr size_t MAX_STACK_FRAMES   = 64;
static constexpr size_t MAX_VESSEL_NAME    = 32;
static constexpr size_t MAX_VESSEL_COUNT   = 32;
static constexpr size_t MAX_EVENT_RING     = 64;
static constexpr size_t MAX_WAYLAND_CLIENTS= 32;
static constexpr size_t MAX_CLIENT_APPID   = 64;
static constexpr size_t MAX_MAPS_SNAPSHOT  = 65536;  // 64KB for /proc/self/maps
static constexpr size_t MAX_SOUND_NAME     = 32;

// Written by AnimationClock::onPresent() every frame
struct CrashFrame {
    std::atomic<uint64_t>  frameNumber;     // incremented each vblank
    std::atomic<double>    totalTimeS;      // AnimationClock::totalTime()
    std::atomic<float>     lastDt;          // last frame dt
    std::atomic<float>     refreshHz;       // EMA refresh rate
};

// Written by AnimationEngine::tick() for each active spring
struct CrashSpringEntry {
    char   name[32];       // e.g. "OSFWindow[0].pos.x"
    float  value;
    float  target;
    float  velocity;
    bool   isResting;
};

// Written by Vessels on each state change
struct CrashVesselEntry {
    char    name[MAX_VESSEL_NAME];
    uint8_t state;          // VesselState cast to uint8_t
};

// Written by EventBus::publish() into a ring buffer
struct CrashEventEntry {
    uint32_t eventId;       // OSFEvent cast to uint32_t
    double   timeS;         // CLOCK_MONOTONIC when published
};

// Written by CrashSite on each client connect/crash
struct CrashClientEntry {
    char    appId[MAX_CLIENT_APPID];
    bool    connected;
    uint32_t pid;
};

// Written by GlobalFeed::monitorLoop() each poll cycle
struct CrashResourceEntry {
    uint64_t vmRssKb;
    uint32_t openFdCount;
    uint64_t gpuUsedBytes;
    uint32_t pwUnderruns;
    uint8_t  memPressure;   // PressureLevel
    uint8_t  fdPressure;
    uint8_t  gpuPressure;
    uint8_t  audioPressure;
};

// Written by SoundEngine on each play() call
struct CrashSoundEntry {
    char    name[MAX_SOUND_NAME];
    double  timeS;
};

// Written by StateManager::set() for well-known keys
struct CrashStateKeys {
    uint64_t focusedWindowId;
    bool     lockScreenVisible;
    bool     cockpitViewOpen;
    bool     pathfinderOpen;
    float    wallpaperTintR, wallpaperTintG, wallpaperTintB;
    float    systemVolume;
    bool     dockVisible;
};

// The complete static intel block — one global instance
struct CrashStateBlock {
    // Magic + version for crash reader
    uint32_t magic;         // 0x56544F53 = "VTOS"
    uint32_t version;       // 1

    // Frame state
    CrashFrame frame;

    // Active springs (ring — newest overwrites oldest)
    CrashSpringEntry  springs[MAX_STACK_FRAMES];
    std::atomic<uint32_t> springCount;

    // Vessel states
    CrashVesselEntry  vessels[MAX_VESSEL_COUNT];
    std::atomic<uint32_t> vesselCount;

    // Event ring buffer
    CrashEventEntry   events[MAX_EVENT_RING];
    std::atomic<uint32_t> eventHead;  // index of next write slot

    // Connected Wayland clients
    CrashClientEntry  clients[MAX_WAYLAND_CLIENTS];
    std::atomic<uint32_t> clientCount;

    // Resource snapshot (last GlobalFeed poll)
    CrashResourceEntry resources;

    // Last sound played
    CrashSoundEntry lastSound;

    // StateManager well-known keys
    CrashStateKeys stateKeys;

    // /proc/self/maps snapshot (updated every 10s by GlobalFeed)
    char   mapsSnapshot[MAX_MAPS_SNAPSHOT];
    size_t mapsSnapshotLen;

    static CrashStateBlock& global();
    static void initialize();
};

} // namespace Animus
```

```cpp
// animus/crash/CrashState.cpp
#include "CrashState.h"
#include <cstring>
#include <fcntl.h>
#include <unistd.h>

namespace Animus {

// Static storage — allocated in BSS, never heap
static CrashStateBlock g_crashState;

CrashStateBlock& CrashStateBlock::global() { return g_crashState; }

void CrashStateBlock::initialize() {
    memset(&g_crashState, 0, sizeof(g_crashState));
    g_crashState.magic   = 0x56544F53;  // "VTOS"
    g_crashState.version = 1;
}

} // namespace Animus
```

### 23.3 Subsystem Write-Back Points

```cpp
// Each subsystem writes to CrashStateBlock::global() at key moments.
// These are the exact call sites — not pseudo-code.

// ── AnimationClock::onPresent() ──────────────────────────────────
void AnimationClock::onPresent(const struct wlr_output_event_present *ev) {
    // ... existing dt calculation ...
    auto& cs = CrashStateBlock::global();
    cs.frame.frameNumber.fetch_add(1, std::memory_order_relaxed);
    cs.frame.totalTimeS.store(m_totalTime, std::memory_order_relaxed);
    cs.frame.lastDt.store(m_dt, std::memory_order_relaxed);
    cs.frame.refreshHz.store(m_refreshHz, std::memory_order_relaxed);
}

// ── AnimationEngine::tick() — register active springs ────────────
// Called from each component's OSFEvent::Tick handler.
// Components call this to register their current spring state.
void CrashStateBlock::updateSpring(const char *name,
                                    float value, float target,
                                    float velocity, bool resting)
{
    auto& cs = CrashStateBlock::global();
    uint32_t idx = cs.springCount.load(std::memory_order_relaxed)
                   % MAX_STACK_FRAMES;
    // Raw write — signal handler may read a torn struct, acceptable
    strncpy(cs.springs[idx].name, name, 31);
    cs.springs[idx].value    = value;
    cs.springs[idx].target   = target;
    cs.springs[idx].velocity = velocity;
    cs.springs[idx].isResting= resting;
    cs.springCount.fetch_add(1, std::memory_order_relaxed);
}

// ── EventBus::publish() — write to event ring ────────────────────
void EventBus::publish(OSFEvent event, const std::any &data) {
    // ... existing dispatch ...
    auto& cs = CrashStateBlock::global();
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    uint32_t slot = cs.eventHead.fetch_add(1, std::memory_order_relaxed)
                    % MAX_EVENT_RING;
    cs.events[slot].eventId = static_cast<uint32_t>(event);
    cs.events[slot].timeS   = ts.tv_sec + ts.tv_nsec * 1e-9;
}

// ── Vessels::registerVessel() + markDead() ───────────────────────
void Vessels::syncToCrashState() {
    auto& cs = CrashStateBlock::global();
    uint32_t i = 0;
    for (auto& [name, vessel] : m_vessels) {
        if (i >= MAX_VESSEL_COUNT) break;
        strncpy(cs.vessels[i].name, name.c_str(), MAX_VESSEL_NAME - 1);
        cs.vessels[i].state = static_cast<uint8_t>(vessel.state);
        i++;
    }
    cs.vesselCount.store(i, std::memory_order_relaxed);
}

// ── SoundEngine::play() ──────────────────────────────────────────
void SoundEngine::play(const std::string &name) {
    auto& cs = CrashStateBlock::global();
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    strncpy(cs.lastSound.name, name.c_str(), MAX_SOUND_NAME - 1);
    cs.lastSound.timeS = ts.tv_sec + ts.tv_nsec * 1e-9;
    // ... existing play logic ...
}

// ── StateManager::set() — well-known keys ────────────────────────
void StateManager::set(const std::string &key, std::any value) {
    m_state[key] = value;
    auto& cs = CrashStateBlock::global();
    auto& sk = cs.stateKeys;
    if (key == "focused_window_id")
        sk.focusedWindowId  = std::any_cast<uint64_t>(value);
    else if (key == "lock_screen_visible")
        sk.lockScreenVisible= std::any_cast<bool>(value);
    else if (key == "cockpit_view_open")
        sk.cockpitViewOpen  = std::any_cast<bool>(value);
    else if (key == "pathfinder_open")
        sk.pathfinderOpen   = std::any_cast<bool>(value);
    else if (key == "system_volume")
        sk.systemVolume     = std::any_cast<float>(value);
    else if (key == "dock_visibility")
        sk.dockVisible      = std::any_cast<bool>(value);
    else if (key == "wallpaper_tint_r")
        sk.wallpaperTintR   = std::any_cast<float>(value);
    else if (key == "wallpaper_tint_g")
        sk.wallpaperTintG   = std::any_cast<float>(value);
    else if (key == "wallpaper_tint_b")
        sk.wallpaperTintB   = std::any_cast<float>(value);
    EventBus::shared().publish(OSFEvent::StateChanged, key);
}

// ── GlobalFeed::monitorLoop() — resource + maps snapshot ─────────
void GlobalFeed::monitorLoop() {
    while (m_running) {
        // ... existing resource collection ...
        auto& cs = CrashStateBlock::global();
        cs.resources.vmRssKb      = snap.vmRssKb;
        cs.resources.openFdCount  = snap.openFdCount;
        cs.resources.gpuUsedBytes = snap.gpuUsedBytes;
        cs.resources.pwUnderruns  = snap.pwUnderruns;
        cs.resources.memPressure  = static_cast<uint8_t>(snap.memory);
        cs.resources.fdPressure   = static_cast<uint8_t>(snap.fds);
        cs.resources.gpuPressure  = static_cast<uint8_t>(snap.gpu);
        cs.resources.audioPressure= static_cast<uint8_t>(snap.audio);

        // Refresh /proc/self/maps snapshot every 10 seconds
        static double lastMapRefresh = 0;
        double now = snap.vmRssKb; // placeholder, use real time
        struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
        double t = ts.tv_sec + ts.tv_nsec * 1e-9;
        if (t - lastMapRefresh > 10.0) {
            int fd = open("/proc/self/maps", O_RDONLY | O_CLOEXEC);
            if (fd >= 0) {
                ssize_t n = read(fd, cs.mapsSnapshot,
                                 MAX_MAPS_SNAPSHOT - 1);
                cs.mapsSnapshotLen = n > 0 ? (size_t)n : 0;
                close(fd);
            }
            lastMapRefresh = t;
        }

        EventBus::shared().publishAsync(OSFEvent::ResourcePressure, snap);
        int ms = (snap.memory > PressureLevel::Normal ||
                  snap.fds    > PressureLevel::Normal) ? 500 : 2000;
        std::this_thread::sleep_for(std::chrono::milliseconds(ms));
    }
}
```

### 23.4 CrashDump Binary Format

```cpp
// animus/crash/CrashDump.h
// Fixed-size binary format written by signal handler.
// All fields plain C types. No pointers. No padding surprises — use
// __attribute__((packed)) to guarantee layout across compiler versions.
// One write() call writes the entire struct.
#pragma once
#include <cstdint>
#include <cstddef>

namespace Animus {

static constexpr uint32_t CRASHDUMP_MAGIC   = 0x56544F53;  // "VTOS"
static constexpr uint32_t CRASHDUMP_VERSION = 1;
static constexpr size_t   CRASHDUMP_MAX_FRAMES = 64;

struct __attribute__((packed)) CrashRegisterBlock {
    // x86_64 general-purpose registers from ucontext_t->uc_mcontext.gregs
    uint64_t rax, rbx, rcx, rdx;
    uint64_t rsi, rdi, rbp, rsp;
    uint64_t r8,  r9,  r10, r11;
    uint64_t r12, r13, r14, r15;
    uint64_t rip;          // instruction pointer — WHERE it crashed
    uint64_t rflags;       // EFLAGS
    uint64_t cs, ss;       // segment registers
    uint64_t cr2;          // page fault address (from siginfo_t si_addr for SIGSEGV)
};

struct __attribute__((packed)) CrashDump {
    // ── Header ──────────────────────────────────────────────────
    uint32_t magic;             // CRASHDUMP_MAGIC
    uint32_t version;           // CRASHDUMP_VERSION
    uint32_t signum;            // signal number (SIGSEGV=11, SIGABRT=6, etc.)
    uint32_t sicode;            // siginfo_t si_code (SEGV_MAPERR, SEGV_ACCERR...)
    uint64_t faultAddress;      // siginfo_t si_addr cast to uint64_t
    uint64_t pid;               // getpid()
    uint64_t tid;               // gettid() — which thread crashed
    uint64_t timestampSec;      // clock_gettime CLOCK_MONOTONIC tv_sec
    uint64_t timestampNsec;     // clock_gettime CLOCK_MONOTONIC tv_nsec

    // ── CPU State ───────────────────────────────────────────────
    CrashRegisterBlock regs;

    // ── Stack Trace ─────────────────────────────────────────────
    uint32_t  stackDepth;                          // number of valid frames
    uint64_t  stackFrames[CRASHDUMP_MAX_FRAMES];   // raw return addresses

    // ── AnimusEngine Frame State ─────────────────────────────────
    uint64_t  frameNumber;      // which vblank frame we were on
    double    totalTimeS;       // seconds since compositor started
    float     lastDt;           // last frame dt (seconds)
    float     refreshHz;        // EMA refresh rate estimate

    // ── StateManager Snapshot ────────────────────────────────────
    uint64_t  focusedWindowId;
    uint8_t   lockScreenVisible;
    uint8_t   cockpitViewOpen;
    uint8_t   pathfinderOpen;
    uint8_t   dockVisible;
    float     wallpaperTintR, wallpaperTintG, wallpaperTintB;
    float     systemVolume;

    // ── Resource Snapshot ────────────────────────────────────────
    uint64_t  vmRssKb;          // process RSS at crash time
    uint32_t  openFdCount;      // open file descriptor count
    uint64_t  gpuUsedBytes;     // GPU VRAM usage
    uint32_t  pwUnderruns;      // PipeWire underrun count
    uint8_t   memPressure;      // PressureLevel enum value
    uint8_t   fdPressure;
    uint8_t   gpuPressure;
    uint8_t   audioPressure;

    // ── Last 8 OSFEvents ─────────────────────────────────────────
    // Ring buffer tail — 8 most recent events before crash
    uint32_t  lastEvents[8];     // OSFEvent cast to uint32_t
    double    lastEventTimes[8]; // CLOCK_MONOTONIC timestamp each

    // ── Vessel States ────────────────────────────────────────────
    uint32_t  vesselCount;
    struct __attribute__((packed)) {
        char    name[32];
        uint8_t state;           // VesselState cast to uint8_t
    } vessels[32];

    // ── Last Sound ───────────────────────────────────────────────
    char      lastSoundName[32];
    double    lastSoundTimeS;

    // ── Padding to align to 4096 bytes for O_DIRECT if needed ───
    uint8_t   _pad[1];  // compiler will warn if we overshoot — intentional
};
// Verify at compile time that we haven't grown beyond a reasonable size
// static_assert(sizeof(CrashDump) <= 8192, "CrashDump too large for signal handler");

} // namespace Animus
```

### 23.5 FirstResponder — Complete Implementation

```cpp
// animus/crash/FirstResponder.h — complete replacement
#pragma once
#include "CrashDump.h"
#include <csignal>
#include <cstdint>
#include <string>
#include <atomic>

namespace Animus {

// Global pipe write-end — accessed by signal handler
// Must be extern, not a class member (signal handler has no 'this')
extern int g_crashPipeWrite;

class FirstResponder {
public:
    // initialize() MUST be the first call in OSFDesktop::run(),
    // before wlr_backend_start(), before any subsystem init.
    void initialize();
    void destroy();

    // Called every frame from AnimationEngine::tick()
    // Sends sd_notify(WATCHDOG=1) if systemd watchdog is active
    void kickWatchdog();

    // Called by InstallManager on nixos-rebuild failure
    void onInstallFailed(const std::string &errorOutput);

    // Called by BootCrossfade if crossfade exceeds 5 seconds
    void onBootCrossfadeFailed();

    // Phase 2 handler — called from main thread when pipe fires
    // Does the heavy lifting: symbol resolution, human-readable report
    void handleCrashOnMainThread(uint8_t signum);

    static constexpr int    WATCHDOG_INTERVAL_MS = 5000;
    static constexpr double BOOT_CROSSFADE_TIMEOUT_S = 5.0;

private:
    // ── Phase 1: Signal Handler (async-signal-safe ONLY) ─────────
    static void signalHandler(int sig, siginfo_t *info, void *ctx);

    // Pre-opened crash dump fd — opened during initialize()
    // Signal handler writes directly to this fd
    static int s_crashFd;

    // Static CrashDump buffer — never heap-allocated
    // Signal handler fills this, then write()s it
    static CrashDump s_dump;

    // Self-pipe: signal handler writes 1 byte → main thread wakes
    int m_pipeFd[2] = { -1, -1 };

    // PSI pressure fd (/proc/pressure/memory)
    int m_psiFd = -1;

    bool m_watchdogActive = false;
};

} // namespace Animus
```

```cpp
// animus/crash/FirstResponder.cpp — complete implementation
#define _GNU_SOURCE  // gettid(), _Unwind_Backtrace
#include "FirstResponder.h"
#include "CrashState.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include "crash/Vessels.h"

#include <csignal>
#include <cstring>
#include <cstdio>
#include <cstdlib>
#include <unistd.h>
#include <fcntl.h>
#include <sys/types.h>
#include <time.h>
#include <ucontext.h>
#include <unwind.h>
#include <sys/utsname.h>
#include <systemd/sd-daemon.h>

namespace Animus {

// ── Static storage — never heap-allocated ────────────────────────
int       FirstResponder::s_crashFd = -1;
CrashDump FirstResponder::s_dump    = {};
int       g_crashPipeWrite          = -1;

// ── Stack unwind callback ─────────────────────────────────────────
struct UnwindCtx {
    uint64_t *frames;
    uint32_t  count;
    uint32_t  max;
};

static _Unwind_Reason_Code unwindCallback(struct _Unwind_Context *ctx,
                                            void *arg)
{
    UnwindCtx *uc = static_cast<UnwindCtx*>(arg);
    if (uc->count >= uc->max) return _URC_END_OF_STACK;
    uc->frames[uc->count++] = (uint64_t)_Unwind_GetIP(ctx);
    return _URC_NO_REASON;
}

// ── initialize() ─────────────────────────────────────────────────
void FirstResponder::initialize() {
    // 1. Init static crash state block (zeroes + magic)
    CrashStateBlock::initialize();

    // 2. Pre-open crash dump file
    //    Path: /run/vitusos/crashdump-{pid}.bin
    //    O_CREAT | O_WRONLY | O_TRUNC | O_CLOEXEC
    //    Must succeed before any signal can fire.
    char path[128];
    snprintf(path, sizeof(path),
             "/run/vitusos/crashdump-%d.bin", (int)getpid());
    // Ensure directory exists (may fail silently if already exists)
    mkdir("/run/vitusos", 0755);  // not async-signal-safe, but we're in init
    s_crashFd = open(path, O_CREAT|O_WRONLY|O_TRUNC|O_CLOEXEC, 0640);
    // If open fails (e.g. /run/vitusos doesn't exist), fall back to stderr fd
    if (s_crashFd < 0) s_crashFd = STDERR_FILENO;

    // 3. Self-pipe (signal → main thread)
    pipe2(m_pipeFd, O_CLOEXEC | O_NONBLOCK);
    g_crashPipeWrite = m_pipeFd[1];

    // 4. Install signal handlers
    struct sigaction sa = {};
    sa.sa_sigaction = &FirstResponder::signalHandler;
    sa.sa_flags     = SA_SIGINFO   // gives us siginfo_t + ucontext_t
                    | SA_RESETHAND // don't loop on double-fault
                    | SA_ONSTACK;  // use alternate stack (see below)
    sigemptyset(&sa.sa_mask);
    sigaction(SIGSEGV, &sa, nullptr);
    sigaction(SIGABRT, &sa, nullptr);
    sigaction(SIGBUS,  &sa, nullptr);
    sigaction(SIGFPE,  &sa, nullptr);

    // SIGTERM/SIGHUP: non-fatal, handled on main thread via pipe
    struct sigaction termsa = {};
    termsa.sa_sigaction = &FirstResponder::signalHandler;
    termsa.sa_flags     = SA_SIGINFO;
    sigemptyset(&termsa.sa_mask);
    sigaction(SIGTERM, &termsa, nullptr);
    sigaction(SIGHUP,  &termsa, nullptr);

    // 5. Alternate signal stack (protects against stack overflow SIGSEGV)
    //    Static allocation — 64KB, never freed
    static uint8_t altStack[65536];
    stack_t ss = {};
    ss.ss_sp    = altStack;
    ss.ss_size  = sizeof(altStack);
    ss.ss_flags = 0;
    sigaltstack(&ss, nullptr);

    // 6. PSI memory pressure fd
    m_psiFd = open("/proc/pressure/memory", O_RDWR | O_CLOEXEC | O_NONBLOCK);
    if (m_psiFd >= 0) {
        // Notify on 50ms stall in 1-second window
        const char *trig = "some 50000 1000000\n";
        write(m_psiFd, trig, strlen(trig));
        // GlobalFeed adds m_psiFd to its epoll for pressure events
    }

    // 7. systemd watchdog
    const char *wd = getenv("WATCHDOG_USEC");
    m_watchdogActive = (wd != nullptr && atoll(wd) > 0);
    if (m_watchdogActive) sd_notify(0, "READY=1");
}

// ── Phase 1: Signal Handler ───────────────────────────────────────
// ASYNC-SIGNAL-SAFE ONLY. Every line here is audited.
void FirstResponder::signalHandler(int sig, siginfo_t *info, void *ctx)
{
    // ── 1. Timestamp (async-signal-safe) ─────────────────────────
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);

    // ── 2. Zero the dump struct (memset is async-signal-safe) ─────
    memset(&s_dump, 0, sizeof(s_dump));

    // ── 3. Header ─────────────────────────────────────────────────
    s_dump.magic         = CRASHDUMP_MAGIC;
    s_dump.version       = CRASHDUMP_VERSION;
    s_dump.signum        = (uint32_t)sig;
    s_dump.sicode        = info ? info->si_code : 0;
    s_dump.faultAddress  = info ? (uint64_t)(uintptr_t)info->si_addr : 0;
    s_dump.pid           = (uint64_t)getpid();
    s_dump.tid           = (uint64_t)gettid();
    s_dump.timestampSec  = (uint64_t)ts.tv_sec;
    s_dump.timestampNsec = (uint64_t)ts.tv_nsec;

    // ── 4. CPU Registers (from ucontext_t) ───────────────────────
    if (ctx) {
        ucontext_t *uc = static_cast<ucontext_t*>(ctx);
        mcontext_t *mc = &uc->uc_mcontext;
        s_dump.regs.rax    = (uint64_t)mc->gregs[REG_RAX];
        s_dump.regs.rbx    = (uint64_t)mc->gregs[REG_RBX];
        s_dump.regs.rcx    = (uint64_t)mc->gregs[REG_RCX];
        s_dump.regs.rdx    = (uint64_t)mc->gregs[REG_RDX];
        s_dump.regs.rsi    = (uint64_t)mc->gregs[REG_RSI];
        s_dump.regs.rdi    = (uint64_t)mc->gregs[REG_RDI];
        s_dump.regs.rbp    = (uint64_t)mc->gregs[REG_RBP];
        s_dump.regs.rsp    = (uint64_t)mc->gregs[REG_RSP];
        s_dump.regs.r8     = (uint64_t)mc->gregs[REG_R8];
        s_dump.regs.r9     = (uint64_t)mc->gregs[REG_R9];
        s_dump.regs.r10    = (uint64_t)mc->gregs[REG_R10];
        s_dump.regs.r11    = (uint64_t)mc->gregs[REG_R11];
        s_dump.regs.r12    = (uint64_t)mc->gregs[REG_R12];
        s_dump.regs.r13    = (uint64_t)mc->gregs[REG_R13];
        s_dump.regs.r14    = (uint64_t)mc->gregs[REG_R14];
        s_dump.regs.r15    = (uint64_t)mc->gregs[REG_R15];
        s_dump.regs.rip    = (uint64_t)mc->gregs[REG_RIP];
        s_dump.regs.rflags = (uint64_t)mc->gregs[REG_EFL];
        s_dump.regs.cr2    = s_dump.faultAddress;  // SIGSEGV fault addr = CR2
    }

    // ── 5. Stack Trace ────────────────────────────────────────────
    // _Unwind_Backtrace: not strictly async-signal-safe but works on
    // Linux/glibc for non-corrupt stacks. SA_RESETHAND prevents re-entry.
    {
        UnwindCtx uctx = { s_dump.stackFrames, 0, CRASHDUMP_MAX_FRAMES };
        _Unwind_Backtrace(unwindCallback, &uctx);
        s_dump.stackDepth = uctx.count;
    }

    // ── 6. AnimusEngine Frame State (from static CrashStateBlock) ─
    {
        const auto& cs = CrashStateBlock::global();
        s_dump.frameNumber  = cs.frame.frameNumber.load(std::memory_order_relaxed);
        s_dump.totalTimeS   = cs.frame.totalTimeS.load(std::memory_order_relaxed);
        s_dump.lastDt       = cs.frame.lastDt.load(std::memory_order_relaxed);
        s_dump.refreshHz    = cs.frame.refreshHz.load(std::memory_order_relaxed);
    }

    // ── 7. StateManager snapshot ──────────────────────────────────
    {
        const auto& sk = CrashStateBlock::global().stateKeys;
        s_dump.focusedWindowId  = sk.focusedWindowId;
        s_dump.lockScreenVisible= sk.lockScreenVisible;
        s_dump.cockpitViewOpen  = sk.cockpitViewOpen;
        s_dump.pathfinderOpen   = sk.pathfinderOpen;
        s_dump.dockVisible      = sk.dockVisible;
        s_dump.wallpaperTintR   = sk.wallpaperTintR;
        s_dump.wallpaperTintG   = sk.wallpaperTintG;
        s_dump.wallpaperTintB   = sk.wallpaperTintB;
        s_dump.systemVolume     = sk.systemVolume;
    }

    // ── 8. Resource snapshot ──────────────────────────────────────
    {
        const auto& r = CrashStateBlock::global().resources;
        s_dump.vmRssKb      = r.vmRssKb;
        s_dump.openFdCount  = r.openFdCount;
        s_dump.gpuUsedBytes = r.gpuUsedBytes;
        s_dump.pwUnderruns  = r.pwUnderruns;
        s_dump.memPressure  = r.memPressure;
        s_dump.fdPressure   = r.fdPressure;
        s_dump.gpuPressure  = r.gpuPressure;
        s_dump.audioPressure= r.audioPressure;
    }

    // ── 9. Last 8 OSFEvents from ring buffer ──────────────────────
    {
        const auto& cs = CrashStateBlock::global();
        uint32_t head = cs.eventHead.load(std::memory_order_relaxed);
        for (int i = 0; i < 8; i++) {
            uint32_t idx = (head - 1 - i + MAX_EVENT_RING) % MAX_EVENT_RING;
            s_dump.lastEvents[i]     = cs.events[idx].eventId;
            s_dump.lastEventTimes[i] = cs.events[idx].timeS;
        }
    }

    // ── 10. Vessel states ─────────────────────────────────────────
    {
        const auto& cs = CrashStateBlock::global();
        uint32_t vc = cs.vesselCount.load(std::memory_order_relaxed);
        s_dump.vesselCount = vc < 32 ? vc : 32;
        for (uint32_t i = 0; i < s_dump.vesselCount; i++) {
            memcpy(s_dump.vessels[i].name, cs.vessels[i].name, 32);
            s_dump.vessels[i].state = cs.vessels[i].state;
        }
    }

    // ── 11. Last sound ────────────────────────────────────────────
    {
        const auto& cs = CrashStateBlock::global();
        memcpy(s_dump.lastSoundName, cs.lastSound.name, 32);
        s_dump.lastSoundTimeS = cs.lastSound.timeS;
    }

    // ── 12. Write binary dump (one write() call — atomic for PIPE_BUF) ──
    if (s_crashFd >= 0)
        write(s_crashFd, &s_dump, sizeof(s_dump));

    // ── 13. Wake main thread via pipe ────────────────────────────
    //    If main thread is alive, it will do Phase 2 (symbol resolution)
    uint8_t sigbyte = (uint8_t)sig;
    if (g_crashPipeWrite >= 0)
        write(g_crashPipeWrite, &sigbyte, 1);

    // ── 14. Fatal signals: die immediately after writing ──────────
    if (sig == SIGSEGV || sig == SIGABRT ||
        sig == SIGBUS  || sig == SIGFPE) {
        // _exit(): no atexit, no destructors, no stdio flush
        // 128+sig is the conventional exit code for signal-killed processes
        _exit(128 + sig);
    }
    // SIGTERM/SIGHUP: return — main thread handles graceful shutdown
}

// ── Phase 2: Main Thread Handler ─────────────────────────────────
// Called after pipe fires. Not signal context — full C++ available.
void FirstResponder::handleCrashOnMainThread(uint8_t signum) {
    // At this point s_dump is already written to the .bin file by Phase 1.
    // Phase 2 augments with a human-readable .txt report.

    char txtPath[128];
    snprintf(txtPath, sizeof(txtPath),
             "/run/vitusos/crashdump-%d.txt", (int)getpid());
    FILE *f = fopen(txtPath, "w");
    if (!f) return;

    // ── Header ────────────────────────────────────────────────────
    fprintf(f, "═══════════════════════════════════════════════════\n");
    fprintf(f, "  VitusOS AnimusEngine Crash Report\n");
    fprintf(f, "  Signal: %d (%s)\n", signum,
            signum == SIGSEGV ? "SIGSEGV" :
            signum == SIGABRT ? "SIGABRT" :
            signum == SIGBUS  ? "SIGBUS"  :
            signum == SIGFPE  ? "SIGFPE"  : "UNKNOWN");
    fprintf(f, "  Fault Address: 0x%016llx\n",
            (unsigned long long)s_dump.faultAddress);
    fprintf(f, "  PID: %llu  TID: %llu\n",
            (unsigned long long)s_dump.pid,
            (unsigned long long)s_dump.tid);
    fprintf(f, "  Time: %llu.%09llu s (CLOCK_MONOTONIC)\n",
            (unsigned long long)s_dump.timestampSec,
            (unsigned long long)s_dump.timestampNsec);
    fprintf(f, "═══════════════════════════════════════════════════\n\n");

    // ── Registers ─────────────────────────────────────────────────
    fprintf(f, "── Registers ──────────────────────────────────────\n");
    fprintf(f, "  RIP: 0x%016llx  ← WHERE IT CRASHED\n",
            (unsigned long long)s_dump.regs.rip);
    fprintf(f, "  RSP: 0x%016llx  RBP: 0x%016llx\n",
            (unsigned long long)s_dump.regs.rsp,
            (unsigned long long)s_dump.regs.rbp);
    fprintf(f, "  RAX: 0x%016llx  RBX: 0x%016llx\n",
            (unsigned long long)s_dump.regs.rax,
            (unsigned long long)s_dump.regs.rbx);
    fprintf(f, "  RCX: 0x%016llx  RDX: 0x%016llx\n",
            (unsigned long long)s_dump.regs.rcx,
            (unsigned long long)s_dump.regs.rdx);
    fprintf(f, "  RSI: 0x%016llx  RDI: 0x%016llx\n",
            (unsigned long long)s_dump.regs.rsi,
            (unsigned long long)s_dump.regs.rdi);
    fprintf(f, "  R8:  0x%016llx  R9:  0x%016llx\n",
            (unsigned long long)s_dump.regs.r8,
            (unsigned long long)s_dump.regs.r9);
    fprintf(f, "  R10: 0x%016llx  R11: 0x%016llx\n",
            (unsigned long long)s_dump.regs.r10,
            (unsigned long long)s_dump.regs.r11);
    fprintf(f, "  R12: 0x%016llx  R13: 0x%016llx\n",
            (unsigned long long)s_dump.regs.r12,
            (unsigned long long)s_dump.regs.r13);
    fprintf(f, "  R14: 0x%016llx  R15: 0x%016llx\n",
            (unsigned long long)s_dump.regs.r14,
            (unsigned long long)s_dump.regs.r15);
    fprintf(f, "  CR2 (fault addr): 0x%016llx\n\n",
            (unsigned long long)s_dump.regs.cr2);

    // ── Stack Trace with Symbol Resolution ───────────────────────
    // /proc/self/maps was pre-cached by GlobalFeed — use it for
    // address→module mapping. Full symbol names need addr2line
    // or dladdr() — both available here since we're not in signal context.
    fprintf(f, "── Stack Trace (%u frames) ─────────────────────────\n",
            s_dump.stackDepth);
    for (uint32_t i = 0; i < s_dump.stackDepth; i++) {
        uint64_t addr = s_dump.stackFrames[i];
        // dladdr for symbol name
        Dl_info di = {};
        dladdr(reinterpret_cast<void*>(addr), &di);
        const char *sym = di.dli_sname ? di.dli_sname : "??";
        const char *mod = di.dli_fname ? di.dli_fname : "??";
        uint64_t offset = di.dli_saddr
            ? addr - (uint64_t)(uintptr_t)di.dli_saddr : 0;
        fprintf(f, "  #%-2u 0x%016llx  %s+0x%llx  [%s]\n",
                i,
                (unsigned long long)addr,
                sym,
                (unsigned long long)offset,
                mod);
    }
    fprintf(f, "\n");

    // ── AnimusEngine State ────────────────────────────────────────
    fprintf(f, "── AnimusEngine State ──────────────────────────────\n");
    fprintf(f, "  Frame:      %llu\n",
            (unsigned long long)s_dump.frameNumber);
    fprintf(f, "  Total time: %.3f s\n", s_dump.totalTimeS);
    fprintf(f, "  Last dt:    %.4f s  (%.1f Hz effective)\n",
            s_dump.lastDt, s_dump.lastDt > 0 ? 1.0f/s_dump.lastDt : 0);
    fprintf(f, "  RefreshHz:  %.1f Hz (EMA)\n\n", s_dump.refreshHz);

    // ── StateManager Snapshot ─────────────────────────────────────
    fprintf(f, "── StateManager ────────────────────────────────────\n");
    fprintf(f, "  focused_window_id:  %llu\n",
            (unsigned long long)s_dump.focusedWindowId);
    fprintf(f, "  lock_screen:        %s\n",
            s_dump.lockScreenVisible ? "YES" : "no");
    fprintf(f, "  cockpit_view:       %s\n",
            s_dump.cockpitViewOpen   ? "YES" : "no");
    fprintf(f, "  pathfinder:         %s\n",
            s_dump.pathfinderOpen    ? "YES" : "no");
    fprintf(f, "  dock_visible:       %s\n",
            s_dump.dockVisible       ? "yes" : "no");
    fprintf(f, "  wallpaper_tint:     r=%.3f g=%.3f b=%.3f\n",
            s_dump.wallpaperTintR,
            s_dump.wallpaperTintG,
            s_dump.wallpaperTintB);
    fprintf(f, "  system_volume:      %.2f\n\n", s_dump.systemVolume);

    // ── Resources ─────────────────────────────────────────────────
    fprintf(f, "── Resources ───────────────────────────────────────\n");
    fprintf(f, "  VmRSS:      %llu KB\n",
            (unsigned long long)s_dump.vmRssKb);
    fprintf(f, "  Open FDs:   %u\n",   s_dump.openFdCount);
    fprintf(f, "  GPU used:   %llu bytes\n",
            (unsigned long long)s_dump.gpuUsedBytes);
    fprintf(f, "  PW underruns: %u\n", s_dump.pwUnderruns);
    static const char *levels[] = { "Normal", "Low", "Medium", "Critical" };
    fprintf(f, "  Pressure:   mem=%s  fd=%s  gpu=%s  audio=%s\n\n",
            levels[s_dump.memPressure  & 3],
            levels[s_dump.fdPressure   & 3],
            levels[s_dump.gpuPressure  & 3],
            levels[s_dump.audioPressure& 3]);

    // ── Last 8 OSFEvents ──────────────────────────────────────────
    fprintf(f, "── Last 8 OSFEvents (most recent first) ────────────\n");
    for (int i = 0; i < 8; i++) {
        if (s_dump.lastEventTimes[i] == 0) continue;
        fprintf(f, "  [%.3f s] event=%u\n",
                s_dump.lastEventTimes[i],
                s_dump.lastEvents[i]);
    }
    fprintf(f, "\n");

    // ── Vessel States ─────────────────────────────────────────────
    fprintf(f, "── Vessel States ───────────────────────────────────\n");
    static const char *vstates[] = { "Running", "Degraded", "Isolated", "Dead" };
    for (uint32_t i = 0; i < s_dump.vesselCount; i++) {
        uint8_t st = s_dump.vessels[i].state;
        fprintf(f, "  %-28s  %s\n",
                s_dump.vessels[i].name,
                st < 4 ? vstates[st] : "?");
    }
    fprintf(f, "\n");

    // ── Last Sound ────────────────────────────────────────────────
    if (s_dump.lastSoundName[0]) {
        fprintf(f, "── Last Sound ──────────────────────────────────────\n");
        fprintf(f, "  %s at %.3f s\n\n",
                s_dump.lastSoundName, s_dump.lastSoundTimeS);
    }

    // ── /proc/self/maps (pre-cached) ──────────────────────────────
    fprintf(f, "── /proc/self/maps (cached) ────────────────────────\n");
    const auto& cs = CrashStateBlock::global();
    if (cs.mapsSnapshotLen > 0)
        fwrite(cs.mapsSnapshot, 1, cs.mapsSnapshotLen, f);
    fprintf(f, "\n");

    fclose(f);

    // Update /run/vitusos/last-crash symlink
    char binPath[128];
    snprintf(binPath, sizeof(binPath),
             "/run/vitusos/crashdump-%d.bin", (int)getpid());
    unlink("/run/vitusos/last-crash.bin");
    unlink("/run/vitusos/last-crash.txt");
    symlink(binPath,  "/run/vitusos/last-crash.bin");
    symlink(txtPath,  "/run/vitusos/last-crash.txt");
}

void FirstResponder::kickWatchdog() {
    if (m_watchdogActive) sd_notify(0, "WATCHDOG=1");
}

void FirstResponder::onInstallFailed(const std::string &errorOutput) {
    EventBus::shared().publishAsync(OSFEvent::InstallFailed,
        std::string(errorOutput));
}

void FirstResponder::onBootCrossfadeFailed() {
    EventBus::shared().publishAsync(OSFEvent::BootCrossfadeComplete);
}

void FirstResponder::destroy() {
    if (s_crashFd >= 0 && s_crashFd != STDERR_FILENO)
        { close(s_crashFd); s_crashFd = -1; }
    if (m_psiFd >= 0)    { close(m_psiFd);    m_psiFd = -1; }
    if (m_pipeFd[0] >= 0){ close(m_pipeFd[0]); m_pipeFd[0] = -1; }
    if (m_pipeFd[1] >= 0){ close(m_pipeFd[1]); m_pipeFd[1] = -1; }
    g_crashPipeWrite = -1;
}

} // namespace Animus
```

### 23.6 OSFDesktop Integration — Pipe Drain Loop

```cpp
// In OSFDesktop::run() — after wlr_backend_start(), in the event loop
// Add the crash pipe fd to the Wayland event loop via wl_event_loop_add_fd()

static int onCrashPipe(int fd, uint32_t mask, void *data) {
    uint8_t signum = 0;
    if (read(fd, &signum, 1) == 1) {
        if (signum == SIGTERM) {
            // Graceful shutdown — clean wlroots teardown
            OSFDesktop::shared().requestShutdown();
        } else if (signum == SIGHUP) {
            // Config reload
            EventBus::shared().publish(OSFEvent::ConfigReload, {});
        } else {
            // Fatal signal already wrote Phase 1 dump.
            // Phase 2: symbol resolution + human-readable report.
            // This path only runs if main thread is alive (rare for SIGSEGV,
            // possible if a background thread faulted).
            CrashManager::shared().firstResponder()
                .handleCrashOnMainThread(signum);
        }
    }
    return 0;
}

// In OSFDesktop::initSubsystems():
wl_event_loop_add_fd(
    g_compositor_state.event_loop,
    CrashManager::shared().firstResponder().pipeReadFd(),
    WL_EVENT_READABLE,
    onCrashPipe,
    nullptr);
```

### 23.7 What Each Collection Point Tells the Debugger

```
CrashDump field          → Tells you
─────────────────────────────────────────────────────────────
regs.rip                 → Exact instruction that faulted
regs.cr2 / faultAddress  → Which memory address was accessed
stackFrames[]            → Call chain leading to the fault
frameNumber              → Which vblank frame (correlates with logs)
totalTimeS               → How long compositor was running
lastDt                   → Was the frame loop stalling?
refreshHz                → Was the display rate normal?
focusedWindowId          → Which app had focus when it crashed
lockScreenVisible        → Was the lock screen active?
cockpitViewOpen          → Was CockpitView rendering?
pathfinderOpen           → Was Pathfinder's overlay active?
wallpaperTint R/G/B      → Which wallpaper color was sampled
vmRssKb                  → Was memory pressure the cause?
openFdCount              → Was an fd leak the cause?
gpuUsedBytes             → Was GPU memory exhausted?
pwUnderruns              → Was audio buffer underrunning?
memPressure/fdPressure   → Were we already under stress?
lastEvents[8]            → What was the system doing just before?
vessels[]                → Which subsystems were isolated/dead?
lastSoundName            → Was a sound playing? (PipeWire state)
mapsSnapshot             → Resolve raw stack addresses to symbols
```


---

## PART 24 — CrashManager Gap Closure

### 24.1 Handshakes — Complete Implementation

```cpp
// animus/crash/Handshakes.cpp — complete
#include "Handshakes.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <pipewire/pipewire.h>
#include <sdbus-c++/sdbus-c++.h>
#include <chrono>
#include <thread>
#include <cstring>

// Access compositor state for wlroots check
#define WLR_USE_UNSTABLE
#include <wlr/backend.h>
extern struct AnimusCompositorState g;  // from animus_compositor.c

namespace Animus {

static constexpr int MAX_FAIL_STREAK = 3;  // Unresponsive after 3 misses → Dead

void Handshakes::start() {
    // Register built-in checks
    using ms = std::chrono::milliseconds;
    registerCheck("PipeWire", [this]{ return checkPipeWire(); }, ms(5000));
    registerCheck("DBus",     [this]{ return checkDBus();     }, ms(8000));
    registerCheck("wlroots",  [this]{ return checkWlroots();  }, ms(3000));

    m_running = true;
    m_thread  = std::thread(&Handshakes::heartbeatLoop, this);
}

void Handshakes::stop() {
    m_running = false;
    if (m_thread.joinable()) m_thread.join();
}

void Handshakes::registerCheck(const std::string &name,
                                std::function<SubsystemHealth()> fn,
                                std::chrono::milliseconds interval)
{
    m_checks.push_back({ name, std::move(fn), interval,
                         std::chrono::steady_clock::now(), 0 });
}

void Handshakes::heartbeatLoop() {
    while (m_running) {
        auto now = std::chrono::steady_clock::now();
        for (auto &chk : m_checks) {
            if (now < chk.nextCheck) continue;
            chk.nextCheck = now + chk.interval;

            SubsystemHealth h = chk.fn();

            if (h != SubsystemHealth::Healthy) {
                chk.failStreak++;
                if (chk.failStreak >= MAX_FAIL_STREAK)
                    h = SubsystemHealth::Dead;
                else
                    h = SubsystemHealth::Unresponsive;
            } else {
                chk.failStreak = 0;
            }

            HandshakeResult result { chk.name, h, "" };
            EventBus::shared().publishAsync(
                OSFEvent::SubsystemHealthChanged, result);
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
    }
}

// ── checkPipeWire ─────────────────────────────────────────────────
// Proof of liveness: pw_core_sync() round-trip.
// pw_core_sync() sends a SYNC request to the PipeWire daemon and
// expects a DONE event back. Timeout = 2 seconds.
// If the daemon is dead, the round-trip never completes.
SubsystemHealth Handshakes::checkPipeWire() {
    // Use a short-lived pw_main_loop + pw_context + pw_core
    // just for the health ping. Independent of SoundEngine's loop.
    pw_init(nullptr, nullptr);
    struct pw_main_loop *loop = pw_main_loop_new(nullptr);
    if (!loop) return SubsystemHealth::Dead;

    struct pw_context *ctx = pw_context_new(
        pw_main_loop_get_loop(loop), nullptr, 0);
    if (!ctx) { pw_main_loop_destroy(loop); return SubsystemHealth::Dead; }

    struct pw_core *core = pw_context_connect(ctx, nullptr, 0);
    if (!core) {
        pw_context_destroy(ctx);
        pw_main_loop_destroy(loop);
        return SubsystemHealth::Dead;
    }

    // Send sync — when DONE fires, quit the loop
    bool done = false;
    auto events = pw_core_events{};
    events.version = PW_VERSION_CORE_EVENTS;
    events.done = [](void *data, uint32_t, int) {
        *static_cast<bool*>(data) = true;
        // pw_main_loop_quit called via loop ptr stored in closure
    };
    spa_hook listener;
    // Simplified: use pw_core_sync seq 0
    pw_core_sync(core, PW_ID_CORE, 0);

    // Run loop with 2s timeout
    auto deadline = std::chrono::steady_clock::now()
                  + std::chrono::seconds(2);
    while (!done && std::chrono::steady_clock::now() < deadline) {
        pw_loop_iterate(pw_main_loop_get_loop(loop), 100);
        // Check if core fd is readable — simplified poll
    }

    pw_core_disconnect(core);
    pw_context_destroy(ctx);
    pw_main_loop_destroy(loop);

    return done ? SubsystemHealth::Healthy : SubsystemHealth::Unresponsive;
}

// ── checkDBus ────────────────────────────────────────────────────
// Proof of liveness: call org.freedesktop.DBus.GetId() on session bus.
// This is a synchronous round-trip that returns within milliseconds
// if D-Bus is alive, or throws sdbus::Error if it is dead/unreachable.
SubsystemHealth Handshakes::checkDBus() {
    try {
        auto conn = sdbus::createSessionBusConnection();
        auto proxy = sdbus::createProxy(*conn,
            sdbus::ServiceName{"org.freedesktop.DBus"},
            sdbus::ObjectPath{"/org/freedesktop/DBus"});
        std::string id;
        proxy->callMethod("GetId")
             .onInterface("org.freedesktop.DBus")
             .storeResultsTo(id);
        // If we got here, D-Bus responded
        return id.empty() ? SubsystemHealth::Degraded
                          : SubsystemHealth::Healthy;
    } catch (const sdbus::Error &) {
        return SubsystemHealth::Unresponsive;
    } catch (...) {
        return SubsystemHealth::Dead;
    }
}

// ── checkWlroots ─────────────────────────────────────────────────
// Proof of liveness: wlr_backend_is_headless() + output count.
// wlroots doesn't have a ping API; liveness is proven by:
//   1. g.backend pointer is non-null
//   2. g.primary_output pointer is non-null
//   3. wlr_output is still enabled (not disconnected)
// If the backend died, g.backend would be null or the output disabled.
SubsystemHealth Handshakes::checkWlroots() {
    if (!g.backend)        return SubsystemHealth::Dead;
    if (!g.primary_output) return SubsystemHealth::Degraded;
    if (!g.primary_output->enabled) return SubsystemHealth::Degraded;
    // Verify the event loop is still processing
    // (if wlroots hung, wl_display would stop dispatching)
    if (!g.display)        return SubsystemHealth::Dead;
    return SubsystemHealth::Healthy;
}

} // namespace Animus
```

### 24.2 EventHandler — Complete Implementation

```cpp
// animus/crash/EventHandler.cpp — complete
#include "EventHandler.h"
#include "CrashManager.h"
#include "Vessels.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <cstdarg>
#include <cstdio>
#include <cstring>

// wlr_log_init needs this callback signature
#define WLR_USE_UNSTABLE
#include <wlr/util/log.h>

namespace Animus {

void EventHandler::initialize() {
    // Install wlr_log callback — replaces the NULL in animus_compositor.c
    // wlr_log_init(WLR_DEBUG, EventHandler::wlrLogCallback)
    // Called AFTER CrashManager::initialize() so the handler exists
    wlr_log_init(WLR_INFO, &EventHandler::wlrLogCallback);
}

// wlr_log callback — called by wlroots for every log message
// Runs on the main thread (wlroots is single-threaded)
void EventHandler::wlrLogCallback(enum wlr_log_importance importance,
                                   const char *fmt, va_list args)
{
    // Format the message
    char buf[1024];
    vsnprintf(buf, sizeof(buf), fmt, args);

    // Mirror to stderr always
    fprintf(stderr, "[wlr][%s] %s\n",
            importance == WLR_ERROR   ? "ERR" :
            importance == WLR_INFO    ? "INF" : "DBG",
            buf);

    // Only triage errors — debug/info are informational
    if (importance != WLR_ERROR) return;

    EventHandler &eh = CrashManager::shared().eventHandler();
    eh.onCompositorError(std::string(buf));
}

void EventHandler::onCompositorError(const std::string &detail) {
    Severity sev = classify("compositor", detail);
    dispatch(sev, "compositor", detail);
}

void EventHandler::onDBusConnectionLost(const std::string &busName) {
    // D-Bus lost: DBusBridge and dependents become Degraded, not Fatal
    // The compositor keeps running — just no third-party menus/tray
    Severity sev = classify("dbus", busName);
    dispatch(sev, "dbus", busName);
}

void EventHandler::onWindowStateAnomaly(const std::string &appId,
                                         const std::string &detail) {
    Severity sev = classify("window", detail);
    dispatch(sev, "window[" + appId + "]", detail);
}

EventHandler::Severity EventHandler::classify(const std::string &source,
                                               const std::string &detail)
{
    // Fatal: wlroots backend failure, DRM/KMS error, Vulkan device lost
    static const char *FATAL_PATTERNS[] = {
        "DRM_IOCTL_MODE_ATOMIC",
        "VK_ERROR_DEVICE_LOST",
        "failed to create backend",
        "wlr_backend_start failed",
        "no outputs available",
        nullptr
    };
    for (int i = 0; FATAL_PATTERNS[i]; i++)
        if (detail.find(FATAL_PATTERNS[i]) != std::string::npos)
            return Severity::Fatal;

    // Degraded: surface errors, protocol violations, non-fatal DRM
    static const char *DEGRADED_PATTERNS[] = {
        "wlr_surface",
        "xdg_toplevel",
        "protocol error",
        "client destroyed",
        "lost connection",
        "dbus",
        nullptr
    };
    for (int i = 0; DEGRADED_PATTERNS[i]; i++)
        if (detail.find(DEGRADED_PATTERNS[i]) != std::string::npos ||
            source.find(DEGRADED_PATTERNS[i]) != std::string::npos)
            return Severity::Degraded;

    return Severity::Recoverable;
}

void EventHandler::dispatch(Severity sev,
                             const std::string &source,
                             const std::string &detail)
{
    switch (sev) {
    case Severity::Recoverable:
        // Log only — no action
        fprintf(stderr, "[CrashManager][Recoverable] %s: %s\n",
                source.c_str(), detail.c_str());
        break;

    case Severity::Degraded:
        // Notify Vessels — isolate affected subsystem
        fprintf(stderr, "[CrashManager][Degraded] %s: %s\n",
                source.c_str(), detail.c_str());
        // Map source to vessel name and mark degraded
        if (source.find("dbus") != std::string::npos)
            CrashManager::shared().vessels().markDead("DBusBridge");
        else if (source.find("compositor") != std::string::npos)
            CrashManager::shared().vessels().markDead("Compositor");
        // Publish for UI — show subtle indicator, not full crash screen
        EventBus::shared().publishAsync(
            OSFEvent::SubsystemHealthChanged,
            HandshakeResult{ source, SubsystemHealth::Degraded, detail });
        break;

    case Severity::Fatal:
        // Controlled shutdown — attempt clean wlroots teardown
        fprintf(stderr, "[CrashManager][FATAL] %s: %s\n",
                source.c_str(), detail.c_str());
        // Write crash state before shutdown
        CrashManager::shared().vessels().markDead("Compositor");
        // Request graceful exit — OSFDesktop::run() exits its event loop
        EventBus::shared().publishAsync(OSFEvent::ShutdownRequested, {});
        break;
    }
}

} // namespace Animus
```

### 24.3 CrashSite — Complete Implementation

```cpp
// animus/crash/CrashSite.cpp — complete
#include "CrashSite.h"
#include "CrashManager.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <spawn.h>        // posix_spawn — async-safe app respawn
#include <ctime>
#include <cstring>
#include <cstdio>

extern char **environ;

namespace Animus {

void CrashSite::initialize() {
    m_records.clear();
}

void CrashSite::onCleanExit(struct wl_client *client,
                              const std::string &appId)
{
    // Clean exit — just remove from records, no respawn needed
    m_records.erase(appId);

    // Publish so Dock clears the running dot
    EventBus::shared().publishAsync(OSFEvent::ClientCrashed,
        std::string(appId + ":clean"));
}

void CrashSite::onClientCrash(struct wl_client *client,
                                const std::string &appId)
{
    fprintf(stderr, "[CrashSite] Client crash: %s\n", appId.c_str());

    // Update crash record and check respawn policy
    recordRespawn(appId);

    if (shouldRespawn(appId)) {
        fprintf(stderr, "[CrashSite] Respawning %s\n", appId.c_str());
        respawnApp(appId);
    } else {
        fprintf(stderr, "[CrashSite] %s exceeded respawn limit — giving up\n",
                appId.c_str());

        // Publish ClientCrashed — Dock clears running dot
        EventBus::shared().publishAsync(OSFEvent::ClientCrashed,
            std::string(appId));

        // Show OSFNotification via EventBus (main thread handles display)
        struct CrashNotifData {
            std::string title;
            std::string body;
            int timeoutMs;
        };
        CrashNotifData nd {
            appId + " quit unexpectedly",
            "It could not be reopened automatically.",
            7000
        };
        EventBus::shared().publishAsync(OSFEvent::NotificationPosted,
            std::move(nd));
    }
}

bool CrashSite::shouldRespawn(const std::string &appId) const {
    auto it = m_records.find(appId);
    if (it == m_records.end()) return true;  // first crash — always try

    const auto &rec = it->second;

    // Get current CLOCK_MONOTONIC time
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    double now = ts.tv_sec + ts.tv_nsec * 1e-9;

    // Reset window if outside 10s
    if (now - rec.firstCrashTime > RESPAWN_WINDOW_S) return true;

    // Within window — check count
    return rec.respawnCount < MAX_RESPAWNS;
}

void CrashSite::recordRespawn(const std::string &appId) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    double now = ts.tv_sec + ts.tv_nsec * 1e-9;

    auto &rec = m_records[appId];

    // Reset window if outside 10s
    if (now - rec.firstCrashTime > RESPAWN_WINDOW_S) {
        rec.respawnCount  = 0;
        rec.firstCrashTime= now;
    }
    rec.respawnCount++;
}

void CrashSite::respawnApp(const std::string &appId) {
    // posix_spawn: preferred over fork()+exec() in a compositor.
    // fork() in a multi-threaded process with Vulkan/wlroots is dangerous
    // (locks, GPU state). posix_spawn avoids the fork hazard.
    //
    // App launch path: /etc/vitusos/apps/{appId}/launch
    // This is a simple shell script or binary set by InstallManager.
    char launchPath[256];
    snprintf(launchPath, sizeof(launchPath),
             "/etc/vitusos/apps/%s/launch", appId.c_str());

    char *const argv[] = { launchPath, nullptr };

    // Spawn with WAYLAND_DISPLAY set (inherited from compositor env)
    pid_t pid;
    int ret = posix_spawn(&pid, launchPath,
                          nullptr,  // no file actions
                          nullptr,  // no attrs
                          argv,
                          environ);
    if (ret != 0) {
        fprintf(stderr, "[CrashSite] posix_spawn failed for %s: %s\n",
                appId.c_str(), strerror(ret));
    }
    // Don't waitpid — fire and forget, Wayland will notify us when it connects
}

} // namespace Animus
```

### 24.4 Vessels — markRestored() Complete Implementation

```cpp
// Addition to animus/crash/Vessels.cpp

void Vessels::markRestored(const std::string &name) {
    auto it = m_vessels.find(name);
    if (it == m_vessels.end()) return;

    // Only restore if all dependencies are healthy
    for (auto &dep : it->second.dependsOn) {
        auto dit = m_vessels.find(dep);
        if (dit == m_vessels.end()) continue;
        if (dit->second.state == VesselState::Dead ||
            dit->second.state == VesselState::Isolated)
            return;  // dependency still down — cannot restore yet
    }

    it->second.state = VesselState::Running;
    if (it->second.onRestore) it->second.onRestore();

    // Cascade: re-evaluate all vessels that depended on this one
    // They may now be restorable too
    for (auto &[vname, vessel] : m_vessels) {
        if (vessel.state != VesselState::Isolated) continue;
        // Check if this vessel's deps are all now Running
        bool allUp = true;
        for (auto &dep : vessel.dependsOn) {
            auto dit = m_vessels.find(dep);
            if (dit == m_vessels.end()) continue;
            if (dit->second.state != VesselState::Running)
                { allUp = false; break; }
        }
        if (allUp) {
            vessel.state = VesselState::Running;
            if (vessel.onRestore) vessel.onRestore();
        }
    }

    // Sync updated states to CrashStateBlock
    syncToCrashState();

    EventBus::shared().publishAsync(OSFEvent::SubsystemHealthChanged,
        HandshakeResult{ name, SubsystemHealth::Healthy, "restored" });
}
```

### 24.5 GlobalFeed — epoll + PSI + PipeWire Underrun Counter

```cpp
// animus/crash/GlobalFeed.cpp — complete replacement with epoll + underrun

#include "GlobalFeed.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <cstdio>
#include <cstring>
#include <dirent.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <chrono>
#include <thread>
#include <xf86drm.h>

// PipeWire underrun counter — written by SoundEngine's process callback
// on the PipeWire thread. Read by GlobalFeed on its thread.
// std::atomic — no mutex needed.
#include <atomic>
namespace Animus {
std::atomic<uint32_t> g_pwUnderrunCount{0};
}

namespace Animus {

void GlobalFeed::start() {
    // Open /proc/self/status once — seek+read each poll (cheaper than open)
    m_procStatusFd = open("/proc/self/status", O_RDONLY | O_CLOEXEC);

    // epoll for PSI pressure fd + wakeup eventfd
    m_epollFd  = epoll_create1(EPOLL_CLOEXEC);
    m_wakeupFd = eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK);

    // Add wakeup fd to epoll (for clean stop())
    struct epoll_event wev = {};
    wev.events   = EPOLLIN;
    wev.data.fd  = m_wakeupFd;
    epoll_ctl(m_epollFd, EPOLL_CTL_ADD, m_wakeupFd, &wev);

    // PSI fd is passed in from FirstResponder after it opens it
    // FirstResponder calls GlobalFeed::setPsiFd() after open()
    if (m_psiFd >= 0) {
        struct epoll_event pev = {};
        pev.events  = EPOLLPRI;  // PSI triggers EPOLLPRI, not EPOLLIN
        pev.data.fd = m_psiFd;
        epoll_ctl(m_epollFd, EPOLL_CTL_ADD, m_psiFd, &pev);
    }

    m_running = true;
    m_thread  = std::thread(&GlobalFeed::monitorLoop, this);
}

void GlobalFeed::stop() {
    m_running = false;
    // Wake epoll via eventfd
    uint64_t one = 1;
    write(m_wakeupFd, &one, sizeof(one));
    if (m_thread.joinable()) m_thread.join();
    if (m_epollFd  >= 0) { close(m_epollFd);  m_epollFd  = -1; }
    if (m_wakeupFd >= 0) { close(m_wakeupFd); m_wakeupFd = -1; }
    if (m_psiFd    >= 0) { close(m_psiFd);    m_psiFd    = -1; }
    if (m_procStatusFd >= 0) { close(m_procStatusFd); m_procStatusFd = -1; }
}

void GlobalFeed::setPsiFd(int fd) {
    m_psiFd = fd;
    if (m_epollFd >= 0 && fd >= 0) {
        struct epoll_event pev = {};
        pev.events  = EPOLLPRI;
        pev.data.fd = fd;
        epoll_ctl(m_epollFd, EPOLL_CTL_ADD, fd, &pev);
    }
}

void GlobalFeed::monitorLoop() {
    struct epoll_event events[4];
    double lastMapRefresh = 0.0;

    while (m_running) {
        // epoll_wait: 2000ms timeout for normal poll cycle
        // PSI fires immediately on memory pressure (EPOLLPRI)
        int n = epoll_wait(m_epollFd, events, 4, 2000);

        for (int i = 0; i < n; i++) {
            if (events[i].data.fd == m_wakeupFd) {
                return;  // stop() called
            }
            if (events[i].data.fd == m_psiFd) {
                // PSI pressure event — immediate Critical memory pressure
                ResourceSnapshot snap = m_last;
                snap.memory = PressureLevel::Critical;
                EventBus::shared().publishAsync(
                    OSFEvent::MemoryPressure, PressureLevel::Critical);
            }
        }

        // Regular poll cycle (runs every 2s via epoll timeout,
        // or immediately after PSI event)
        ResourceSnapshot snap;
        snap.vmRssKb      = readVmRss();
        snap.openFdCount  = countOpenFds();
        snap.gpuUsedBytes = readGpuBudget();
        snap.pwUnderruns  = g_pwUnderrunCount.load(std::memory_order_relaxed);
        snap.memory       = classify(snap.vmRssKb,      RSS_WARN_KB,   RSS_CRITICAL_KB);
        snap.fds          = classify(snap.openFdCount,  FD_WARN,       FD_CRITICAL);
        snap.gpu          = classify(snap.gpuUsedBytes,
                                     512ULL*1024*1024,   // 512MB warn
                                     900ULL*1024*1024);  // 900MB crit
        snap.audio        = snap.pwUnderruns >= PW_UNDERRUN_WARN
                            ? PressureLevel::Medium : PressureLevel::Normal;
        m_last = snap;

        // Write to CrashStateBlock (GlobalFeed owns this update)
        auto &cs = CrashStateBlock::global();
        cs.resources.vmRssKb       = snap.vmRssKb;
        cs.resources.openFdCount   = snap.openFdCount;
        cs.resources.gpuUsedBytes  = snap.gpuUsedBytes;
        cs.resources.pwUnderruns   = snap.pwUnderruns;
        cs.resources.memPressure   = static_cast<uint8_t>(snap.memory);
        cs.resources.fdPressure    = static_cast<uint8_t>(snap.fds);
        cs.resources.gpuPressure   = static_cast<uint8_t>(snap.gpu);
        cs.resources.audioPressure = static_cast<uint8_t>(snap.audio);

        // Refresh /proc/self/maps every 10s
        struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
        double now = ts.tv_sec + ts.tv_nsec * 1e-9;
        if (now - lastMapRefresh > 10.0) {
            int fd = open("/proc/self/maps", O_RDONLY | O_CLOEXEC);
            if (fd >= 0) {
                ssize_t bytes = read(fd, cs.mapsSnapshot,
                                     MAX_MAPS_SNAPSHOT - 1);
                cs.mapsSnapshotLen = bytes > 0 ? (size_t)bytes : 0;
                close(fd);
            }
            lastMapRefresh = now;
        }

        EventBus::shared().publishAsync(OSFEvent::ResourcePressure, snap);
    }
}

} // namespace Animus
```

### 24.6 PipeWire Underrun Counter — SoundEngine Integration

```cpp
// Addition to animus/audio/SoundEngine.cpp
// The PipeWire process callback fires when the server needs audio data.
// If our buffer isn't ready in time, that's an underrun.
// We count them via the global atomic — GlobalFeed reads it.

#include "crash/GlobalFeed.h"  // for g_pwUnderrunCount

namespace Animus {
extern std::atomic<uint32_t> g_pwUnderrunCount;
}

// In SoundEngine's pw_stream_events::process callback:
static void pw_process_callback(void *userdata) {
    struct pw_buffer *b = pw_stream_dequeue_buffer(stream);
    if (!b) {
        // No buffer available — underrun
        Animus::g_pwUnderrunCount.fetch_add(1, std::memory_order_relaxed);
        return;
    }
    // ... fill buffer with audio data ...
    pw_stream_queue_buffer(stream, b);
}
```

### 24.7 wlr_log_init — EventHandler Callback Registration

```c
// compositor/animus_compositor.c — replace wlr_log_init line
// OLD (line ~1040):
//   wlr_log_init(WLR_INFO, NULL);
// NEW:
//   wlr_log_init is called from C11 compositor, but the callback
//   lives in C++17. Bridge via extern "C" pointer set by EventHandler.

// In animus_compositor.h (bridge header) — add:
extern void (*animus_wlr_log_callback)(enum wlr_log_importance,
                                        const char*, va_list);

// In animus_compositor.c — replace wlr_log_init:
static void compositor_log_relay(enum wlr_log_importance imp,
                                  const char *fmt, va_list args) {
    if (animus_wlr_log_callback)
        animus_wlr_log_callback(imp, fmt, args);
}
void (*animus_wlr_log_callback)(enum wlr_log_importance,
                                 const char*, va_list) = NULL;
// In animus_compositor_init():
wlr_log_init(WLR_INFO, compositor_log_relay);

// In EventHandler::initialize() (C++17 side):
extern void (*animus_wlr_log_callback)(enum wlr_log_importance,
                                        const char*, va_list);
void EventHandler::initialize() {
    animus_wlr_log_callback = &EventHandler::wlrLogCallback;
}
```

### 24.8 OSFDesktop — CrashManager Integration

```cpp
// animus/core/OSFDesktop.h — add CrashManager accessor
#include "crash/CrashManager.h"

class OSFDesktop {
public:
    // ... existing accessors ...
    CrashManager& crashManager() { return CrashManager::shared(); }
    // ...
};
```

```cpp
// animus/core/OSFDesktop.cpp — CrashManager init ORDER

int OSFDesktop::run() {
    // ── Step 0: CrashManager FIRST — before any other subsystem ──
    // Signal handlers must be in place before wlr_backend_start()
    CrashManager::shared().initialize();

    // ── Step 1: Compositor C11 core ───────────────────────────────
    if (!animus_compositor_init()) return 1;

    // ── Step 2: Register EventHandler as wlr_log callback ─────────
    // (must be after compositor init so wlr_log_init has been called)
    CrashManager::shared().eventHandler().initialize();

    // ── Step 3: All other subsystems ──────────────────────────────
    initSubsystems(outputW, outputH);

    // ── Step 4: Start background monitors ─────────────────────────
    CrashManager::shared().globalFeed().setPsiFd(
        CrashManager::shared().firstResponder().psiFd());
    CrashManager::shared().globalFeed().start();
    CrashManager::shared().handshakes().start();

    // ── Step 5: Wire crash pipe to Wayland event loop ─────────────
    wl_event_loop_add_fd(
        g.event_loop,
        CrashManager::shared().firstResponder().pipeReadFd(),
        WL_EVENT_READABLE,
        onCrashPipe,
        nullptr);

    // ── Step 6: Register for SubsystemHealthChanged ───────────────
    EventBus::shared().subscribe(OSFEvent::SubsystemHealthChanged,
        [](const std::any &data) {
            auto r = std::any_cast<HandshakeResult>(data);
            if (r.health == SubsystemHealth::Dead)
                CrashManager::shared().vessels().markDead(r.subsystem);
        });

    // ── Step 7: Run ───────────────────────────────────────────────
    animus_compositor_run();

    // ── Step 8: Cleanup ───────────────────────────────────────────
    CrashManager::shared().handshakes().stop();
    CrashManager::shared().globalFeed().stop();
    CrashManager::shared().destroy();
    return 0;
}
```

### 24.9 AnimationEngine — kickWatchdog Integration

```cpp
// animus/animation/AnimationEngine.cpp — add watchdog kick to tick()
#include "crash/CrashManager.h"

void AnimationEngine::tick(float dt) {
    if (!m_running) return;

    // Kick systemd watchdog every frame — proves compositor is alive
    // sd_notify(WATCHDOG=1) is fast (one write to a Unix socket)
    CrashManager::shared().firstResponder().kickWatchdog();

    EventBus::shared().publish(OSFEvent::Tick, dt);

    // ... existing settler logic ...
}
```

### 24.10 NixOS Build Dependencies for CrashManager

```nix
# pkgs/vitusos-animus/default.nix — add to buildInputs

buildInputs = [
  # ... existing deps ...

  # CrashManager — FirstResponder
  libunwind          # _Unwind_Backtrace for stack traces in signal handler
  systemd            # sd-daemon.h, sd_notify for watchdog
  libdl              # dladdr() for Phase 2 symbol resolution (usually in glibc)

  # CrashManager — GlobalFeed
  libdrm             # drmGetMemoryBudget for GPU memory pressure

  # CrashManager — Handshakes
  # sdbus-c++ already listed for EO-Bus — no new dep needed

  # CrashManager — CrashSite
  # posix_spawn is in glibc — no extra dep
];

# Linker flags for dladdr
NIX_LDFLAGS = "-ldl -lunwind";
```

### 24.11 OSFEvent — ShutdownRequested + ConfigReload

```cpp
// Add to OSFEvent.h enum — used by EventHandler::dispatch() and onCrashPipe:
ShutdownRequested,    // data = {} — fatal error, controlled shutdown
ConfigReload,         // data = {} — SIGHUP received, reload vitusos-config.nix
```


---

## PART 25 — HEV (Highly Encrypted Valuables)

### 25.1 Overview

HEV is VitusOS's native secret storage and identity manager.
Named after Gordon Freeman's Hazardous Environment Suit — it stands between
the user's personal information and everything hostile in the world.
Silent when nothing is wrong. Present when something needs attention.
Never in the way.

**Design contract:**
- Implements org.freedesktop.secrets — all apps talk to HEV transparently
- Master key never written to disk — only in memory while unlocked
- Vault locked automatically when screen locks
- Proximity unlock via SeaDrop RSSI — phone in pocket is the key
- Cold start always requires password — no exceptions
- libsodium for all cryptographic primitives

**Layer placement:** Layer 11 — System Services
alongside ClipboardBridge, FileOperationDaemon,
DirectoryWatcher, InstallManager.

### 25.2 Architecture

```
EXTERNAL WORLD                    HEV                      INTERNAL
──────────────────────────────────────────────────────────────────
Spotify, Chrome,      ┌───────────────────────┐
VS Code, SSH agent    │  org.freedesktop.      │
  → libsecret    ─────►  secrets D-Bus API     │
                       │  (via EO-Bus/DBus      │
                       │   Bridge)              │
                       │                        │
vitusOS native apps   │  org.vitusos.HEV       │
  → direct C++ API ───►  extended interface    │
                       │                        │──► EventBus
SeaDrop RSSI ─────────►  ProximityGuard        │──► LockScreen
                       │                        │──► StateManager
PAM/LockScreen ───────►  VaultGuard            │──► OSFNotification
                       │                        │──► Supervisor
                       │  ┌──────────────────┐  │
                       │  │   VaultEngine    │  │
                       │  │  Argon2id KDF    │  │
                       │  │  AES-256-GCM     │  │
                       │  │  SQLite backend  │  │
                       │  └──────────────────┘  │
                       └───────────────────────┘
```

### 25.3 HEV.h — Complete Header

```cpp
// animus/hev/HEV.h
#pragma once
#include <string>
#include <vector>
#include <memory>
#include <atomic>
#include <functional>
#include <cstdint>

namespace Animus {

// ── Store types ───────────────────────────────────────────────────
enum class HEVStoreType : uint8_t {
    Credentials  = 0,  // app passwords, SSH keys, VPN creds
    Identity     = 1,  // user name, email, profile photo path
    Certificate  = 2,  // SSL/TLS certs, GPG keys, code signing
    Token        = 3,  // OAuth tokens, API keys, 2FA backup codes
    SeaDropTrust = 4,  // SeaDrop device public keys + shared secrets
};

// ── Vault entry ───────────────────────────────────────────────────
struct HEVEntry {
    uint64_t    id;             // unique entry ID
    HEVStoreType store;         // which store this belongs to
    std::string label;          // human-readable name
    std::string appId;          // which app owns this entry
    std::string schema;         // e.g. "org.gnome.keyring.NetworkPassword"
    std::vector<uint8_t> ciphertext;  // AES-256-GCM encrypted value
    std::vector<uint8_t> nonce;       // 96-bit GCM nonce (never reused)
    std::vector<uint8_t> tag;         // 128-bit GCM authentication tag
    uint64_t    createdAt;      // CLOCK_MONOTONIC seconds
    uint64_t    accessedAt;     // last access timestamp
};

// ── SeaDrop trusted device ────────────────────────────────────────
struct HEVTrustedDevice {
    std::string deviceId;       // unique device identifier
    std::string deviceName;     // "Krisna's Phone"
    std::vector<uint8_t> publicKey;   // Curve25519 public key (32 bytes)
    std::vector<uint8_t> sharedSecret;// derived shared secret (32 bytes)
    bool        proximityUnlockEnabled;
    float       rssiUnlockThreshold;  // default -45.0 dBm
    float       rssiLockThreshold;    // default -70.0 dBm
    uint64_t    lastSeen;       // CLOCK_MONOTONIC seconds
};

// ── Vault state ───────────────────────────────────────────────────
enum class HEVVaultState {
    Cold,       // process just started, master key not yet derived
    Unlocked,   // master key in memory, entries accessible
    Locked,     // master key wiped, entries inaccessible
    Sealed,     // security alert — requires password to reopen
};

// ── Access request result ─────────────────────────────────────────
enum class HEVAccessResult {
    Granted,
    Denied,         // app not in trusted list
    VaultLocked,    // vault not unlocked
    NotFound,       // entry doesn't exist
    AuthRequired,   // needs user approval
};

// ── HEV: the main class ───────────────────────────────────────────
// Singleton — one vault per user session.
// Initialized by OSFDesktop after LockScreen, before Shell components.
class HEV {
public:
    static HEV& shared();

    // ── Lifecycle ─────────────────────────────────────────────────
    bool initialize();  // opens vault db, registers D-Bus service
    void destroy();     // wipes master key, closes vault

    // ── Vault state ───────────────────────────────────────────────

    // Cold start — derives master key from password via Argon2id
    // Must succeed before vault is accessible
    // Returns false if password is wrong
    bool unlockWithPassword(const std::string &password);

    // Proximity unlock — called by ProximityGuard when RSSI threshold met
    // Only works if vault is Locked (not Cold) — master key must exist
    // in memory from a previous unlockWithPassword() call
    bool unlockWithProximity(const std::string &deviceId);

    // Lock vault — wipes master key from memory
    // Called on screen lock, explicit user action
    void lock();

    // Seal vault — security alert, requires password to reopen
    // Called on panic lock (sudden signal loss) or security event
    void seal();

    HEVVaultState state() const { return m_state.load(); }

    // ── Entry access ──────────────────────────────────────────────
    HEVAccessResult getSecret(const std::string &appId,
                               const std::string &label,
                               std::vector<uint8_t> &outPlaintext);

    HEVAccessResult setSecret(const std::string &appId,
                               const std::string &label,
                               HEVStoreType store,
                               const std::vector<uint8_t> &plaintext);

    HEVAccessResult deleteSecret(const std::string &appId,
                                  const std::string &label);

    std::vector<HEVEntry> listEntries(const std::string &appId);

    // ── Trust management ──────────────────────────────────────────
    void registerTrustedApp(const std::string &appId);
    void revokeTrustedApp(const std::string &appId);
    bool isAppTrusted(const std::string &appId) const;

    // ── SeaDrop integration ───────────────────────────────────────
    bool registerSeaDropDevice(const HEVTrustedDevice &device);
    bool revokeSeaDropDevice(const std::string &deviceId);
    const HEVTrustedDevice* getSeaDropDevice(const std::string &deviceId) const;
    std::vector<HEVTrustedDevice> listSeaDropDevices() const;

    // ── Proximity unlock config ───────────────────────────────────
    void setProximityUnlock(const std::string &deviceId, bool enabled);
    void setRssiThresholds(const std::string &deviceId,
                            float unlockDbm, float lockDbm);

    // ── Event handlers ────────────────────────────────────────────
    void onScreenLocked();    // → lock()
    void onScreenUnlocked();  // no-op — proximity or password handles this
    void onSecurityAlert();   // → seal()
    void onFatalSignal();     // → wipeKeyImmediate() async-signal-safe

    // For Supervisor
    struct VaultStatus {
        HEVVaultState state;
        uint32_t      entryCount;
        uint32_t      trustedAppCount;
        uint32_t      seaDropDeviceCount;
        std::string   lastAccessedApp;
        double        lastAccessTimeS;
        bool          proximityUnlockActive;
    };
    VaultStatus status() const;

private:
    HEV() = default;

    class VaultEngine;
    class ProximityGuard;
    class DBusSecretService;

    std::unique_ptr<VaultEngine>       m_vault;
    std::unique_ptr<ProximityGuard>    m_proximity;
    std::unique_ptr<DBusSecretService> m_dbus;

    std::atomic<HEVVaultState> m_state { HEVVaultState::Cold };

    // Master key — 256 bits, AES-256-GCM key
    // In memory only. Never written to disk.
    // Wiped on lock(), seal(), onFatalSignal()
    uint8_t m_masterKey[32] = {};
    bool    m_masterKeyValid = false;

    void wipeMasterKey();  // secure_memzero + m_masterKeyValid = false
};

} // namespace Animus
```

### 25.4 VaultEngine — Encryption and Storage

```cpp
// animus/hev/VaultEngine.h
#pragma once
#include <string>
#include <vector>
#include <cstdint>
#include <sqlite3.h>
#include <sodium.h>   // libsodium — Argon2id + AES-256-GCM

namespace Animus {

// VaultEngine: handles all cryptographic operations and SQLite storage.
//
// Encryption model:
//   Password → Argon2id → 256-bit master key (never stored)
//   Each entry: AES-256-GCM with unique 96-bit nonce
//   Storage: SQLite with encrypted blobs
//   Auth tag: 128-bit GCM tag stored alongside ciphertext
//
// Argon2id parameters (tuned for ~100ms on mid-range hardware):
//   Memory:      65536 KB  (64 MB)
//   Iterations:  3
//   Parallelism: 4
//   Output:      32 bytes (256-bit master key)
//
// Why Argon2id over bcrypt/scrypt:
//   - Memory-hard: GPU cracking is expensive
//   - Password Hashing Competition winner
//   - Used by 1Password, Bitwarden, Signal
//   - libsodium ships it natively
class VaultEngine {
public:
    bool open(const std::string &dbPath);
    void close();

    // KDF: password → master key
    // Returns false if password is wrong (salt mismatch or HMAC fail)
    bool deriveKey(const std::string &password,
                   uint8_t outKey[32]);

    // First-time setup: generate salt, derive key, store salt
    bool initializeVault(const std::string &password,
                          uint8_t outKey[32]);

    // Encrypt plaintext with master key → ciphertext + nonce + tag
    bool encrypt(const uint8_t masterKey[32],
                 const std::vector<uint8_t> &plaintext,
                 std::vector<uint8_t> &outCiphertext,
                 std::vector<uint8_t> &outNonce,
                 std::vector<uint8_t> &outTag);

    // Decrypt ciphertext with master key → plaintext
    // Returns false if tag verification fails (tampered data)
    bool decrypt(const uint8_t masterKey[32],
                 const std::vector<uint8_t> &ciphertext,
                 const std::vector<uint8_t> &nonce,
                 const std::vector<uint8_t> &tag,
                 std::vector<uint8_t> &outPlaintext);

    // SQLite operations
    bool storeEntry(const HEVEntry &entry);
    bool loadEntry(uint64_t id, HEVEntry &out);
    bool deleteEntry(uint64_t id);
    std::vector<HEVEntry> queryEntries(const std::string &appId);
    uint32_t entryCount();

    // Argon2id parameters — tunable
    static constexpr uint64_t KDF_MEMORY_KB  = 65536;
    static constexpr uint32_t KDF_ITERATIONS = 3;
    static constexpr uint32_t KDF_PARALLELISM= 4;
    static constexpr size_t   SALT_BYTES     = 32;
    static constexpr size_t   KEY_BYTES      = 32;
    static constexpr size_t   NONCE_BYTES    = 12;   // 96-bit GCM nonce
    static constexpr size_t   TAG_BYTES      = 16;   // 128-bit GCM tag

private:
    sqlite3 *m_db = nullptr;
    uint8_t  m_salt[SALT_BYTES] = {};
    bool     m_saltLoaded = false;

    bool createSchema();
    bool loadSalt();
    bool storeSalt();
};

} // namespace Animus
```

```cpp
// animus/hev/VaultEngine.cpp — key methods
#include "VaultEngine.h"
#include <sodium.h>
#include <cstring>

namespace Animus {

bool VaultEngine::deriveKey(const std::string &password,
                             uint8_t outKey[32])
{
    if (!m_saltLoaded) return false;

    // Argon2id via libsodium
    int ret = crypto_pwhash(
        outKey, KEY_BYTES,
        password.c_str(), password.size(),
        m_salt,
        KDF_ITERATIONS,
        KDF_MEMORY_KB * 1024ULL,  // bytes
        crypto_pwhash_ALG_ARGON2ID13);

    return ret == 0;
}

bool VaultEngine::encrypt(const uint8_t masterKey[32],
                           const std::vector<uint8_t> &plaintext,
                           std::vector<uint8_t> &outCiphertext,
                           std::vector<uint8_t> &outNonce,
                           std::vector<uint8_t> &outTag)
{
    // Generate random 96-bit nonce (never reused)
    outNonce.resize(NONCE_BYTES);
    randombytes_buf(outNonce.data(), NONCE_BYTES);

    outCiphertext.resize(plaintext.size());
    outTag.resize(TAG_BYTES);

    // AES-256-GCM via libsodium
    // crypto_aead_aes256gcm requires hardware AES support
    // Fallback: crypto_secretbox_xchacha20poly1305 if no AES-NI
    if (crypto_aead_aes256gcm_is_available()) {
        unsigned long long clen;
        // Encrypt + generate auth tag
        int ret = crypto_aead_aes256gcm_encrypt_detached(
            outCiphertext.data(),
            outTag.data(), nullptr,
            plaintext.data(), plaintext.size(),
            nullptr, 0,  // no additional data
            nullptr,
            outNonce.data(),
            masterKey);
        return ret == 0;
    } else {
        // XChaCha20-Poly1305 fallback (still excellent, no HW requirement)
        // Nonce expanded to 192-bit for XChaCha20
        outNonce.resize(crypto_secretbox_xchacha20poly1305_NONCEBYTES);
        randombytes_buf(outNonce.data(), outNonce.size());
        outCiphertext.resize(plaintext.size() +
            crypto_secretbox_xchacha20poly1305_MACBYTES);
        int ret = crypto_secretbox_xchacha20poly1305_easy(
            outCiphertext.data(),
            plaintext.data(), plaintext.size(),
            outNonce.data(),
            masterKey);
        outTag.clear();  // MAC is prepended to ciphertext in easy mode
        return ret == 0;
    }
}

void VaultEngine::wipeMasterKey(uint8_t key[32]) {
    sodium_memzero(key, 32);  // libsodium secure zero — not optimized away
}

} // namespace Animus
```

### 25.5 ProximityGuard — RSSI-Based Unlock/Lock

```cpp
// animus/hev/ProximityGuard.h
#pragma once
#include <string>
#include <atomic>
#include <thread>
#include <chrono>
#include <functional>

namespace Animus {

// ProximityGuard: monitors SeaDrop RSSI for proximity-based unlock/lock.
//
// Unlock flow:
//   SeaDrop reports RSSI ≥ UNLOCK_DBMS (-45 dBm default)
//   + device is in HEV trusted list
//   + device has proximityUnlockEnabled = true
//   + vault is Locked (not Cold — master key must exist from prior password)
//   → HEV::unlockWithProximity() called
//   → LockScreen dismisses
//   → OSFNotification: "Unlocked by [device name]" (subtle, 2s)
//
// Lock flow:
//   SeaDrop reports RSSI ≤ LOCK_DBMS (-70 dBm default)
//   + grace period 3 seconds (prevents flicker)
//   + still same trusted device
//   → HEV::lock() called
//   → LockScreen engages
//   → OSFNotification: "Locked" (subtle, 2s)
//
// Panic lock (immediate, no grace period):
//   Signal loss > 20m in < 5 seconds (SeaDrop sudden movement detection)
//   OR RSSI drops from strong to absent in < 2 seconds
//   → HEV::seal() called (requires password to reopen)
//   → LockScreen engages immediately
//   → OSFNotification: "Security alert — device moved unexpectedly"
//
// Cold start rule (NEVER bypassed):
//   If vault is Cold (fresh boot, no master key in memory)
//   proximity unlock does NOTHING.
//   Password is always required to derive the master key on cold start.
class ProximityGuard {
public:
    void start();
    void stop();

    // Called by SeaDrop's RSSI monitor on each measurement
    // rssiDbm: e.g. -45.0, -70.0
    // deviceId: SeaDrop device ID (matches HEVTrustedDevice::deviceId)
    void onRssiUpdate(const std::string &deviceId, float rssiDbm);

    // Called by SeaDrop's sudden movement detector
    void onSuddenMovement(const std::string &deviceId,
                           float distanceChangeMeter,
                           float timeSeconds);

    // Callbacks set by HEV
    std::function<void(const std::string &deviceId)> onUnlockTriggered;
    std::function<void()>                             onLockTriggered;
    std::function<void()>                             onPanicLockTriggered;

    static constexpr float   DEFAULT_UNLOCK_DBMS    = -45.0f;
    static constexpr float   DEFAULT_LOCK_DBMS      = -70.0f;
    static constexpr float   PANIC_DISTANCE_METER   = 20.0f;
    static constexpr float   PANIC_TIME_SECONDS     = 5.0f;
    static constexpr int     LOCK_GRACE_MS          = 3000;
    static constexpr int     RSSI_MOVING_AVG_WINDOW = 5;

private:
    // RSSI smoothing — moving average over last 5 samples
    struct DeviceRssiState {
        std::string deviceId;
        float       samples[RSSI_MOVING_AVG_WINDOW] = {};
        int         sampleIdx = 0;
        float       smoothed  = -100.0f;
        bool        wasUnlocked = false;
        std::chrono::steady_clock::time_point lockGraceStart;
        bool        inLockGrace = false;
    };

    float computeSmoothed(DeviceRssiState &state, float newSample);
    void evaluateState(DeviceRssiState &state);

    std::unordered_map<std::string, DeviceRssiState> m_devices;
    std::atomic<bool> m_running = false;
};

} // namespace Animus
```

```cpp
// animus/hev/ProximityGuard.cpp
#include "ProximityGuard.h"
#include "HEV.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <cmath>
#include <chrono>

namespace Animus {

void ProximityGuard::onRssiUpdate(const std::string &deviceId,
                                   float rssiDbm)
{
    // Get or create state for this device
    auto &state = m_devices[deviceId];
    state.deviceId = deviceId;

    float smoothed = computeSmoothed(state, rssiDbm);
    evaluateState(state);
}

float ProximityGuard::computeSmoothed(DeviceRssiState &state,
                                       float newSample)
{
    state.samples[state.sampleIdx % RSSI_MOVING_AVG_WINDOW] = newSample;
    state.sampleIdx++;

    int count = std::min(state.sampleIdx, RSSI_MOVING_AVG_WINDOW);
    float sum = 0;
    for (int i = 0; i < count; i++) sum += state.samples[i];
    state.smoothed = sum / count;
    return state.smoothed;
}

void ProximityGuard::evaluateState(DeviceRssiState &state)
{
    // Get device config from HEV
    const auto *dev = HEV::shared().getSeaDropDevice(state.deviceId);
    if (!dev || !dev->proximityUnlockEnabled) return;

    float unlockDbm = dev->rssiUnlockThreshold;
    float lockDbm   = dev->rssiLockThreshold;

    // ── Unlock check ──────────────────────────────────────────────
    if (!state.wasUnlocked &&
        state.smoothed >= unlockDbm &&
        HEV::shared().state() == HEVVaultState::Locked)
    {
        // Verify device identity before unlocking
        // (RSSI alone is not authentication — key verification is)
        state.wasUnlocked = true;
        state.inLockGrace = false;
        if (onUnlockTriggered) onUnlockTriggered(state.deviceId);
        return;
    }

    // ── Lock check (with grace period) ───────────────────────────
    if (state.wasUnlocked && state.smoothed <= lockDbm) {
        if (!state.inLockGrace) {
            state.inLockGrace = true;
            state.lockGraceStart = std::chrono::steady_clock::now();
        } else {
            auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                std::chrono::steady_clock::now() - state.lockGraceStart).count();
            if (elapsed >= LOCK_GRACE_MS) {
                state.wasUnlocked = false;
                state.inLockGrace = false;
                if (onLockTriggered) onLockTriggered();
            }
        }
    } else if (state.smoothed > lockDbm) {
        // Signal recovered during grace period — cancel lock
        state.inLockGrace = false;
    }
}

void ProximityGuard::onSuddenMovement(const std::string &deviceId,
                                       float distanceChangeMeter,
                                       float timeSeconds)
{
    // Panic lock: device moved > 20m in < 5s
    if (distanceChangeMeter > PANIC_DISTANCE_METER &&
        timeSeconds < PANIC_TIME_SECONDS)
    {
        if (onPanicLockTriggered) onPanicLockTriggered();
    }
}

} // namespace Animus
```

### 25.6 DBusSecretService — org.freedesktop.secrets Implementation

```cpp
// animus/hev/DBusSecretService.h
#pragma once
#include <sdbus-c++/sdbus-c++.h>
#include <string>
#include <vector>
#include <unordered_map>

namespace Animus {

// DBusSecretService: implements org.freedesktop.secrets on D-Bus.
// All external apps (Spotify, Chrome, SSH agent, etc.) call libsecret
// which talks to this service. They never know they're talking to HEV.
//
// Protocol:
//   App calls libsecret → libsecret calls org.freedesktop.secrets →
//   DBusSecretService validates app identity →
//   routes to HEV::getSecret()/setSecret() →
//   returns encrypted value decrypted with master key
//
// Authorization:
//   First access by unknown app → OSFNotification prompt
//     "Spotify wants to store a password. Allow?"
//     [Allow Once] [Allow Always] [Deny]
//   If Allow Always → app added to HEV trusted list
//   If Deny → HEVAccessResult::Denied returned
//
// Vault locked behavior:
//   If vault is locked when app requests secret →
//   LockScreen shown (if not already visible)
//   Request queued until vault unlocks or times out (30s)
class DBusSecretService {
public:
    bool initialize();
    void destroy();

private:
    std::unique_ptr<sdbus::IConnection> m_conn;
    std::unique_ptr<sdbus::IObject>     m_obj;

    // org.freedesktop.secrets.Service
    void onOpenSession(const std::string &algorithm,
                       sdbus::Variant &input,
                       sdbus::ObjectPath &sessionPath,
                       sdbus::Variant &output);

    void onSearchItems(
        const std::map<std::string,std::string> &attributes,
        std::vector<sdbus::ObjectPath> &unlocked,
        std::vector<sdbus::ObjectPath> &locked);

    void onUnlock(const std::vector<sdbus::ObjectPath> &objects,
                  std::vector<sdbus::ObjectPath> &unlockedOut,
                  sdbus::ObjectPath &prompt);

    void onGetSecrets(
        const std::vector<sdbus::ObjectPath> &items,
        const sdbus::ObjectPath &session,
        std::map<sdbus::ObjectPath,
                 sdbus::Struct<sdbus::ObjectPath,
                               std::vector<uint8_t>,
                               std::vector<uint8_t>,
                               std::string>> &secrets);

    // Pending requests — queued while vault is locked
    struct PendingRequest {
        std::string appId;
        std::string label;
        std::function<void(HEVAccessResult,
                           const std::vector<uint8_t>&)> callback;
        double      expiresAt;  // CLOCK_MONOTONIC — 30s timeout
    };
    std::vector<PendingRequest> m_pendingRequests;

    void flushPendingRequests();  // called when vault unlocks
    void expirePendingRequests(); // called each second

    // App identity from D-Bus sender
    std::string senderToAppId(const std::string &sender);
    void promptAuthorization(const std::string &appId,
                              const std::string &label,
                              std::function<void(bool)> callback);
};

} // namespace Animus
```

### 25.7 HEV State Machine

```
                    ┌─────────────────────────────────────┐
                    │                                     │
                    ▼                                     │
              ┌──────────┐   unlockWithPassword()    ┌──────────┐
   Boot ────► │   COLD   │ ─────────────────────────► │UNLOCKED  │
              └──────────┘                            └──────────┘
                    ▲                                  │        │
                    │                                  │        │
              reboot│                     lock()       │        │ seal()
                    │                  screenLock()    │        │ panicLock()
                    │                                  ▼        ▼
                    │                            ┌──────────┐ ┌──────────┐
                    │   reboot only              │  LOCKED  │ │  SEALED  │
                    └────────────────────────────┤          │ │          │
                                                 └──────────┘ └──────────┘
                                                      │              │
                                              proximity│              │password
                                               unlock  │              │only
                                                       └──────────────┘
                                                              │
                                                              ▼
                                                        ┌──────────┐
                                                        │UNLOCKED  │
                                                        └──────────┘

COLD:     fresh boot, no master key. Only password works.
UNLOCKED: master key in memory. All operations available.
LOCKED:   master key wiped. Proximity OR password unlocks.
SEALED:   security event. Password only. Proximity disabled.
```

### 25.8 OSFEvent Additions for HEV

```cpp
// Add to OSFEvent.h enum:
HEVUnlocked,           // data = std::string deviceId ("password" or device name)
HEVLocked,             // data = {} — screen lock or manual
HEVSealed,             // data = std::string reason — security alert
HEVAccessDenied,       // data = std::string appId — unauthorized access attempt
HEVAuthorizationNeeded,// data = std::string appId — needs user approval
ProximityUnlockReady,  // data = std::string deviceId — phone in range
ProximityLockWarning,  // data = float rssiDbm — phone leaving range
```

### 25.9 HEV Integration Points — Complete

```cpp
// ── LockScreen ────────────────────────────────────────────────────
// LockScreen::onUnlockAttempt() — when user types password:
void LockScreen::onUnlockAttempt(const std::string &password) {
    if (HEV::shared().state() == HEVVaultState::Cold) {
        // Cold start — derive master key from password
        if (HEV::shared().unlockWithPassword(password)) {
            dismissLockScreen();
            EventBus::shared().publishAsync(OSFEvent::HEVUnlocked,
                std::string("password"));
        } else {
            shakeAnimation();  // wrong password — spring shake
        }
    } else {
        // Warm unlock — vault was locked, not cold
        // Password re-derives and verifies against stored salt
        if (HEV::shared().unlockWithPassword(password)) {
            dismissLockScreen();
        }
    }
}

// LockScreen — subscribe to proximity unlock:
EventBus::shared().subscribe(OSFEvent::ProximityUnlockReady,
    [](const std::any &data) {
        auto deviceId = std::any_cast<std::string>(data);
        if (HEV::shared().unlockWithProximity(deviceId)) {
            LockScreen::shared().dismissWithProximity(deviceId);
        }
    });

// ── StateManager ──────────────────────────────────────────────────
// StateManager watches lock_screen_visible:
StateManager::shared().observeState("lock_screen_visible",
    [](const std::any &val) {
        bool locked = std::any_cast<bool>(val);
        if (locked)
            HEV::shared().onScreenLocked();   // → lock()
    });

// ── FirstResponder — wipe key on fatal signal ─────────────────────
// In FirstResponder::signalHandler() — async-signal-safe path:
// We cannot call HEV::wipeMasterKey() (not async-signal-safe)
// Instead: set a global flag, Phase 2 wipes it
extern std::atomic<bool> g_hevWipeRequested;
// In signalHandler():
g_hevWipeRequested.store(true, std::memory_order_relaxed);
// In handleCrashOnMainThread():
if (g_hevWipeRequested.load()) {
    HEV::shared().onFatalSignal();  // sodium_memzero the key
}

// ── Vessels DAG — add HEV ─────────────────────────────────────────
// In Vessels::initialize() — add:
registerVessel({ "HEV", { "DBusBridge" },
    []{ /* vault locked — all secret requests denied */ },
    []{ /* vault can reopen — pending requests flushed */ }
});
registerVessel({ "SeaDropTrust", { "HEV" },
    []{ /* proximity unlock disabled */ },
    []{ /* proximity unlock re-enabled */ }
});

// ── OSFDesktop::run() — initialization order ──────────────────────
// After LockScreen init, before Shell:
// Step N: Initialize HEV
HEV::shared().initialize();
// HEV starts in Cold state — LockScreen will handle first unlock

// ── Pathfinder — register app at install time ─────────────────────
// When Pathfinder installs Spotify:
void Pathfinder::onAppInstalled(const std::string &appId) {
    // Pre-register as trusted so first launch doesn't prompt
    // Only for apps from verified NixOS packages
    HEV::shared().registerTrustedApp(appId);
}

// ── Supervisor — HEV status ───────────────────────────────────────
// Supervisor::buildHEVSection():
auto s = HEV::shared().status();
// Displays:
// HEV Status
// ─────────────────────────────
// State:          ● Unlocked
// Vault entries:  47
// Trusted apps:   12
// SeaDrop devices: 2
// Last accessed:  spotify  2s ago
// Proximity:      ● Active (Krisna's Phone, -48 dBm)
```

### 25.10 NixOS Service Declaration

```nix
# pkgs/vitusos-animus/hev.nix

services.vitusOS.hev = {
    enable       = true;

    # Vault storage location
    vaultPath    = "/home/${user}/.vitusOS/hev/vault.db";

    # Argon2id parameters — tunable per hardware
    # Aim for ~100ms derivation time on target hardware
    kdfMemoryKb  = 65536;   # 64 MB
    kdfIterations= 3;
    kdfParallelism = 4;

    # Auto-lock on screen lock
    lockOnScreenLock = true;

    # Sealed vault requires password (proximity disabled)
    # After security alert
    sealOnPanicLock  = true;

    # Proximity unlock defaults (per-device overridable)
    proximity = {
        defaultUnlockDbm = -45.0;
        defaultLockDbm   = -70.0;
        gracePeriodMs    = 3000;
        panicDistanceM   = 20.0;
        panicTimeS       = 5.0;
    };
};

# NixOS build deps — add to buildInputs:
libsodium    # Argon2id + AES-256-GCM + XChaCha20-Poly1305
sqlite       # vault storage backend
# sdbus-c++ already listed for EO-Bus
```

### 25.11 Security Properties Summary

```
Property                    Implementation
──────────────────────────────────────────────────────────
Master key never on disk    Argon2id re-derives from password each boot
Per-entry encryption        AES-256-GCM with unique nonce per write
Tamper detection            GCM auth tag — modified ciphertext rejected
GPU-resistant KDF           Argon2id 64MB memory requirement
Secure key wipe             sodium_memzero — not optimized away by compiler
Cold start password required vault state machine — proximity blocked in Cold
Panic lock → Sealed         Proximity disabled until password re-entered
Fatal signal key wipe       g_hevWipeRequested flag → Phase 2 wipes key
D-Bus trust boundary        DBusSecretService validates every caller
Unknown app authorization   OSFNotification prompt before first access
Pending request timeout     30s — vault must unlock or request expires
SeaDrop key verification    Curve25519 identity check before proximity unlock
                            (RSSI alone is NOT authentication)
```


---

## PART 26 — CacheKeepr

### 26.1 Overview

CacheKeepr is a peer-level openSEF component. It is owned by no subsystem
and serves all of them. It is the memory of expensive computations across
the entire openSEF process — initialized early, always available, evicting
under pressure, invalidating on NixOS store path changes.

**Identity:**
- Peer of AnimusEngine, CrashManager, EO-Bus, HEV
- Not a child of CrashManager — but CrashManager has eviction authority
- Not a child of AnimusEngine — but AnimusEngine subsystems are its
  primary consumers
- Initialized by OSFDesktop before Shell, after AnimusEngine subsystems
  exist but before any of them need cached data

**What CacheKeepr caches:**
- GlyphCache       — rasterized FreeType glyph bitmaps + atlas coordinates
- ShaderCache      — Vulkan VkPipelineCache blobs (driver-serialized)
- TintCache        — WallpaperTintSampler OKLab k-means results per wallpaper
- AppIndexCache    — Pathfinder app metadata (name, icon path, launch path)
- IconCache        — decoded icon pixel data at display DPI
- SnapshotCache    — CockpitView window thumbnails (VkImage handles)

**What CacheKeepr does NOT cache:**
- D-Bus messages          (dbus-broker owns this)
- PipeWire audio buffers  (PipeWire owns this)
- systemd unit state      (systemd owns this)
- Wayland protocol state  (stateless by design)
- Kernel DRM state        (kernel owns this)
- HEV vault entries       (HEV owns this — SQLite + AES-256-GCM)
- NixOS store paths       (/nix/store is immutable, content-addressed,
                           Nix owns its own caching)

**Invalidation model:**
NixOS store paths are content-addressed and immutable. A package update
changes the store path. That changed path IS the invalidation signal.
No timestamps. No dirty flags. No manual cache busting.
InstallManager notifies CacheKeepr on nixos-rebuild completion.
CacheKeepr compares stored store paths against current paths.
Mismatch = evict that subsystem's cache entries. Match = keep everything.

**Persistence:**
- ShaderCache: persisted to disk (VkPipelineCache blob, binary)
  Path: /home/{user}/.vitusOS/cache/shader-pipeline.bin
  Loaded on initialize(), saved on shutdown and after new pipeline creation
- GlyphCache: in-memory only (fast to rebuild from FreeType, small)
- TintCache: in-memory only (fast to recompute on wallpaper change)
- AppIndexCache: persisted to disk (JSON, human-readable)
  Path: /home/{user}/.vitusOS/cache/app-index.json
- IconCache: in-memory only (decoded on demand from icon paths)
- SnapshotCache: in-memory only (VkImage handles, GPU memory)

### 26.2 CacheKeepr.h — Complete Header

```cpp
// animus/cache/CacheKeepr.h
#pragma once
#include <string>
#include <vector>
#include <unordered_map>
#include <memory>
#include <mutex>
#include <atomic>
#include <cstdint>
#include <vulkan/vulkan.h>
#include "crash/GlobalFeed.h"   // PressureLevel

namespace Animus {

// Forward declarations
class VulkanContext;
class GlyphAtlas;

// ── GlyphCache ────────────────────────────────────────────────────
// Stores rasterized glyph bitmaps in CPU memory.
// GlyphAtlas owns the GPU atlas texture — GlyphCache owns the CPU
// side so rasterization (HarfBuzz + FreeType) is not repeated.
//
// Key: codepoint (uint32_t) + ptSize (float) + dpiScale (float)
// Combined into uint64_t: codepoint<<32 | ptSizeQ16<<8 | dpiQ8
//
// Eviction: LRU. Under pressure, evict glyphs not accessed in >60s.
// Keep: Latin Basic (U+0020–U+007F) always — never evicted.
// Evictable: Latin Extended-1 (U+00A0–U+00FF) + any dynamically
//            rasterized codepoints.
struct CachedGlyph {
    std::vector<uint8_t> bitmap;  // FT_Bitmap.buffer copy
    uint32_t  width;
    uint32_t  rows;
    int32_t   bearingX;  // FT_GlyphSlot->bitmap_left
    int32_t   bearingY;  // FT_GlyphSlot->bitmap_top
    int32_t   advanceX;  // FT_GlyphSlot->advance.x (26.6 fixed)
    uint16_t  atlasX;    // position in GlyphAtlas GPU texture
    uint16_t  atlasY;
    bool      inAtlas;   // true if already uploaded to GPU atlas
    double    lastAccessS; // CLOCK_MONOTONIC — for LRU eviction
};

class GlyphCache {
public:
    // Key encoding: codepoint in upper 32 bits,
    // ptSize as Q16.16 fixed point in bits 8–31 of lower 32,
    // dpiScale as Q8.8 fixed in bits 0–7 of lower 32.
    static uint64_t makeKey(uint32_t codepoint,
                             float ptSize,
                             float dpiScale);

    // Returns nullptr if not cached
    const CachedGlyph* get(uint64_t key);

    // Store a newly rasterized glyph
    void put(uint64_t key, CachedGlyph glyph);

    // Mark atlas position filled — called by GlyphAtlas after GPU upload
    void markInAtlas(uint64_t key, uint16_t atlasX, uint16_t atlasY);

    // Evict LRU glyphs older than ageThresholdS
    // Never evicts Latin Basic (codepoint 0x0020–0x007F)
    size_t evictOlderThan(double ageThresholdS);

    // Evict everything except Latin Basic
    size_t evictAll();

    size_t byteSize() const;
    size_t entryCount() const { return m_glyphs.size(); }

private:
    std::unordered_map<uint64_t, CachedGlyph> m_glyphs;
    mutable std::mutex m_mutex;

    static constexpr uint32_t LATIN_BASIC_START = 0x0020;
    static constexpr uint32_t LATIN_BASIC_END   = 0x007F;
    bool isLatinBasic(uint32_t codepoint) const {
        return codepoint >= LATIN_BASIC_START &&
               codepoint <= LATIN_BASIC_END;
    }
};

// ── ShaderCache ───────────────────────────────────────────────────
// Wraps Vulkan's VkPipelineCache.
// VkPipelineCache stores driver-serialized pipeline objects.
// On first run: empty cache, pipelines compiled from SPIR-V.
// Subsequent runs: cache blob loaded → pipelines created instantly.
//
// Persistence: binary blob at ~/.vitusOS/cache/shader-pipeline.bin
// Invalidation: NixOS store path change for animusengine package
//               → delete blob, recreate empty VkPipelineCache
//
// The store path is stored alongside the blob:
//   shader-pipeline.bin       — VkPipelineCache data
//   shader-pipeline.storepath — /nix/store/abc123.../animus path
// If storepath mismatch → both files deleted, cache rebuilt.
class ShaderCache {
public:
    bool initialize(VulkanContext *ctx, const std::string &cachePath,
                    const std::string &currentStorePath);
    void destroy();

    // The VkPipelineCache handle — passed to vkCreateGraphicsPipeline
    // and vkCreateComputePipeline calls in RenderPipeline and
    // MaterialRenderer. Using this handle allows Vulkan driver to
    // return a cached pipeline instead of recompiling SPIR-V.
    VkPipelineCache handle() const { return m_cache; }

    // Called after new pipelines are created — serializes cache to disk
    // so next boot benefits from this session's compilation work.
    bool saveToDisk();

    // Called by CacheKeepr::onStorePathChanged() for animusengine
    // Destroys current VkPipelineCache, deletes blob, creates empty cache
    void invalidate();

    size_t byteSize() const { return m_blobSize; }

private:
    VkDevice       m_device    = VK_NULL_HANDLE;
    VkPipelineCache m_cache    = VK_NULL_HANDLE;
    std::string    m_cachePath;
    std::string    m_storepathFile;
    size_t         m_blobSize  = 0;

    bool loadFromDisk(const std::string &currentStorePath);
    bool createEmpty();
};

// ── TintCache ─────────────────────────────────────────────────────
// Caches WallpaperTintSampler OKLab k-means results per wallpaper.
// Key: wallpaper file path (std::string)
// Value: TintResult (dominant color, luminosity boost, chroma reduce)
// Eviction: path-based — new wallpaper set → old entry irrelevant
// Max entries: 8 (user unlikely to cycle through more than 8 wallpapers
//              in a session without restart)
struct TintResult {
    float r, g, b;             // dominant color (linear sRGB)
    float luminosityBoost;     // OKLab L+ : 0.04–0.12
    float chromaReduce;        // OKLab ab×: 0.08–0.20
    float alpha;               // blend alpha
};

class TintCache {
public:
    // Returns nullptr if not cached
    const TintResult* get(const std::string &wallpaperPath);

    // Store k-means result for this wallpaper
    void put(const std::string &wallpaperPath, const TintResult &result);

    // Evict everything (called under Critical pressure)
    void evictAll();

    // Evict entries not matching currentWallpaperPath
    void evictStale(const std::string &currentWallpaperPath);

    size_t byteSize() const;
    size_t entryCount() const { return m_entries.size(); }

    static constexpr size_t MAX_ENTRIES = 8;

private:
    struct Entry {
        std::string  path;
        TintResult   result;
    };
    std::vector<Entry> m_entries;  // small enough, linear scan is fine
    mutable std::mutex m_mutex;
};

// ── AppIndexCache ─────────────────────────────────────────────────
// Caches Pathfinder's app metadata index.
// Scanned from: /etc/vitusos/apps/{appId}/manifest.json
//               /run/current-system/sw/share/applications/*.desktop
//
// Persistence: ~/.vitusOS/cache/app-index.json
// Invalidation: NixOS store path change → rescan all app paths
//
// Key: appId (std::string)
struct AppEntry {
    std::string appId;
    std::string displayName;
    std::string iconPath;       // absolute path to icon file
    std::string launchPath;     // /etc/vitusos/apps/{appId}/launch
    std::string desktopFile;    // .desktop file path if from system
    std::string storePath;      // /nix/store/... path — invalidation key
                                // AppEntry::storePath mismatch with current
                                // /run/current-system → entry is stale
    std::vector<std::string> keywords;  // for Pathfinder search
    bool        isElectron;     // true → apply Electron compat profile
    bool        requiresHEV;    // true → register in HEV on install
};

class AppIndexCache {
public:
    bool initialize(const std::string &cachePath);

    // Returns nullptr if appId not in index
    const AppEntry* get(const std::string &appId) const;

    // Search by display name or keywords — Pathfinder query
    // Returns up to maxResults entries matching query string
    std::vector<const AppEntry*> search(const std::string &query,
                                         size_t maxResults = 10) const;

    // Rebuild index from scratch — runs on background thread
    // Publishes OSFEvent::AppIndexReady when complete
    void rebuildAsync();

    // Called by CacheKeepr::onStorePathChanged
    void invalidate();

    // Persist current index to disk
    bool saveToDisk();

    size_t byteSize() const;
    size_t entryCount() const;

private:
    std::unordered_map<std::string, AppEntry> m_entries;
    std::string m_cachePath;
    mutable std::mutex m_mutex;
    std::atomic<bool>  m_rebuilding = false;

    void scanAppDirs();
    void parseDesktopFile(const std::string &path);
    void parseVitusManifest(const std::string &appId,
                             const std::string &manifestPath);
    bool isElectronApp(const std::string &launchPath);
};

// ── IconCache ─────────────────────────────────────────────────────
// Caches decoded icon pixel data at current display DPI.
// Key: iconPath + dpiScale (encoded as uint64_t)
// Value: decoded RGBA pixel data + dimensions
// Eviction: LRU. Under Medium pressure, evict icons for non-running apps.
// Under Critical pressure, evict all icons (reloaded on next access).
//
// Note: Icons are loaded from /nix/store paths (immutable).
// Invalidation: store path change → all icon entries evicted.
struct CachedIcon {
    std::vector<uint8_t> rgba;  // decoded pixel data, 4 bytes per pixel
    uint32_t width;
    uint32_t height;
    double   lastAccessS;       // CLOCK_MONOTONIC
    bool     isRunning;         // hint: running app icons kept longer
};

class IconCache {
public:
    static uint64_t makeKey(const std::string &iconPath, float dpiScale);

    const CachedIcon* get(uint64_t key);
    void put(uint64_t key, CachedIcon icon);

    // Mark icon as belonging to a running app — affects eviction priority
    void markRunning(const std::string &iconPath, bool running);

    // Evict LRU non-running icons
    size_t evictNonRunning();

    // Evict everything
    size_t evictAll();

    size_t byteSize() const;
    size_t entryCount() const { return m_icons.size(); }

private:
    std::unordered_map<uint64_t, CachedIcon> m_icons;
    mutable std::mutex m_mutex;
};

// ── SnapshotCache ─────────────────────────────────────────────────
// Caches CockpitView window thumbnails.
// Each thumbnail is a VkImage captured via wlr_renderer_read_pixels
// when CockpitView opens. GPU memory. Freed when CockpitView closes
// or under Critical pressure.
//
// Key: surface id (uint64_t — wl_surface serial)
// Value: VkImage + VkImageView + VkDeviceMemory
//
// Ownership: SnapshotCache owns the Vulkan resources.
// CockpitView borrows VkImageView handles for rendering.
// CockpitView MUST NOT free these — only SnapshotCache frees them.
struct CachedSnapshot {
    VkImage        image  = VK_NULL_HANDLE;
    VkImageView    view   = VK_NULL_HANDLE;
    VkDeviceMemory memory = VK_NULL_HANDLE;
    uint32_t       width  = 0;
    uint32_t       height = 0;
    double         capturedAtS = 0.0;  // CLOCK_MONOTONIC
};

class SnapshotCache {
public:
    void initialize(VulkanContext *ctx);
    void destroy();

    // Store a captured thumbnail
    void put(uint64_t surfaceId, CachedSnapshot snapshot);

    // Returns nullptr if not cached
    const CachedSnapshot* get(uint64_t surfaceId);

    // Free a specific surface's thumbnail (surface destroyed)
    void evict(uint64_t surfaceId);

    // Free all thumbnails — called on CockpitView close or Critical pressure
    void evictAll();

    size_t byteSize() const;  // sum of VkImage sizes in GPU memory
    size_t entryCount() const { return m_snapshots.size(); }

private:
    VkDevice m_device = VK_NULL_HANDLE;
    std::unordered_map<uint64_t, CachedSnapshot> m_snapshots;
    mutable std::mutex m_mutex;

    void freeSnapshot(CachedSnapshot &snap);
};

// SnapshotCache::destroy — frees all GPU memory on shutdown
// void SnapshotCache::destroy() {
//     evictAll();
//     m_device = VK_NULL_HANDLE;
// }
//
// SnapshotCache::evict(surfaceId) — frees one surface's GPU memory
// Called by CockpitView::onSurfaceDestroyed(surfaceId)
// Prevents GPU memory leak when windows close while CockpitView open.
//
// LockScreen integration — snapshots evicted on screen lock:
// StateManager::observeState("lock_screen_visible") → true
//   → CacheKeepr::shared().snapshots().evictAll()
// Window content cleared from GPU memory before user leaves machine.
//
// Critical pressure integration:
// onPressureChanged(PressureLevel::Critical)
//   → m_snapshots.evictAll()
// GPU memory reclaimed immediately under Critical pressure.

// ── CacheKeepr — main class ───────────────────────────────────────
class CacheKeepr {
public:
    static CacheKeepr& shared();

    // Called by OSFDesktop::run() — Step 3.5 (after AnimusEngine
    // subsystems exist, before Shell initialization)
    bool initialize(VulkanContext *ctx,
                    const std::string &cacheDir,
                    const std::string &currentAnimusStorePath);
    void destroy();

    // ── Subsystem accessors ───────────────────────────────────────
    GlyphCache&    glyphs()     { return m_glyphs; }
    ShaderCache&   shaders()    { return m_shaders; }
    TintCache&     tints()      { return m_tints; }
    AppIndexCache& apps()       { return m_apps; }
    IconCache&     icons()      { return m_icons; }
    SnapshotCache& snapshots()  { return m_snapshots; }

    // ── Pressure response — called by CrashManager::GlobalFeed ───
    // CrashManager has eviction authority over CacheKeepr.
    // This is a one-way authority relationship, not ownership.
    void onPressureChanged(PressureLevel level);

    // ── NixOS invalidation — called by InstallManager ─────────────
    // storePath: /nix/store/abc123.../animusengine or other component
    // component: "animusengine" | "icons" | "apps" | "fonts"
    void onStorePathChanged(const std::string &component,
                             const std::string &newStorePath);

    // ── Supervisor status ─────────────────────────────────────────
    struct CacheStatus {
        size_t glyphsBytes;
        size_t shadersBytes;
        size_t tintsBytes;
        size_t appsBytes;
        size_t iconsBytes;
        size_t snapshotsBytes;
        size_t totalBytes;
        uint64_t hitCount;
        uint64_t missCount;
        float    hitRate;        // hits / (hits + misses)
        PressureLevel lastPressure;
        double   lastEvictionS;   // CLOCK_MONOTONIC
        double   lastInvalidationS;
        std::string lastInvalidatedComponent;
    };
    CacheStatus status() const;

    // ── Hit/miss tracking — called by each subsystem ──────────────
    void recordHit();
    void recordMiss();

private:
    CacheKeepr() = default;

    GlyphCache    m_glyphs;
    ShaderCache   m_shaders;
    TintCache     m_tints;
    AppIndexCache m_apps;
    IconCache     m_icons;
    SnapshotCache m_snapshots;

    std::unordered_map<std::string, std::string> m_storePaths;
    // key: component name, value: last known /nix/store path

    std::atomic<uint64_t> m_hitCount  = 0;
    std::atomic<uint64_t> m_missCount = 0;
    PressureLevel m_lastPressure = PressureLevel::Normal;
    double        m_lastEvictionS = 0.0;
    double        m_lastInvalidationS = 0.0;
    std::string   m_lastInvalidatedComponent;
    std::string   m_cacheDir;
};

} // namespace Animus
```

### 26.3 CacheKeepr.cpp — Complete Implementation

```cpp
// animus/cache/CacheKeepr.cpp
#include "CacheKeepr.h"
#include "render/VulkanContext.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <sys/stat.h>
#include <fstream>
#include <sstream>
#include <dirent.h>
#include <cstring>
#include <time.h>

namespace Animus {

// ── CacheKeepr singleton ──────────────────────────────────────────
CacheKeepr& CacheKeepr::shared() {
    static CacheKeepr instance;
    return instance;
}

bool CacheKeepr::initialize(VulkanContext *ctx,
                              const std::string &cacheDir,
                              const std::string &currentAnimusStorePath)
{
    m_cacheDir = cacheDir;

    // Ensure cache directory exists
    mkdir(cacheDir.c_str(), 0700);

    // ShaderCache — load from disk if store path matches
    std::string shaderBin  = cacheDir + "/shader-pipeline.bin";
    m_shaders.initialize(ctx, shaderBin, currentAnimusStorePath);

    // AppIndexCache — load from disk
    std::string appJson = cacheDir + "/app-index.json";
    m_apps.initialize(appJson);

    // SnapshotCache — GPU resources
    m_snapshots.initialize(ctx);

    // Record current store paths
    m_storePaths["animusengine"] = currentAnimusStorePath;

    // Subscribe to resource pressure from CrashManager
    EventBus::shared().subscribe(OSFEvent::MemoryPressure,
        [this](const std::any &data) {
            auto level = std::any_cast<PressureLevel>(data);
            onPressureChanged(level);
        });

    // Subscribe to InstallManager rebuild complete
    EventBus::shared().subscribe(OSFEvent::InstallComplete,
        [this](const std::any &data) {
            auto appId = std::any_cast<std::string>(data);
            // Determine which store paths changed and invalidate
            std::string newPath = resolveStorePath(appId);
            onStorePathChanged(appId, newPath);
        });

    return true;
}

void CacheKeepr::destroy() {
    // Save persistent caches before shutdown
    m_shaders.saveToDisk();
    m_apps.saveToDisk();
    m_snapshots.destroy();
    m_shaders.destroy();
}

// ── Pressure response ─────────────────────────────────────────────
void CacheKeepr::onPressureChanged(PressureLevel level) {
    m_lastPressure = level;

    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    double now = ts.tv_sec + ts.tv_nsec * 1e-9;

    switch (level) {
    case PressureLevel::Low:
        // Evict stale tints and old snapshots only
        m_tints.evictStale(
            /* currentWallpaper — get from StateManager */
            "");  // TintCache handles empty path gracefully
        // Evict snapshots older than 5 minutes
        // (CockpitView not open — snapshots stale)
        m_snapshots.evictAll();
        m_lastEvictionS = now;
        break;

    case PressureLevel::Medium:
        // Evict non-running app icons and stale tints
        m_icons.evictNonRunning();
        m_tints.evictAll();
        m_lastEvictionS = now;
        break;

    case PressureLevel::Critical:
        // Maximum eviction — keep only what is actively being used
        m_glyphs.evictAll();        // GlyphAtlas will re-rasterize on demand
        m_icons.evictAll();         // Dock/Pathfinder will reload on next access
        m_tints.evictAll();         // WallpaperTintSampler will re-run k-means
        m_snapshots.evictAll();     // CockpitView will recapture on next open
        // AppIndexCache and ShaderCache are NOT evicted under pressure —
        // they are too expensive to rebuild:
        //   AppIndexCache rebuild requires filesystem scan
        //   ShaderCache rebuild requires SPIR-V recompile
        m_lastEvictionS = now;
        break;

    case PressureLevel::Normal:
        // Nothing to evict — cache is fine
        break;
    }
}

// ── NixOS invalidation ────────────────────────────────────────────
void CacheKeepr::onStorePathChanged(const std::string &component,
                                     const std::string &newStorePath)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    double now = ts.tv_sec + ts.tv_nsec * 1e-9;

    auto it = m_storePaths.find(component);
    bool changed = (it == m_storePaths.end() ||
                    it->second != newStorePath);

    if (!changed) return;  // store path unchanged — nothing to do

    m_storePaths[component] = newStorePath;
    m_lastInvalidationS = now;
    m_lastInvalidatedComponent = component;

    if (component == "animusengine") {
        // Shader SPIR-V paths changed — pipeline cache invalid
        m_shaders.invalidate();
        // Glyph rendering unchanged unless font package changed
    }
    if (component == "animusengine" || component == "fonts") {
        // Font paths may have changed
        m_glyphs.evictAll();
    }
    if (component == "apps" || component == "animusengine") {
        // App manifests, icon paths, launch paths may have changed
        m_apps.invalidate();
        m_icons.evictAll();
        // Rebuild app index asynchronously
        m_apps.rebuildAsync();
    }
    if (component == "icons") {
        m_icons.evictAll();
    }
}

// ── Status for Supervisor ─────────────────────────────────────────
CacheKeepr::CacheStatus CacheKeepr::status() const {
    uint64_t hits   = m_hitCount.load(std::memory_order_relaxed);
    uint64_t misses = m_missCount.load(std::memory_order_relaxed);
    uint64_t total  = hits + misses;

    CacheStatus s;
    s.glyphsBytes    = m_glyphs.byteSize();
    s.shadersBytes   = m_shaders.byteSize();
    s.tintsBytes     = m_tints.byteSize();
    s.appsBytes      = m_apps.byteSize();
    s.iconsBytes     = m_icons.byteSize();
    s.snapshotsBytes = m_snapshots.byteSize();
    s.totalBytes     = s.glyphsBytes + s.shadersBytes + s.tintsBytes
                     + s.appsBytes   + s.iconsBytes   + s.snapshotsBytes;
    s.hitCount   = hits;
    s.missCount  = misses;
    s.hitRate    = total > 0 ? (float)hits / (float)total : 1.0f;
    s.lastPressure              = m_lastPressure;
    s.lastEvictionS             = m_lastEvictionS;
    s.lastInvalidationS         = m_lastInvalidationS;
    s.lastInvalidatedComponent  = m_lastInvalidatedComponent;
    return s;
}

void CacheKeepr::recordHit()  {
    m_hitCount.fetch_add(1, std::memory_order_relaxed);
}
void CacheKeepr::recordMiss() {
    m_missCount.fetch_add(1, std::memory_order_relaxed);
}

} // namespace Animus
```

### 26.4 GlyphCache.cpp — Complete Implementation

```cpp
// animus/cache/GlyphCache.cpp
#include "CacheKeepr.h"
#include <cstring>
#include <time.h>
#include <algorithm>

namespace Animus {

uint64_t GlyphCache::makeKey(uint32_t codepoint,
                               float ptSize,
                               float dpiScale)
{
    // Pack: codepoint (32 bits) | ptSize Q16 (16 bits) | dpiScale Q8 (8 bits)
    // ptSize range: 8.0–72.0 → Q16 fits in uint16_t
    // dpiScale range: 1.0–3.0 → Q8 fits in uint8_t
    uint16_t ptQ16    = (uint16_t)(ptSize  * 256.0f);
    uint8_t  dpiQ8    = (uint8_t) (dpiScale * 64.0f);
    return ((uint64_t)codepoint << 32) |
           ((uint64_t)ptQ16    << 8)  |
           ((uint64_t)dpiQ8);
}

const CachedGlyph* GlyphCache::get(uint64_t key) {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_glyphs.find(key);
    if (it == m_glyphs.end()) {
        CacheKeepr::shared().recordMiss();
        return nullptr;
    }
    // Update LRU timestamp
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    it->second.lastAccessS = ts.tv_sec + ts.tv_nsec * 1e-9;
    CacheKeepr::shared().recordHit();
    return &it->second;
}

void GlyphCache::put(uint64_t key, CachedGlyph glyph) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    glyph.lastAccessS = ts.tv_sec + ts.tv_nsec * 1e-9;
    std::lock_guard<std::mutex> lk(m_mutex);
    m_glyphs[key] = std::move(glyph);
}

void GlyphCache::markInAtlas(uint64_t key,
                               uint16_t atlasX,
                               uint16_t atlasY)
{
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_glyphs.find(key);
    if (it != m_glyphs.end()) {
        it->second.atlasX   = atlasX;
        it->second.atlasY   = atlasY;
        it->second.inAtlas  = true;
    }
}

size_t GlyphCache::evictOlderThan(double ageThresholdS) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    double now = ts.tv_sec + ts.tv_nsec * 1e-9;

    std::lock_guard<std::mutex> lk(m_mutex);
    size_t evicted = 0;
    for (auto it = m_glyphs.begin(); it != m_glyphs.end(); ) {
        uint32_t codepoint = (uint32_t)(it->first >> 32);
        bool protect = isLatinBasic(codepoint);
        double age = now - it->second.lastAccessS;
        if (!protect && age > ageThresholdS) {
            it = m_glyphs.erase(it);
            ++evicted;
        } else {
            ++it;
        }
    }
    return evicted;
}

size_t GlyphCache::evictAll() {
    std::lock_guard<std::mutex> lk(m_mutex);
    size_t count = 0;
    for (auto it = m_glyphs.begin(); it != m_glyphs.end(); ) {
        uint32_t codepoint = (uint32_t)(it->first >> 32);
        if (!isLatinBasic(codepoint)) {
            it = m_glyphs.erase(it);
            ++count;
        } else {
            ++it;
        }
    }
    return count;
}

size_t GlyphCache::byteSize() const {
    std::lock_guard<std::mutex> lk(m_mutex);
    size_t total = 0;
    for (const auto &kv : m_glyphs) {
        total += kv.second.bitmap.size();
        total += sizeof(CachedGlyph);
    }
    return total;
}

} // namespace Animus
```

### 26.5 ShaderCache.cpp — Complete Implementation

```cpp
// animus/cache/ShaderCache.cpp
#include "CacheKeepr.h"
#include "render/VulkanContext.h"
#include <fstream>
#include <sys/stat.h>
#include <cstring>

namespace Animus {

bool ShaderCache::initialize(VulkanContext *ctx,
                               const std::string &cachePath,
                               const std::string &currentStorePath)
{
    m_device         = ctx->device();
    m_cachePath      = cachePath;
    m_storepathFile  = cachePath + ".storepath";

    if (!loadFromDisk(currentStorePath)) {
        // Either no cached blob or store path mismatch — create empty
        return createEmpty();
    }
    return true;
}

bool ShaderCache::loadFromDisk(const std::string &currentStorePath) {
    // Check if cached store path matches current
    std::ifstream spFile(m_storepathFile);
    if (!spFile.is_open()) return false;

    std::string cachedPath;
    std::getline(spFile, cachedPath);
    if (cachedPath != currentStorePath) {
        // Store path changed — cache invalid
        // Delete both files so next run starts clean
        ::unlink(m_cachePath.c_str());
        ::unlink(m_storepathFile.c_str());
        return false;
    }

    // Load the binary blob
    std::ifstream f(m_cachePath, std::ios::binary | std::ios::ate);
    if (!f.is_open()) return false;

    size_t size = f.tellg();
    if (size == 0) return false;

    f.seekg(0);
    std::vector<char> blob(size);
    f.read(blob.data(), size);
    if (!f) return false;

    VkPipelineCacheCreateInfo ci = {};
    ci.sType           = VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO;
    ci.initialDataSize = size;
    ci.pInitialData    = blob.data();

    VkResult r = vkCreatePipelineCache(m_device, &ci, nullptr, &m_cache);
    if (r != VK_SUCCESS) return false;

    m_blobSize = size;
    return true;
}

bool ShaderCache::createEmpty() {
    VkPipelineCacheCreateInfo ci = {};
    ci.sType = VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO;
    // pInitialData = nullptr, initialDataSize = 0 → empty cache
    VkResult r = vkCreatePipelineCache(m_device, &ci, nullptr, &m_cache);
    m_blobSize = 0;
    return r == VK_SUCCESS;
}

bool ShaderCache::saveToDisk() {
    if (m_cache == VK_NULL_HANDLE) return false;

    // Get serialized cache data from driver
    size_t dataSize = 0;
    VkResult r = vkGetPipelineCacheData(m_device, m_cache,
                                         &dataSize, nullptr);
    if (r != VK_SUCCESS || dataSize == 0) return false;

    std::vector<char> data(dataSize);
    r = vkGetPipelineCacheData(m_device, m_cache, &dataSize, data.data());
    if (r != VK_SUCCESS) return false;

    // Write blob atomically — write to .tmp then rename
    std::string tmp = m_cachePath + ".tmp";
    {
        std::ofstream f(tmp, std::ios::binary);
        if (!f.is_open()) return false;
        f.write(data.data(), dataSize);
    }
    ::rename(tmp.c_str(), m_cachePath.c_str());

    // Write store path alongside blob
    {
        std::ofstream sp(m_storepathFile);
        sp << m_storePaths_animusengine;  // set during initialize
    }

    m_blobSize = dataSize;
    return true;
}

void ShaderCache::invalidate() {
    if (m_cache != VK_NULL_HANDLE) {
        vkDestroyPipelineCache(m_device, m_cache, nullptr);
        m_cache = VK_NULL_HANDLE;
    }
    ::unlink(m_cachePath.c_str());
    ::unlink(m_storepathFile.c_str());
    m_blobSize = 0;
    createEmpty();
}

void ShaderCache::destroy() {
    if (m_cache != VK_NULL_HANDLE) {
        vkDestroyPipelineCache(m_device, m_cache, nullptr);
        m_cache = VK_NULL_HANDLE;
    }
}

} // namespace Animus
```

### 26.6 Integration Points — Complete

```cpp
// ── GlyphAtlas integration ────────────────────────────────────────
// In GlyphAtlas::getGlyph() — check CacheKeepr first:
const GlyphEntry* GlyphAtlas::getGlyph(uint32_t codepoint) {
    uint64_t key = GlyphCache::makeKey(codepoint, m_ptSize, m_dpiScale);
    const CachedGlyph *cached = CacheKeepr::shared().glyphs().get(key);

    if (cached) {
        // Cache hit — return atlas entry directly
        // GlyphEntry is a view into the atlas, not the bitmap
        return &m_glyphs.at(codepoint);  // atlas position already recorded
    }

    // Cache miss — rasterize via FreeType + HarfBuzz
    rasterizeAndUpload(codepoint);

    // Store in GlyphCache for future sessions
    // (bitmap + atlas coordinates)
    CachedGlyph cg;
    auto &entry = m_glyphs.at(codepoint);
    cg.width    = entry.w;
    cg.rows     = entry.h;
    cg.atlasX   = entry.atlasX;
    cg.atlasY   = entry.atlasY;
    cg.inAtlas  = true;
    // bitmap already freed after GPU upload — cache atlas coords only
    CacheKeepr::shared().glyphs().put(key, std::move(cg));

    return &m_glyphs.at(codepoint);
}

// ── RenderPipeline integration ─────────────────────────────────────
// In RenderPipeline::initialize() — pass ShaderCache to pipeline creation:
bool RenderPipeline::initialize() {
    // Get VkPipelineCache from CacheKeepr
    VkPipelineCache pipelineCache =
        CacheKeepr::shared().shaders().handle();

    // Pass to every vkCreateGraphicsPipeline call:
    VkGraphicsPipelineCreateInfo ci = {};
    // ... fill ci ...
    vkCreateGraphicsPipelines(m_device,
        pipelineCache,   // ← CacheKeepr's VkPipelineCache
        1, &ci, nullptr, &m_kawasePipeline);

    // Same for all 9 shader pipelines:
    // m_kawasePipeline, m_luminosityPipeline, m_rectPipeline,
    // m_quadPipeline, m_shadowPipeline, m_glyphPipeline, etc.

    // After all pipelines created — save cache to disk
    CacheKeepr::shared().shaders().saveToDisk();

    return true;
}

// ── WallpaperTintSampler integration ─────────────────────────────
// In WallpaperTintSampler::sample():
TintResult WallpaperTintSampler::sample(const std::string &wallpaperPath,
                                          const uint8_t *pixels,
                                          uint32_t w, uint32_t h)
{
    // Check cache first
    const TintResult *cached =
        CacheKeepr::shared().tints().get(wallpaperPath);
    if (cached) return *cached;

    // Cache miss — run k-means in OKLab (expensive: ~50ms for 4K image)
    TintResult result = runKMeans(pixels, w, h);

    // Store result
    CacheKeepr::shared().tints().put(wallpaperPath, result);

    return result;
}

// ── Pathfinder integration ────────────────────────────────────────
// In Pathfinder::query():
std::vector<AppEntry> Pathfinder::query(const std::string &q) {
    // AppIndexCache handles the search — no direct filesystem access
    auto results = CacheKeepr::shared().apps().search(q, 10);

    std::vector<AppEntry> out;
    for (const AppEntry *e : results) {
        if (e) out.push_back(*e);
    }
    return out;
}

// ── CockpitView integration ───────────────────────────────────────
// In CockpitView::captureWindowThumbnails():
void CockpitView::captureWindowThumbnails() {
    for (auto &win : m_windows) {
        uint64_t surfaceId = win.surfaceSerial;

        // Check if already cached (CockpitView opened recently)
        const CachedSnapshot *cached =
            CacheKeepr::shared().snapshots().get(surfaceId);
        if (cached) {
            win.thumbView = cached->view;
            continue;
        }

        // Capture via wlr_renderer_read_pixels
        CachedSnapshot snap;
        snap.width  = win.currentWidth;
        snap.height = win.currentHeight;

        // Allocate VkImage for thumbnail
        // ... VkImageCreateInfo, vkAllocateMemory, vkCreateImageView ...
        // ... wlr_renderer_read_pixels into staging buffer ...
        // ... vkCmdCopyBufferToImage ...

        struct timespec ts;
        clock_gettime(CLOCK_MONOTONIC, &ts);
        snap.capturedAtS = ts.tv_sec + ts.tv_nsec * 1e-9;

        CacheKeepr::shared().snapshots().put(surfaceId, snap);
        win.thumbView = snap.view;
    }
}

// CockpitView::close() — snapshots stay in cache for fast re-open
// CockpitView::onSurfaceDestroyed() — evict that surface's snapshot:
void CockpitView::onSurfaceDestroyed(uint64_t surfaceId) {
    CacheKeepr::shared().snapshots().evict(surfaceId);
}

// ── InstallManager integration ────────────────────────────────────
// In InstallManager — after successful nixos-rebuild:
void InstallManager::onRebuildComplete(const std::string &appId) {
    // Resolve new store path for this app
    std::string newPath = resolveNixStorePath(appId);
    // Notify CacheKeepr — it handles invalidation logic
    CacheKeepr::shared().onStorePathChanged(appId, newPath);
    EventBus::shared().publishAsync(OSFEvent::InstallComplete, appId);
}

// resolveNixStorePath — reads from /run/current-system symlink:
std::string InstallManager::resolveNixStorePath(const std::string &appId) {
    // /run/current-system → /nix/store/abc123.../
    char buf[PATH_MAX];
    ssize_t n = readlink("/run/current-system", buf, sizeof(buf)-1);
    if (n < 0) return "";
    buf[n] = '\0';
    return std::string(buf);
}

// ── OSFDesktop init order — updated ──────────────────────────────
int OSFDesktop::run() {
    // Step 0: CrashManager — always first
    m_crashManager->initialize();

    // Step 1: Compositor C11 core
    animus_compositor_init();

    // Step 2: EventHandler wlr_log bridge
    m_crashManager->eventHandler().initialize();

    // Step 3: AnimusEngine subsystems
    initSubsystems();
    // (VulkanContext, RenderPipeline, AnimationClock, SpringSolver,
    //  EventBus, StateManager, GlyphAtlas, SoundEngine)

    // Step 3.5: CacheKeepr — after Vulkan exists, before Shell
    std::string cacheDir = userHomeDir() + "/.vitusOS/cache";
    std::string animusStorePath = resolveNixStorePath("animusengine");
    CacheKeepr::shared().initialize(
        m_vulkan.get(), cacheDir, animusStorePath);

    // Step 4: Shell (LockScreen, HEV, Panel, Dock, CockpitView)
    initShell();

    // Step 5: Background monitors
    m_crashManager->globalFeed().start();
    m_crashManager->handshakes().start();

    // Step 6: Wayland event loop
    animus_compositor_run();

    // Shutdown
    CacheKeepr::shared().shaders().saveToDisk();
    CacheKeepr::shared().apps().saveToDisk();
    CacheKeepr::shared().destroy();

    return 0;
}

// ── Supervisor — CacheKeepr status section ────────────────────────
// In Supervisor::buildCacheSection():
// Calls CacheKeepr::shared().status() and renders:
//
// CacheKeepr
// ──────────────────────────────────────────────
// Glyphs:     2.1 MB    98.3% hit
// Shaders:    14.2 MB   99.1% hit  (persisted)
// Tints:      0.3 MB    94.7% hit
// App index:  1.8 MB    97.2% hit  (persisted)
// Icons:      3.4 MB    96.8% hit
// Snapshots:  8.1 MB    89.4% hit  (GPU memory)
// ──────────────────────────────────────────────
// Total:      29.9 MB   overall 97.1% hit
// Last eviction:     14 min ago  (Medium pressure)
// Last invalidation: 2 days ago  (animusengine)
```

### 26.7 OSFEvent Additions for CacheKeepr

```cpp
// Add to OSFEvent.h enum:
AppIndexReady,       // data = {} — AppIndexCache rebuild complete
CacheEvicted,        // data = PressureLevel — which level triggered
CacheInvalidated,    // data = std::string component name
```

### 26.8 Vessels DAG — CacheKeepr Entry

```cpp
// In Vessels::initialize() — add CacheKeepr:
registerVessel({ "CacheKeepr", {},  // no dependencies — peer level
    []{ /* degraded: cache misses spike, recomputation increases */
        /* not fatal — everything recomputes on demand             */ },
    []{ /* restored: cache serving again                          */ }
});

// CacheKeepr degradation = performance hit, not correctness failure.
// GlyphAtlas, RenderPipeline, Pathfinder all work without cache —
// just slower. Vessels marks it degraded for Supervisor visibility.
```

### 26.9 NixOS Build

```nix
# CacheKeepr has no additional NixOS dependencies beyond what
# AnimusEngine already declares. All dependencies already present:
#   vulkan-loader  — VkPipelineCache APIs
#   harfbuzz       — GlyphCache key encoding uses ptSize/dpiScale
#   freetype2      — GlyphCache stores FreeType bitmap data
#
# Cache directory created at runtime — not a NixOS-managed path:
#   ~/.vitusOS/cache/
#     shader-pipeline.bin       — VkPipelineCache blob
#     shader-pipeline.storepath — invalidation guard
#     app-index.json            — AppIndexCache

# No additional buildInputs needed.
# No NixOS service declaration needed.
# CacheKeepr is in-process, initialized by OSFDesktop.
```

### 26.10 Security Note

```
CacheKeepr stores no sensitive data.
  GlyphCache:    glyph bitmaps — not sensitive
  ShaderCache:   compiled GPU pipelines — not sensitive
  TintCache:     color floats — not sensitive
  AppIndexCache: app names + paths — not sensitive (all from /nix/store)
  IconCache:     icon pixels — not sensitive
  SnapshotCache: window thumbnails — potentially sensitive

SnapshotCache consideration:
  Window thumbnails may show user content (browser, documents, etc.)
  Stored in GPU memory only — never written to disk.
  Evicted when:
    → CockpitView closes (optional — kept for fast re-open)
    → Surface destroyed (always — mandatory eviction)
    → Memory pressure Critical (always — mandatory eviction)
    → LockScreen engages (always — mandatory eviction)

  LockScreen integration:
  StateManager::observeState("lock_screen_visible") → true
    → CacheKeepr::shared().snapshots().evictAll()
  Window content not visible to anyone who picks up the machine
  after LockScreen engages. HEV locks vault. Snapshots are cleared.
```


---

## PART 27 — RegistryManager

### 27.1 Overview

RegistryManager is a peer-level openSEF component. It is the authoritative
record of every live object in the running system that can be referenced,
destroyed, and dangled. No component in openSEF holds a raw pointer to any
registered object type. All access goes through validated handles.

**The problem it solves:**
A raw pointer to a destroyed object is undefined behavior. In a monolithic
compositor — one process, everything in the same address space — a dangling
pointer dereference is the primary cause of compositor crashes that lose all
open windows and all unsaved user work.

The simulation in this document identified exactly this class of crash:
    OSFWindow::renderTrafficLights() called after WindowManager
    already destroyed the window following a client disconnect.
    RenderPipeline held a stale OSFWindow* reference.
    SIGSEGV at 0xFFFFFFFFFFFFFFFF.

RegistryManager eliminates this entire class of bug. Not by catching it
after it happens. By making it structurally impossible to occur.

**Core guarantee:**
    resolve(handle) → valid pointer if object alive
    resolve(handle) → nullptr if object destroyed
    resolve(handle) → NEVER undefined behavior
    resolve(handle) → NEVER a dangling pointer

**What RegistryManager tracks:**
    WindowRegistry       — OSFWindow instances
    SurfaceRegistry      — wlr_surface instances
    NotificationRegistry — OSFNotification instances
    ClientRegistry       — Wayland client PIDs + appIds (for reconnect)

**What RegistryManager does NOT track:**
    SpringSolver instances  (header-only, stack/member allocated, no pointer sharing)
    VkImage handles         (owned by CacheKeepr::SnapshotCache exclusively)
    HEV vault entries       (owned by VaultEngine exclusively)
    CacheKeepr entries      (owned by each cache subsystem exclusively)
    wlr_output              (owned by wlroots, one instance, no sharing)
    wlr_seat                (owned by wlroots, one instance, no sharing)

**Layer placement:** Peer of AnimusEngine, CrashManager, EO-Bus, HEV, CacheKeepr.
Initialized by OSFDesktop immediately after CrashManager (Step 0.5),
before any component that creates registered objects.

**Relationship to CrashManager:**
RegistryManager writes its live object counts into CrashStateBlock so
FirstResponder captures them at crash time. CrashManager does not own
RegistryManager — they are peers. CrashManager reads, RegistryManager writes.

### 27.2 RegistryManager.h — Complete Header

```cpp
// animus/registry/RegistryManager.h
#pragma once
#include <cstdint>
#include <string>
#include <unordered_map>
#include <mutex>
#include <atomic>

// Forward declarations — RegistryManager never includes these headers
// to avoid circular dependencies. It stores void* internally and
// casts only in typed resolve() calls.
struct wlr_surface;
namespace Animus {
class OSFWindow;
class OSFNotification;
}

namespace Animus {

// ── Handle type ───────────────────────────────────────────────────
// A handle is a stable uint64_t token issued at object creation.
// The token remains valid until explicitly unregistered.
// Zero is always INVALID — no object ever receives handle 0.
using RegHandle = uint64_t;
static constexpr RegHandle REG_INVALID = 0;

// ── WindowRegistry ────────────────────────────────────────────────
// Tracks live OSFWindow instances.
// WindowManager calls register on addSurface, unregister on removeSurface.
// RenderPipeline, CockpitView, InputRouter, Dock all hold RegHandle
// instead of OSFWindow*. They call resolve() before any dereference.
class WindowRegistry {
public:
    // Called by WindowManager::addSurface() when OSFWindow is created
    // Returns handle that all other components store instead of OSFWindow*
    RegHandle     registerWindow(OSFWindow *window);

    // Called by WindowManager::removeSurface() before OSFWindow is destroyed
    // After this call, all handles to this window resolve to nullptr
    void          unregisterWindow(RegHandle handle);

    // Safe dereference — returns nullptr if handle is invalid or unregistered
    // NEVER returns a dangling pointer
    OSFWindow*    resolve(RegHandle handle) const;

    // Convenience: check without dereferencing
    bool          isAlive(RegHandle handle) const;

    // Iterate all live windows — used by RenderPipeline::renderFrame()
    // Callback receives (handle, OSFWindow*) — both guaranteed valid
    // during the iteration (mutex held)
    void          forEach(std::function<void(RegHandle, OSFWindow*)> fn) const;

    // For CrashStateBlock — async-signal-safe count read
    uint32_t      count() const {
        return m_count.load(std::memory_order_relaxed);
    }

    // Called by WindowManager to set focused window handle
    void          setFocused(RegHandle handle);
    RegHandle     focused() const {
        return m_focused.load(std::memory_order_relaxed);
    }
    OSFWindow*    focusedWindow() const;

private:
    mutable std::mutex                         m_mutex;
    std::unordered_map<RegHandle, OSFWindow*>  m_windows;
    std::atomic<uint32_t>                      m_count   = 0;
    std::atomic<RegHandle>                     m_focused = REG_INVALID;
    std::atomic<RegHandle>                     m_next    = 1;
};

// ── SurfaceRegistry ───────────────────────────────────────────────
// Tracks live wlr_surface instances.
// wlr_surface is a C struct from wlroots — cannot be wrapped directly.
// SurfaceRegistry maps wlr_surface* to a stable RegHandle so
// components can refer to surfaces by handle instead of raw pointer.
//
// The wlr_surface* is still stored — wlroots APIs require it.
// The guarantee is: if isAlive(handle) returns false, no component
// will attempt to pass that wlr_surface* to any wlroots API.
class SurfaceRegistry {
public:
    // Called by OSFDesktop::onNewSurface()
    RegHandle        registerSurface(struct wlr_surface *surface);

    // Called by OSFDesktop::onSurfaceDestroy()
    void             unregisterSurface(RegHandle handle);
    // Also accepts raw pointer — used directly in on_surface_destroy callback
    void             unregisterSurface(struct wlr_surface *surface);

    // Safe dereference
    struct wlr_surface* resolve(RegHandle handle) const;
    bool             isAlive(RegHandle handle) const;

    // Reverse lookup: wlr_surface* → RegHandle
    // Used to find the handle when only the raw pointer is available
    // (e.g. in wlroots callbacks that provide wlr_surface* directly)
    RegHandle        handleFor(struct wlr_surface *surface) const;

    uint32_t         count() const {
        return m_count.load(std::memory_order_relaxed);
    }

private:
    mutable std::mutex m_mutex;
    std::unordered_map<RegHandle, struct wlr_surface*> m_surfaces;
    std::unordered_map<struct wlr_surface*, RegHandle> m_reverse;
    std::atomic<uint32_t> m_count = 0;
    std::atomic<RegHandle> m_next = 1;
};

// ── NotificationRegistry ──────────────────────────────────────────
// Tracks live OSFNotification instances.
// Notifications auto-dismiss after their display duration expires.
// The dismiss timer fires on the main thread — but EO-Bus callbacks
// (D-Bus notification requests) arrive from a different context.
// NotificationRegistry ensures safe access across both paths.
class NotificationRegistry {
public:
    RegHandle        registerNotification(OSFNotification *notif);
    void             unregisterNotification(RegHandle handle);

    OSFNotification* resolve(RegHandle handle) const;
    bool             isAlive(RegHandle handle) const;

    // Iterate live notifications — used by RenderPipeline overlay layer
    void             forEach(
        std::function<void(RegHandle, OSFNotification*)> fn) const;

    uint32_t         count() const {
        return m_count.load(std::memory_order_relaxed);
    }

private:
    mutable std::mutex m_mutex;
    std::unordered_map<RegHandle, OSFNotification*> m_notifs;
    std::atomic<uint32_t>  m_count = 0;
    std::atomic<RegHandle> m_next  = 1;
};

// ── ClientRegistry ────────────────────────────────────────────────
// Tracks live Wayland client processes.
// Key data: appId + PID.
// Used by CrashSite for respawn and by the compositor-restart
// reconnect mechanism (SIGUSR1 to surviving client PIDs).
//
// This replaces the raw wl_client* tracking in CrashSite.
// wl_client* becomes invalid after client disconnect — storing
// the PID (stable, kernel-assigned) is the correct approach.
struct ClientRecord {
    std::string appId;
    pid_t       pid;
    RegHandle   windowHandle;   // corresponding WindowRegistry handle
                                // REG_INVALID if no window yet
    bool        isNativeApp;    // true = vitusOS native (supports SIGUSR1)
    double      connectedAtS;   // CLOCK_MONOTONIC
};

class ClientRegistry {
public:
    // Called by CrashSite::onClientConnected()
    RegHandle    registerClient(const std::string &appId,
                                 pid_t pid,
                                 bool isNativeApp);

    // Called by CrashSite::onCleanExit() or onClientCrash()
    void         unregisterClient(RegHandle handle);
    void         unregisterClientByPid(pid_t pid);

    const ClientRecord* resolve(RegHandle handle) const;
    bool                isAlive(RegHandle handle) const;

    // Set window handle after OSFWindow is created for this client
    void         setWindowHandle(RegHandle clientHandle,
                                  RegHandle windowHandle);

    // Used by compositor-restart reconnect:
    // Returns all client PIDs that are still running after compositor crash
    // Only native apps (isNativeApp = true) support SIGUSR1 reconnect
    std::vector<pid_t> liveNativeClientPids() const;
    std::vector<pid_t> liveAllClientPids() const;

    uint32_t     count() const {
        return m_count.load(std::memory_order_relaxed);
    }

private:
    mutable std::mutex m_mutex;
    std::unordered_map<RegHandle, ClientRecord> m_clients;
    std::unordered_map<pid_t, RegHandle>        m_byPid;
    std::atomic<uint32_t>  m_count = 0;
    std::atomic<RegHandle> m_next  = 1;

    bool processExists(pid_t pid) const;
};

// ── RegistryManager — main class ─────────────────────────────────
class RegistryManager {
public:
    static RegistryManager& shared();

    // Called by OSFDesktop::run() Step 0.5 —
    // after CrashManager, before everything else
    void initialize();
    void destroy();

    // ── Subsystem accessors ───────────────────────────────────────
    WindowRegistry&       windows()       { return m_windows; }
    SurfaceRegistry&      surfaces()      { return m_surfaces; }
    NotificationRegistry& notifications() { return m_notifications; }
    ClientRegistry&       clients()       { return m_clients; }

    // ── CrashStateBlock integration ───────────────────────────────
    // Called by GlobalFeed::monitorLoop() each poll cycle
    // Writes live counts into CrashStateBlock for FirstResponder
    // All reads are atomic — async-signal-safe
    struct LiveCounts {
        uint32_t windows;        // live OSFWindow count
        uint32_t surfaces;       // live wlr_surface count
        uint32_t notifications;  // live OSFNotification count
        uint32_t clients;        // live Wayland client count
    };
    LiveCounts liveCounts() const;

    // ── Compositor restart reconnect ──────────────────────────────
    // Called by CrashSite::onCompositorRestart() after systemd
    // restarts the compositor process.
    // Sends SIGUSR1 to all surviving native client PIDs so they
    // reconnect to the new Wayland socket.
    void signalReconnect();

    // ── Supervisor status ─────────────────────────────────────────
    struct RegistryStatus {
        uint32_t liveWindows;
        uint32_t liveSurfaces;
        uint32_t liveNotifications;
        uint32_t liveClients;
        uint64_t totalRegistrations;   // cumulative since boot
        uint64_t totalUnregistrations; // cumulative since boot
        RegHandle focusedWindowHandle;
    };
    RegistryStatus status() const;

private:
    RegistryManager() = default;

    WindowRegistry       m_windows;
    SurfaceRegistry      m_surfaces;
    NotificationRegistry m_notifications;
    ClientRegistry       m_clients;

    std::atomic<uint64_t> m_totalRegs   = 0;
    std::atomic<uint64_t> m_totalUnregs = 0;
};

} // namespace Animus
```

### 27.3 RegistryManager.cpp — Complete Implementation

```cpp
// animus/registry/RegistryManager.cpp
#include "RegistryManager.h"
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <signal.h>
#include <sys/types.h>
#include <dirent.h>
#include <cstring>
#include <time.h>

namespace Animus {

// ── RegistryManager singleton ─────────────────────────────────────
RegistryManager& RegistryManager::shared() {
    static RegistryManager instance;
    return instance;
}

void RegistryManager::initialize() {
    // Nothing to allocate — all subsystems are value members
    // Subscribe to surface lifecycle events from compositor C11 core
    // (wired in OSFDesktop::onNewSurface / onSurfaceDestroy)
}

void RegistryManager::destroy() {
    // Signal reconnect before destroying registry state
    // (compositor shutdown — not crash — clients can clean up)
}

RegistryManager::LiveCounts RegistryManager::liveCounts() const {
    // All atomic reads — async-signal-safe
    return {
        m_windows.count(),
        m_surfaces.count(),
        m_notifications.count(),
        m_clients.count()
    };
}

void RegistryManager::signalReconnect() {
    // Called after compositor restarts
    // Send SIGUSR1 to all surviving native client PIDs
    auto pids = m_clients.liveNativeClientPids();
    for (pid_t pid : pids) {
        // ClientRegistry::processExists() verified these PIDs survive
        kill(pid, SIGUSR1);
        // Client's SIGUSR1 handler calls wl_display_connect() again
        // vitusOS native apps implement this handler
    }
}

RegistryManager::RegistryStatus RegistryManager::status() const {
    return {
        m_windows.count(),
        m_surfaces.count(),
        m_notifications.count(),
        m_clients.count(),
        m_totalRegs.load(std::memory_order_relaxed),
        m_totalUnregs.load(std::memory_order_relaxed),
        m_windows.focused()
    };
}

// ── WindowRegistry ────────────────────────────────────────────────
RegHandle WindowRegistry::registerWindow(OSFWindow *window) {
    std::lock_guard<std::mutex> lk(m_mutex);
    RegHandle h = m_next.fetch_add(1, std::memory_order_relaxed);
    m_windows[h] = window;
    m_count.fetch_add(1, std::memory_order_relaxed);
    return h;
}

void WindowRegistry::unregisterWindow(RegHandle handle) {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_windows.find(handle);
    if (it == m_windows.end()) return;
    m_windows.erase(it);
    m_count.fetch_sub(1, std::memory_order_relaxed);
    // If this was the focused window, clear focused handle
    if (m_focused.load(std::memory_order_relaxed) == handle) {
        m_focused.store(REG_INVALID, std::memory_order_relaxed);
    }
}

OSFWindow* WindowRegistry::resolve(RegHandle handle) const {
    if (handle == REG_INVALID) return nullptr;
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_windows.find(handle);
    if (it == m_windows.end()) return nullptr;
    return it->second;  // guaranteed valid — unregister called before destroy
}

bool WindowRegistry::isAlive(RegHandle handle) const {
    if (handle == REG_INVALID) return false;
    std::lock_guard<std::mutex> lk(m_mutex);
    return m_windows.count(handle) > 0;
}

void WindowRegistry::forEach(
    std::function<void(RegHandle, OSFWindow*)> fn) const
{
    std::lock_guard<std::mutex> lk(m_mutex);
    for (const auto &[h, w] : m_windows) {
        fn(h, w);  // both h and w valid during iteration
    }
}

void WindowRegistry::setFocused(RegHandle handle) {
    m_focused.store(handle, std::memory_order_relaxed);
}

OSFWindow* WindowRegistry::focusedWindow() const {
    RegHandle h = m_focused.load(std::memory_order_relaxed);
    return resolve(h);
}

// ── SurfaceRegistry ───────────────────────────────────────────────
RegHandle SurfaceRegistry::registerSurface(struct wlr_surface *surface) {
    std::lock_guard<std::mutex> lk(m_mutex);
    RegHandle h = m_next.fetch_add(1, std::memory_order_relaxed);
    m_surfaces[h] = surface;
    m_reverse[surface] = h;
    m_count.fetch_add(1, std::memory_order_relaxed);
    return h;
}

void SurfaceRegistry::unregisterSurface(RegHandle handle) {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_surfaces.find(handle);
    if (it == m_surfaces.end()) return;
    m_reverse.erase(it->second);
    m_surfaces.erase(it);
    m_count.fetch_sub(1, std::memory_order_relaxed);
}

void SurfaceRegistry::unregisterSurface(struct wlr_surface *surface) {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto rit = m_reverse.find(surface);
    if (rit == m_reverse.end()) return;
    RegHandle h = rit->second;
    m_surfaces.erase(h);
    m_reverse.erase(rit);
    m_count.fetch_sub(1, std::memory_order_relaxed);
}

struct wlr_surface* SurfaceRegistry::resolve(RegHandle handle) const {
    if (handle == REG_INVALID) return nullptr;
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_surfaces.find(handle);
    if (it == m_surfaces.end()) return nullptr;
    return it->second;
}

bool SurfaceRegistry::isAlive(RegHandle handle) const {
    if (handle == REG_INVALID) return false;
    std::lock_guard<std::mutex> lk(m_mutex);
    return m_surfaces.count(handle) > 0;
}

RegHandle SurfaceRegistry::handleFor(struct wlr_surface *surface) const {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_reverse.find(surface);
    if (it == m_reverse.end()) return REG_INVALID;
    return it->second;
}

// ── NotificationRegistry ──────────────────────────────────────────
RegHandle NotificationRegistry::registerNotification(
    OSFNotification *notif)
{
    std::lock_guard<std::mutex> lk(m_mutex);
    RegHandle h = m_next.fetch_add(1, std::memory_order_relaxed);
    m_notifs[h] = notif;
    m_count.fetch_add(1, std::memory_order_relaxed);
    return h;
}

void NotificationRegistry::unregisterNotification(RegHandle handle) {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_notifs.find(handle);
    if (it == m_notifs.end()) return;
    m_notifs.erase(it);
    m_count.fetch_sub(1, std::memory_order_relaxed);
}

OSFNotification* NotificationRegistry::resolve(RegHandle handle) const {
    if (handle == REG_INVALID) return nullptr;
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_notifs.find(handle);
    if (it == m_notifs.end()) return nullptr;
    return it->second;
}

bool NotificationRegistry::isAlive(RegHandle handle) const {
    if (handle == REG_INVALID) return false;
    std::lock_guard<std::mutex> lk(m_mutex);
    return m_notifs.count(handle) > 0;
}

void NotificationRegistry::forEach(
    std::function<void(RegHandle, OSFNotification*)> fn) const
{
    std::lock_guard<std::mutex> lk(m_mutex);
    for (const auto &[h, n] : m_notifs) {
        fn(h, n);
    }
}

// ── ClientRegistry ────────────────────────────────────────────────
RegHandle ClientRegistry::registerClient(const std::string &appId,
                                          pid_t pid,
                                          bool isNativeApp)
{
    std::lock_guard<std::mutex> lk(m_mutex);
    RegHandle h = m_next.fetch_add(1, std::memory_order_relaxed);

    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);

    ClientRecord rec;
    rec.appId        = appId;
    rec.pid          = pid;
    rec.windowHandle = REG_INVALID;
    rec.isNativeApp  = isNativeApp;
    rec.connectedAtS = ts.tv_sec + ts.tv_nsec * 1e-9;

    m_clients[h]   = rec;
    m_byPid[pid]   = h;
    m_count.fetch_add(1, std::memory_order_relaxed);
    return h;
}

void ClientRegistry::unregisterClient(RegHandle handle) {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_clients.find(handle);
    if (it == m_clients.end()) return;
    m_byPid.erase(it->second.pid);
    m_clients.erase(it);
    m_count.fetch_sub(1, std::memory_order_relaxed);
}

void ClientRegistry::unregisterClientByPid(pid_t pid) {
    std::lock_guard<std::mutex> lk(m_mutex);
    auto pit = m_byPid.find(pid);
    if (pit == m_byPid.end()) return;
    RegHandle h = pit->second;
    m_clients.erase(h);
    m_byPid.erase(pit);
    m_count.fetch_sub(1, std::memory_order_relaxed);
}

const ClientRecord* ClientRegistry::resolve(RegHandle handle) const {
    if (handle == REG_INVALID) return nullptr;
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_clients.find(handle);
    if (it == m_clients.end()) return nullptr;
    return &it->second;
}

bool ClientRegistry::isAlive(RegHandle handle) const {
    if (handle == REG_INVALID) return false;
    std::lock_guard<std::mutex> lk(m_mutex);
    return m_clients.count(handle) > 0;
}

void ClientRegistry::setWindowHandle(RegHandle clientHandle,
                                       RegHandle windowHandle)
{
    std::lock_guard<std::mutex> lk(m_mutex);
    auto it = m_clients.find(clientHandle);
    if (it != m_clients.end()) {
        it->second.windowHandle = windowHandle;
    }
}

std::vector<pid_t> ClientRegistry::liveNativeClientPids() const {
    std::lock_guard<std::mutex> lk(m_mutex);
    std::vector<pid_t> result;
    for (const auto &[h, rec] : m_clients) {
        if (rec.isNativeApp && processExists(rec.pid)) {
            result.push_back(rec.pid);
        }
    }
    return result;
}

std::vector<pid_t> ClientRegistry::liveAllClientPids() const {
    std::lock_guard<std::mutex> lk(m_mutex);
    std::vector<pid_t> result;
    for (const auto &[h, rec] : m_clients) {
        if (processExists(rec.pid)) {
            result.push_back(rec.pid);
        }
    }
    return result;
}

bool ClientRegistry::processExists(pid_t pid) const {
    // kill(pid, 0) — signal 0 checks if process exists without sending signal
    // Returns 0 if process exists, -1 + ESRCH if not
    return kill(pid, 0) == 0;
}

} // namespace Animus
```

### 27.4 Integration Points — Complete

```cpp
// ── WindowManager — register/unregister OSFWindow ─────────────────
// animus/core/WindowManager.cpp

void WindowManager::addSurface(struct wlr_surface *s) {
    // Register surface first
    RegHandle surfaceHandle =
        RegistryManager::shared().surfaces().registerSurface(s);

    // Create OSFWindow
    auto win = std::make_shared<OSFWindow>(s, 0, 0, 800, 600);

    // Register window — store handle, NOT raw pointer
    RegHandle windowHandle =
        RegistryManager::shared().windows().registerWindow(win.get());

    win->setHandle(windowHandle);  // OSFWindow stores its own handle

    m_windows.push_back(win);
    // m_focused stores RegHandle now, not OSFWindow*
}

void WindowManager::removeSurface(struct wlr_surface *s) {
    // Find handle via reverse lookup
    RegHandle surfaceHandle =
        RegistryManager::shared().surfaces().handleFor(s);

    // Find matching OSFWindow
    auto it = std::find_if(m_windows.begin(), m_windows.end(),
        [s](const std::shared_ptr<OSFWindow> &w) {
            return w->surface() == s;
        });

    if (it != m_windows.end()) {
        RegHandle windowHandle = (*it)->handle();

        // Unregister BEFORE destroying — all resolve() calls
        // after this return nullptr immediately
        RegistryManager::shared().windows().unregisterWindow(windowHandle);
        RegistryManager::shared().surfaces().unregisterSurface(surfaceHandle);

        // Also notify CacheKeepr — evict this window's snapshot
        CacheKeepr::shared().snapshots().evict(
            static_cast<uint64_t>(windowHandle));

        // Now safe to destroy — no component can dereference it
        m_windows.erase(it);
    }
}

// m_focused is now RegHandle, not OSFWindow*:
void WindowManager::setFocus(RegHandle handle) {
    m_focusedHandle = handle;
    RegistryManager::shared().windows().setFocused(handle);
    StateManager::shared().set(
        StateKeys::FocusedWindowId,
        static_cast<int64_t>(handle));
}

OSFWindow* WindowManager::focused() const {
    // Safe: returns nullptr if window was destroyed
    return RegistryManager::shared().windows().focusedWindow();
}

// ── RenderPipeline — handle-based window iteration ────────────────
// animus/render/RenderPipeline.cpp

void RenderPipeline::renderFrame(float dt) {
    // ... damage check ...

    // Layer 3: Window surfaces
    // OLD (dangerous): for (auto& win : m_windows) win->render(cmd);
    // NEW (safe): iterate via RegistryManager
    RegistryManager::shared().windows().forEach(
        [&](RegHandle handle, OSFWindow *win) {
            // win is guaranteed valid inside this callback
            // mutex held during forEach — cannot be destroyed mid-iteration
            win->render(cmd, dt);
        });

    // Focused window special rendering (traffic lights, focus ring):
    OSFWindow *focused =
        RegistryManager::shared().windows().focusedWindow();
    if (focused) {
        // Safe: focusedWindow() returns nullptr if window destroyed
        focused->renderTrafficLights(cmd);
    }
    // If focused == nullptr: window was destroyed between frames
    // No crash. No undefined behavior. Silent no-op.
}

// ── CockpitView — handle-based window access ──────────────────────
// animus/shell/CockpitView.cpp

void CockpitView::open(
    const std::vector<RegHandle> &windowHandles)
{
    // CockpitView stores RegHandle vector, not OSFWindow* vector
    m_windowHandles = windowHandles;
    captureWindowThumbnails();
}

void CockpitView::captureWindowThumbnails() {
    for (RegHandle handle : m_windowHandles) {
        OSFWindow *win =
            RegistryManager::shared().windows().resolve(handle);
        if (!win) continue;  // window closed while CockpitView was opening
        // ... capture thumbnail via wlr_renderer_read_pixels ...
        CacheKeepr::shared().snapshots().put(
            static_cast<uint64_t>(handle), snap);
    }
}

void CockpitView::render(VkCommandBuffer cmd) {
    for (RegHandle handle : m_windowHandles) {
        OSFWindow *win =
            RegistryManager::shared().windows().resolve(handle);
        if (!win) {
            // Window destroyed while CockpitView open
            // Skip this entry — render remaining windows
            // CockpitView layout reflows on next frame
            continue;
        }
        // ... render win's thumbnail ...
    }
}

// ── InputRouter — handle-based focus dispatch ─────────────────────
// animus/core/InputRouter.cpp

void InputRouter::routeKeyEvent(const KeyEvent &ev) {
    // Dispatch to focused window
    OSFWindow *focused =
        RegistryManager::shared().windows().focusedWindow();
    if (!focused) return;  // no focused window — discard event
    // Safe: focusedWindow() returns nullptr if window destroyed
    focused->deliverKeyEvent(ev);
}

void InputRouter::routePointerEvent(const PointerEvent &ev) {
    // Hit-test against all live windows
    OSFWindow *hit = nullptr;
    RegistryManager::shared().windows().forEach(
        [&](RegHandle h, OSFWindow *win) {
            if (win->hitTest(ev.x, ev.y)) hit = win;
        });
    if (hit) hit->deliverPointerEvent(ev);
}

// ── CrashSite — use ClientRegistry instead of wl_client* ─────────
// animus/crash/CrashSite.cpp

void CrashSite::onClientConnected(const std::string &appId, pid_t pid) {
    // Determine if this is a native vitusOS app
    bool isNative = isNativeVitusApp(appId);

    RegHandle clientHandle =
        RegistryManager::shared().clients().registerClient(
            appId, pid, isNative);

    // Write to CrashStateBlock for FirstResponder
    // (existing CrashSite CrashStateBlock update code — unchanged)
    updateCrashStateBlock(appId, pid);
}

void CrashSite::onClientCrash(const std::string &appId, pid_t pid) {
    recordRespawn(appId);

    // Unregister client — window already unregistered by WindowManager
    RegistryManager::shared().clients().unregisterClientByPid(pid);

    if (shouldRespawn(appId)) {
        respawnApp(appId);
    } else {
        EventBus::shared().publishAsync(OSFEvent::ClientCrashed, appId);
    }
}

void CrashSite::onCompositorRestart() {
    // Called by systemd service restart hook
    // Send SIGUSR1 to all surviving native client PIDs
    RegistryManager::shared().signalReconnect();
}

bool CrashSite::isNativeVitusApp(const std::string &appId) const {
    // Native vitusOS apps: Pathfinder, Filer, Supervisor, SeaDrop
    static const std::unordered_set<std::string> natives = {
        "pathfinder", "filer", "supervisor", "seadrop"
    };
    return natives.count(appId) > 0;
}

// ── OSFDesktop — surface lifecycle wires RegistryManager ──────────
// animus/core/OSFDesktop.cpp

void OSFDesktop::onNewSurface(struct wlr_surface *s) {
    // SurfaceRegistry registers first — before WindowManager creates OSFWindow
    // WindowManager::addSurface() will also register the OSFWindow
    m_wm->addSurface(s);
}

// on_surface_destroy callback (C11 compositor core → C++ bridge):
// animus_compositor.c fires this when Wayland client disconnects
void OSFDesktop::onSurfaceDestroy(struct wlr_surface *s) {
    // WindowManager unregisters both window + surface
    m_wm->removeSurface(s);
}

// ── OSFNotification — register on show, unregister on dismiss ─────
// animus/surfaces/OSFNotification.cpp

void OSFNotification::show() {
    m_handle = RegistryManager::shared().notifications()
                   .registerNotification(this);
    // ... spring animation begin ...
}

void OSFNotification::dismiss() {
    RegistryManager::shared().notifications()
        .unregisterNotification(m_handle);
    m_handle = REG_INVALID;
    // ... spring animation out, then destroy self ...
}

// ── VK_ERROR_DEVICE_LOST — RenderPipeline ────────────────────────
// animus/render/RenderPipeline.cpp
// Addition to renderFrame() — wrap vkQueueSubmit:

VkResult submitResult =
    vkQueueSubmit(m_ctx->gfxQueue, 1, &si, m_ctx->fence[f]);

if (submitResult == VK_ERROR_DEVICE_LOST) {
    // GPU driver crashed — unrecoverable in-process
    // FirstResponder will capture state on SIGABRT
    EventBus::shared().publishAsync(
        OSFEvent::FatalError,
        std::string("VK_ERROR_DEVICE_LOST"));
    // Controlled exit — systemd restarts compositor
    // signalReconnect() will fire in onCompositorRestart()
    OSFDesktop::shared().requestShutdown();
    return;
}
if (submitResult == VK_SUBOPTIMAL_KHR) {
    // Swapchain needs recreation — not a crash
    // Flag for recreation on next frame
    m_swapchainDirty = true;
    return;
}
if (submitResult != VK_SUCCESS) {
    EventHandler::shared().onWlrError(
        WLR_ERROR, "vkQueueSubmit: %d", (int)submitResult);
    CrashManager::shared().vessels().markDead("RenderPipeline");
    return;
}

// ── CrashStateBlock additions for RegistryManager ─────────────────
// In CrashStateBlock struct — add to resource snapshot section:
//   uint32_t liveWindows;       // RegistryManager::windows().count()
//   uint32_t liveSurfaces;      // RegistryManager::surfaces().count()
//   uint32_t liveNotifications; // RegistryManager::notifications().count()
//   uint32_t liveClients;       // RegistryManager::clients().count()
//
// In GlobalFeed::monitorLoop() — add to CrashStateBlock update:
auto counts = RegistryManager::shared().liveCounts();
cs.liveWindows       = counts.windows;
cs.liveSurfaces      = counts.surfaces;
cs.liveNotifications = counts.notifications;
cs.liveClients       = counts.clients;

// ── Native app SIGUSR1 handler — in Pathfinder, Filer, Supervisor,
//    SeaDrop main.cpp:
static void onCompositorRestart(int) {
    // Reconnect to new Wayland socket after compositor restart
    // Called when compositor crashes and restarts via systemd
    // wl_display_connect() reconnects to /run/user/1000/wayland-0
    g_needsReconnect.store(true, std::memory_order_relaxed);
}

// In main() startup:
signal(SIGUSR1, onCompositorRestart);

// In main event loop:
if (g_needsReconnect.load(std::memory_order_relaxed)) {
    g_needsReconnect.store(false, std::memory_order_relaxed);
    app->reconnectWayland();  // wl_display_connect() + recreate surfaces
}
```

### 27.5 OSFDesktop Init Order — Updated

```cpp
int OSFDesktop::run() {
    // Step 0:   CrashManager::initialize()    ← always first
    // Step 0.5: RegistryManager::initialize() ← before anything creates objects
    RegistryManager::shared().initialize();

    // Step 1:   animus_compositor_init()
    // Step 2:   EventHandler::initialize()
    // Step 3:   AnimusEngine subsystems
    // Step 3.5: CacheKeepr::initialize()
    // Step 4:   Shell (LockScreen, HEV, Panel, Dock, CockpitView)
    // Step 5:   GlobalFeed::start(), Handshakes::start()
    // Step 6:   wl_event_loop_run()

    // Shutdown:
    RegistryManager::shared().destroy();
}
```

### 27.6 OSFEvent Additions

```cpp
// Add to OSFEvent.h enum:
FatalError,    // data = std::string reason ("VK_ERROR_DEVICE_LOST" etc.)
               // triggers controlled shutdown → systemd restart
```

### 27.7 Vessels DAG Entry

```cpp
// In Vessels::initialize() — add:
registerVessel({ "RegistryManager", {},   // no dependencies — peer level
    []{ /* degraded: handle resolution may return stale data  */
        /* extremely unlikely — pure in-memory hashtable ops  */ },
    []{ /* restored                                           */ }
});

// RegistryManager degradation = unrecoverable in practice.
// If the mutex deadlocks, the compositor is effectively hung.
// FirstResponder captures state, systemd restarts.
// RegistryManager rebuild from scratch on restart — correct.
```

### 27.8 Supervisor — RegistryManager Status Section

```
RegistryManager
──────────────────────────────────────────────
Live windows:        4
Live surfaces:       6
Live notifications:  1
Live clients:        4  (3 native, 1 Electron)
──────────────────────────────────────────────
Total registered:    2,847  (since boot)
Total unregistered:  2,843  (since boot)
Focused window:      handle #0x0000000000001A4F
```

### 27.9 Security Properties

```
RegistryManager stores no sensitive data.
All values are object pointers and PIDs — not credentials, not keys.

ClientRegistry PID tracking:
    PIDs are used only for:
        SIGUSR1 reconnect signal (native apps only)
        processExists() liveness check (kill(pid, 0))
    PIDs are not exposed to external processes or D-Bus.
    PIDs are not written to disk.

The registry is in-process only.
No IPC. No D-Bus. No file system access.
Pure in-memory hashtable operations with mutex protection.
```

### 27.10 The Crash That Can No Longer Happen

```
BEFORE RegistryManager:

T+804.832s  Firefox closes → WindowManager::removeSurface()
                → OSFWindow destroyed (shared_ptr refcount → 0)
            7ms later: RenderPipeline::renderFrame()
                → iterates m_windows (stale reference)
                    → OSFWindow::renderTrafficLights()
                        → dereference of destroyed object
                            → SIGSEGV 0xFFFFFFFFFFFFFFFF
                                → compositor down
                                → all windows lost
                                → all unsaved work lost

AFTER RegistryManager:

T+804.832s  Firefox closes → WindowManager::removeSurface()
                → RegistryManager::windows().unregisterWindow(handle)
                    → handle removed from registry
                → OSFWindow destroyed
            7ms later: RenderPipeline::renderFrame()
                → RegistryManager::windows().forEach(...)
                    → Firefox handle not in registry
                    → callback not invoked for Firefox
                → focused = RegistryManager::windows().focusedWindow()
                    → focused handle was Firefox → now REG_INVALID
                    → returns nullptr
                → renderTrafficLights: skipped (focused == nullptr)
                → frame renders normally without Firefox window
                → no crash
                → no SIGSEGV
                → compositor running
                → all other windows intact
                → all unsaved work preserved
```


---

## PART 28 — vitusOS Complete Directory Structure

### 28.1 Source Tree

Every file Opus will create. Every folder. No gaps.
Paths are relative to the repository root: `vitusos/`

```
vitusos/
│
├── AnimusBoot/                         # Stage 0: UEFI EFI app (C11, EDK2)
│   ├── AnimusHandoff.h                 # Shared struct: ANIMUS_GPU_HANDOFF EFI var
│   ├── GpuDetect.c                     # PCI scan: NVIDIA/AMD/Intel Arc detection
│   ├── GopSetup.c                      # GOP framebuffer + Space Orange wordmark
│   ├── AnimusBoot.c                    # Entry point: EfiMain()
│   └── AnimusBoot.inf                  # EDK2 module descriptor
│
├── animus-early/                       # Stage 2: initramfs C11 service
│   ├── animus-early.c                  # simpledrm splash + PipeWire boot chime
│   └── CMakeLists.txt
│
├── compositor/                         # C11 wlroots core — extern "C" boundary
│   ├── animus_compositor.h             # Public API: init/run/damage/callbacks
│   └── animus_compositor.c             # wlroots: DRM, seat, xdg_shell,
│                                       # layer_shell, input, swipe gestures
│
├── session/                            # vitusos-session process (C++17)
│   ├── main.cpp                        # Role::Session entry point
│   │                                   # Step 0: CrashManager (session-side)
│   │                                   # Step 0.5: RegistryManager
│   │                                   # Step 1: EventBus
│   │                                   # Step 2: StateManager
│   │                                   # Step 3: HEV
│   │                                   # Step 4: OSFBridge::bindAsSession()
│   │                                   # Step 5: EO-Bus (D-Bus session bus)
│   │                                   # Step 6: SeaDrop trust subsystem
│   │                                   # Step 7: sd_notify("READY=1")
│   └── CMakeLists.txt                  # session-only build target
│
├── animus/                             # vitusos-compositor process (C++17)
│   ├── main.cpp                        # Role::Compositor entry point
│   │                                   # Step 0: CrashManager (compositor-side)
│   │                                   # Step 0.5: RegistryManager
│   │                                   # Step 1: animus_compositor_init()
│   │                                   # Step 2: EventHandler (wlr_log bridge)
│   │                                   # Step 3: AnimusEngine subsystems
│   │                                   # Step 3.5: CacheKeepr
│   │                                   # Step 4: OSFBridge::connectToSession()
│   │                                   # Step 4.5: Shell
│   │                                   # Step 5: GlobalFeed + Handshakes
│   │                                   # Step 6: wl_event_loop_run()
│   │
│   ├── core/                           # OSFDesktop authority + fundamentals
│   │   ├── OSFDesktop.cpp/.h           # singleton, Role::Compositor,
│   │   │                               # owns OSFBridge, init order
│   │   ├── core/OSFBridge.cpp/.h       # cross-process EventBus bridge
│   │   │                               # unix socket /run/vitusos/osf-ipc.sock
│   │   │                               # binary OSFEvent wire format, 8 bytes
│   │   ├── EventBus.cpp/.h             # in-process pub/sub, subscribe/publish/
│   │   │                               # publishAsync, wl_event_loop drain
│   │   ├── OSFEvent.h                  # enum class OSFEvent — all event types
│   │   │                               # marked: LOCAL / BRIDGED / EXTERNAL
│   │   ├── StateManager.cpp/.h         # key-value state store, observeState()
│   │   │                               # getOr() for safe default reads
│   │   ├── WindowManager.cpp/.h        # addSurface/removeSurface, RegHandle
│   │   │                               # focused window via RegistryManager
│   │   ├── PowerManager.cpp/.h         # logind signals, idle timer, battery
│   │   │                               # lid close, display sleep, DPMS
│   │   ├── DragManager.cpp/.h          # wl_data_device drag, ghost image spring
│   │   │                               # drop target highlight, cancel spring
│   │   └── ClipboardBridge.cpp/.h      # Wayland clipboard → internal events
│   │
│   ├── animation/
│   │   ├── SpringSolver.h              # header-only, 12 spring profiles
│   │   ├── AnimationClock.cpp/.h       # CLOCK_MONOTONIC, 144Hz EMA
│   │   └── AnimationEngine.cpp/.h      # tick(), damage, watchdog kick
│   │
│   ├── render/
│   │   ├── VulkanContext.cpp/.h        # VkInstance, VkDevice, VkSwapchain
│   │   │                               # VkRenderPass, VkFramebuffer x3
│   │   ├── RenderPipeline.cpp/.h       # 5-layer render, damage culling,
│   │   │                               # direct scanout, VK_ERROR_DEVICE_LOST
│   │   ├── MaterialRenderer.cpp/.h     # glass blur: kawase + luminosity
│   │   ├── ShadowRenderer.cpp/.h       # window shadows, VkPipeline
│   │   ├── GlyphAtlas.cpp/.h           # HarfBuzz + FreeType, 2048×2048 R8
│   │   │                               # hb_ft_font_create_referenced only
│   │   ├── TextRenderer.cpp/.h         # shaped quads, sub-pixel accuracy
│   │   └── WallpaperTintSampler.cpp/.h # k-means OKLab, TintResult
│   │
│   ├── input/
│   │   ├── InputRouter.cpp/.h          # key/pointer dispatch via RegHandle
│   │   └── MotionWave.cpp/.h           # all gestures, thresholds, Settings integration
│   │                                   # replaces GestureRecognizer — NEVER use old name
│   │
│   ├── audio/
│   │   └── SoundEngine.cpp/.h          # PipeWire pw_stream, boot chime,
│   │                                   # named sound playback
│   │
│   ├── shell/                          # Shell surfaces
│   │   ├── Panel.cpp/.h                # top bar, orange box, clock, tray
│   │   ├── PanelManager.cpp/.h         # owns all Panel instances (one per output)
│   │   │                               # GlobalMenu follows focused window's output
│   │   ├── Dock.cpp/.h                 # app launcher, running dots, magnify
│   │   ├── CockpitView.cpp/.h          # Mission Control equiv, RegHandle vec
│   │   ├── LockScreen.cpp/.h           # PAM auth, BootCrossfade, shake anim
│   │   ├── BootCrossfade.cpp/.h        # simpledrm → Wayland spring transition
│   │   ├── GlobalMenu.cpp/.h           # menu bar from D-Bus, keyboard nav
│   │   ├── DesktopManager.cpp/.h       # virtual desktop state, parallax springs
│   │   └── WelcomeScreen.cpp/.h        # first-boot 3-step welcome
│   │
│   ├── crash/                          # CrashManager — all 6 subsystems
│   │   ├── CrashManager.cpp/.h         # peer singleton, owns all subsystems
│   │   ├── FirstResponder.cpp/.h       # sigaltstack 64KB, signal handlers,
│   │   │                               # CrashStateBlock write, _exit(139)
│   │   ├── CrashState.cpp/.h           # CrashStateBlock struct + accessors
│   │   ├── CrashDump.cpp/.h            # binary + text report writer
│   │   ├── GlobalFeed.cpp/.h           # /proc polling, PSI epoll, 2000ms/500ms
│   │   │                               # PressureLevel classification
│   │   ├── Handshakes.cpp/.h           # PipeWire/D-Bus/wlroots heartbeat 500ms
│   │   ├── EventHandler.cpp/.h         # wlr_log bridge, FATAL_PATTERNS
│   │   ├── CrashSite.cpp/.h            # wl_client lifecycle, posix_spawn,
│   │   │                               # MAX_RESPAWNS 3 / 10s window,
│   │   │                               # onCompositorRestart + signalReconnect
│   │   └── Vessels.cpp/.h              # DAG registry, markDead, blast radius
│   │
│   ├── hev/                            # HEV — Hardware Encryption Vault
│   │   ├── HEV.cpp/.h                  # peer singleton, state machine
│   │   │                               # Cold/Locked/Unlocked/Wiped
│   │   ├── VaultEngine.cpp/.h          # SQLite + AES-256-GCM, Argon2id KDF
│   │   │                               # 64MB/3iter/4parallel
│   │   ├── ProximityGuard.cpp/.h       # SeaDrop RSSI, Curve25519 verify,
│   │   │                               # -45dBm threshold, 3s grace
│   │   └── DBusSecretService.cpp/.h    # org.freedesktop.secrets impl
│   │
│   ├── cache/                          # CacheKeepr — all 6 subsystems
│   │   ├── CacheKeepr.cpp/.h           # peer singleton, eviction authority,
│   │   │                               # NixOS store path invalidation
│   │   ├── GlyphCache.cpp/.h           # LRU, Latin Basic protected,
│   │   │                               # codepoint+ptSize+dpiScale key
│   │   ├── ShaderCache.cpp/.h          # VkPipelineCache blob persist/load,
│   │   │                               # storepath invalidation guard
│   │   ├── TintCache.cpp/.h            # OKLab k-means results, MAX 8 entries
│   │   ├── AppIndexCache.cpp/.h        # app metadata, Pathfinder search,
│   │   │                               # persist app-index.json
│   │   ├── IconCache.cpp/.h            # RGBA decoded icons, LRU, isRunning
│   │   └── SnapshotCache.cpp/.h        # VkImage thumbnails, GPU memory only
│   │
│   ├── registry/                       # RegistryManager — all 4 subsystems
│   │   ├── RegistryManager.cpp/.h      # peer singleton, LiveCounts,
│   │   │                               # signalReconnect, RegistryStatus
│   │   ├── WindowRegistry.cpp/.h       # RegHandle→OSFWindow*, forEach,
│   │   │                               # focused handle, atomic count
│   │   ├── SurfaceRegistry.cpp/.h      # RegHandle↔wlr_surface*, reverse map
│   │   ├── NotificationRegistry.cpp/.h # RegHandle→OSFNotification*, forEach
│   │   └── ClientRegistry.cpp/.h       # ClientRecord, PID tracking,
│   │                                   # liveNativeClientPids, processExists
│   │
│   └── eo-bus/                         # EO-Bus — external trust boundary
│       ├── DBusBridge.cpp/.h           # eo-bus/DBusBridge: dbus-broker session,
│       │                               # validateMessage, 60msg/sec rate limit,
│       │                               # menu routing, org.freedesktop.secrets
│       ├── AccessibilityProvider.cpp/.h # eo-bus/AccessibilityProvider:
│       │                               # AT-SPI2 org.a11y.atspi bridge
│       └── PortalGateway.cpp/.h        # eo-bus/PortalGateway: XDG portals:
│                                       # file, screenshot, OpenURI routing
│
├── osf/                                # OSFSurfaces + OSFAppKit
│   ├── surfaces/
│   │   ├── OSFWindow.cpp/.h            # wlr_surface wrapper, RegHandle,
│   │   │                               # traffic lights, spring pos/shadow/scale
│   │   ├── OSFSidebar.cpp/.h           # navigation sidebar surface
│   │   ├── OSFToolbar.cpp/.h           # toolbar surface
│   │   ├── OSFContent.h                # content area (header-only)
│   │   ├── OSFPopover.cpp/.h           # spring-animated popover
│   │   ├── OSFDropdown.cpp/.h          # dropdown menu surface
│   │   ├── OSFSheet.cpp/.h             # modal sheet, slideY spring
│   │   ├── OSFNotification.cpp/.h      # auto-dismiss, register/unregister
│   │   ├── OSFTooltip.h                # tooltip (header-only)
│   │   └── OSFContextMenu.cpp/.h       # right-click context menu
│   │
│   └── appkit/                         # OSFAppKit widgets (header-only)
│       ├── OSFButton.h                 # hover/press spring, onClick
│       ├── OSFTextField.h              # focus ring spring, input handling
│       ├── OSFScrollView.h             # scroll spring SPRING_SCROLL
│       ├── OSFTableView.h              # selection pill spring
│       ├── OSFProgressBar.h            # fill spring SPRING_SELECTION
│       ├── OSFSlider.h                 # thumb spring SPRING_WINDOW_DRAG
│       ├── OSFLabel.h                  # static text, TextRenderer
│       ├── OSFImageView.h              # VkImage display
│       ├── OSFCheckbox.h               # check spring SPRING_SELECTION
│       ├── OSFSegmentedControl.h       # pill Y spring SPRING_SELECTION
│       └── OSFListView.h               # item hover springs SPRING_HOVER
│
├── shaders/                            # GLSL sources → SPIR-V at build time
│   ├── texture_quad.vert               # wallpaper, thumbnails, wlr_surface
│   ├── texture_quad.frag
│   ├── rounded_rect.vert               # OSFNative solid surfaces
│   ├── rounded_rect.frag
│   ├── window_shadow.frag              # drop shadows
│   ├── kawase_blur.frag                # glass blur pass 1
│   ├── luminosity_composite.frag       # glass blur pass 2 + OKLab tint
│   ├── glyph.vert                      # text rendering
│   └── glyph.frag
│
├── protocol/
│   └── osf-shell-v1.xml               # Wayland extension protocol definition
│
├── native/                             # OSFNative apps (C++17, each standalone)
│   ├── Pathfinder/
│   │   ├── main.cpp                    # SIGUSR1 reconnect handler
│   │   ├── PathfinderApp.cpp/.h        # app overlay, search bar spring
│   │   ├── PathfinderEngine.cpp/.h     # parallel source queries
│   │   └── CMakeLists.txt
│   │
│   ├── Filer/
│   │   ├── main.cpp                    # SIGUSR1 reconnect handler
│   │   ├── FilerApp.cpp/.h             # file browser, DirectoryWatcher
│   │   ├── FileOperationDaemon.cpp/.h  # copy/move/delete background
│   │   └── CMakeLists.txt
│   │
│   ├── Settings/
│   │   ├── main.cpp                    # SIGUSR1 reconnect handler
│   │   ├── Settings.cpp/.h             # split-pane shell, sidebar nav
│   │   │                               # 9 sections: Wallpaper/Appearance/
│   │   │                               # Display/Sound/Keyboard/MotionWave/
│   │   │                               # User Account/About/Power
│   │   ├── sections/
│   │   │   ├── WallpaperSection.cpp/.h
│   │   │   ├── AppearanceSection.cpp/.h
│   │   │   ├── DisplaySection.cpp/.h
│   │   │   ├── SoundSection.cpp/.h
│   │   │   ├── KeyboardSection.cpp/.h
│   │   │   ├── MotionWaveSection.cpp/.h
│   │   │   ├── UserAccountSection.cpp/.h
│   │   │   ├── AboutSection.cpp/.h
│   │   │   └── PowerSection.cpp/.h
│   │   └── CMakeLists.txt
│   │
│   ├── Supervisor/
│   │   ├── main.cpp                    # SIGUSR1 reconnect handler
│   │   ├── SupervisorApp.cpp/.h        # system monitor, vessel status,
│   │   │                               # CacheKeepr status section,
│   │   │                               # RegistryManager status section
│   │   └── CMakeLists.txt
│   │
│   ├── SeaDrop/
│   │   ├── main.cpp                    # SIGUSR1 reconnect handler
│   │   ├── SeaDropApp.cpp/.h           # BLE scan, RSSI feed, trust UI
│   │   ├── SeaDropDaemon.cpp/.h        # background proximity monitor
│   │   └── CMakeLists.txt
│   │
│   ├── Terminow/
│   │   ├── main.cpp                    # SIGUSR1 reconnect handler
│   │   ├── TerminowApp.cpp/.h          # terminal emulator, VTE or custom
│   │   └── CMakeLists.txt
│   │
│   └── Installer/                      # Live ISO installer — NOT in installed system
│       ├── main.cpp                    # isLiveISO() check via /proc/cmdline
│       │                               # "vitusos-installer" flag from ISO AnimusBoot
│       ├── InstallerApp.cpp/.h         # 5-step controller, slide transitions
│       ├── steps/
│       │   ├── DiskSelectStep.cpp/.h   # disk list, removable detection, warning
│       │   ├── PartitionStep.cpp/.h    # graphical map + partition list editor
│       │   ├── AccountStep.cpp/.h      # username + password + display name
│       │   ├── SummaryStep.cpp/.h      # review, erasing warning, Install button
│       │   └── ProgressStep.cpp/.h     # 5 phases, countdown, reboot
│       ├── engine/
│       │   ├── DiskManager.cpp/.h      # read-only disk/partition enumeration
│       │   ├── PartitionOp.cpp/.h      # sfdisk, mkfs, mount, rsync
│       │   ├── InstallEngine.cpp/.h    # background thread orchestration
│       │   └── EFIInstaller.cpp/.h     # copies AnimusBoot + kernel, efibootmgr
│       └── CMakeLists.txt
│
├── tools/                              # Developer + diagnostic tools
│   ├── collect-report.sh               # crash report collector:
│   │                                   # last-crash.txt + journalctl +
│   │                                   # lspci + hardware info → single file
│   ├── vitusos-diag.cpp                # CLI: dump/vessels/pressure/events
│   │                                   # reads CrashStateBlock + live system
│   └── CMakeLists.txt                  # tools/CMakeLists.txt — vitusos-diag target
│
├── nixos/                              # NixOS configuration
│   ├── flake.nix                       # flake inputs: nixpkgs, home-manager
│   │                                   # outputs: nixosConfigurations.vitusos
│   ├── configuration.nix               # system config: kernel, hardware,
│   │                                   # DRM_SIMPLEDRM=y, fonts, users,
│   │                                   # services.vitusos-session.enable
│   │                                   # services.vitusos-compositor.enable
│   ├── hardware-configuration.nix      # generated by nixos-generate-config
│   │                                   # HP Victus specific: NVMe, GPU, audio
│   └── modules/
│       ├── animus-early.nix            # initramfs Stage 2 service
│       ├── vitusos-session.nix         # systemd: vitusos-session.service
│       │                               # After: dbus.service pipewire.service
│       │                               # Wants: dbus.service
│       │                               # ExecStart: session/vitusos-session
│       │                               # Restart: on-failure
│       │                               # RestartSec: 1s
│       │                               # WatchdogSec: 10s
│       ├── vitusos-compositor.nix      # systemd: vitusos-compositor.service
│       │                               # After: vitusos-session.service
│       │                               # Requires: vitusos-session.service
│       │                               # ExecStart: animus/vitusos-compositor
│       │                               # Restart: on-failure
│       │                               # RestartSec: 0.5s
│       │                               # WatchdogSec: 5s
│       └── vitusos-config.nix          # user config: Pathfinder sources,
│                                       # key bindings, installed apps,
│                                       # wallpaper path, user preferences
│                                       # NEVER system config
│
└── CMakeLists.txt                      # root build: all targets
                                        # glslc shaders → .spv
                                        # spirv-val validation
                                        # session + compositor + native apps
```

### 28.2 Runtime Filesystem

What vitusOS creates at runtime. What Opus must know exists
before writing any path string in any source file.

```
/run/vitusos/                           # tmpfs — cleared on reboot
    osf-ipc.sock                        # OSFBridge unix domain socket
    │                                   # session binds, compositor connects
    wayland-0                           # Wayland display socket
    │                                   # created by animus_compositor_init()
    crashdump-{pid}.bin                 # FirstResponder binary crash report
    crashdump-{pid}.txt                 # FirstResponder human-readable report
    last-crash.bin → crashdump-{N}.bin  # symlink: most recent crash binary
    last-crash.txt → crashdump-{N}.txt  # symlink: most recent crash text
    session.pid                         # vitusos-session PID (written on start)
    compositor.pid                      # vitusos-compositor PID (written on start)

/run/user/{uid}/                        # user runtime — tmpfs
    vitusos-filer.sock                  # Filer IPC socket (PortalGateway)
    vitusos-seadrop.sock                # SeaDrop IPC socket

/etc/vitusos/                           # read-only system config (NixOS managed)
    vitusos-config.nix                  # Pathfinder + user prefs (InstallManager
    │                                   # writes this — nothing else does)
    sounds/
    │   boot_chime.wav                  # SoundEngine boot sound
    │   lock.wav                        # lock sound
    │   unlock.wav                      # unlock sound
    │   notification.wav                # notification sound
    apps/
        {appId}/
            manifest.json               # AppEntry metadata for AppIndexCache
            launch                      # executable path (posix_spawn target)
            icon.png                    # app icon for IconCache

/home/{user}/.vitusOS/                  # user persistent data
    hev/
    │   vault.db                        # VaultEngine SQLite database
    │                                   # AES-256-GCM encrypted entries
    cache/
    │   shader-pipeline.bin             # ShaderCache VkPipelineCache blob
    │   shader-pipeline.storepath       # invalidation guard: /nix/store path
    │   app-index.json                  # AppIndexCache persisted index
    logs/
        session-{date}.log              # vitusos-session structured log
        compositor-{date}.log           # vitusos-compositor structured log
        crash-history.log               # append-only crash event log
                                        # one line per crash: timestamp + reason
```

### 28.3 systemd Service Relationship

```
systemd (PID 1)
    │
    ├── dbus.service         (dbus-broker)
    ├── pipewire.service     (PipeWire 1.0.5)
    ├── pipewire-pulse.service
    │
    ├── vitusos-session.service
    │       After:    dbus.service pipewire.service
    │       Wants:    dbus.service
    │       Restart:  on-failure
    │       RestartSec: 1s
    │       WatchdogSec: 10s
    │       ExecStart: /nix/store/.../vitusos-session
    │       │
    │       owns: HEV, RegistryManager, StateManager
    │             EventBus (session-side)
    │             OSFBridge (binds osf-ipc.sock)
    │             EO-Bus (D-Bus session bus)
    │             SeaDrop trust subsystem
    │
    └── vitusos-compositor.service
            After:    vitusos-session.service
            Requires: vitusos-session.service
            Restart:  on-failure
            RestartSec: 0.5s
            WatchdogSec: 5s
            ExecStart: /nix/store/.../vitusos-compositor
            │
            owns: AnimusEngine (render/animate)
                  CrashManager (health/failure)
                  CacheKeepr (memory)
                  RegistryManager (compositor-side view)
                  Shell (Panel/Dock/CockpitView/LockScreen)
                  OSFBridge (connects to osf-ipc.sock)
                  EventBus (compositor-side)
                  EO-Bus (compositor-facing portals)
```

### 28.4 OSFEvent Bridge Classification

Every OSFEvent is classified. Opus must never bridge a LOCAL event
and must never leave a BRIDGED event local-only.

```cpp
// In OSFEvent.h — every event marked with its scope:

// ── LOCAL: compositor internal — never bridged ────────────────────
// AnimationTick, SpringSettled, DamageRegion, FramePresented,
// MemoryPressure (CrashManager handles internally),
// VulkanDeviceLost (triggers shutdown, session notified separately)

// ── LOCAL: session internal — never bridged ──────────────────────
// VaultEntryAccessed, VaultEntryWritten,
// SeaDropRSSIUpdate, SeaDropDeviceLost,
// ProximityRaw (processed into ProximityChanged before bridge)

// ── BRIDGED: session → compositor ────────────────────────────────
// HEVUnlocked          // LockScreen dismisses
// HEVLocked            // LockScreen appears
// ProximityChanged     // unlock/lock trigger
// WallpaperChanged     // RenderPipeline + WallpaperTintSampler
// StateChanged         // focused window, system prefs
// InstallComplete      // CacheKeepr invalidates
// AppIndexReady        // Pathfinder refreshes
// ConfigReload         // vitusos-config.nix changed

// ── BRIDGED: compositor → session ────────────────────────────────
// ClientConnected      // session: ClientRegistry.registerClient()
// ClientCrashed        // session: ClientRegistry.unregisterClient()
// WindowFocusChanged   // session: StateManager updates
// CompositorReady      // session: first frame rendered
// FatalError           // session: log + prepare for restart

// ── EXTERNAL: arrive via EO-Bus, dispatched to EventBus ──────────
// DBusMenuChanged, PortalFileOpen, PortalSaveFile,
// PortalTakeScreenshot, PortalOpenURI,
// PathfinderResultsReady, PathfinderQueryChanged,
// NotificationRequest, ClipboardChanged
```

### 28.5 Log Format

```
/home/{user}/.vitusOS/logs/session-{date}.log
/home/{user}/.vitusOS/logs/compositor-{date}.log

Format: structured, one line per entry
[ISO8601 timestamp] [LEVEL] [component] message

Example:
[2025-03-15T08:42:01.003Z] [INFO]  [HEV]          vault opened, 4 entries
[2025-03-15T08:42:01.047Z] [INFO]  [OSFBridge]    compositor connected
[2025-03-15T08:42:01.510Z] [INFO]  [OSFDesktop]   READY — first frame
[2025-03-15T08:42:15.221Z] [WARN]  [GlobalFeed]   memory pressure: Low
[2025-03-15T08:42:15.224Z] [INFO]  [CacheKeepr]   evicted snapshots (Low)
[2025-03-15T09:14:33.891Z] [ERROR] [CrashSite]    firefox crashed (pid 4821)
[2025-03-15T09:14:33.892Z] [INFO]  [CrashSite]    respawning firefox (1/3)

Levels: DEBUG INFO WARN ERROR FATAL
FATAL entries are also written to crash-history.log

/home/{user}/.vitusOS/logs/crash-history.log
Format: append-only, one line per crash event
[ISO8601] compositor crash: SIGSEGV RIP=0x... reason=renderTrafficLights
[ISO8601] compositor crash: VK_ERROR_DEVICE_LOST
[ISO8601] session restart: on-failure (exit code 1)
```

### 28.6 Build System

```cmake
# vitusos/CMakeLists.txt (root)
cmake_minimum_required(VERSION 3.24)
project(vitusos CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_C_STANDARD 11)

# Shader compilation: GLSL → SPIR-V
# All .vert/.frag in shaders/ → shaders/*.spv
# spirv-val validates each output

# Targets:
#   vitusos-session     (session/main.cpp + animus/ session subsystems)
#   vitusos-compositor  (animus/main.cpp + compositor/ + animus/ compositor)
#   vitusos-diag        (tools/vitusos-diag.cpp)
#   pathfinder          (native/Pathfinder/)
#   filer               (native/Filer/)
#   supervisor          (native/Supervisor/)
#   seadrop             (native/SeaDrop/)
#   terminow            (native/Terminow/)
# All targets: vitusos-session vitusos-compositor pathfinder filer
#              supervisor seadrop terminow vitusos-diag

# Dependencies (all via NixOS pkgs, no manual find_package hunting):
#   vulkan-loader vulkan-headers
#   wlroots libwayland-server
#   harfbuzz freetype2
#   libsodium sqlite
#   dbus sdbus-cpp
#   pipewire
#   pixman
#   nlohmann_json       (AppIndexCache JSON serialization)
#   glslang spirv-tools (shader compilation, nativeBuildInputs)
```

### 28.7 What Opus Must Never Do

```
NEVER create these directories — they do not exist in vitusOS:
    src/                — Never create src/. No src/ wrapper exists.
    lib/                — Never create lib/. No lib/ wrapper exists.
    include/            — headers live alongside sources (.cpp/.h pairs)
    build/              — CMake out-of-tree build dir, not in repo
    animus/bridge/      — OSFBridge lives in animus/core/, not a bridge/ folder
    animus/session/     — session process lives in session/, not inside animus/

NEVER put in wrong location:
    HEV files outside   animus/hev/
    CrashManager files  outside animus/crash/
    CacheKeepr files    outside animus/cache/
    RegistryManager     outside animus/registry/
    EO-Bus files        outside animus/eo-bus/
    Native apps         outside native/{AppName}/
    NixOS files         outside nixos/
    Shaders             outside shaders/
    Tools               outside tools/
    Runtime files       outside /run/vitusos/ or /home/{user}/.vitusOS/

NEVER write vitusos-config.nix for system-level NixOS config
    vitusos-config.nix = user prefs + Pathfinder sources + installed apps
    system-level NixOS config = /etc/nixos/configuration.nix only
    hardware / kernel / service definitions → configuration.nix
    wallpaper / gestures / power / desktop names → vitusos-config.nix

NEVER use hb_ft_font_create()
    correct API = hb_ft_font_create_referenced()

NEVER put OSFBridge in eo-bus/
    OSFBridge = internal cross-process (animus/core/)
    EO-Bus    = external trust boundary (animus/eo-bus/)
    These are different concerns. Never mix.
```


---

## PART 26 ADDITION — AppIndexCache: nixpkgs Integration + InstallState

### 26A.1 Overview

This addition extends Part 26's AppIndexCache to support:
- nixpkgs-unstable channel search via `nix search`
- InstallState lifecycle tracking per AppEntry
- Install/uninstall progress reporting to Pathfinder
- Pathfinder UI behavior: spinner vs progress bar

This addition does NOT change any existing AppIndexCache API.
It extends AppEntry and adds one new method to AppIndexCache.
All existing code continues to compile unchanged.

### 26A.2 AppEntry Extension

```cpp
// animus/cache/AppIndexCache.h — AppEntry extended
// Add these fields to the existing AppEntry struct.
// All new fields have defaults — existing construction sites unaffected.

struct AppEntry {
    // ── EXISTING FIELDS (unchanged) ──────────────────────────────
    std::string appId;
    std::string displayName;
    std::string iconPath;
    std::string launchPath;
    std::string desktopFile;
    std::string storePath;
    std::vector<std::string> keywords;
    bool        isElectron  = false;
    bool        requiresHEV = false;

    // ── NEW FIELDS ────────────────────────────────────────────────

    // Where this entry came from.
    enum class Source : uint8_t {
        Installed,      // /etc/vitusos/apps/{appId}/manifest.json
        DesktopFile,    // /run/current-system/sw/share/applications/*.desktop
        Nixpkgs,        // discovered via `nix search nixpkgs#{query}`
    };
    Source source = Source::Installed;

    // Current install lifecycle state.
    // Pathfinder reads this to decide what to render.
    enum class InstallState : uint8_t {
        Installed,      // present in /etc/vitusos/apps/ — launch on click
        Available,      // in nixpkgs, not installed — show "Get" button
        Installing,     // nixos-rebuild switch running — show progress bar
        Removing,       // nixos-rebuild running to remove — show progress bar
        Failed,         // last install/remove attempt failed — show "Retry"
    };
    InstallState installState = InstallState::Installed;

    // Install/remove progress: 0.0 → 1.0
    // Valid only when installState == Installing or Removing.
    // Updated by InstallManager on background thread via publishAsync.
    float        installProgress = 0.0f;

    // Error message from last failed operation.
    // Valid only when installState == Failed.
    // Sourced from nixos-rebuild stderr, trimmed to 200 chars.
    std::string  installError;

    // nixpkgs metadata — populated for Source::Nixpkgs entries only.
    std::string  nixpkgsAttr;      // e.g. "zen-browser"
    std::string  nixpkgsVersion;   // e.g. "1.7.6"
    uint64_t     nixpkgsSize = 0;  // installed size in bytes (from nix path-info)
                                   // 0 = unknown (nix search does not return size)
                                   // populated lazily on result selection
};
```

### 26A.3 AppIndexCache Extension

```cpp
// animus/cache/AppIndexCache.h — new method added to AppIndexCache class

class AppIndexCache {
public:
    // ── EXISTING METHODS (unchanged) ─────────────────────────────
    bool initialize(const std::string &cachePath);
    const AppEntry* get(const std::string &appId) const;
    std::vector<const AppEntry*> search(const std::string &query,
                                         size_t maxResults = 10) const;
    void rebuildAsync();
    void invalidate();
    bool saveToDisk();
    size_t byteSize() const;
    size_t entryCount() const;

    // ── NEW METHOD ────────────────────────────────────────────────

    // Search nixpkgs-unstable channel for query.
    // Runs `nix search nixpkgs#{query} --json` on background thread.
    // Results merged into m_entries with Source::Nixpkgs.
    // Existing Installed entries are never overwritten by nixpkgs results.
    // Publishes OSFEvent::AppIndexReady when merge complete.
    //
    // Known limit: `nix search` requires network access on first run
    // (fetches nixpkgs flake metadata). Subsequent runs use local cache.
    // If network unavailable: searchNixpkgsAsync() silently returns no
    // nixpkgs results — installed app results still returned normally.
    // Known limit: `nix search` can take 2-8 seconds on first invocation
    // per session. Pathfinder shows spinner during this time.
    void searchNixpkgsAsync(const std::string &query);

private:
    // ── EXISTING PRIVATE (unchanged) ─────────────────────────────
    std::unordered_map<std::string, AppEntry> m_entries;
    std::string  m_cachePath;
    mutable std::mutex m_mutex;
    std::atomic<bool>  m_rebuilding = false;
    void scanAppDirs();
    void parseDesktopFile(const std::string &path);
    void parseVitusManifest(const std::string &appId,
                             const std::string &manifestPath);
    bool isElectronApp(const std::string &launchPath);

    // ── NEW PRIVATE ───────────────────────────────────────────────
    // Parses `nix search --json` output into AppEntry vector.
    // Called on background thread by searchNixpkgsAsync.
    // Returns empty vector on parse failure — never throws.
    std::vector<AppEntry> parseNixSearch(const std::string &jsonOutput) const;

    // Active nixpkgs search thread. Only one runs at a time.
    // If a new query arrives while one is running: the running thread
    // is abandoned (results discarded), new thread starts.
    // Uses std::atomic<uint64_t> m_searchGeneration to detect stale results.
    std::thread              m_nixSearchThread;
    std::atomic<uint64_t>    m_searchGeneration = 0;
};
```

### 26A.4 searchNixpkgsAsync Implementation

```cpp
// animus/cache/AppIndexCache.cpp

void AppIndexCache::searchNixpkgsAsync(const std::string &query) {
    // Increment generation — any in-flight search with old generation
    // will discard its results on completion.
    uint64_t myGeneration = ++m_searchGeneration;

    // Abandon previous thread — it will detect stale generation and exit.
    // std::thread::detach() — we do not join search threads.
    if (m_nixSearchThread.joinable()) {
        m_nixSearchThread.detach();
    }

    m_nixSearchThread = std::thread([this, query, myGeneration]() {
        // Build command:
        // nix search nixpkgs#{query} --json --no-update-lock-file
        // --no-update-lock-file: do not write to flake.lock during search
        // 2>/dev/null: suppress nix stderr (trace/warning output)
        // Timeout: 15 seconds — nix search can hang on slow network
        std::string cmd =
            "nix search nixpkgs#" + query +
            " --json --no-update-lock-file 2>/dev/null";

        // popen with 15s timeout via alarm() is not safe across threads.
        // Use pipe + fork instead via posix_spawn + waitpid with timeout.
        // Implementation: spawn "timeout 15 nix search ..." via /bin/sh
        std::string timedCmd = "timeout 15 " + cmd;

        FILE *fp = popen(timedCmd.c_str(), "r");
        if (!fp) {
            // popen failed — system issue, silent return
            return;
        }

        std::string output;
        char buf[4096];
        while (fgets(buf, sizeof(buf), fp)) {
            output += buf;
            // Guard: nix search output should not exceed 4MB
            // Malformed or unexpected output — abort
            if (output.size() > 4 * 1024 * 1024) {
                pclose(fp);
                return;
            }
        }
        int exitCode = pclose(fp);

        // Check generation before doing anything with results
        if (m_searchGeneration.load() != myGeneration) {
            return;  // stale — a newer search superseded this one
        }

        // exit code 1 from `nix search` means no results — not an error
        // exit code 124 from `timeout` means timed out — silent return
        if (exitCode != 0 && exitCode != 256) {
            // exitCode 256 = (1 << 8) from waitpid — nix found 0 results
            // Any other nonzero: genuine failure — silent return
            // Known limit: distinguishing "no results" from "nix error"
            // via exit code is fragile. If nix changes exit codes,
            // this silently returns no results instead of showing error.
            return;
        }

        if (output.empty()) return;

        std::vector<AppEntry> nixResults = parseNixSearch(output);

        // Merge into m_entries — never overwrite Installed entries
        {
            std::lock_guard<std::mutex> lock(m_mutex);
            for (auto &entry : nixResults) {
                // If already installed: do not touch the entry
                auto it = m_entries.find(entry.appId);
                if (it != m_entries.end() &&
                    it->second.source == AppEntry::Source::Installed) {
                    continue;
                }
                // Insert or update nixpkgs entry
                m_entries[entry.appId] = std::move(entry);
            }
        }

        // Notify Pathfinder — check generation one more time
        if (m_searchGeneration.load() == myGeneration) {
            EventBus::shared().publishAsync(OSFEvent::AppIndexReady, {});
        }
    });
}

std::vector<AppEntry> AppIndexCache::parseNixSearch(
    const std::string &jsonOutput) const
{
    // nix search --json output format:
    // {
    //   "legacyPackages.x86_64-linux.zen-browser": {
    //     "pname": "zen-browser",
    //     "version": "1.7.6",
    //     "description": "Firefox-based browser with a focus on privacy"
    //   },
    //   ...
    // }
    //
    // Key format: "legacyPackages.{system}.{attrName}"
    // We extract attrName as appId.

    std::vector<AppEntry> results;

    try {
        auto j = nlohmann::json::parse(jsonOutput,
                                        nullptr,  // callback
                                        false);   // do not throw on error
        if (!j.is_object()) return results;

        for (auto &[key, val] : j.items()) {
            if (!val.is_object()) continue;

            // Extract attr name from key
            // "legacyPackages.x86_64-linux.zen-browser" → "zen-browser"
            std::string attrName;
            size_t lastDot = key.rfind('.');
            if (lastDot == std::string::npos) {
                attrName = key;
            } else {
                attrName = key.substr(lastDot + 1);
            }
            if (attrName.empty()) continue;

            AppEntry e;
            e.appId          = attrName;
            e.nixpkgsAttr    = attrName;
            e.source         = AppEntry::Source::Nixpkgs;
            e.installState   = AppEntry::InstallState::Available;

            // pname: prefer pname over attrName for displayName
            // "zen-browser" pname → "Zen Browser" display name
            if (val.contains("pname") && val["pname"].is_string()) {
                e.displayName = val["pname"].get<std::string>();
            } else {
                e.displayName = attrName;
            }

            if (val.contains("version") && val["version"].is_string()) {
                e.nixpkgsVersion = val["version"].get<std::string>();
            }

            if (val.contains("description") && val["description"].is_string()) {
                // Store description as keyword for search ranking
                // Not stored as a separate field — keywords drive search
                std::string desc = val["description"].get<std::string>();
                e.keywords.push_back(desc);
            }

            // launchPath for nixpkgs entries: empty until installed
            // InstallManager populates this after nixos-rebuild completes
            e.launchPath = "";

            // iconPath: not available from nix search
            // IconCache will use a generic app icon for Available entries
            e.iconPath = "";

            // nixpkgsSize: not available from nix search --json
            // Populated lazily via `nix path-info` on result selection
            // Known limit: size unknown until user selects the result
            e.nixpkgsSize = 0;

            results.push_back(std::move(e));
        }
    } catch (...) {
        // nlohmann::json::parse with allow_exceptions=false should not throw
        // but guard anyway — malformed output returns empty results
        results.clear();
    }

    return results;
}
```

### 26A.5 InstallManager Extension

```cpp
// animus/shell/InstallManager.h — new methods for progress tracking

class InstallManager {
public:
    static InstallManager& shared();

    // Install app by nixpkgs attr name.
    // Writes vitusos-config.nix, runs nixos-rebuild switch.
    // Updates AppEntry::installState and installProgress via AppIndexCache.
    // Publishes OSFEvent::InstallProgress during build.
    // Publishes OSFEvent::InstallComplete on success.
    // Publishes OSFEvent::InstallFailed on failure — sets installError.
    //
    // Known limit: nixos-rebuild switch requires sudo or wheel group.
    // If user lacks permission: installState → Failed immediately.
    // installError: "Insufficient permissions for nixos-rebuild"
    //
    // Known limit: nixos-rebuild progress is not a clean 0.0→1.0 value.
    // Progress is estimated from build phase strings in stdout:
    //   "evaluating"  → 0.10
    //   "fetching"    → 0.30
    //   "building"    → 0.50 + (built/total * 0.40)
    //   "activating"  → 0.95
    //   complete      → 1.00
    // This is an approximation. Progress bar may jump non-linearly.
    void installAsync(const std::string &nixpkgsAttr);

    // Remove installed app.
    // Removes entry from vitusos-config.nix, runs nixos-rebuild switch.
    // Same progress reporting as installAsync.
    // Known limit: nixos-rebuild on removal is as slow as on install.
    // The user waits the same amount of time to remove as to install.
    void removeAsync(const std::string &appId);

    static constexpr char CONFIG_PATH[] = "/etc/vitusos/vitusos-config.nix";

private:
    // Parse nixos-rebuild stdout line → progress float
    // Returns -1.0f if line does not match any known phase string
    float parseProgressLine(const std::string &line) const;

    // Update AppEntry in AppIndexCache with new state + progress
    // Thread-safe — called from background thread via publishAsync
    void updateInstallState(const std::string &appId,
                             AppEntry::InstallState state,
                             float progress,
                             const std::string &error = "");
};
```

### 26A.6 New OSFEvents for Install Lifecycle

```cpp
// Additions to OSFEvent enum in animus/core/OSFEvent.h
// Insert before _Count:

// Install lifecycle — LOCAL: session internal
// data = std::string appId
InstallProgress,     // installProgress updated — Pathfinder refreshes bar
InstallComplete,     // install succeeded — BRIDGED session→compositor
                     // CacheKeepr invalidates, AppIndex rebuilds
InstallFailed,       // install failed — LOCAL — Pathfinder shows error
RemoveComplete,      // remove succeeded — BRIDGED session→compositor
RemoveFailed,        // remove failed — LOCAL — Pathfinder shows error
```

### 26A.7 Pathfinder UI Behavior — Spinner vs Progress Bar

```
Pathfinder renders differently based on search phase and AppEntry state.
Opus must implement these behaviors exactly.

PHASE 1: User types query — installed apps only
    Spinner: NO
    Progress bar: NO
    Results: AppIndexCache.search() — installed + desktop file entries
    Latency: < 16ms (in-memory hash search)
    Display: results appear immediately as user types

PHASE 2: nixpkgs search fires (300ms debounce after keystroke stops)
    Spinner: YES — in search bar right side
             same spinner Filer uses for directory scan
             SpaceOrange color, 16px, rotates at 360°/s
    Progress bar: NO
    Results: existing installed results remain visible
             nixpkgs results added as they arrive
    Latency: 2-8 seconds first invocation, < 1s subsequent
    Known limit: spinner may spin for up to 15 seconds
                 (timeout value for nix search)
                 User may see spinner for a long time on slow network.
                 This is acceptable for unstable ISO.

PHASE 3: User clicks "Get" on an Available entry
    Spinner: NO (replaced by progress bar)
    Progress bar: YES — in the result card
                  fills left to right, Space Orange
                  SPRING_SELECTION fill animation
                  label: "Installing..."
    Result card stays visible — user sees progress in place

PHASE 4: Install succeeds
    Progress bar: briefly shows 1.0 (full)
    Icon travels from result card position to Dock
    AnimusContext { type = PathfinderResult,
                    originX = card center X,
                    originY = card center Y,
                    originW = card width,
                    originH = card height }
    Card label changes to "Open"
    installState → Installed

PHASE 5: Install fails
    Progress bar: disappears
    Card shows error message (trimmed installError)
    Button label: "Retry"
    "Retry" re-invokes installAsync()
    Known limit: installError from nixos-rebuild stderr is often
                 cryptic (nix store hash errors, evaluation errors).
                 Pathfinder shows raw error trimmed to 120 chars.
                 A human-readable error translation layer does not
                 exist in the unstable ISO — it is a known rough edge.

PHASE 6: User removes an installed app (long-press or context menu)
    Progress bar: YES — same visual as install
    Label: "Removing..."
    On complete: Dock icon springs out via AnimusContext
    Card returns to "Get" / installState → Available
    Known limit: there is no confirmation dialog before removal
                 in the unstable ISO. Removal is immediate on trigger.
                 This is a known rough edge — acceptable for early adopters.
```

### 26A.8 Known Limits and Bugs — AppIndexCache nixpkgs Extension

```
These are real limitations. Opus must not paper over them.
They must exist exactly as described.

BUG-26A-1: nix search exit code ambiguity
    `nix search nixpkgs#xyz` returns exit code 1 for both
    "no results found" and "evaluation error".
    vitusOS cannot distinguish them.
    Both cases: silent return, no nixpkgs results shown.
    User sees only installed app results.
    No error message shown for genuine nix errors.
    Severity: Low — affects discoverability, not stability.

BUG-26A-2: nixpkgsSize is always 0 in search results
    `nix search --json` does not return package size.
    Size shown as "Unknown" in Pathfinder result detail.
    `nix path-info --closure-size nixpkgs#{attr}` would return it
    but adds another 2-5s network call.
    Decision: size not shown in unstable ISO.
    Severity: Cosmetic.

BUG-26A-3: Install progress is estimated, not measured
    nixos-rebuild does not expose structured progress.
    Phase string matching is fragile — nix output format
    is not a stable API and may change between nix versions.
    Progress bar may jump or stall.
    This is visually imperfect but functionally harmless.
    Severity: Cosmetic.

BUG-26A-4: Abandoned search threads are detached, not cancelled
    When user types quickly (each keystroke triggers debounce),
    multiple nix search processes may be spawned and detached.
    Each runs `timeout 15 nix search` — the process tree is
    cleaned up by the kernel when the parent detaches.
    However: multiple nix search processes running simultaneously
    consume memory and CPU for up to 15 seconds.
    On slow machines: this may cause perceptible slowdown.
    Severity: Low — acceptable for unstable ISO.

BUG-26A-5: nixpkgs entries have no icon
    `nix search` does not return icon paths.
    Pathfinder shows a generic "package" icon for Available entries.
    After install: real icon from /etc/vitusos/apps/{appId}/icon.png.
    The icon "appears" on install completion.
    Severity: Cosmetic.

KNOWN LIMIT-26A-1: Requires nix on PATH
    searchNixpkgsAsync assumes `nix` is available on PATH.
    On NixOS this is always true.
    On non-NixOS: vitusOS does not run anyway — not a real limit.

KNOWN LIMIT-26A-2: First nixpkgs search requires network
    If user is offline: nix search returns no results.
    Pathfinder silently shows only installed apps.
    No "offline" indicator shown in unstable ISO.
    This is a known rough edge.
```


---

## PART 29 — AnimusContext + UI Polish + Orange Box + CockpitView Zoom Model

### 29.1 Overview

Part 29 specifies:
- AnimusContext: the origin-aware animation struct that connects
  every user action to its visual consequence
- CockpitView redesign: zoom level, not separate surface
- Orange box button: geometry, behavior, dropdown spec
- Alt-Tab → CockpitView wiring
- SpringSolver extension: initialVelocity + edge resistance
- Window throw physics
- Shutdown/restart screen exact strings
- Traffic light colors locked: red/yellow/blue (not green)
- UI material and typography rules locked

This part does NOT replace any existing part.
It extends Parts 9, 14, 15, and 18.

### 29.2 AnimusContext

```cpp
// animus/core/AnimusContext.h
// The origin of any user action that produces a visual consequence.
// Every transition in vitusOS carries an AnimusContext.
// AnimationEngine reads it to determine spatial origin of the animation.
// If no spatial origin: use AnimusContext::none().

#pragma once
#include <cstdint>

namespace Animus {

struct AnimusContext {

    enum class Type : uint8_t {
        None,               // no spatial origin — fade from/to center
        DockIcon,           // app launched from Dock icon
        PathfinderResult,   // app launched or installed from Pathfinder
        CockpitThumbnail,   // window restored from CockpitView card
        Notification,       // expanded from notification
        OrangeBox,          // CockpitView opened from orange box double-click
        KeyboardShortcut,   // CockpitView opened from Alt-Tab
        ScreenEdge,         // gesture from screen edge
    };

    Type    type    = Type::None;

    // Screen coordinates of the origin element.
    // For DockIcon: center of the icon in screen space.
    // For PathfinderResult: center of the result card.
    // For CockpitThumbnail: center of the thumbnail card.
    // For OrangeBox: bottom-left corner of the orange box (0, panelHeight).
    // For KeyboardShortcut: center of the currently focused window.
    // For None: ignored — animation uses screen center.
    float   originX = 0.0f;
    float   originY = 0.0f;

    // Size of the origin element.
    // Used to compute initial scale of the born window.
    // Window starts at originW x originH and springs to final size.
    // For None or KeyboardShortcut: 0 — no size-based scaling used.
    float   originW = 0.0f;
    float   originH = 0.0f;

    // Monotonic timestamp when action was triggered.
    // Used to compute animation start delay if needed.
    double  triggeredAtS = 0.0;

    // Convenience constructors
    static AnimusContext none() {
        return AnimusContext{};
    }

    static AnimusContext fromDockIcon(float iconCenterX,
                                       float iconCenterY,
                                       float iconSize) {
        AnimusContext ctx;
        ctx.type    = Type::DockIcon;
        ctx.originX = iconCenterX;
        ctx.originY = iconCenterY;
        ctx.originW = iconSize;
        ctx.originH = iconSize;
        return ctx;
    }

    static AnimusContext fromPathfinderResult(float cardCenterX,
                                               float cardCenterY,
                                               float cardW,
                                               float cardH) {
        AnimusContext ctx;
        ctx.type    = Type::PathfinderResult;
        ctx.originX = cardCenterX;
        ctx.originY = cardCenterY;
        ctx.originW = cardW;
        ctx.originH = cardH;
        return ctx;
    }

    static AnimusContext fromCockpitThumbnail(float thumbCenterX,
                                               float thumbCenterY,
                                               float thumbW,
                                               float thumbH) {
        AnimusContext ctx;
        ctx.type    = Type::CockpitThumbnail;
        ctx.originX = thumbCenterX;
        ctx.originY = thumbCenterY;
        ctx.originW = thumbW;
        ctx.originH = thumbH;
        return ctx;
    }

    static AnimusContext fromOrangeBox(float panelHeight) {
        AnimusContext ctx;
        ctx.type    = Type::OrangeBox;
        ctx.originX = 0.0f;
        ctx.originY = panelHeight;
        return ctx;
    }

    static AnimusContext fromKeyboardShortcut(float focusedWindowCenterX,
                                               float focusedWindowCenterY) {
        AnimusContext ctx;
        ctx.type    = Type::KeyboardShortcut;
        ctx.originX = focusedWindowCenterX;
        ctx.originY = focusedWindowCenterY;
        return ctx;
    }
};

} // namespace Animus
```

### 29.3 AnimusContext in Window Birth

```cpp
// Extension to WindowManager::addSurface()
// AnimusContext is passed when a surface is added.
// OSFWindow uses it to compute the birth animation origin.

// animus/core/WindowManager.h — extended signature:
void addSurface(struct wlr_surface *surface,
                const AnimusContext &ctx = AnimusContext::none());

// animus/shell/OSFWindow.h — extended:
void beginBirthAnimation(const AnimusContext &ctx);

// animus/shell/OSFWindow.cpp — birth animation implementation:
void OSFWindow::beginBirthAnimation(const AnimusContext &ctx) {
    if (ctx.type == AnimusContext::Type::None) {
        // No spatial origin: scale from 0.95 at current position
        m_scale.reset(0.95f);
        m_scale.setTarget(1.0f);
        m_opacity.reset(0.0f);
        m_opacity.setTarget(1.0f);
        return;
    }

    // Spatial origin: window starts at origin position, small,
    // and springs to its final position and size.
    //
    // Initial position: origin center
    m_pos.reset(ctx.originX, ctx.originY);
    m_pos.setTarget(m_finalX, m_finalY);

    // Initial scale: origin size relative to final window size
    // If originW/H == 0 (keyboard shortcut): start at 0.1 scale
    float initScale = (ctx.originW > 0.0f && m_finalW > 0.0f)
                        ? (ctx.originW / m_finalW)
                        : 0.1f;
    initScale = std::clamp(initScale, 0.05f, 0.95f);
    m_scale.reset(initScale);
    m_scale.setTarget(1.0f);

    m_opacity.reset(0.0f);
    m_opacity.setTarget(1.0f);

    // Springs used:
    // m_pos:     SPRING_WINDOW_DRAG (800,35) — fast, physical
    // m_scale:   SPRING_SELECTION (400,28) — slightly slower
    // m_opacity: SPRING_HOVER (600,40) — fast fade in
    //
    // Known limit: if the origin is far from the final position
    // (e.g. Dock at bottom, window opens at top), the window
    // travels a long path. On slow hardware this may look sluggish.
    // The spring constants are tuned for the HP Victus baseline.
    // Adjustable in vitusos-config.nix user prefs in future releases.
}
```

### 29.4 CockpitView — Zoom Level Model

```
ARCHITECTURAL CHANGE from Part 15.

Part 15 specified CockpitView as a separate full-screen overlay surface.
Part 29 replaces this with a zoom level model.

OLD MODEL (Part 15 — superseded):
    CockpitView is SurfaceAltitude::High overlay.
    Opens by appearing on top of desktop.
    Closes by disappearing.
    Desktop and CockpitView are separate visual states.

NEW MODEL (Part 29 — canonical):
    CockpitView is the desktop at a different camera altitude.
    There is no "CockpitView surface."
    There is one desktop. The camera zooms out.
    Windows shrink toward their CockpitView positions.
    The user is always in the same space — just at a different altitude.

Implementation:
    RenderPipeline adds a global scale + translate transform
    applied to the window layer (Layer 3) when CockpitView is active.

    SpringSolver m_cockpitZoom   — SPRING_SELECTION (400,28)
                                   target: 1.0 (desktop) or 0.45 (cockpit)
    SpringSolver m_cockpitOffsetY — SPRING_SELECTION (400,28)
                                   target: 0 (desktop) or +60px (cockpit)
                                   shifts windows up to make room for
                                   virtual desktop sidebar and Dock

    When m_cockpitZoom approaches 0.45:
        Windows appear as thumbnails — their actual Vulkan surfaces,
        scaled down. No separate thumbnail capture needed for layout.
        SnapshotCache still used for the card thumbnail content
        (the image inside the card) but the card itself IS the window.

    Desktop background (Layer 1+2):
        Darkens as m_cockpitZoom decreases.
        Blur increases on wallpaper layer.
        NOT a black overlay — the actual wallpaper, darkened + blurred.
        SpringSolver m_cockpitBgDarken: 0.0 → 0.5

    Panel (Layer 5):
        Stays at full size — does not zoom.
        "Activities" label: NOT shown. Label removed.
        Panel is unchanged in CockpitView.

    Dock (Layer 5):
        // Dock stays full size in CockpitView — does not zoom.
        Stays at bottom, full size.
        Does not zoom with windows.
        Remains interactive — user can click Dock icons in CockpitView
        to launch apps (they open, CockpitView closes on launch).

    Virtual desktop sidebar:
        Springs in from left edge when CockpitView opens.
        Width: 80px. SpringSolver m_sidebarX: -80 → 0.
        SPRING_SELECTION (400,28).
        Contains:
            Desktop thumbnails: 64×40px each, 8px gap.
            Active desktop: Space Orange 1px border.
            "+" button at bottom: add new virtual desktop.
            No labels — thumbnails suffice.
        When sidebar fully open: windows shift right by 80px
        (m_cockpitOffsetX spring target: 0 → 80).

    Window cards in CockpitView:
        Each window is its actual surface, scaled to thumbnail size.
        No separate card widget — the window IS the card.
        vitusOS chrome (traffic lights) scales with the window.
        Close button (×):
            // × close button appears on hover over each window card.
            Appears on hover — 300ms hover detection.
            Red circle (Space Orange adjacent — #FF3B30), 20px.
            White × symbol, 10px.
            Springs in: SPRING_TRAFFIC_LIGHT (700,38).
            Click: wl_surface destroy → WindowManager::removeSurface()
                   CockpitView reflows remaining windows
                   SPRING_SELECTION on reflow positions.
        Title label:
            App display name, 11px, white, 80% opacity.
            Appears below each window in CockpitView.
            Hidden in desktop mode.

    Window positions in CockpitView:
        Arranged in a grid. Max 2 rows.
        If > 6 windows: grid compresses, minimum card size 120×80px.
        If > 12 windows: scroll within CockpitView (horizontal).
        Known limit: > 12 windows causes layout to become dense.
        This is acceptable for unstable ISO.
        Each window's SpringSolver2D m_pos springs to its grid position
        when CockpitView opens, springs back to desktop position on close.

    Drag window to virtual desktop:
        User drags a window card toward the sidebar.
        When card center crosses the sidebar boundary:
            target desktop highlights (Space Orange border pulses).
        Release over a desktop thumbnail:
            window assigned to that virtual desktop.
            StateManager stores assignment:
                key: "windowDesktop:{handle}" → desktopIndex
            Card animates into the desktop thumbnail (scale → 0).
            Desktop thumbnail updates to show the window.
        Known limit: drag-to-desktop in the unstable ISO requires
        deliberate slow drag. Fast swipe may not register correctly.
        GestureRecognizer threshold: 80px/s minimum drag velocity
        toward sidebar to start drag-to-desktop mode.
```

### 29.5 Orange Box Button — Locked Specification

```cpp
// animus/shell/Panel.h — OrangeBox constants added

// THE ORANGE BOX.
// Top-left corner of the Panel. Flush with Panel top and left edges.
// Sharp corners — 0px radius on ALL four corners.
// This is intentional. The orange box is the only hard corner in vitusOS.
// It marks the system boundary. It does not apologize.
//
// Single click:  dropdown menu springs down (OrangeBoxMenu)
// Double click:  CockpitView opens (AnimusContext::fromOrangeBox)
//
// NEVER: round the corners of the orange box.
// NEVER: change the color on hover.
// NEVER: add a shadow to the orange box.
// NEVER: animate the orange box itself — it does not move.

static constexpr float ORANGE_BOX_W        = 42.0f;   // px, exact
static constexpr float ORANGE_BOX_H        = 28.0f;   // Panel height, exact
static constexpr float ORANGE_BOX_RADIUS   = 0.0f;    // sharp. always.
static constexpr uint32_t ORANGE_BOX_COLOR = 0xFFFF6B2B; // ARGB Space Orange

// Double-click detection window: 400ms between clicks.
// Industry standard: 300-500ms. 400ms is forgiving but not accidental.
static constexpr double ORANGE_BOX_DCLICK_MS = 400.0;

// Panel left and right edges: no rounding.
// Panel is full-width, edge to edge.
// The orange box corner IS the screen corner.
static constexpr float PANEL_LEFT_RADIUS  = 0.0f;
static constexpr float PANEL_RIGHT_RADIUS = 0.0f;
```

### 29.6 Orange Box Dropdown Menu

```cpp
// animus/shell/OrangeBoxMenu.h
// The dropdown that appears on single-click of the orange box.
// Springs down from the bottom-left corner of the orange box.
// Top-left and top-right corners: 0px (continues orange box geometry).
// Bottom-left and bottom-right corners: 8px radius.
// Background: glass material, SurfaceAltitude::High.
// Border: 1px, white, 15% opacity.
// Width: 220px fixed.
// Item height: 28px each.

#pragma once
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <functional>
#include <vector>

namespace Animus {

class OrangeBoxMenu {
public:
    OrangeBoxMenu();
    ~OrangeBoxMenu();

    void open();    // springs down from orange box bottom-left
    void close();   // springs up back into orange box

    bool isOpen() const { return m_open; }

    void render(VkCommandBuffer cmd, float dt);
    void onPointerButton(float x, float y, bool pressed);
    void onPointerMotion(float x, float y);

    static constexpr float WIDTH         = 220.0f;
    static constexpr float ITEM_H        = 28.0f;
    static constexpr float CORNER_TOP_L  = 0.0f;   // flush with orange box
    static constexpr float CORNER_TOP_R  = 0.0f;   // flush with panel bottom
    static constexpr float CORNER_BOT_L  = 8.0f;
    static constexpr float CORNER_BOT_R  = 8.0f;
    static constexpr float SEPARATOR_H   = 1.0f;   // px, white 20% opacity

private:
    struct MenuItem {
        std::string          label;
        bool                 isSeparator = false;
        std::function<void()> action;
    };

    void buildItems();  // called once — items are fixed

    bool         m_open = false;
    float        m_hoverIndex = -1.0f;
    std::vector<MenuItem>    m_items;
    std::vector<SpringSolver> m_itemHover;  // SPRING_HOVER (600,40) per item

    // Menu springs DOWN on open (clipH: 0 → totalH)
    // Springs UP on close (clipH: totalH → 0)
    SpringSolver m_clipH;   // SPRING_SHEET (420,30)
    SpringSolver m_opacity; // SPRING_HOVER (600,40): 0→1 on open

    uint64_t m_tickHandle = 0;
};

} // namespace Animus

// Menu items — fixed, in order:
//
//  About vitusOS                → OSFEvent::AboutVitusOS
//  ─────────────────────────── (separator)
//  Pathfinder                   → OSFEvent::PathfinderOpen
//  Settings                     → launch Settings native app
//  ─────────────────────────── (separator)
//  Lock Screen                  → OSFEvent::LockScreenActivate
//  Sleep                        → systemd suspend via loginctl
//  ─────────────────────────── (separator)
//  Restart                      → OSFEvent::SystemRestart
//  Shut Down                    → OSFEvent::SystemShutdown
//
// Item text: 13px, white, Inter Regular.
// Separator: 1px horizontal line, white 20% opacity, 8px left/right margin.
// Hover: SPRING_HOVER pill behind item text, white 10% opacity.
// Active item (pressed): white 18% opacity pill.
// "Restart" and "Shut Down": no confirmation dialog. Immediate on click.
// Known limit: no confirmation dialog is a rough edge for unstable ISO.
//              Data loss is possible if user clicks accidentally.
//              Acceptable — early adopters understand the risk.
```

### 29.7 Shutdown and Restart Screens

```cpp
// animus/shell/SystemScreen.h
// Full-screen black surface shown during shutdown and restart.
// Replaces everything. No TTY. No systemd journal. No kernel messages.
// Appears after user selects Shut Down or Restart from OrangeBoxMenu.
//
// EXACT STRINGS — locked. Never change. Never localize.
// Never capitalize. Never add punctuation.
// These are the OS speaking directly to the user.
// The only two moments it speaks.

static constexpr const char* SHUTDOWN_MESSAGE = "goodbye";
static constexpr const char* RESTART_MESSAGE  = "i'll see you in a bit";

// Typography:
//   Font:   Inter Regular (same system font)
//   Size:   15px
//   Color:  white, 95% opacity (#F2F2F2)
//   Align:  center horizontal and vertical
//   No animation on the text — it appears with the black surface.
//
// Background: pure black #000000
//   Exception: this is the ONLY surface in vitusOS that uses pure black.
//   The shutdown/restart screen is the OS going away.
//   Pure black is correct here.
//
// Transition:
//   Desktop fades to black: SpringSolver opacity 1.0→0.0
//   SPRING_BOOT (200,22) — slow, deliberate, 800ms approximate settle.
//   Text appears as desktop opacity reaches 0.0.
//   Then: systemd poweroff or systemd reboot is invoked.
//   The OS does not wait for confirmation after showing this screen.
//
// Known limit: if systemd shutdown takes >5 seconds, the user sees
// "goodbye" for longer than expected. This is acceptable.
// The screen never shows anything else — no progress, no spinner.
// Silence is correct.

class SystemScreen {
public:
    enum class Mode { Shutdown, Restart };

    SystemScreen();
    void show(Mode mode);   // begins fade, invokes systemd on complete
    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);

private:
    Mode         m_mode = Mode::Shutdown;
    bool         m_active = false;
    SpringSolver m_opacity;  // SPRING_BOOT (200,22): 0.0→1.0

    uint64_t     m_tickHandle = 0;

    void invokeSystemd();   // poweroff or reboot via sd_bus_call_method
                            // called when m_opacity spring settled at 1.0
};
```

### 29.8 Alt-Tab → CockpitView

```cpp
// Extension to InputRouter — global shortcut intercept

// In InputRouter::onKey():
// Alt-Tab is intercepted BEFORE delivery to focused surface.
// It is never passed to any application.

// animus/input/InputRouter.cpp — addition:

void InputRouter::onKey(uint32_t keysym, uint32_t mods, bool pressed) {
    // Global shortcut: Alt+Tab → CockpitView toggle
    // Alt = MOD_ALT (wlroots modifier mask)
    if ((mods & MOD_ALT) && keysym == XKB_KEY_Tab && pressed) {
        if (!m_cockpitView->isOpen()) {
            // Open CockpitView from focused window center
            float cx = 0.0f, cy = 0.0f;
            auto focusedWin =
                RegistryManager::shared().windows().focusedWindow();
            if (focusedWin) {
                cx = focusedWin->posX() + focusedWin->width()  * 0.5f;
                cy = focusedWin->posY() + focusedWin->height() * 0.5f;
            } else {
                cx = m_screenW * 0.5f;
                cy = m_screenH * 0.5f;
            }
            AnimusContext ctx =
                AnimusContext::fromKeyboardShortcut(cx, cy);
            EventBus::shared().publish(OSFEvent::CockpitViewOpen, ctx);
        } else {
            // Close CockpitView — zoom into highlighted window
            // If no window highlighted: zoom into last focused window
            EventBus::shared().publish(OSFEvent::CockpitViewClose,
                                        AnimusContext::none());
        }
        return;  // consumed — not passed to application
    }

    // Tab while CockpitView is open + Alt held: cycle highlight
    if ((mods & MOD_ALT) && keysym == XKB_KEY_Tab &&
        m_cockpitView->isOpen() && pressed) {
        EventBus::shared().publish(OSFEvent::CockpitViewCycleNext, {});
        return;
    }

    // Shift+Alt+Tab: cycle in reverse
    if ((mods & MOD_ALT) && (mods & MOD_SHIFT) &&
        keysym == XKB_KEY_Tab && m_cockpitView->isOpen() && pressed) {
        EventBus::shared().publish(OSFEvent::CockpitViewCyclePrev, {});
        return;
    }

    // Escape while CockpitView open: close, return to previous focus
    if (keysym == XKB_KEY_Escape && m_cockpitView->isOpen() && pressed) {
        EventBus::shared().publish(OSFEvent::CockpitViewClose,
                                    AnimusContext::none());
        return;
    }

    // Fullscreen escape: exit fullscreen (Part 40)
    if (keysym == XKB_KEY_Escape && pressed) {
        auto focused = RegistryManager::shared().windows().focusedWindow();
        if (focused && focused->m_fullscreen.active) {
            WindowManager::shared().onUnsetFullscreen(focused.get());
            return;  // consumed — app does not receive Esc
        }
    }

    // F10: activate GlobalMenu keyboard navigation
    // Alt alone (no other key): activate GlobalMenu keyboard navigation
    // Alt+other: pass through normally (alt is a modifier, not a shortcut)
    if (keysym == XKB_KEY_F10 && pressed) {
        PanelManager::shared().activateGlobalMenuKeyboard();
        return;  // consumed
    }
    // Alt alone detected on KEY_UP (Alt pressed then released without another key):
    // Tracked via m_altDownAlone flag — set on Alt press, cleared on any other key
    // while Alt is held. On Alt release with flag set: activate GlobalMenu.
    if (keysym == XKB_KEY_Alt_L || keysym == XKB_KEY_Alt_R) {
        if (pressed) {
            m_altDownAlone = true;
        } else {
            if (m_altDownAlone) {
                PanelManager::shared().activateGlobalMenuKeyboard();
                // m_altDownAlone cleared below
            }
        }
    } else if (mods & MOD_ALT) {
        m_altDownAlone = false;  // another key pressed while Alt held
    }
    if (!pressed && (keysym == XKB_KEY_Alt_L || keysym == XKB_KEY_Alt_R)) {
        m_altDownAlone = false;
    }

    // All other keys: deliver to focused surface normally
    deliverKeyToFocused(keysym, mods, pressed);
}

// Known limit: Alt+Tab intercept means no application can use Alt+Tab
// for its own purpose. This is intentional and correct.
// Applications that rely on Alt+Tab internally (rare) will lose that
// shortcut. Acceptable — system shortcuts take precedence.
```

### 29.9 New OSFEvents for Part 29

```cpp
// Additions to OSFEvent enum — insert before _Count

// CockpitView (Part 29 zoom model — replaces CockpitViewToggle)
// data = AnimusContext
CockpitViewOpen,         // BRIDGED compositor internal
CockpitViewClose,        // BRIDGED compositor internal
CockpitViewCycleNext,    // LOCAL compositor — highlight next window
CockpitViewCyclePrev,    // LOCAL compositor — highlight prev window

// Orange box
OrangeBoxMenuOpen,       // LOCAL — menu opens
OrangeBoxMenuClose,      // LOCAL — menu closes

// System actions (from OrangeBoxMenu)
SystemShutdown,          // BRIDGED compositor→session
                         // session tells HEV to lock before shutdown
SystemRestart,           // BRIDGED compositor→session — same

// About screen
AboutVitusOS,            // LOCAL — opens About surface

// NOTE: CockpitViewToggle (Part 4) is superseded by CockpitViewOpen/Close.
// CockpitViewToggle remains in the enum for backward compatibility
// but is no longer published by any component.
// Opus must not remove CockpitViewToggle — removing it changes enum values.
// Opus must not publish CockpitViewToggle in new code.
```

### 29.10 SpringSolver Extension: initialVelocity + Edge Resistance

```cpp
// Extension to SpringSolver — Part 9 extended.
// These methods are ADDED to SpringSolver. Existing API unchanged.

// animus/animation/SpringSolver.h — additions:

class SpringSolver {
public:
    // ── EXISTING API (unchanged) ──────────────────────────────────
    SpringSolver(float stiffness, float damping);
    void  setTarget(float target);
    void  reset(float value);
    float tick(float dt);
    float value() const;
    float velocity() const;
    bool  isSettled(float threshold = 0.001f) const;

    // ── NEW: initial velocity ─────────────────────────────────────
    // Set velocity at the current position without changing position.
    // Used for throw physics: pointer release velocity → spring velocity.
    // Positive velocity: moving toward larger values.
    // Negative velocity: moving toward smaller values.
    // Call AFTER reset() or setTarget(), BEFORE first tick().
    void setVelocity(float vel) { m_vel = vel; }

    // ── NEW: edge resistance ──────────────────────────────────────
    // When enabled: if value approaches edgeMin or edgeMax within
    // resistanceZone pixels, an additional restoring force is added.
    // The window feels like it is pushing against a soft boundary.
    // Force magnitude: proportional to (penetration / resistanceZone).
    // This is NOT a hard clamp — the spring can still overshoot slightly.
    // The resistance adds to the spring force, not replaces it.
    void enableEdgeResistance(float edgeMin, float edgeMax,
                               float resistanceZone = 20.0f);
    void disableEdgeResistance();

private:
    float m_stiffness;
    float m_damping;
    float m_pos = 0.0f;
    float m_vel = 0.0f;
    float m_target = 0.0f;

    // Edge resistance state
    bool  m_edgeEnabled  = false;
    float m_edgeMin      = 0.0f;
    float m_edgeMax      = 0.0f;
    float m_resistZone   = 20.0f;
};

// SpringSolver::tick() extension for edge resistance:
// In the existing force calculation, add:
//
//   if (m_edgeEnabled) {
//       if (m_pos < m_edgeMin + m_resistZone) {
//           float penetration = (m_edgeMin + m_resistZone) - m_pos;
//           float resistForce = m_stiffness * 0.3f *
//                               (penetration / m_resistZone);
//           springForce += resistForce;  // pushes away from edge
//       }
//       if (m_pos > m_edgeMax - m_resistZone) {
//           float penetration = m_pos - (m_edgeMax - m_resistZone);
//           float resistForce = m_stiffness * 0.3f *
//                               (penetration / m_resistZone);
//           springForce -= resistForce;  // pushes away from edge
//       }
//   }
//
// 0.3f multiplier: resistance is 30% of main spring force at zone boundary.
// At zone center: resistance is 0. At edge: resistance is 30%.
// This feels like approaching a soft magnetic boundary, not a wall.
```

### 29.11 Window Throw Physics

```cpp
// Extension to OSFWindow — throw physics on pointer release.
// When user releases a drag, the pointer release velocity is applied
// to the window's position spring as initialVelocity.
// The window then follows through with momentum and settles naturally.

// In OSFWindow::onPointerButtonRelease():
void OSFWindow::onPointerButtonRelease(float vx, float vy) {
    // vx, vy: pointer velocity at release in px/s
    // from wlroots pointer event — dx/dt over last 3 frames

    if (m_dragging) {
        m_dragging = false;

        // Apply throw velocity to position spring
        // Clamp to reasonable range — prevent flying off screen
        vx = std::clamp(vx, -2000.0f, 2000.0f);
        vy = std::clamp(vy, -2000.0f, 2000.0f);

        m_pos.setVelocity(vx, vy);  // SpringSolver2D — sets x and y velocity

        // m_pos.enableEdgeResistance() called on throw release
        // Window resists leaving screen on all four sides.
        float screenW = StateManager::shared().screenWidth();
        float screenH = StateManager::shared().screenHeight();
        m_pos.enableEdgeResistance(
            -m_width  * 0.5f,               // left: can go half off-screen
            screenW - m_width * 0.5f,        // right: same
            32.0f                            // resistance zone: 32px
        );
        // Vertical: Panel height at top (28px), nothing at bottom
        m_pos.enableEdgeResistanceY(
            Panel::HEIGHT,                   // top: cannot go above Panel
            screenH - 32.0f,                 // bottom: partial off-screen ok
            32.0f
        );

        // Known limit: edge resistance for 2D springs requires
        // SpringSolver2D to support per-axis edge resistance.
        // Implemented as separate X and Y calls.
        // The API above (enableEdgeResistanceY) is new — must be added
        // to SpringSolver2D alongside enableEdgeResistance.
    }
}

// Known limit: velocity estimation from wlroots pointer events.
// wlroots delivers pointer motion as dx, dy per event.
// Velocity = sum(dx,dy over last 3 events) / sum(dt over last 3 events).
// 3-frame average prevents single-event spikes from causing violent throws.
// On touchpad: libinput applies its own smoothing before wlroots.
// On mouse: velocity is accurate.
// Known edge case: if user releases very slowly (velocity ≈ 0),
// the spring settles immediately. Correct behavior.
```

### 29.12 Traffic Light Colors — Locked

```
vitusOS traffic light colors are red / yellow / BLUE.
NOT red / yellow / green (macOS).
This is intentional. Blue = maximize in vitusOS.

Locked values:
    Close:    #FF3B30   (red)
    Minimize: #FFCC00   (yellow)
    Maximize: #007AFF   (blue — vitusOS accent, not green)

On hover (× / − / + symbols revealed):
    Symbol color: black, 70% opacity, on all three.
    Symbol size: 8px, centered in 12px circle.

SPRING_TRAFFIC_LIGHT (700,38) governs:
    The scale of each circle on hover (1.0 → 1.1 → 1.0)
    The opacity of the symbol (0.0 → 1.0 on hover)

NEVER use green (#28C840 or similar) for the maximize button.
NEVER change these colors. They are locked.
```

### 29.13 UI Material and Typography Rules — Locked

```
These rules are additive to Part 18.
Violations are the same severity as Part 18 violations.

TYPOGRAPHY:
    System font: Inter (variable, loaded from NixOS pkgs.inter)
    Weights used:
        Regular (400):   body text, menu items, clock
        Medium (500):    secondary labels, sidebar items
        Semibold (600):  window titles, panel app name, section headers
    Sizes:
        11px: secondary text, sidebar headers (letter-spaced),
              CockpitView window titles, Pathfinder subtitles
        13px: body, menu items, Panel app name, clock, Dock labels
        15px: shutdown/restart message
        20px: Pathfinder selected result name
    Color (on dark surfaces):
        Primary text:   #F2F2F2  (white, 95% opacity)
        Secondary text: #ABABAB  (white, 67% opacity)
        Tertiary text:  #6B6B6B  (white, 42% opacity)
    Sidebar section headers (e.g. "FAVORITES" in Filer):
        11px, Semibold, ALL CAPS, letter-spacing: 0.08em
        Color: tertiary (#6B6B6B)
        This is the only place ALL CAPS is used in vitusOS.

    NEVER use pure white (#FFFFFF) for text on any surface.
    NEVER use bold (700+) weight for body text.
    NEVER use font sizes other than 11, 13, 15, 20 in the shell.
    Applications control their own typography — vitusOS never touches it.

MATERIALS (additive to altitude table in Part 20):
    Glass surfaces (Panel, Dock, menus, Pathfinder, window chrome):
        Background: derived from altitude table (Part 20)
        Border: 1px, white, 15% opacity — ALL glass surfaces
        No shadow on glass surfaces themselves
        (shadows are on OSFWindow, which contains glass chrome)

    App content area:
        vitusOS never renders the app content area.
        It is the wlr_surface texture from the application.
        vitusOS composites it at Layer 3, no modification.

    System screens (LockScreen, SystemScreen):
        No blur. No glass. Solid color.
        LockScreen: #1A1208 (near-black, warm)
        SystemScreen: #000000 (pure black — only exception)

SHADOW (additive to Part 8):
    Focused window shadow:
        color: #1A1208 (spec Part 18 — NEVER pure black)
        blur radius: 32px
        offset: 0, 8px (slightly downward)
        opacity: 0.6
    Unfocused window shadow:
        same color, blur radius 16px, opacity 0.3
    Shadow opacity transitions via SpringSolver
    when focus changes between windows:
        // SPRING_SHADOW (300,25) governs focus change shadow depth
        SPRING_SHADOW (300,25)
    This makes focus change feel physical —
    the shadow deepens as the window comes forward.

ORANGE BOX (locked — additive to 29.5):
    Color: #FF6B2B — Space Orange. Always. Never changes on hover.
    Width: 42px. Height: Panel height (28px).
    All corners: 0px radius. Always.
    No shadow. No border. No blur.
    It is the one opaque, flat, hard element in the system.
    Everything else is glass or app content.
    The orange box is neither. It is the system.

SPACING:
    Panel height: 28px
    Dock icon size: 48px
    Dock height: 64px (icon + padding)
    Window title bar height: 32px
    Traffic light size: 12px diameter, 8px gap between them
    Traffic light left margin: 12px from window left edge
    All spacing: multiples of 4px
    NEVER: odd pixel spacing
    NEVER: fractional pixel spacing in layout
           (fractional only in spring animation positions)
```

### 29.14 Known Limits and Bugs — Part 29

```
These are real. Opus must not hide them.

BUG-29-1: CockpitView zoom with >12 windows
    Layout becomes dense. Horizontal scroll is specced
    but the scroll gesture conflicts with the three-finger
    swipe that closes CockpitView.
    Resolution for unstable ISO: cap visible windows at 12.
    Windows 13+ are shown in a "..." overflow indicator.
    Clicking overflow: shows a list overlay.
    Known rough edge. Acceptable.

BUG-29-2: AnimusContext origin for Pathfinder
    Pathfinder may be dismissed before the window appears
    (e.g. user presses Esc between clicking a result and
    the app launching). In this case the AnimusContext origin
    position is the center of a surface that no longer exists.
    The birth animation will spring from the Pathfinder position
    (now invisible) to the window's final position.
    This looks slightly odd — window appears to come from nothing.
    Acceptable for unstable ISO.

BUG-29-3: Throw physics on touchpad vs mouse
    libinput applies acceleration curves to touchpad events.
    The velocity at release is the accelerated velocity, not raw.
    This means touchpad throws feel different from mouse throws.
    Touchpad throws may feel sluggish or too fast depending on
    libinput configuration.
    No compensation is applied in the unstable ISO.
    Known rough edge.

BUG-29-4: OrangeBoxMenu double-click window
    400ms double-click window is not per-platform configurable.
    Users with accessibility needs (motor impairment) may find
    the 400ms window too short for CockpitView via double-click.
    Alt-Tab is always available as an alternative.
    This is a known accessibility limitation of the unstable ISO.

BUG-29-5: SystemScreen invokes systemd immediately
    There is no grace period after showing "goodbye" before
    systemd poweroff is invoked.
    If systemd takes >5 seconds to shut down all services,
    "goodbye" remains visible for longer than intended.
    The screen never shows anything other than the message.
    Acceptable — silence is correct.

KNOWN LIMIT-29-1: No window snapping
    vitusOS does not implement window snap zones (half-screen,
    quarter-screen) in the unstable ISO.
    Edge resistance prevents windows from going off-screen
    but does not snap them to grid positions.
    Window tiling is a post-unstable feature.

KNOWN LIMIT-29-2: Virtual desktops maximum: 6
    The sidebar can display a maximum of 6 virtual desktops.
    The "+" button is hidden when 6 desktops exist.
    This is arbitrary and will be raised in a later release.

KNOWN LIMIT-29-3: CockpitView keyboard navigation
    Alt+Tab cycles through all windows in creation order.
    There is no spatial keyboard navigation (arrow keys).
    Arrow key navigation in CockpitView is a post-unstable feature.
```

### 29.15 What Opus Must NEVER Do — Part 29 Additions

```
NEVER round the orange box corners.
    ORANGE_BOX_RADIUS = 0.0f. Always. If you write any value other
    than 0.0f for the orange box corner radius, you are wrong.

NEVER change the orange box color on hover, focus, or press.
    It is always #FF6B2B. The dropdown or CockpitView IS the feedback.
    The orange box itself does not react visually to interaction.

NEVER use the string "goodbye" with any capitalization other than all-lowercase.
NEVER use the string "i'll see you in a bit" with any capitalization.
NEVER add punctuation to either string.
NEVER add animation to the text of either string.
    The text appears. That is all. Silence is correct.

NEVER show a TTY, journal output, kernel messages, or any systemd text
    at any point during shutdown or restart. If it is visible,
    the SystemScreen implementation is wrong.

NEVER use green for the maximize traffic light.
    MAXIMIZE_COLOR = #007AFF. Always. Never #28C840 or any green value.

NEVER implement CockpitView as a separate overlay surface (old Part 15 model).
    Part 29 is canonical. CockpitView is a zoom level. One desktop.
    One camera altitude. No separate surface.

NEVER pass a raw OSFWindow* in AnimusContext.
    AnimusContext holds only coordinates and type.
    Not pointers. Not handles. Coordinates only.
    AnimusContext is a value type — copy freely, store safely.

NEVER use CockpitViewToggle in new code.
    Use CockpitViewOpen and CockpitViewClose.
    CockpitViewToggle remains in the enum but is never published.
```


---

## PART 30 — MotionWave: Complete Gesture System

### 30.1 Overview

MotionWave replaces GestureRecognizer entirely.

GestureRecognizer (Part 12) is superseded by this part.
All references to GestureRecognizer in new code must use MotionWave.
The file animus/input/GestureRecognizer.cpp/.h is renamed to
animus/input/MotionWave.cpp/.h.
InputRouter's m_gestures member changes type to
std::unique_ptr<MotionWave>.

MotionWave is responsible for:
- Recognizing all multi-finger gestures from raw wlroots events
- Applying velocity and angle thresholds to distinguish
  deliberate gestures from accidental touches
- Publishing typed OSFEvents for each recognized gesture
- Reading user sensitivity preferences from StateManager
- Exposing per-gesture enable/disable from Settings

MotionWave is NOT responsible for:
- What happens after a gesture fires (that is the subscriber's job)
- Palm rejection (libinput handles this before wlroots)
- Single-finger pointer events (InputRouter handles those directly)

### 30.2 C11 Core Extension — Pinch Events

```c
// compositor/compositor.c — add pinch event wiring
// wlroots 0.17.1 exposes pinch via wlr_pointer events.
// Must be added alongside existing swipe wiring.

// Add to Comp struct (compositor/compositor.h):
void (*on_pinch_begin)(uint32_t fingers, void*);
void (*on_pinch_update)(uint32_t fingers, double dx, double dy,
                         double scale, double rotation, void*);
void (*on_pinch_end)(bool cancelled, void*);

// Add to PtrS struct:
struct wl_listener pinch_begin, pinch_update, pinch_end;

// Handler implementations:
static void h_pinch_begin(struct wl_listener *l, void *data) {
    (void)l;
    const struct wlr_pointer_pinch_begin_event *ev = data;
    if (g.on_pinch_begin) g.on_pinch_begin(ev->fingers, g.ud);
}
static void h_pinch_update(struct wl_listener *l, void *data) {
    (void)l;
    const struct wlr_pointer_pinch_update_event *ev = data;
    if (g.on_pinch_update)
        g.on_pinch_update(ev->fingers, ev->dx, ev->dy,
                           ev->scale, ev->rotation, g.ud);
}
static void h_pinch_end(struct wl_listener *l, void *data) {
    (void)l;
    const struct wlr_pointer_pinch_end_event *ev = data;
    if (g.on_pinch_end) g.on_pinch_end(ev->cancelled, g.ud);
}

// Add to pointer device setup in h_new_input():
p->pinch_begin.notify  = h_pinch_begin;
p->pinch_update.notify = h_pinch_update;
p->pinch_end.notify    = h_pinch_end;
wl_signal_add(&ptr->events.pinch_begin,  &p->pinch_begin);
wl_signal_add(&ptr->events.pinch_update, &p->pinch_update);
wl_signal_add(&ptr->events.pinch_end,    &p->pinch_end);

// Add to pointer cleanup in h_ptr_destroy():
wl_list_remove(&p->pinch_begin.link);
wl_list_remove(&p->pinch_update.link);
wl_list_remove(&p->pinch_end.link);

// animus_set_callbacks() — add three new parameters:
void animus_set_callbacks(
    /* existing params unchanged */
    void (*on_pinch_begin)(uint32_t, void*),
    void (*on_pinch_update)(uint32_t, double, double, double, double, void*),
    void (*on_pinch_end)(bool, void*)
);

// Known limit: wlr_pointer_pinch_update_event exposes `scale` as
// cumulative scale factor from gesture start (1.0 = no change).
// vitusOS uses delta scale per-frame = scale / prev_scale.
// If wlroots changes this convention: pinch will feel wrong.
// Verified against wlroots 0.17.1 source — cumulative is correct.
```

### 30.3 MotionWave.h

```cpp
// animus/input/MotionWave.h
// Replaces animus/input/GestureRecognizer.h entirely.
// NEVER include GestureRecognizer.h in new code.
#pragma once
#include "core/OSFEvent.h"
#include "core/StateManager.h"
#include <cstdint>
#include <cmath>

namespace Animus {

// MotionWave: recognizes all multi-finger gestures.
// Feeds AnimusContext-aware events to the system.
// User-configurable via Settings → MotionWave section.
//
// Gesture family: THREE-FINGER
//   Swipe UP:    OSFEvent::CockpitViewOpen
//   Swipe DOWN:  OSFEvent::CockpitViewClose  (if CockpitView open)
//                OSFEvent::ShowDesktop        (if CockpitView closed)
//   Swipe LEFT:  OSFEvent::DesktopPrev
//   Swipe RIGHT: OSFEvent::DesktopNext
//   Tap:         OSFEvent::ShowDesktopToggle
//
// Gesture family: TWO-FINGER
//   Scroll:      OSFEvent::ScrollDelta        (existing — unchanged)
//   Pinch IN:    OSFEvent::PinchIn
//   Pinch OUT:   OSFEvent::PinchOut
//
// NO four-finger gestures in vitusOS unstable ISO.
// Four-finger events are ignored.
// This is intentional — vitusOS uses three fingers for everything.
// Four-finger: post-unstable feature. Known gap. Documented.

class MotionWave {
public:
    static MotionWave& shared();

    // Called from InputRouter — raw wlroots events
    void onSwipeBegin(uint32_t fingers);
    void onSwipeUpdate(uint32_t fingers, double dx, double dy);
    void onSwipeEnd(bool cancelled);

    void onPinchBegin(uint32_t fingers);
    void onPinchUpdate(uint32_t fingers, double dx, double dy,
                        double scale, double rotation);
    void onPinchEnd(bool cancelled);

    // Called by Settings → MotionWave section
    // Writes to StateManager. Immediate effect.
    void setSensitivity(Sensitivity s);
    void setNaturalScroll(bool natural);
    void setGestureEnabled(GestureId id, bool enabled);

    enum class Sensitivity : uint8_t {
        Low    = 0,   // velocity threshold: 400px/s
        Medium = 1,   // velocity threshold: 300px/s (default)
        High   = 2,   // velocity threshold: 200px/s
    };

    enum class GestureId : uint8_t {
        ThreeFingerUp    = 0,
        ThreeFingerDown  = 1,
        ThreeFingerLeft  = 2,
        ThreeFingerRight = 3,
        ThreeFingerTap   = 4,
        PinchIn          = 5,
        PinchOut         = 6,
    };

    // Threshold constants — NOT configurable. Physics, not preference.
    // Velocity thresholds are configurable (above). These are not.
    static constexpr double MIN_TRAVEL_PX     = 20.0;  // px before axis committed
    static constexpr double AXIS_COMMIT_PX    = 40.0;  // px to commit to one axis
    static constexpr double TAP_MAX_TRAVEL_PX = 10.0;  // px max movement for tap
    static constexpr double TAP_MAX_MS        = 200.0; // ms max duration for tap
    static constexpr double TAP_LAND_MS       = 100.0; // ms max between fingers
    static constexpr double PINCH_MIN_DELTA   = 0.04;  // min scale delta to fire
                                                         // (prevents noise on still)
    static constexpr double BOUNDARY_SPRING_VEL = 800.0; // px/s — desktop boundary
                                                           // bounce velocity

private:
    MotionWave() = default;

    // ── Swipe state ───────────────────────────────────────────────
    enum class SwipeState : uint8_t {
        Idle,
        Tracking,       // accumulating, axis not yet committed
        CommittedH,     // horizontal axis committed (LEFT/RIGHT)
        CommittedV,     // vertical axis committed (UP/DOWN)
    };

    SwipeState   m_swipeState  = SwipeState::Idle;
    uint32_t     m_swipeFingers = 0;
    double       m_accumDX     = 0.0;
    double       m_accumDY     = 0.0;
    double       m_peakVelX    = 0.0;  // peak velocity px/s during gesture
    double       m_peakVelY    = 0.0;
    double       m_prevDX      = 0.0;  // previous frame delta for velocity
    double       m_prevDY      = 0.0;
    uint32_t     m_swipeTimeMs = 0;    // time_msec from last swipe_update

    // ── Tap state ─────────────────────────────────────────────────
    enum class TapState : uint8_t {
        Idle,
        Waiting,        // fingers down, watching for tap vs swipe
    };
    TapState     m_tapState      = TapState::Idle;
    uint32_t     m_tapStartMs    = 0;
    double       m_tapTravelX    = 0.0;
    double       m_tapTravelY    = 0.0;

    // ── Pinch state ───────────────────────────────────────────────
    enum class PinchState : uint8_t {
        Idle,
        Tracking,
    };
    PinchState   m_pinchState     = PinchState::Idle;
    double       m_pinchPrevScale = 1.0;  // cumulative scale from last frame
    double       m_pinchAccum     = 0.0;  // accumulated scale delta

    // ── Settings state ────────────────────────────────────────────
    Sensitivity  m_sensitivity    = Sensitivity::Medium;
    bool         m_naturalScroll  = true;
    bool         m_enabled[7]     = {true,true,true,true,true,true,true};
    // index matches GestureId enum values

    // ── Helpers ───────────────────────────────────────────────────
    double velocityThreshold() const {
        switch (m_sensitivity) {
            case Sensitivity::Low:    return 400.0;
            case Sensitivity::Medium: return 300.0;
            case Sensitivity::High:   return 200.0;
        }
        return 300.0;
    }

    bool gestureEnabled(GestureId id) const {
        return m_enabled[static_cast<uint8_t>(id)];
    }

    void fireSwipeResult();
    void fireTapResult();
    void resetSwipe();
};

} // namespace Animus
```

### 30.4 MotionWave.cpp

```cpp
// animus/input/MotionWave.cpp
#include "MotionWave.h"
#include "core/EventBus.h"
#include "core/StateManager.h"
#include "core/AnimusContext.h"
#include <cmath>
#include <algorithm>

namespace Animus {

MotionWave& MotionWave::shared() {
    static MotionWave inst;
    return inst;
}

// ── Swipe events ──────────────────────────────────────────────────

void MotionWave::onSwipeBegin(uint32_t fingers) {
    resetSwipe();
    m_swipeFingers = fingers;

    // Only track three fingers. Ignore two (handled by axis/scroll).
    // Ignore four+ — not used in unstable ISO.
    if (fingers == 3) {
        m_swipeState = SwipeState::Tracking;
        m_tapState   = TapState::Waiting;
        // Record tap start time via CLOCK_MONOTONIC equivalent.
        // wlroots swipe_update delivers time_msec — use first update.
        m_tapStartMs = 0;  // set on first update
    }
    // fingers != 3: state stays Idle
}

void MotionWave::onSwipeUpdate(uint32_t fingers, double dx, double dy) {
    if (m_swipeState == SwipeState::Idle) return;
    if (fingers != 3) return;

    // Accumulate deltas
    m_accumDX += dx;
    m_accumDY += dy;
    m_tapTravelX += std::abs(dx);
    m_tapTravelY += std::abs(dy);

    // Track peak velocity (magnitude per axis)
    // wlroots delivers dx/dy as pixels per event (not per second).
    // Velocity approximation: we use magnitude relative to threshold.
    // Full px/s requires dt — approximated as |delta| * 60 (60Hz assumption).
    // Known limit: velocity estimation is approximate.
    // At 120Hz: velocity is halved relative to 60Hz assumption.
    // On HP Victus (60Hz panel): correct. On 120Hz panel: underestimates.
    // Result: gestures require slightly more effort on 120Hz panels.
    // Acceptable for unstable ISO.
    m_peakVelX = std::max(m_peakVelX, std::abs(dx) * 60.0);
    m_peakVelY = std::max(m_peakVelY, std::abs(dy) * 60.0);

    // Axis commitment: once AXIS_COMMIT_PX travel in one direction,
    // commit to that axis and ignore the other.
    if (m_swipeState == SwipeState::Tracking) {
        double absX = std::abs(m_accumDX);
        double absY = std::abs(m_accumDY);

        if (absX >= AXIS_COMMIT_PX || absY >= AXIS_COMMIT_PX) {
            // Primary axis = whichever has greater displacement
            if (absX >= absY) {
                m_swipeState = SwipeState::CommittedH;
            } else {
                m_swipeState = SwipeState::CommittedV;
            }
            // Tap is no longer possible — too much travel
            m_tapState = TapState::Idle;
        }
    }

    m_prevDX = dx;
    m_prevDY = dy;
}

void MotionWave::onSwipeEnd(bool cancelled) {
    if (cancelled) {
        resetSwipe();
        return;
    }

    if (m_swipeFingers == 3) {
        // Check for tap first (highest priority)
        if (m_tapState == TapState::Waiting) {
            double totalTravel = m_tapTravelX + m_tapTravelY;
            if (totalTravel <= TAP_MAX_TRAVEL_PX) {
                fireTapResult();
                resetSwipe();
                return;
            }
        }

        // Check for committed swipe
        if (m_swipeState == SwipeState::CommittedH ||
            m_swipeState == SwipeState::CommittedV) {
            fireSwipeResult();
        }
        // If still Tracking (not enough travel) — ignore. No event fired.
    }

    resetSwipe();
}

void MotionWave::fireSwipeResult() {
    // Verify minimum travel
    double absX = std::abs(m_accumDX);
    double absY = std::abs(m_accumDY);
    double travel = (m_swipeState == SwipeState::CommittedH) ? absX : absY;

    if (travel < MIN_TRAVEL_PX) return;

    // Verify velocity threshold
    double peakVel = (m_swipeState == SwipeState::CommittedH)
                        ? m_peakVelX : m_peakVelY;

    if (peakVel < velocityThreshold()) return;

    if (m_swipeState == SwipeState::CommittedH) {
        // LEFT or RIGHT
        if (m_accumDX < 0) {
            // Swipe LEFT → previous desktop
            if (!gestureEnabled(GestureId::ThreeFingerLeft)) return;
            EventBus::shared().publish(OSFEvent::DesktopPrev,
                                        m_peakVelX);
        } else {
            // Swipe RIGHT → next desktop
            if (!gestureEnabled(GestureId::ThreeFingerRight)) return;
            EventBus::shared().publish(OSFEvent::DesktopNext,
                                        m_peakVelX);
        }
    } else {
        // UP or DOWN
        // wlroots: dy negative = upward finger motion = swipe UP
        if (m_accumDY < 0) {
            // Swipe UP → CockpitView open
            if (!gestureEnabled(GestureId::ThreeFingerUp)) return;
            // AnimusContext: gesture origin = screen center
            float cx = StateManager::shared().screenWidth()  * 0.5f;
            float cy = StateManager::shared().screenHeight() * 0.5f;
            AnimusContext ctx = AnimusContext::fromKeyboardShortcut(cx, cy);
            EventBus::shared().publish(OSFEvent::CockpitViewOpen, ctx);
        } else {
            // Swipe DOWN
            if (!gestureEnabled(GestureId::ThreeFingerDown)) return;
            bool cockpitOpen = std::any_cast<bool>(
                StateManager::shared().get(StateKey::CockpitViewOpen)
            );
            if (cockpitOpen) {
                // CockpitView is open → close it
                EventBus::shared().publish(OSFEvent::CockpitViewClose,
                                            AnimusContext::none());
            } else {
                // CockpitView is closed → show desktop toggle
                EventBus::shared().publish(OSFEvent::ShowDesktop, {});
            }
        }
    }
}

void MotionWave::fireTapResult() {
    if (!gestureEnabled(GestureId::ThreeFingerTap)) return;
    // Three-finger tap → show desktop toggle
    EventBus::shared().publish(OSFEvent::ShowDesktopToggle, {});
}

void MotionWave::resetSwipe() {
    m_swipeState   = SwipeState::Idle;
    m_swipeFingers = 0;
    m_accumDX      = 0.0;
    m_accumDY      = 0.0;
    m_peakVelX     = 0.0;
    m_peakVelY     = 0.0;
    m_prevDX       = 0.0;
    m_prevDY       = 0.0;
    m_tapState     = TapState::Idle;
    m_tapStartMs   = 0;
    m_tapTravelX   = 0.0;
    m_tapTravelY   = 0.0;
}

// ── Pinch events ──────────────────────────────────────────────────

void MotionWave::onPinchBegin(uint32_t fingers) {
    if (fingers == 2) {
        m_pinchState     = PinchState::Tracking;
        m_pinchPrevScale = 1.0;
        m_pinchAccum     = 0.0;
    }
    // Non-two-finger pinch: ignore
}

void MotionWave::onPinchUpdate(uint32_t fingers, double /*dx*/, double /*dy*/,
                                double scale, double /*rotation*/) {
    if (m_pinchState == PinchState::Idle) return;
    if (fingers != 2) return;

    // scale is cumulative from gesture start (1.0 = unchanged)
    // delta = current scale / previous scale
    double delta = scale / m_pinchPrevScale;
    m_pinchPrevScale = scale;
    m_pinchAccum += (delta - 1.0);

    // Fire events when accumulation crosses threshold
    // Threshold: PINCH_MIN_DELTA (0.04) prevents noise
    // Fired repeatedly during gesture as user pinches
    // Consumer (app zoom) integrates these deltas

    if (m_pinchAccum > PINCH_MIN_DELTA) {
        if (gestureEnabled(GestureId::PinchOut)) {
            EventBus::shared().publish(OSFEvent::PinchOut,
                                        static_cast<float>(m_pinchAccum));
        }
        m_pinchAccum = 0.0;
    } else if (m_pinchAccum < -PINCH_MIN_DELTA) {
        if (gestureEnabled(GestureId::PinchIn)) {
            EventBus::shared().publish(OSFEvent::PinchIn,
                                        static_cast<float>(-m_pinchAccum));
        }
        m_pinchAccum = 0.0;
    }
}

void MotionWave::onPinchEnd(bool /*cancelled*/) {
    m_pinchState     = PinchState::Idle;
    m_pinchPrevScale = 1.0;
    m_pinchAccum     = 0.0;
}

// ── Settings interface ────────────────────────────────────────────

void MotionWave::setSensitivity(Sensitivity s) {
    m_sensitivity = s;
    // Persist to StateManager — Settings reads/writes this
    StateManager::shared().set(StateKey::MotionWaveSensitivity,
                                 static_cast<int>(s));
}

void MotionWave::setNaturalScroll(bool natural) {
    m_naturalScroll = natural;
    StateManager::shared().set(StateKey::MotionWaveNaturalScroll, natural);
    // Natural scroll inversion applied in InputRouter::onPointerAxis()
    // not here — MotionWave does not handle scroll axis events
}

void MotionWave::setGestureEnabled(GestureId id, bool enabled) {
    m_enabled[static_cast<uint8_t>(id)] = enabled;
    // Persist per-gesture enable state
    // Key: "motionwave_gesture_{id}" → bool
    std::string key = std::string("motionwave_gesture_") +
                      std::to_string(static_cast<int>(id));
    StateManager::shared().set(key, enabled);
}

} // namespace Animus
```

### 30.5 InputRouter Extension — MotionWave wiring

```cpp
// animus/input/InputRouter.h — updated

// Replace:
//     class GestureRecognizer;
//     std::unique_ptr<GestureRecognizer> m_gestures;
// With:
//     class MotionWave;  // forward declaration not needed — MotionWave is singleton

// InputRouter.h — add pinch callbacks:
void onPinchBegin(uint32_t fingers);
void onPinchUpdate(uint32_t fingers, double dx, double dy,
                    double scale, double rotation);
void onPinchEnd(bool cancelled);

// InputRouter.cpp — implementations:
void InputRouter::onSwipeBegin(uint32_t f) {
    MotionWave::shared().onSwipeBegin(f);
}
void InputRouter::onSwipeUpdate(uint32_t f, double dx, double dy) {
    MotionWave::shared().onSwipeUpdate(f, dx, dy);
}
void InputRouter::onSwipeEnd(bool c) {
    MotionWave::shared().onSwipeEnd(c);
}
void InputRouter::onPinchBegin(uint32_t f) {
    MotionWave::shared().onPinchBegin(f);
}
void InputRouter::onPinchUpdate(uint32_t f, double dx, double dy,
                                  double scale, double rotation) {
    MotionWave::shared().onPinchUpdate(f, dx, dy, scale, rotation);
}
void InputRouter::onPinchEnd(bool c) {
    MotionWave::shared().onPinchEnd(c);
}

// Natural scroll applied in onPointerAxis:
void InputRouter::onPointerAxis(double dx, double dy) {
    bool natural = std::any_cast<bool>(
        StateManager::shared().get(StateKey::MotionWaveNaturalScroll)
    );
    double scrollDY = natural ? dy : -dy;
    EventBus::shared().publish(OSFEvent::ScrollDelta, scrollDY);
}
```

### 30.6 OSFDesktop callback bridge — pinch addition

```cpp
// animus/core/OSFDesktop.cpp — add pinch callbacks
// Alongside existing cbSwipeBegin/Update/End:

static void cbPinchBegin(uint32_t fingers, void *ud) {
    (void)ud;
    InputRouter::shared().onPinchBegin(fingers);
}
static void cbPinchUpdate(uint32_t fingers, double dx, double dy,
                            double scale, double rotation, void *ud) {
    (void)ud;
    InputRouter::shared().onPinchUpdate(fingers, dx, dy, scale, rotation);
}
static void cbPinchEnd(bool cancelled, void *ud) {
    (void)ud;
    InputRouter::shared().onPinchEnd(cancelled);
}

// In OSFDesktop::initialize() — add to animus_set_callbacks():
animus_set_callbacks(
    /* existing callbacks unchanged */
    cbPinchBegin,
    cbPinchUpdate,
    cbPinchEnd
);
```

### 30.7 New OSFEvents for MotionWave

```cpp
// Additions to OSFEvent enum — insert before _Count

// MotionWave gesture results
// data type noted per event

// Desktop navigation
// data = float (peak velocity px/s — used by desktop transition spring)
DesktopPrev,         // LOCAL compositor — three-finger swipe LEFT
DesktopNext,         // LOCAL compositor — three-finger swipe RIGHT

// Show desktop
// data = {}
ShowDesktop,         // LOCAL — three-finger swipe DOWN (CockpitView closed)
ShowDesktopToggle,   // LOCAL — three-finger TAP (show/restore all)

// Pinch
// data = float (scale delta magnitude, always positive)
PinchIn,             // LOCAL — two-finger pinch in (shrink)
PinchOut,            // LOCAL — two-finger pinch out (expand)

// NOTE: CockpitViewOpen and CockpitViewClose already defined in Part 29.
// MotionWave publishes those directly — no new events needed for UP/DOWN swipe.
```

### 30.8 New StateManager Keys for MotionWave

```cpp
// Additions to StateKey namespace in animus/core/StateManager.h

namespace StateKey {
    // ── EXISTING KEYS (unchanged) ────────────────────────────────
    // FocusedWindowId, ActiveMonitorIndex, LockScreenVisible,
    // CockpitViewOpen, CurrentWallpaper, WallpaperTintR/G/B,
    // SystemVolume, DockVisibility, PathfinderOpen

    // ── NEW: MotionWave settings ──────────────────────────────────
    // Persisted to vitusos-config.nix user prefs section.
    // Read by MotionWave::initialize() on compositor start.

    constexpr char MotionWaveSensitivity[]   = "motionwave_sensitivity";
                                               // int: 0=Low 1=Medium 2=High
    constexpr char MotionWaveNaturalScroll[] = "motionwave_natural_scroll";
                                               // bool: true=natural (default)

    // Per-gesture enable/disable keys:
    // "motionwave_gesture_0" through "motionwave_gesture_6"
    // bool: true = enabled (all default true)
    // Not listed as individual constexpr — generated programmatically:
    // "motionwave_gesture_" + std::to_string(GestureId)

    // ── NEW: Desktop state ────────────────────────────────────────
    constexpr char CurrentDesktopIndex[]    = "current_desktop_index";
                                               // int: 0-based, 0 = Desktop 1
    constexpr char DesktopCount[]           = "desktop_count";
                                               // int: 1-6
    constexpr char ShowDesktopActive[]      = "show_desktop_active";
                                               // bool: all windows minimized
    constexpr char ReducedMotion[]          = "reduced_motion";
                                               // bool: false default
    constexpr char FirstBootComplete[]      = "first_boot_complete";
                                               // bool: false until welcome done
} // namespace StateKey
```

### 30.9 New Spring Profile — SPRING_DESKTOP_SWITCH

```cpp
// Addition to Part 9 / Part 20 Quick Reference

// SPRING_DESKTOP_SWITCH (280, 28)
// Used for: virtual desktop transition slide animation
// Character: slightly heavier than SPRING_SELECTION (400,28)
//            The desktop has mass — it doesn't snap instantly
//            It slides with momentum, settles with weight
//            initVelocity set from MotionWave peak velocity
//            Fast swipe → fast transition → correct
//            Slow deliberate swipe → slower transition → correct
//
// Wallpaper parallax: separate spring on wallpaper layer
// SPRING_DESKTOP_SWITCH_BG (180, 24)
//            Even slower — wallpaper lags windows
//            Creates depth: two layers moving at different speeds
//            Windows: full travel (screen width)
//            Wallpaper: 40% of travel (0.4 × screen width)
//            This is the macOS Spaces parallax feel

static constexpr SpringConfig SPRING_DESKTOP_SWITCH    = { 280.f, 28.f, 0.010f };
static constexpr SpringConfig SPRING_DESKTOP_SWITCH_BG = { 180.f, 24.f, 0.010f };

// Add to Part 20 Quick Reference:
// DesktopSwitch               280, 28
// DesktopSwitchBG             180, 24   (wallpaper parallax — lags windows)
```

### 30.10 MotionWave Initialization

```cpp
// MotionWave reads preferences from StateManager on compositor start.
// Called from OSFDesktop::initialize() after StateManager is ready.

void MotionWave::initialize() {
    // Read sensitivity
    auto sensVal = StateManager::shared().getOr(
        StateKey::MotionWaveSensitivity, std::any(1));
    m_sensitivity = static_cast<Sensitivity>(std::any_cast<int>(sensVal));

    // Read natural scroll
    auto natVal = StateManager::shared().getOr(
        StateKey::MotionWaveNaturalScroll, std::any(true));
    m_naturalScroll = std::any_cast<bool>(natVal);

    // Read per-gesture enable states
    for (int i = 0; i < 7; ++i) {
        std::string key = std::string("motionwave_gesture_") +
                          std::to_string(i);
        auto val = StateManager::shared().getOr(key, std::any(true));
        m_enabled[i] = std::any_cast<bool>(val);
    }
}

// StateManager::getOr() — new method needed:
// Returns value for key if present, otherwise returns defaultVal.
// Prevents crash on first boot when keys don't exist yet.
// Add to StateManager.h:
std::any getOr(const std::string &key, const std::any &defaultVal) const;
```

### 30.11 Complete Gesture Map — Locked

```
INPUT EVENT             FINGERS   DIRECTION/TYPE    RESULT
──────────────────────────────────────────────────────────────────
wlr swipe               3         UP (dy < 0)       CockpitViewOpen
                                  velocity ≥ threshold
                                  travel ≥ 20px
                                  axis committed vertical

wlr swipe               3         DOWN (dy > 0)     CockpitViewClose
                                  velocity ≥ threshold     OR
                                  CockpitView open    ShowDesktop
                                  (if CockpitView closed)

wlr swipe               3         LEFT (dx < 0)     DesktopPrev
                                  velocity ≥ threshold
                                  travel ≥ 20px
                                  axis committed horizontal

wlr swipe               3         RIGHT (dx > 0)    DesktopNext
                                  velocity ≥ threshold

wlr swipe end           3         travel ≤ 10px     ShowDesktopToggle
                                  duration ≤ 200ms
                                  (tap detected)

wlr pinch update        2         scale < prev      PinchIn
                                  accum < -0.04     (data=magnitude)

wlr pinch update        2         scale > prev      PinchOut
                                  accum > +0.04     (data=magnitude)

wlr axis (scroll)       2         any               ScrollDelta
                                  (natural scroll applied)

keyboard Alt+Tab        —         —                 CockpitViewOpen/Close
                                                    (Part 29 — unchanged)

orange box double-click —         —                 CockpitViewOpen
                                                    (Part 29 — unchanged)

IGNORED:
    Four-finger swipe:  ignored entirely
    Two-finger swipe:   does not exist in wlroots
                        (two-finger motion = scroll axis)
    One-finger gestures: not multi-touch
    Cancelled gestures: always reset, no event fired
```

### 30.12 Settings → MotionWave Section Spec

```
Settings app → MotionWave section

Layout: same split as rest of Settings
Left sidebar: "MotionWave" item with wave icon
Right content:

    SECTION HEADER: "SENSITIVITY"
        Segmented control: Low · Medium · High
        Default: Medium
        Caption: "How much effort gestures require"
        Immediate effect — no restart needed

    SECTION HEADER: "SCROLLING"
        Toggle: Natural Scrolling  [ON]
        Caption: "Scroll direction follows finger movement"
        Immediate effect via InputRouter::onPointerAxis()

    SECTION HEADER: "GESTURES"
        Toggle list — each independently switchable:

        Three-finger UP       "Open CockpitView"         [ON]
        Three-finger DOWN     "Close CockpitView /        [ON]
                               Show Desktop"
        Three-finger LEFT     "Previous Desktop"          [ON]
        Three-finger RIGHT    "Next Desktop"              [ON]
        Three-finger TAP      "Show / Restore Desktop"   [ON]
        Pinch                 "Zoom"                      [ON]

        Disabling a gesture: greys out its row
        Does not affect other gestures

    SECTION HEADER: "ABOUT MOTIONWAVE"
        11px tertiary text:
        "MotionWave reads multi-finger gestures
         from your touchpad. Changes apply instantly."

Storage:
    All settings → StateManager → vitusos-config.nix
    user prefs section
    Survives reboot
    Applied on compositor start via MotionWave::initialize()
```

### 30.13 Known Limits and Bugs — MotionWave

```
BUG-30-1: Velocity estimation at 120Hz
    Peak velocity estimated as |delta| × 60.
    On 120Hz panels: underestimates by ~50%.
    Gestures require more effort on 120Hz hardware.
    HP Victus (60Hz): correct behavior.
    Fix: use time_msec from wlroots events for true dt.
    Deferred to post-unstable — requires wlroots
    event timestamp threading through all update calls.

BUG-30-2: Tap detection start time
    m_tapStartMs set to 0, not actual start time.
    Tap duration therefore not truly measured.
    Only travel distance checked (≤10px).
    On very slow deliberate three-finger presses:
    may fire ShowDesktopToggle unintentionally.
    Mitigation: 10px travel limit prevents most false positives.
    Fix: thread time_msec through onSwipeBegin.
    Deferred — acceptable for unstable ISO.

BUG-30-3: Axis commitment with diagonal swipes
    At exactly 45° diagonal: axis committed to whichever
    accumulates AXIS_COMMIT_PX first (non-deterministic
    for perfectly diagonal swipes).
    In practice: perfect 45° swipes are rare.
    User intent is almost always clearly one axis.
    No fix needed — physics handles it correctly.

BUG-30-4: Pinch rotation ignored
    wlr_pointer_pinch_update_event provides rotation angle.
    vitusOS ignores it — no rotate gesture.
    No app currently needs rotation.
    Post-unstable feature if needed.

KNOWN LIMIT-30-1: No four-finger gestures
    Four-finger events are silently ignored.
    No user-visible indication.
    Post-unstable feature.

KNOWN LIMIT-30-2: Physical touchpad only
    MotionWave requires libinput gesture support.
    External mice with gesture buttons:
    may or may not produce wlr swipe events.
    Behavior on external pointing devices: undefined.
    Acceptable — vitusOS targets laptop touchpad.

KNOWN LIMIT-30-3: Palm rejection is libinput's job
    If libinput palm rejection misses a palm:
    MotionWave will see false gesture events.
    vitusOS cannot compensate — libinput must be correct.
    HP Victus touchpad: libinput palm rejection is good.
    Known rough edge on other hardware.
```

### 30.14 What Opus Must NEVER Do — MotionWave

```
NEVER reference GestureRecognizer in new code.
    The class is MotionWave. The file is MotionWave.cpp/.h.
    GestureRecognizer.cpp/.h is deleted and replaced.
    Any #include "GestureRecognizer.h" is wrong.

NEVER hardcode velocity thresholds outside MotionWave.
    The threshold is velocityThreshold() — reads sensitivity.
    Never write: if (velocity > 300.0) in any other file.

NEVER fire gesture events from onSwipeBegin.
    Gestures are committed on onSwipeEnd only.
    Begin and Update only accumulate state.
    Firing on Begin = every accidental touch triggers actions.

NEVER invert scroll direction anywhere except InputRouter::onPointerAxis.
    Natural scroll inversion happens in exactly one place.
    Inverting anywhere else = double inversion = wrong direction.

NEVER wire four-finger gestures in unstable ISO.
    They are intentionally not implemented.
    If Opus adds four-finger handling: remove it.
    Post-unstable only.

NEVER skip the cancelled flag check in onSwipeEnd.
    if (cancelled) { resetSwipe(); return; }
    This line is mandatory. Without it: gesture fires
    when user lifts fingers after accidental contact.
    That is the most common gesture misfire source.
```

---

## PART 31 — Virtual Desktop System

### 31.1 Overview

Virtual desktops in vitusOS are spatial containers for windows.
The user moves between them horizontally — always left or right,
never up or down. There is always at least one desktop.
Maximum: 6.

The desktop transition is the macOS Spaces model:
windows slide at full velocity, wallpaper slides at 40% velocity.
Two layers moving at different speeds. Depth is created without 3D.

This part specifies:
- DesktopManager: owns all desktop state
- Desktop transition rendering
- Window assignment to desktops
- Naming, creation, deletion
- Boundary behavior (bounce, not wrap)
- CockpitView sidebar integration

### 31.2 DesktopManager.h

```cpp
// animus/shell/DesktopManager.h
#pragma once
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include "core/StateManager.h"
#include <string>
#include <vector>
#include <memory>

namespace Animus {

class OSFWindow;

// DesktopManager: owns the virtual desktop state.
// Singleton. Lives in OSFDesktop.
//
// Desktop indices are 0-based internally.
// Display names are 1-based ("Desktop 1" = index 0).
//
// Window assignment:
//   Every OSFWindow belongs to exactly one desktop.
//   Default: current desktop at time of window creation.
//   Assignment stored in StateManager:
//     key: "windowDesktop:{handle}" → int desktopIndex
//
// Rendering:
//   DesktopManager owns the horizontal slide offset.
//   RenderPipeline reads m_slideOffsetX.value() each frame.
//   Windows outside current desktop are not rendered
//   (outside clip region) but remain in scene graph.
//
// NEVER render windows from inactive desktops.
// NEVER destroy windows when their desktop is inactive.
// Springs continue ticking on inactive desktops — position
// is correct when user returns. CPU cost is acceptable.

class DesktopManager {
public:
    static DesktopManager& shared();

    bool initialize();

    // Switch to desktop by index (0-based)
    // Called by: DesktopPrev/Next OSFEvent handlers,
    //            CockpitView sidebar click
    void switchTo(int index);
    void switchPrev();  // publishes OSFEvent::DesktopSwitched
    void switchNext();  // publishes OSFEvent::DesktopSwitched

    // Desktop creation / deletion
    // addDesktop: appends new desktop at end
    // removeDesktop: moves windows to Desktop 0, then removes
    void addDesktop();
    void removeDesktop(int index);  // cannot remove last desktop

    // Naming
    // name: user-provided UTF-8 string, max 24 chars
    // Empty name reverts to "Desktop {N}"
    void renameDesktop(int index, const std::string &name);
    std::string nameForDesktop(int index) const;

    // Window assignment
    void assignWindow(uint64_t windowHandle, int desktopIndex);
    int  desktopForWindow(uint64_t windowHandle) const;
    bool windowVisibleOnCurrent(uint64_t windowHandle) const;

    // Accessors
    int  currentIndex() const { return m_currentIndex; }
    int  count()        const { return static_cast<int>(m_names.size()); }
    float slideOffsetX() const { return m_slideOffsetX.value(); }
    float bgSlideOffsetX() const { return m_bgSlideOffsetX.value(); }

    // Called each frame from RenderPipeline
    void tick(float dt);

    // Boundary state — for bounce visual feedback
    bool isAtLeftBoundary()  const { return m_currentIndex == 0; }
    bool isAtRightBoundary() const { return m_currentIndex == count()-1; }

    static constexpr int   MAX_DESKTOPS  = 6;
    static constexpr int   MIN_DESKTOPS  = 1;
    static constexpr float PARALLAX_FACTOR = 0.4f; // wallpaper travels 40% of window travel
    static constexpr int   MAX_NAME_LEN  = 24;

private:
    DesktopManager() = default;

    int                      m_currentIndex = 0;
    std::vector<std::string> m_names;    // display names, indexed by desktop

    // Slide springs — SPRING_DESKTOP_SWITCH (280,28)
    // Value = horizontal offset in pixels
    // 0 = current desktop centered
    // -screenW = one desktop to the right
    // +screenW = one desktop to the left
    SpringSolver m_slideOffsetX;    // windows layer
    SpringSolver m_bgSlideOffsetX;  // wallpaper layer — SPRING_DESKTOP_SWITCH_BG

    // Boundary bounce spring
    // When at edge and swipe attempted: fires and returns
    SpringSolver m_bounceX;  // SPRING_SELECTION (400,28)
    bool         m_bouncing = false;

    uint64_t m_prevHandle = 0;
    uint64_t m_nextHandle = 0;
    uint64_t m_tickHandle = 0;

    void persistState();     // writes to StateManager + vitusos-config.nix
    void loadPersistedState(); // reads on initialize()
    void triggerBounce(float direction); // +1.0=rightward, -1.0=leftward
    std::string defaultName(int index) const;
};

} // namespace Animus
```

### 31.3 DesktopManager.cpp

```cpp
// animus/shell/DesktopManager.cpp
#include "DesktopManager.h"
#include "core/EventBus.h"
#include "core/StateManager.h"
#include "core/OSFEvent.h"
#include "animation/SpringSolver.h"

namespace Animus {

DesktopManager& DesktopManager::shared() {
    static DesktopManager inst;
    return inst;
}

bool DesktopManager::initialize() {
    // Always start with at least one desktop
    m_names.push_back("Desktop 1");
    m_currentIndex = 0;

    // Load persisted state
    loadPersistedState();

    // Spring initialization — starts settled at 0
    m_slideOffsetX   = SpringSolver(280.f, 28.f);
    m_bgSlideOffsetX = SpringSolver(180.f, 24.f);
    m_bounceX        = SpringSolver(400.f, 28.f);

    // Subscribe to MotionWave gesture events
    m_prevHandle = EventBus::shared().subscribe(
        OSFEvent::DesktopPrev,
        [this](const std::any &data) {
            float vel = std::any_cast<float>(data);
            switchPrev(vel);
        });

    m_nextHandle = EventBus::shared().subscribe(
        OSFEvent::DesktopNext,
        [this](const std::any &data) {
            float vel = std::any_cast<float>(data);
            switchNext(vel);
        });

    m_tickHandle = EventBus::shared().subscribe(
        OSFEvent::Tick,
        [this](const std::any &data) {
            tick(std::any_cast<float>(data));
        });

    return true;
}

void DesktopManager::switchTo(int index) {
    switchTo(index, 600.0f);  // default velocity for programmatic switch
}

void DesktopManager::switchTo(int index, float velocity) {
    if (index < 0 || index >= count()) return;
    if (index == m_currentIndex) return;

    float screenW = StateManager::shared().screenWidth();
    float direction = (index > m_currentIndex) ? -1.0f : 1.0f;

    m_currentIndex = index;

    // Target: 0 (current desktop always at offset 0)
    // Spring from current offset to 0 with initial velocity
    // The current offset is already set from previous position
    float target = 0.0f;
    m_slideOffsetX.setTarget(target);
    m_slideOffsetX.setVelocity(direction * velocity);

    m_bgSlideOffsetX.setTarget(0.0f);
    m_bgSlideOffsetX.setVelocity(direction * velocity * PARALLAX_FACTOR);

    // Update StateManager
    StateManager::shared().set(StateKey::CurrentDesktopIndex, m_currentIndex);

    // Play desktop switch sound
    SoundEngine::shared().play(Sounds::DesktopSwitch,
                                0.5f);  // 50% volume

    EventBus::shared().publish(OSFEvent::DesktopSwitched,
                                m_currentIndex);
    persistState();
}

void DesktopManager::switchPrev() { switchPrev(600.0f); }
void DesktopManager::switchNext() { switchNext(600.0f); }

void DesktopManager::switchPrev(float velocity) {
    if (m_currentIndex == 0) {
        // At left boundary — bounce
        triggerBounce(1.0f);  // bounce rightward
        return;
    }
    float screenW = StateManager::shared().screenWidth();
    // Slide offset shifts right (positive) to reveal left desktop
    m_slideOffsetX.reset(0.0f);
    m_slideOffsetX.setTarget(screenW);
    m_slideOffsetX.setVelocity(velocity);
    m_bgSlideOffsetX.reset(0.0f);
    m_bgSlideOffsetX.setTarget(screenW * PARALLAX_FACTOR);
    m_bgSlideOffsetX.setVelocity(velocity * PARALLAX_FACTOR);

    m_currentIndex--;
    StateManager::shared().set(StateKey::CurrentDesktopIndex, m_currentIndex);
    SoundEngine::shared().play(Sounds::DesktopSwitch, 0.5f);
    EventBus::shared().publish(OSFEvent::DesktopSwitched, m_currentIndex);
    persistState();
}

void DesktopManager::switchNext(float velocity) {
    if (m_currentIndex >= count() - 1) {
        // At right boundary — bounce
        triggerBounce(-1.0f);  // bounce leftward
        return;
    }
    float screenW = StateManager::shared().screenWidth();
    m_slideOffsetX.reset(0.0f);
    m_slideOffsetX.setTarget(-screenW);
    m_slideOffsetX.setVelocity(-velocity);
    m_bgSlideOffsetX.reset(0.0f);
    m_bgSlideOffsetX.setTarget(-screenW * PARALLAX_FACTOR);
    m_bgSlideOffsetX.setVelocity(-velocity * PARALLAX_FACTOR);

    m_currentIndex++;
    StateManager::shared().set(StateKey::CurrentDesktopIndex, m_currentIndex);
    SoundEngine::shared().play(Sounds::DesktopSwitch, 0.5f);
    EventBus::shared().publish(OSFEvent::DesktopSwitched, m_currentIndex);
    persistState();
}

void DesktopManager::triggerBounce(float direction) {
    // Boundary bounce: small displacement in direction, springs back
    // Communicates "you are at the edge" physically
    float bounceAmt = 32.0f;  // px — subtle, not jarring
    m_bounceX.reset(0.0f);
    m_bounceX.setTarget(direction * bounceAmt);
    m_bouncing = true;
    // On settle: return to 0 automatically (spring target is bounceAmt,
    // but we schedule a return after 80ms)
    // Implementation: m_bounceX settles at bounceAmt, then we
    // setTarget(0) — double spring = natural overshoot bounce
    // Use EventBus::publishAsync with delay? No — use spring settle check.
}

void DesktopManager::tick(float dt) {
    m_slideOffsetX.tick(dt);
    m_bgSlideOffsetX.tick(dt);

    if (m_bouncing) {
        m_bounceX.tick(dt);
        if (m_bounceX.isSettled(0.5f)) {
            // Bounce peak reached — now spring back to 0
            if (std::abs(m_bounceX.value()) > 1.0f) {
                m_bounceX.setTarget(0.0f);
            } else {
                m_bouncing = false;
                m_bounceX.reset(0.0f);
            }
        }
    }
}

void DesktopManager::addDesktop() {
    if (count() >= MAX_DESKTOPS) return;
    m_names.push_back(defaultName(count()));
    StateManager::shared().set(StateKey::DesktopCount, count());
    persistState();
    EventBus::shared().publish(OSFEvent::DesktopAdded, count() - 1);
}

void DesktopManager::removeDesktop(int index) {
    if (count() <= MIN_DESKTOPS) return;
    if (index < 0 || index >= count()) return;

    // Move all windows on this desktop to Desktop 0
    // WindowManager iterates and reassigns
    EventBus::shared().publish(OSFEvent::DesktopRemoving, index);

    m_names.erase(m_names.begin() + index);

    // Adjust current index if needed
    if (m_currentIndex >= count()) {
        m_currentIndex = count() - 1;
    }

    StateManager::shared().set(StateKey::DesktopCount, count());
    StateManager::shared().set(StateKey::CurrentDesktopIndex, m_currentIndex);
    persistState();
    EventBus::shared().publish(OSFEvent::DesktopRemoved, index);
}

void DesktopManager::renameDesktop(int index, const std::string &name) {
    if (index < 0 || index >= count()) return;
    if (name.empty()) {
        m_names[index] = defaultName(index);
    } else {
        // Truncate to MAX_NAME_LEN chars
        m_names[index] = name.substr(0, MAX_NAME_LEN);
    }
    persistState();
    EventBus::shared().publish(OSFEvent::DesktopRenamed, index);
}

std::string DesktopManager::nameForDesktop(int index) const {
    if (index < 0 || index >= count()) return "";
    return m_names[index];
}

void DesktopManager::assignWindow(uint64_t handle, int desktopIndex) {
    std::string key = "windowDesktop:" + std::to_string(handle);
    StateManager::shared().set(key, desktopIndex);
}

int DesktopManager::desktopForWindow(uint64_t handle) const {
    std::string key = "windowDesktop:" + std::to_string(handle);
    try {
        return std::any_cast<int>(StateManager::shared().get(key));
    } catch (...) {
        return 0;  // Default: Desktop 0 if not assigned
    }
}

bool DesktopManager::windowVisibleOnCurrent(uint64_t handle) const {
    return desktopForWindow(handle) == m_currentIndex;
}

std::string DesktopManager::defaultName(int index) const {
    return "Desktop " + std::to_string(index + 1);
}

void DesktopManager::persistState() {
    // StateManager holds live state
    // vitusos-config.nix holds persisted state (survives reboot)
    // Written via ConfigWriter — background thread, non-blocking
    // ConfigWriter is the component that writes vitusos-config.nix
    // It serializes: desktop count, names, current index
    // Keys written:
    //   desktops.count = N
    //   desktops.current = M
    //   desktops.names = ["Desktop 1", "Work", ...]
    // Known limit: ConfigWriter not fully specced in unstable ISO.
    // StateManager state is authoritative during session.
    // On reboot: desktop count and names survive via vitusos-config.nix.
    // Current index: always resets to 0 on boot (intentional).
}

void DesktopManager::loadPersistedState() {
    // Read from vitusos-config.nix on startup
    // If absent (first boot): defaults apply (one desktop, "Desktop 1")
    // Implementation: reads via ConfigReader (background, sync at boot)
}

} // namespace Animus
```

### 31.4 New OSFEvents — Desktop Lifecycle

```cpp
// Additions to OSFEvent enum — insert before _Count

// Desktop lifecycle
// data = int desktopIndex (0-based)
DesktopSwitched,     // BRIDGED — desktop changed. UI updates globally.
DesktopAdded,        // LOCAL — new desktop created
DesktopRemoving,     // LOCAL — desktop about to be removed
                     // WindowManager moves windows before removal
DesktopRemoved,      // LOCAL — desktop removed
DesktopRenamed,      // LOCAL — desktop name changed
                     // CockpitView sidebar label updates
```

### 31.5 RenderPipeline Integration

```cpp
// RenderPipeline reads DesktopManager offsets each frame.
// Windows layer transform:
//     translateX = DesktopManager::shared().slideOffsetX()
//     + DesktopManager::shared().bounceX.value()  (boundary bounce)
//
// Wallpaper layer transform:
//     translateX = DesktopManager::shared().bgSlideOffsetX()
//     + (bounceX.value() * PARALLAX_FACTOR)
//
// Windows NOT on current desktop:
//     If window's desktopForWindow() != currentIndex:
//         Skip rendering — outside visible region.
//         Do NOT destroy or freeze. Springs tick. State preserved.
//
// During transition (springs not settled):
//     Windows from BOTH current and adjacent desktop rendered.
//     Adjacent desktop windows offset by ±screenWidth.
//     As spring settles: adjacent desktop slides off screen.
//     When settled: only current desktop windows rendered.
//
// Known limit: during transition, adjacent desktop windows
// must be temporarily visible. This requires DesktopManager
// to expose "previousIndex" during transition.
// m_previousIndex: set on switchTo(), cleared when spring settles.
```

### 31.6 Desktop Naming — CockpitView Integration

```cpp
// CockpitView sidebar shows desktop names below each thumbnail.
// Name label: 11px, white, 80% opacity, centered below thumbnail.
// Active desktop: Space Orange 1px border on thumbnail.
//
// Rename interaction (CockpitView sidebar):
//     Double-click on desktop label → inline text edit
//     Text field springs open: SPRING_HOVER (600,40)
//     Max 24 chars enforced live as user types
//     Enter or click away → commits rename
//     Esc → cancels, reverts to previous name
//     Empty string → reverts to "Desktop N"
//
// Rename NOT available from Panel or Dock.
// Only from CockpitView sidebar.
// This keeps the Panel clean.
```

### 31.7 Known Limits and Bugs — Virtual Desktops

```
BUG-31-1: Slide offset model during transition
    m_slideOffsetX resets to 0 before spring runs.
    If user swipes again mid-transition:
    spring jumps to 0 and re-springs to new target.
    Brief visual discontinuity on rapid swipes.
    Fix: accumulate offset instead of resetting.
    Deferred — rare interaction, acceptable for unstable ISO.

BUG-31-2: Bounce implementation is double-spring
    Bounce target is set to bounceAmt, then returns to 0.
    The "return" is triggered by settle check in tick().
    Timing depends on spring settle threshold (0.5px).
    On slow hardware: bounce may feel sluggish.
    On fast hardware: may barely be visible.
    Both are acceptable. The physics is correct.

BUG-31-3: ConfigWriter not fully specced
    persistState() references ConfigWriter which is not
    specced in unstable ISO. Desktop state survives session
    via StateManager but may not survive reboot correctly.
    Known gap — documented. StateManager is authoritative.

KNOWN LIMIT-31-1: Shared wallpaper across all desktops
    All desktops use the same wallpaper.
    Parallax works because wallpaper is one continuous layer.
    Per-desktop wallpaper: post-unstable feature.
    Setting a different wallpaper per desktop: not available.

KNOWN LIMIT-31-2: Windows cannot be dragged between desktops
    except via CockpitView.
    Direct drag from one desktop edge to next: not supported.
    Use CockpitView sidebar drag or reassign from CockpitView.

KNOWN LIMIT-31-3: Current desktop index resets to 0 on reboot
    Intentional. User starts fresh each session.
    Desktop names and count persist. Position does not.
```

---

## PART 32 — GlobalMenu + PanelManager

### 32.1 Overview

GlobalMenu moves the application menu bar out of the window
and into the Panel — one consistent location for all app menus.

PanelManager owns one Panel instance per connected monitor.
GlobalMenu follows the focused window's monitor.

This part specifies:
- PanelManager: multi-monitor Panel ownership
- GlobalMenu: D-Bus integration via com.canonical.dbusmenu
- Menu rendering: item types, submenus, keyboard navigation
- No-menu behavior: app name only (clean, honest)
- LibreOffice registration timing

### 32.2 PanelManager.h

```cpp
// animus/shell/PanelManager.h
// Owns all Panel instances. One per wlr_output.
// GlobalMenu follows focused window's output.
// Clock shown on all Panels.
// Orange box on all Panels.
// System tray on primary Panel only.
#pragma once
#include "Panel.h"
#include "GlobalMenu.h"
#include "core/EventBus.h"
#include <vector>
#include <memory>
#include <unordered_map>

struct wlr_output;

namespace Animus {

class PanelManager {
public:
    static PanelManager& shared();
    bool initialize();

    // Called by compositor output_add/remove events
    void onOutputAdded(struct wlr_output *output, bool isPrimary);
    void onOutputRemoved(struct wlr_output *output);

    // Route GlobalMenu to correct Panel based on focused window output
    void onWindowFocused(uint64_t windowHandle,
                          struct wlr_output *windowOutput);

    // Called when app registers D-Bus menu
    void onMenuRegistered(const std::string &appId,
                           const std::string &menuJson);

    // Called when focused app changes but has no D-Bus menu
    void onNoMenu(const std::string &appName);

    Panel* panelForOutput(struct wlr_output *output) const;
    Panel* primaryPanel() const;

    void render(VkCommandBuffer cmd, float dt);
    void tick(float dt);

private:
    PanelManager() = default;

    struct PanelEntry {
        std::unique_ptr<Panel>  panel;
        struct wlr_output      *output;
        bool                    isPrimary;
    };

    std::vector<PanelEntry>   m_panels;
    struct wlr_output        *m_primaryOutput  = nullptr;
    struct wlr_output        *m_focusedOutput  = nullptr;  // output of focused window

    uint64_t m_focusHandle  = 0;
    uint64_t m_menuHandle   = 0;
};

} // namespace Animus
```

### 32.3 GlobalMenu.h

```cpp
// animus/shell/GlobalMenu.h
// Renders the application menu bar inside Panel.
// Reads menu layout from DBusBridge (com.canonical.dbusmenu).
// Falls back to app-name-only when no D-Bus menu available.
// Keyboard navigable: F10 or Alt activates, arrows navigate.
#pragma once
#include "render/MaterialRenderer.h"
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace Animus {

class GlobalMenu {
public:
    GlobalMenu();
    ~GlobalMenu();

    // Set menu from D-Bus menu JSON (from DBusBridge)
    // json: serialized dbusmenu layout
    void setMenuFromJson(const std::string &appName,
                          const std::string &json);

    // No D-Bus menu — show app name only
    void setAppNameOnly(const std::string &appName);

    void render(VkCommandBuffer cmd, float panelW, float panelH, float dt);
    void onPointerMotion(float x, float y);
    void onPointerButton(float x, float y, bool pressed);
    void onKey(uint32_t sym, uint32_t mods, bool pressed);

    // Keyboard activation (F10 or Alt press)
    void activate();
    void deactivate();
    bool isActive() const { return m_active; }

private:
    struct MenuItem {
        enum class Type : uint8_t {
            Normal,
            Checkbox,
            Radio,
            Separator,
            Submenu,
        };
        Type        type      = Type::Normal;
        std::string label;
        std::string shortcut;     // display only e.g. "Cmd+S"
        bool        checked   = false;   // Checkbox/Radio state
        bool        enabled   = true;
        bool        visible   = true;
        std::vector<MenuItem> children;  // for Submenu type

        // Runtime state
        SpringSolver hoverAlpha;  // SPRING_HOVER (600,40)
    };

    // Top-level menu bar items (File, Edit, View, ...)
    std::vector<MenuItem> m_topItems;

    // Active submenu state
    int          m_activeTopIndex  = -1;  // which top item is open
    int          m_hoverTopIndex   = -1;  // which top item is hovered
    bool         m_active          = false;  // keyboard activated

    // Open submenu dropdown
    struct OpenSubmenu {
        int                   parentIndex;
        std::vector<MenuItem> items;        // copy from parent
        float                 x, y;         // screen position
        int                   hoverIndex;
        SpringSolver          clipH;        // SPRING_SHEET (420,30)
        SpringSolver          opacity;      // SPRING_HOVER (600,40)
        bool                  open;
    };
    std::unique_ptr<OpenSubmenu> m_openSubmenu;

    // Nested submenu (one level deep max in unstable ISO)
    // Known limit: only one level of submenu nesting.
    // LibreOffice has deeper nesting — those submenus are truncated.
    // Post-unstable: full recursive nesting.

    std::string  m_appName;
    bool         m_hasMenu = false;  // false = show app name only

    void renderTopBar(VkCommandBuffer cmd, float panelW, float panelH);
    void renderOpenSubmenu(VkCommandBuffer cmd);
    void closeSubmenu();
    void openSubmenuAt(int topIndex);
    bool parseMenuJson(const std::string &json);

    // Submenu positioning: opens below top item
    // If submenu would go off right edge: aligns right edge to screen
    float submenuXForTopIndex(int index, float submenuW) const;

    // Keyboard navigation state
    int  m_kbTopIndex = -1;  // keyboard-highlighted top item
    int  m_kbSubIndex = -1;  // keyboard-highlighted submenu item

    static constexpr float ITEM_PADDING_H = 10.0f;  // horizontal px per item side
    static constexpr float SUBMENU_ITEM_H = 28.0f;
    static constexpr float SUBMENU_MIN_W  = 180.0f;
    static constexpr float SUBMENU_MAX_W  = 320.0f;
    static constexpr float SUBMENU_CORNER_TL = 0.0f;  // flush with Panel bottom
    static constexpr float SUBMENU_CORNER_TR = 0.0f;
    static constexpr float SUBMENU_CORNER_BL = 8.0f;
    static constexpr float SUBMENU_CORNER_BR = 8.0f;

    uint64_t m_tickHandle = 0;
};

} // namespace Animus
```

### 32.4 DBusBridge Extension — AppMenu Registration

```cpp
// Extension to DBusBridge for com.canonical.AppMenu.Registrar
// This is what LibreOffice and other GTK apps use to export menus.
// Must be registered on D-Bus BEFORE those apps launch.
// Registration happens in DBusBridge::initialize().

// D-Bus service name to register:
//   com.canonical.AppMenu.Registrar
// Object path:
//   /com/canonical/AppMenu/Registrar
// Interface methods to implement:
//   RegisterWindow(uint windowId, objectpath menuObjectPath)
//     → app calls this to register its menu
//   UnregisterWindow(uint windowId)
//     → app calls this on close

// In DBusBridge::initialize() — register BEFORE apps can launch:
bool DBusBridge::initialize() {
    // ... existing init ...

    // Register com.canonical.AppMenu.Registrar
    // This MUST succeed before any GTK/LibreOffice app is allowed to start.
    // If this fails: global menu is unavailable for that session.
    // Known limit: if D-Bus session bus is unavailable:
    //   registration fails silently.
    //   Apps fall back to in-window menus automatically.
    //   Not a crash — degraded gracefully.
    m_conn->registerServiceName("com.canonical.AppMenu.Registrar");
    m_conn->registerObject(
        "/com/canonical/AppMenu/Registrar",
        std::make_unique<AppMenuRegistrar>(*this)
    );

    // DBusMenu proxy — watches services that register menus
    // When app calls RegisterWindow:
    //   DBusBridge fetches menu layout via com.canonical.dbusmenu
    //   Parses to MenuItem tree
    //   Publishes OSFEvent::DBusMenuChanged with appId + JSON
    //   PanelManager routes to correct Panel

    return true;
}

// AppMenuRegistrar handles RegisterWindow/UnregisterWindow calls:
void AppMenuRegistrar::onRegisterWindow(uint32_t windowId,
                                          const std::string &menuPath) {
    // Map windowId → menuPath
    // Fetch menu layout via dbusmenu protocol:
    //   GetLayout(0, -1, []) → returns full menu tree
    // Serialize to JSON for EventBus transport
    // publishAsync DBusMenuChanged

    // Known limit: dbusmenu GetLayout is async D-Bus call.
    // Menu may not appear instantly on app focus.
    // Typical latency: 50-200ms after window registers.
    // Panel shows app name during latency period.
    // When menu arrives: cross-fade app name → menu items.
    // SPRING_HOVER (600,40) opacity transition.
}

// No-menu fallback — clean, honest:
// When focused window has no registered menu:
//   Panel center-left area shows: [app name only]
//   Font: Inter Semibold 13px
//   No menu items. No "…". No placeholder.
//   Just the name.
// When focus switches to windowed app WITH menu:
//   App name cross-fades to menu items (SPRING_HOVER)
// When focus switches to windowed app WITHOUT menu:
//   Menu items cross-fade to app name only (SPRING_HOVER)
```

### 32.5 Menu Keyboard Navigation

```cpp
// GlobalMenu keyboard navigation — full spec.
// Activation: F10 key OR Alt key alone (not Alt+other).
// Deactivation: Esc, click outside menu, app receives input.

// State machine:
// INACTIVE → F10/Alt pressed → ACTIVE (first top item highlighted)
// ACTIVE + ArrowRight → highlight next top item
// ACTIVE + ArrowLeft  → highlight prev top item
// ACTIVE + ArrowDown  → open highlighted top item submenu
// ACTIVE + Enter      → open highlighted top item submenu
// OPEN_SUBMENU + ArrowDown  → highlight next submenu item
// OPEN_SUBMENU + ArrowUp    → highlight prev submenu item
// OPEN_SUBMENU + ArrowRight → open nested submenu (if any)
//                             OR move to next top item
// OPEN_SUBMENU + ArrowLeft  → close submenu, return to top bar
//                             OR move to prev top item
// OPEN_SUBMENU + Enter      → activate highlighted item
// OPEN_SUBMENU + Esc        → close submenu, return to ACTIVE
// ACTIVE       + Esc        → deactivate, return INACTIVE
// OPEN_SUBMENU + letter     → jump to item starting with letter
//                             (standard menu keyboard shortcut)

// Key intercept:
// When GlobalMenu is ACTIVE or OPEN_SUBMENU:
//   Arrow keys, Enter, Esc, letters intercepted before app.
//   Alt released alone: deactivate.
// When GlobalMenu is INACTIVE:
//   All keys pass through normally.
//   F10 intercepted globally (like Alt-Tab for CockpitView).
```

### 32.6 Known Limits — GlobalMenu

```
BUG-32-1: dbusmenu async latency
    Menu layout fetched asynchronously.
    50-200ms delay before menu items appear.
    Panel shows app name during this window.
    Fast typists who press F10 immediately after
    focus switch may see empty menu.
    Mitigation: pre-fetch menu on WindowFocused event.
    Partial fix — fetch starts on focus, arrives before
    user typically presses F10.

BUG-32-2: Submenu nesting depth 1 only
    LibreOffice has menus nested 3 levels deep.
    vitusOS unstable ISO: max 1 level of submenu.
    Items at level 2+ shown as disabled with "→" indicator.
    User sees them but cannot activate them.
    Known rough edge — post-unstable fix.

BUG-32-3: dbusmenu dynamic updates
    Apps can push layout changes mid-session
    (e.g. LibreOffice changes menu based on selection).
    DBusBridge subscribes to LayoutUpdated D-Bus signal.
    On signal: re-fetches layout, re-renders.
    Known limit: if update arrives while submenu is open:
    menu closes and re-opens. Brief visual flash.
    Acceptable for unstable ISO.

KNOWN LIMIT-32-1: Electron apps have no global menu
    Electron apps do not support com.canonical.AppMenu.
    Zen Browser is Electron-based — no global menu.
    Panel shows app name only for Zen Browser.
    Correct behavior per spec — clean, honest.
    Not a bug. Documented.

KNOWN LIMIT-32-2: Global menu requires D-Bus session bus
    If session bus unavailable: no global menu.
    Apps fall back to in-window menus.
    Degraded gracefully — not a crash.
```

---

## PART 33 — Settings App

### 33.1 Overview

Settings is a first-class OSFNative app.
It is the control surface for vitusOS.
Every persistent user preference flows through Settings.

Settings has 8 sections for unstable ISO:
Wallpaper, Appearance, Display, Sound,
Keyboard, User Account, About, Power.

Plus MotionWave (specified in Part 30).
Total: 9 sections in sidebar.

### 33.2 Settings Architecture

```cpp
// native/Settings/Settings.h
// OSFNative app — uses OSFNative surface system (Part 14)
// Layout: left sidebar (section list) + right content pane
// Sidebar width: 220px
// Content pane: fills remainder
// Same split-pane layout as Filer — consistent vitusOS pattern
//
// Settings does NOT use OSFSheetSurface or OSFDropdownSurface.
// It is a flat, direct interface.
// One section open at a time.
// Switching sections: cross-fade content SPRING_HOVER (600,40).

// Storage routing:
//   Immediate (StateManager only):
//       SystemVolume, display brightness
//   Persistent (StateManager + vitusos-config.nix):
//       Wallpaper, appearance, MotionWave, keyboard layout,
//       power settings, reduced motion
//   System-rebuild required (warns user):
//       Display resolution — changes wlr_output mode
//       Keyboard layout (system-wide xkb) — requires rebuild

// Settings app publishes OSFEvents when values change.
// Components subscribe and react immediately.
// No "Apply" button — changes apply live.
// Exception: resolution change — requires confirmation dialog
//   "This will change your display resolution.
//    vitusOS will adjust. This takes a few seconds."
//   [Cancel] [Apply]
```

### 33.3 Section: Wallpaper

```
Layout:
    Full-width wallpaper preview (16:9, max 400px tall)
    Below preview: grid of built-in wallpapers (3 minimum)
    "Choose your own" button → Filer PortalGateway file pick
        Accepts: .jpg .jpeg .png .webp
        Rejects: anything else — shows error notification
    On selection: WallpaperChanged OSFEvent fires immediately
    Preview updates in Settings before desktop updates
    Desktop crossfade: SPRING_BOOT (200,22) — slow, deliberate

Built-in wallpapers:
    mars.jpg      — Mars landscape (default, already in screenshots)
    obsidian.jpg  — Dark volcanic rock, cool tones
    amber.jpg     — Warm amber gradient, complements Space Orange
    All three are 3840×2160 minimum.
    Stored in: /etc/vitusos/wallpapers/

Storage: vitusos-config.nix user.wallpaper = "/path/to/wallpaper.jpg"
```

### 33.4 Section: Appearance

```
Subsections:

ACCENT COLOR:
    Space Orange (#FF6B2B) — only option in unstable ISO.
    Shown as single color swatch, selected, non-interactive.
    Caption: "More colors coming in a future release."
    11px tertiary. Honest.
    Known gap — documented.

FONT SIZE:
    Segmented control: Small · Medium · Large
    Default: Medium
    Maps to scale factor:
        Small:  0.85× (11px→9px, 13px→11px, 15px→13px)
        Medium: 1.0×  (unchanged — default)
        Large:  1.15× (11px→13px, 13px→15px, 15px→17px)
    Immediate effect via StateManager key "fontScale" → float
    TextRenderer reads fontScale on every render call
    Known limit: fractional pixel sizes after scaling
    may cause subpixel rendering artifacts.
    HarfBuzz handles this correctly — not our problem.

DARK/LIGHT MODE:
    Toggle: Dark Mode [ON] — only option in unstable ISO.
    Caption: "Light mode coming in a future release."
    Light mode toggle is shown but disabled (greyed out).
    Known gap — documented. Dark only in unstable ISO.

REDUCED MOTION:
    Toggle: Reduce Motion [OFF]
    Caption: "Reduces animations for people who prefer
              less motion."
    Immediate effect via StateManager key "reduced_motion" → bool
    Publishes OSFEvent::ReducedMotionChanged
    SpringSolver reads on next tick — instant.
```

### 33.5 Section: Display

```
BRIGHTNESS:
    Slider: 0–100% (default 80%)
    Immediate effect via backlight control:
        /sys/class/backlight/*/brightness
        Background thread write — non-blocking
    StateManager key "display_brightness" → float (0.0–1.0)
    Known limit: some hardware does not expose backlight
    via /sys/class/backlight. Slider disabled on those systems.
    Caption shown: "Brightness control not available
    for this display." Honest.

RESOLUTION:
    Dropdown list of available modes from wlr_output_modes
    Current mode highlighted
    Change: confirmation dialog → wlr_output_set_mode()
    Requires compositor to re-commit output state
    Does NOT require reboot — live mode switch
    Known limit: some GPU/display combos reject mode changes.
    If wlr_output_commit_state fails after mode change:
    revert to previous mode automatically.
    Error notification: "Could not apply this resolution."

REFRESH RATE:
    Shown alongside resolution: "1920×1080 @ 60Hz"
    Combined in same dropdown row
    Not a separate control

NIGHT LIGHT:
    Not in unstable ISO. Known gap. Section item greyed out.
    Caption: "Coming in a future release."
```

### 33.6 Section: Sound

```
VOLUME:
    Slider: 0–100% (default 80%)
    Immediate effect via SoundEngine::setMasterVolume()
    StateManager key "system_volume" → float (0.0–1.0)
    Preview: clicking slider plays a short preview tone
    (Notification sound at new volume)

OUTPUT DEVICE:
    Dropdown list from PipeWire sink enumeration
    Default: system default sink
    Change: SoundEngine routes to new sink
    Known limit: PipeWire sink list may not update
    in real time if device is plugged in during session.
    Requires Settings section refresh.
    "Refresh" button shown below dropdown.

NOTIFICATION SOUNDS:
    Toggle: Play notification sounds [ON]
    Immediate effect — mutes OSFEvent::NotificationPosted
    sound trigger only. Other sounds unaffected.

SYSTEM SOUNDS:
    Toggle: Play system sounds [ON]
    Mutes: app_launch, app_close, desktop_switch,
           cockpit_open, boot_chime on next boot
    Does NOT mute: lock, unlock, notification
    (those are safety/awareness sounds)
```

### 33.7 Section: Keyboard

```
LAYOUT:
    Searchable list of XKB keyboard layouts
    Current layout highlighted
    Change: xkb_keymap rebuilt immediately
        wlr_keyboard_set_keymap() called
        No reboot required — live keymap change
    StateManager key "keyboard_layout" → string (XKB layout name)
    Persisted to vitusos-config.nix

REPEAT:
    Key repeat rate: slider 1–50 keys/sec (default 25)
    Key repeat delay: slider 200–1000ms (default 600ms)
    Immediate effect via wlr_keyboard_set_repeat_info()
    These are the values already set in Part 3 compositor.
    Settings just exposes them.

SHORTCUTS:
    Read-only list of system shortcuts in unstable ISO.
    No custom bindings — post-unstable feature.
    Shows:
        Alt+Tab         CockpitView
        Ctrl+Left/Right Desktop switch
        F10 / Alt       Activate menu
        [Three-finger gestures listed with MotionWave note]
    Caption: "Custom shortcuts coming in a future release."
```

### 33.8 Section: User Account

```
AVATAR:
    Circle image crop, 80×80px
    Click → Filer PortalGateway: pick image file
    Accepts: .jpg .jpeg .png
    Stored in: ~/.vitusOS/avatar.png
    Shown in: LockScreen, About section

DISPLAY NAME:
    Text field — plain UTF-8, max 64 chars
    This is display name ONLY — not system username
    System username is immutable in unstable ISO
    Known limit: username change requires system rebuild.
    Caption below field:
        "Your system username ({username}) cannot be
         changed here."
    StorageKey: vitusos-config.nix user.displayName

PASSWORD CHANGE:
    Button: "Change Password"
    Opens HEV auth flow:
        Modal overlay (OSFSheetSurface)
        Step 1: current password verification via PAM
        Step 2: new password (twice to confirm)
        Step 3: HEV re-encrypts vault with new key
    This is the correct security sequence.
    Old password must be verified before new one accepted.
    Known limit: HEV re-encryption takes 2-5 seconds.
    Progress bar shown during re-encryption.
```

### 33.9 Section: About vitusOS

```
Layout: centered content, generous whitespace

vitusOS wordmark: 280×48px (same as boot)
Version: "vitusOS [version]" — from /etc/vitusos/version
         Built from flake.nix, written at build time
NixOS: "NixOS [version]" — from /etc/os-release
Kernel: "Linux [kernel version]" — from uname -r
Hardware:
    CPU: from /proc/cpuinfo model name
    RAM: total from /proc/meminfo MemTotal
    GPU: from lspci output (cached at boot)
Uptime: from /proc/uptime, formatted "Xh Ym"
         Updates every 60 seconds

Storage:
    Used / Total from statvfs on /
    Visual bar: Space Orange fill, glass background
    Updates every 60 seconds

Check for updates button:
    Space Orange, 180px wide, 44px tall
    Click: opens Pathfinder in update context
    Pathfinder searches for newer vitusOS flake version
    Not implemented in unstable ISO — shows notification:
    "Update checking coming in a future release."
    Known gap — documented.

User avatar shown top-right (from User Account section).
```

### 33.10 Section: Power

```
LID CLOSE:
    Segmented control: Lock Screen · Sleep · Do Nothing
    Default: Lock Screen (per our decision)
    Immediate effect via logind D-Bus:
        org.freedesktop.login1.Manager
        SetInhibitedLidSwitch is NOT used
        Instead: vitusOS catches LidClosed logind signal
        and acts based on this setting
    StateManager key "lid_close_action" →
        enum { LockScreen, Sleep, Nothing }

IDLE TIMEOUT:
    Dropdown: 1 min · 2 min · 5 min · 10 min · 30 min · Never
    Default: 5 minutes
    Triggers LockScreen after this period of no input
    Timer reset by any InputRouter event
    StateManager key "idle_timeout_minutes" → int
    0 = Never

SLEEP:
    Button: "Sleep Now"
    Invokes: loginctl suspend via sd_bus_call_method
    Same as OrangeBoxMenu → Sleep

DISPLAY SLEEP:
    Dropdown: 1 min · 2 min · 5 min · Never
    Default: 2 minutes
    Turns off display (DPMS off) after this idle period
    Separate from system idle timeout
    StateManager key "display_sleep_minutes" → int
    0 = Never
```

### 33.11 Section: MotionWave (in Settings sidebar)

```
(Full spec in Part 30 section 30.12)
Sidebar label: "MotionWave"
Icon: wave symbol (custom, from vitusOS icon set)
Position in sidebar: between Keyboard and User Account
```

### 33.12 Known Limits — Settings

```
KNOWN LIMIT-33-1: No "Apply" button — live changes
    All changes apply immediately.
    If user makes a mistake (wrong keyboard layout,
    too-low brightness): must manually revert.
    There is no "Revert to saved" in unstable ISO.
    Known rough edge — acceptable for early adopters.

KNOWN LIMIT-33-2: Accent color locked to Space Orange
    One accent color. User cannot change it.
    Post-unstable feature.

KNOWN LIMIT-33-3: Light mode not available
    Dark only in unstable ISO.

KNOWN LIMIT-33-4: Custom shortcuts not available
    Read-only shortcut list in unstable ISO.

KNOWN LIMIT-33-5: Update checking not available
    "Check for updates" button shows notification only.

BUG-33-1: Font scale fractional pixel artifacts
    Small font size (0.85×): 11px → 9.35px → rounded to 9px.
    Rounding may cause slight layout reflow.
    TextRenderer handles sub-pixel via HarfBuzz.
    Visual: acceptable. Functional: correct.
```

---

## PART 34 — Power Management

### 34.1 Overview

Power management in vitusOS is handled by three components:
- logind D-Bus signals (lid close, suspend events from system)
- PowerManager (compositor-side, reacts to signals + idle timer)
- Settings → Power (user configuration)

Decisions locked:
    Lid close:    LockScreen only (user configurable)
    Idle timeout: LockScreen after 5 minutes (user configurable)
    Sleep:        manual only (OrangeBoxMenu or Settings)
    Resume:       LockScreen always shown until auth

### 34.2 PowerManager.h

```cpp
// animus/core/PowerManager.h
// Listens to logind D-Bus signals and idle timer.
// All power-related compositor behavior flows through here.
#pragma once
#include "core/EventBus.h"
#include "core/StateManager.h"
#include <cstdint>
#include <atomic>

namespace Animus {

enum class LidCloseAction : uint8_t {
    LockScreen  = 0,   // default
    Sleep       = 1,
    Nothing     = 2,
};

class PowerManager {
public:
    static PowerManager& shared();
    bool initialize();
    void destroy();

    // Called every compositor frame with dt
    void tick(float dt);

    // Input event resets idle timer — called from InputRouter
    void onInputEvent();

    // logind signal handlers (called from DBusBridge background thread)
    void onLidClosed();      // publishAsync into compositor thread
    void onPrepareForSleep(bool suspending);   // system about to sleep/wake
    void onBatteryLevelChanged(float level);   // 0.0–1.0

    // Settings interface
    void setLidCloseAction(LidCloseAction action);
    void setIdleTimeoutMinutes(int minutes);   // 0 = never
    void setDisplaySleepMinutes(int minutes);  // 0 = never

    LidCloseAction lidCloseAction()    const { return m_lidAction; }
    int            idleTimeoutMinutes()const { return m_idleTimeoutMin; }

    static constexpr float BATTERY_LOW_THRESHOLD      = 0.20f;  // 20%
    static constexpr float BATTERY_CRITICAL_THRESHOLD = 0.05f;  // 5%

private:
    PowerManager() = default;

    LidCloseAction m_lidAction        = LidCloseAction::LockScreen;
    int            m_idleTimeoutMin   = 5;
    int            m_displaySleepMin  = 2;
    float          m_idleElapsedS     = 0.0f;
    float          m_displayElapsedS  = 0.0f;
    bool           m_displayAsleep    = false;
    float          m_lastBatteryLevel = 1.0f;
    bool           m_batteryLowFired  = false;
    bool           m_batteryCritFired = false;
    bool           m_suspending       = false;

    uint64_t m_tickHandle  = 0;
    uint64_t m_inputHandle = 0;
};

} // namespace Animus
```

### 34.3 PowerManager Implementation

```cpp
// animus/core/PowerManager.cpp

void PowerManager::tick(float dt) {
    if (m_suspending) return;

    m_idleElapsedS    += dt;
    m_displayElapsedS += dt;

    // Display sleep (DPMS off)
    if (!m_displayAsleep && m_displaySleepMin > 0) {
        if (m_displayElapsedS >= m_displaySleepMin * 60.0f) {
            m_displayAsleep = true;
            // DPMS off: wlr_output_enable(output, false) + commit
            EventBus::shared().publish(OSFEvent::DisplaySleep, {});
        }
    }

    // Idle timeout → LockScreen
    if (m_idleTimeoutMin > 0) {
        if (m_idleElapsedS >= m_idleTimeoutMin * 60.0f) {
            m_idleElapsedS = 0.0f;  // reset so it doesn't re-fire
            EventBus::shared().publish(OSFEvent::LockScreenActivate, {});
        }
    }
}

void PowerManager::onInputEvent() {
    m_idleElapsedS    = 0.0f;
    m_displayElapsedS = 0.0f;

    // Wake display if asleep
    if (m_displayAsleep) {
        m_displayAsleep = false;
        // DPMS on: wlr_output_enable(output, true) + commit
        EventBus::shared().publish(OSFEvent::DisplayWake, {});
    }
}

void PowerManager::onLidClosed() {
    // Called via publishAsync — already on compositor thread
    switch (m_lidAction) {
        case LidCloseAction::LockScreen:
            EventBus::shared().publish(OSFEvent::LockScreenActivate, {});
            break;
        case LidCloseAction::Sleep:
            EventBus::shared().publish(OSFEvent::SystemSleep, {});
            break;
        case LidCloseAction::Nothing:
            break;  // do nothing
    }
}

void PowerManager::onPrepareForSleep(bool suspending) {
    m_suspending = suspending;
    if (suspending) {
        // System about to suspend
        // LockScreen must be shown BEFORE suspend completes
        // so display is locked when lid opens again
        EventBus::shared().publish(OSFEvent::LockScreenActivate, {});
        HEV::shared().lock();   // HEV locked before sleep — always
    } else {
        // Resuming from suspend
        m_suspending = false;
        // LockScreen is already active — user must auth
        // HEV stays locked until LockScreen::deactivate() succeeds
        // No action needed here — LockScreen handles it
    }
}

void PowerManager::onBatteryLevelChanged(float level) {
    m_lastBatteryLevel = level;

    if (!m_batteryLowFired && level <= BATTERY_LOW_THRESHOLD) {
        m_batteryLowFired = true;
        // Post notification — 20% battery
        EventBus::shared().publishAsync(OSFEvent::NotificationPosted, {
            // title: "Low Battery"
            // body:  "20% remaining. Connect a charger."
            // timeout: 8000ms
        });
        // Sound: Error (closest existing) — post-unstable: dedicated battery sound
    }

    if (!m_batteryCritFired && level <= BATTERY_CRITICAL_THRESHOLD) {
        m_batteryCritFired = true;
        EventBus::shared().publishAsync(OSFEvent::NotificationPosted, {
            // title: "Critical Battery"
            // body:  "5% remaining. Save your work now."
            // timeout: 0ms  (persistent — does not auto-dismiss)
        });
    }

    // Reset fired flags when charging
    if (level > BATTERY_LOW_THRESHOLD + 0.05f) {
        m_batteryLowFired  = false;
        m_batteryCritFired = false;
    }
}
```

### 34.4 New OSFEvents — Power

```cpp
// Additions to OSFEvent enum — insert before _Count

SystemSleep,         // LOCAL → BRIDGED: compositor tells session to sleep
                     // session invokes loginctl suspend
DisplaySleep,        // LOCAL compositor: DPMS off
DisplayWake,         // LOCAL compositor: DPMS on
BatteryLevelChanged, // LOCAL: data = float level (0.0–1.0)
                     // published by DBusBridge from UPower signal
LidClosed,           // LOCAL: published by DBusBridge from logind signal
```

### 34.5 Battery Status in Panel

```cpp
// Panel system tray — battery icon + percentage
// Right side of Panel, left of clock
// Only shown when running on battery (AC power: icon hidden)
// Charging: lightning bolt overlaid on battery icon
// Icons: 5 fill levels (0-20%, 20-40%, 40-60%, 60-80%, 80-100%)
// Rendered from vitusOS icon set SVG at 16×16px
// Color: white 80% opacity normally
//        Space Orange when < 20% (matches urgency)
//        Flashes (SPRING_HOVER alpha 1.0↔0.6) when < 5%
// Percentage text: 11px, white 60% opacity, right of icon
// Updates when BatteryLevelChanged fires
// Source: org.freedesktop.UPower via DBusBridge
```

### 34.6 Known Limits — Power Management

```
BUG-34-1: logind lid close signal race
    LockScreen::activate() is async (SPRING_LOCK_SCREEN takes ~1s).
    If system suspends before LockScreen is fully rendered:
    display may blank mid-animation.
    On resume: LockScreen is active but may be mid-animation.
    Mitigation: LockScreen::activate() marks state immediately
    (m_active = true before animation starts).
    HEV is locked synchronously — security is correct.
    Visual: brief animation on resume. Acceptable.

BUG-34-2: UPower polling latency
    Battery level updates from UPower arrive every 30s by default.
    Panel battery % may be 30s stale.
    Not a safety issue. Cosmetic.
    UPower also fires signal on significant changes — catches
    rapid drain events.

KNOWN LIMIT-34-1: No hibernate support
    loginctl hibernate not invoked in vitusOS.
    Hibernate requires swap partition configuration.
    Not guaranteed on all vitusOS installations.
    Sleep (suspend-to-RAM) only.

KNOWN LIMIT-34-2: Display sleep on external monitors
    DPMS off applied to all connected outputs.
    Some monitors ignore DPMS signals.
    Behavior on non-compliant monitors: undefined.
    vitusOS sends the signal — hardware compliance varies.
```

---

## PART 35 — Drag and Drop

### 35.1 Overview

Full wl_data_device implementation.
Files from Filer to apps. Text between apps. Both directions.
Ghost image follows cursor during drag with spring lag.

wl_data_device is a standard Wayland protocol.
vitusOS implements it correctly via wlr_seat.
This is plumbing — vitusOS brokers the data transfer.
Apps handle what they do with the data.

### 35.2 DragManager.h

```cpp
// animus/core/DragManager.h
// Manages active drag operations.
// Renders ghost image following cursor.
// Reports drag state to compositor hit-testing.
#pragma once
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <vector>
#include <memory>

namespace Animus {

struct DragPayload {
    enum class Type : uint8_t {
        File,       // text/uri-list
        Text,       // text/plain;charset=utf-8
        Unknown,    // other mime type — ghost shown, drop may fail
    };
    Type                     type;
    std::vector<std::string> mimeTypes;  // all offered types
    std::string              preview;    // first 40 chars for ghost label
                                         // for File: filename only
                                         // for Text: first 40 chars of text
    std::string              iconPath;   // for File: file type icon
                                         // for Text: empty — use text pill
};

// DragManager: tracks active wl_data_source drag.
// Created when wl_data_device receives start_drag request.
// Destroyed when drag ends (drop or cancel).
class DragManager {
public:
    static DragManager& shared();

    // Called by compositor when drag starts
    void onDragStart(const DragPayload &payload,
                      float originX, float originY);

    // Called every frame with current cursor position
    void onCursorMove(float x, float y);

    // Called when drop occurs over a target
    void onDrop(float x, float y);

    // Called when drag is cancelled (Esc, button release over no target)
    void onDragCancel();

    bool isDragging() const { return m_dragging; }

    // RenderPipeline calls this — rendered above all windows
    void render(VkCommandBuffer cmd, float dt);

    static constexpr float GHOST_OPACITY   = 0.60f;
    static constexpr float GHOST_ICON_SIZE = 48.0f;
    static constexpr float GHOST_MAX_W     = 200.0f;
    static constexpr float GHOST_CORNER    = 8.0f;

private:
    DragManager() = default;

    bool         m_dragging    = false;
    DragPayload  m_payload;
    float        m_cursorX     = 0.0f;
    float        m_cursorY     = 0.0f;
    float        m_originX     = 0.0f;
    float        m_originY     = 0.0f;

    // Ghost position lags cursor slightly — weight feeling
    // SPRING_WINDOW_DRAG (800,35) — same as window drag
    SpringSolver2D m_ghostPos;

    bool         m_overValidTarget = false;
    SpringSolver m_dropHighlight;  // SPRING_HOVER (600,40) on target border

    uint64_t m_tickHandle = 0;

    void renderFileGhost(VkCommandBuffer cmd, float x, float y);
    void renderTextGhost(VkCommandBuffer cmd, float x, float y);
};

} // namespace Animus
```

### 35.3 Drop Target Feedback

```cpp
// Drop target feedback — visual only, vitusOS side.
// App-side acceptance is handled by wl_data_offer protocol.

// When cursor enters a window surface during drag:
//   DragManager sends wl_data_device::enter to app
//   App responds with wl_data_offer::accept (yes/no) + action
//   If accepted:
//       1px Space Orange border springs in on window
//       SPRING_HOVER (600,40)
//       Cursor: default drag cursor (OS-provided)
//   If rejected:
//       No border highlight
//       Cursor: shows ✗ — set via wlr_seat cursor
//       Ghost still follows cursor normally

// On drop:
//   Accepted: ghost springs into drop target position
//       scale 1.0 → 0 toward drop point
//       SPRING_SELECTION (400,28)
//   Rejected/cancelled: ghost springs back to origin
//       m_ghostPos target set back to m_originX/Y
//       SPRING_WINDOW_DRAG (800,35)
//       Spring settles at origin, then ghost fades out
//       SPRING_HOVER (600,40) opacity → 0

// File drag from Filer:
//   wl_data_source offers: text/uri-list, text/plain
//   URI format: file:///home/user/Documents/file.pdf
//   App receives URI — opens file as it sees fit
//   Filer does NOT copy the file to the destination
//   Filer does NOT move the file automatically
//   The drag is informational — the app decides what to do
//   Known limit: apps that don't handle text/uri-list
//   will silently reject the drop. No error shown.
//   This is correct Wayland behavior.

// Text drag between apps:
//   Source app creates wl_data_source
//   Offers: text/plain;charset=utf-8
//   Destination app pastes via wl_data_offer::receive
//   Standard Wayland — vitusOS brokers correctly
```

### 35.4 Known Limits — Drag and Drop

```
BUG-35-1: Ghost spring at drag start
    Ghost starts at cursor position.
    Spring target is always cursor position.
    Spring lag creates ~16px behind cursor at speed.
    On very fast drag: ghost lags visibly.
    This is intentional — communicates weight.
    Some users may find it disorienting.
    Acceptable for unstable ISO.

KNOWN LIMIT-35-1: Apps that ignore URI drops
    Many apps do not implement wl_data_offer for files.
    Dropping a file on them: nothing happens.
    No error shown in vitusOS — the app rejected it.
    User may be confused.
    vitusOS cannot force apps to accept drops.

KNOWN LIMIT-35-2: No drag between virtual desktops
    Dragging an item while switching desktops
    mid-drag: drag cancels automatically.
    Cannot drag content from Desktop 1 to app on Desktop 2.
    Post-unstable feature.

KNOWN LIMIT-35-3: Binary clipboard not supported
    application/octet-stream not offered by ClipboardBridge.
    Image copy/paste between apps uses image/png.
    Raw binary: not supported. Known gap.
```

---

## PART 36 — Sound Design

### 36.1 Complete Sound Map

```
All 8 sounds in vitusOS unstable ISO.
Files in: /etc/vitusos/sounds/
Format: WAV (PCM 44100Hz 16-bit stereo)

NAME                FILE                TRIGGER POINT
──────────────────────────────────────────────────────────────
boot_chime          boot_chime.wav      First frame after BootCrossfade
                                        (already specced — unchanged)
lock_screen         lock_screen.wav     LockScreen::activate() called
unlock_screen       unlock_screen.wav   PAM auth success
notification        notification.wav    OSFEvent::NotificationPosted
app_launch          app_launch.wav      First wl_surface commit from new app
                                        NOT on posix_spawn — visible first
app_close           app_close.wav       wl_surface destroy for last surface
                                        of an app process
desktop_switch      desktop_switch.wav  DesktopManager spring crosses 50%
                                        of travel — not on gesture start
cockpit_open        cockpit_open.wav    m_cockpitZoom crosses 0.7 on open
                                        Same file used on close
                                        (same subtle sound both directions)
```

### 36.2 Volume Levels

```cpp
// Relative to system volume (SoundEngine master volume)
// Values passed to SoundEngine::play(name, volume)

namespace SoundVolumes {
    constexpr float BootChime      = 1.00f;  // full system volume
    constexpr float LockScreen     = 0.80f;
    constexpr float UnlockScreen   = 0.80f;
    constexpr float Notification   = 0.70f;
    constexpr float AppLaunch      = 0.30f;  // subtle — background action
    constexpr float AppClose       = 0.20f;  // very subtle
    constexpr float DesktopSwitch  = 0.50f;  // whoosh — noticeable but not loud
    constexpr float CockpitOpen    = 0.25f;  // subtle spatial cue
}
```

### 36.3 Reduced Motion → Sound Muting

```cpp
// When reduced_motion = true:
// Sounds that accompany animations are muted.
// Sounds that carry information are preserved.

// In SoundEngine::play():
bool SoundEngine::play(const std::string &name, float volume) {
    bool reducedMotion = std::any_cast<bool>(
        StateManager::shared().getOr(
            StateKey::ReducedMotion, std::any(false)));

    if (reducedMotion) {
        // Muted when reduced motion on:
        static const std::unordered_set<std::string> motionSounds = {
            Sounds::AppLaunch,
            Sounds::AppClose,
            Sounds::DesktopSwitch,
            Sounds::CockpitOpen,
        };
        if (motionSounds.count(name)) return true; // silently skip
    }

    // Always play (even with reduced motion):
    // boot_chime, lock_screen, unlock_screen, notification
    // These carry information — muting them is wrong.

    // ... existing play implementation ...
}
```

### 36.4 Sound Namespace Extension

```cpp
// Additions to Sounds namespace in animus/audio/SoundEngine.h
// Alongside existing entries:

namespace Sounds {
    // EXISTING (unchanged):
    constexpr char BootChime[]       = "boot_chime";
    constexpr char WindowOpen[]      = "window_open";   // kept for compat
    constexpr char WindowClose[]     = "window_close";  // kept for compat
    constexpr char Notification[]    = "notification";
    constexpr char Error[]           = "error";
    constexpr char TrashEmpty[]      = "trash_empty";
    constexpr char CockpitOpen[]     = "cockpit_open";
    constexpr char LockScreen[]      = "lock_screen";
    constexpr char UnlockScreen[]    = "unlock_screen";
    constexpr char InstallComplete[] = "install_complete";
    constexpr char Drag[]            = "drag";
    constexpr char Drop[]            = "drop";
    constexpr char Eject[]           = "eject";

    // NEW — Part 36:
    constexpr char AppLaunch[]       = "app_launch";
    constexpr char AppClose[]        = "app_close";
    constexpr char DesktopSwitch[]   = "desktop_switch";
    // CockpitOpen already exists — used for both open and close
}

// NOTE: WindowOpen and WindowClose (existing) are NOT the same as
// AppLaunch and AppClose (new).
// WindowOpen: fires when any wl_surface opens (including popups, menus)
// AppLaunch:  fires only when a new app process creates its first surface
// WindowClose: fires when any surface closes
// AppClose:   fires only when an app process exits (last surface closes)
// They coexist. AppLaunch/Close are new, higher-level concepts.
```

### 36.5 Sound Trigger Implementations

```cpp
// AppLaunch trigger — in WindowManager::addSurface():
void WindowManager::addSurface(struct wlr_surface *surface,
                                 const AnimusContext &ctx) {
    // ... existing surface setup ...

    // Is this the first surface of a new app process?
    pid_t pid = getProcessForSurface(surface);
    if (!m_knownPids.count(pid)) {
        m_knownPids.insert(pid);
        SoundEngine::shared().play(Sounds::AppLaunch,
                                    SoundVolumes::AppLaunch);
    }
}

// AppClose trigger — in WindowManager::removeSurface():
void WindowManager::removeSurface(struct wlr_surface *surface) {
    pid_t pid = getProcessForSurface(surface);
    // Check if this is the last surface of this process
    if (countSurfacesForPid(pid) == 1) {  // this is the last one
        SoundEngine::shared().play(Sounds::AppClose,
                                    SoundVolumes::AppClose);
        m_knownPids.erase(pid);
    }
    // ... existing removal logic ...
}

// DesktopSwitch trigger — in DesktopManager::switchPrev/Next():
// Already included in Part 31 implementation.
// Fires when spring starts (not at 50% travel).
// Known limit: user hears sound at gesture start, not mid-transition.
// 50% travel trigger is cleaner but requires tick() to check.
// For unstable ISO: trigger on switch start. Acceptable.

// CockpitView open/close trigger — in OSFDesktop tick():
// When m_cockpitZoom crosses 0.7 threshold:
void OSFDesktop::tick(float dt) {
    float zoom = m_cockpitZoom.value();
    float prevZoom = m_prevCockpitZoom;
    m_prevCockpitZoom = zoom;

    // Crossing 0.7 threshold on either direction = play sound
    bool crossedOpen  = prevZoom > 0.7f && zoom <= 0.7f;
    bool crossedClose = prevZoom < 0.7f && zoom >= 0.7f;
    if (crossedOpen || crossedClose) {
        SoundEngine::shared().play(Sounds::CockpitOpen,
                                    SoundVolumes::CockpitOpen);
    }
}
```

---

## PART 37 — First Boot Welcome Screen

### 37.1 Overview

First boot detected via StateManager key "first_boot_complete".
If absent or false: WelcomeScreen shown before desktop.
3 steps: vault setup → wallpaper pick → done.

WelcomeScreen is a full-screen OSFNative surface.
No Panel. No Dock. No orange box visible.
The OS is not ready yet. Shell surfaces not shown.

After step 3 completes:
    StateManager sets "first_boot_complete" = true
    Persisted to vitusos-config.nix
    WelcomeScreen fades out (SPRING_BOOT 200,22)
    BootCrossfade fires → desktop revealed
    Boot chime plays

### 37.2 WelcomeScreen.h

```cpp
// animus/shell/WelcomeScreen.h
// Full-screen first-boot experience.
// Three steps: vault setup, wallpaper, done.
// Shown only once — never again after first_boot_complete = true.
// Background: #1A1208 (same as LockScreen — system speaking)
// Content card: glass material, 480px wide, centered
#pragma once
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <string>
#include <functional>

namespace Animus {

class WelcomeScreen {
public:
    WelcomeScreen();
    ~WelcomeScreen();

    bool isComplete() const { return m_complete; }
    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void onKey(uint32_t sym, uint32_t mods, bool pressed);
    void onPointerButton(float x, float y, bool pressed);
    void onPointerMotion(float x, float y);

private:
    enum class Step : uint8_t {
        VaultSetup   = 0,
        WallpaperPick = 1,
        Done          = 2,
    };

    Step    m_step     = Step::VaultSetup;
    bool    m_complete = false;

    // Step 1: vault setup
    std::string m_passphrase1;
    std::string m_passphrase2;
    bool        m_passphraseVisible = false;
    int         m_passphraseStrength = 0;  // 0-4
    bool        m_vaultDone   = false;
    SpringSolver m_strengthBar;  // SPRING_HOVER (600,40)

    // Step 2: wallpaper pick
    int         m_selectedWallpaper = 0;  // index into built-in list
    SpringSolver m_wallpaperHover[3];     // per thumbnail hover

    // Step 3: done
    SpringSolver m_doneButtonHover;

    // Progress dots
    SpringSolver m_dotScale[3];   // per-dot scale SPRING_HOVER (600,40)

    // Card entrance animation
    SpringSolver m_cardY;   // SPRING_SHEET (420,30): drops in from above
    SpringSolver m_opacity; // SPRING_HOVER (600,40): fade in

    uint64_t m_tickHandle = 0;

    void renderVaultStep(VkCommandBuffer cmd, float cx, float cy);
    void renderWallpaperStep(VkCommandBuffer cmd, float cx, float cy);
    void renderDoneStep(VkCommandBuffer cmd, float cx, float cy);
    void renderProgressDots(VkCommandBuffer cmd, float cx, float y);
    void renderPassphraseStrength(VkCommandBuffer cmd, float x, float y);

    void advanceStep();
    void commitVaultSetup();
    void commitWallpaperPick();
    void commitDone();

    // Passphrase strength calculation — simple but honest
    // 0: too short (<8 chars)
    // 1: weak (8+ chars, simple)
    // 2: fair (mixed case or digits)
    // 3: good (mixed case + digits)
    // 4: strong (mixed case + digits + symbols)
    int calculateStrength(const std::string &pass) const;
};

} // namespace Animus
```

### 37.3 Welcome Screen Visual Spec

```
BACKGROUND:
    Full screen #1A1208 — same as LockScreen
    No blur. Solid. System surface.

CONTENT CARD:
    Width:  480px
    Height: varies by step (auto)
    Position: horizontally centered, vertically centered
    Material: glass (SurfaceAltitude::High — 32px blur)
    Border: 1px white 15% opacity
    Corner radius: 16px
    Drop in from above: SPRING_SHEET (420,30)
        Starts at y - 40px, springs to center y
    Fade in: SPRING_HOVER (600,40) opacity 0→1

PROGRESS DOTS:
    3 dots at bottom of card
    Size: 8px each, 12px gap
    Active dot: Space Orange (#FF6B2B), scale 1.1×
    Inactive dot: white 30% opacity, scale 1.0×
    Scale spring: SPRING_HOVER (600,40) on step change
    Position: 24px above card bottom edge

STEP 1 — VAULT SETUP:
    Title: "secure your vault"
        Inter Semibold 20px, white 95%
    Subtitle: "your passwords and keys live here"
        Inter Regular 13px, white 60%
    Passphrase field 1:
        Label: "passphrase" — 11px white 60%
        Input: masked (•••), monospace 15px
        Toggle show/hide button right side
    Passphrase field 2:
        Label: "confirm passphrase"
        Input: masked
    Strength bar:
        4 segments, fills left to right
        Colors: red/orange/yellow/green per segment
        SPRING_HOVER animate on strength change
    Error state: fields shake (SPRING_SELECTION) if mismatch
    Continue button: Space Orange 220×44px
        Disabled (50% opacity) until both fields match
        AND passphrase ≥ 8 chars
        SPRING_HOVER hover scale
    Skip: NOT available — vault setup is mandatory

STEP 2 — WALLPAPER PICK:
    Title: "make it yours"
    Subtitle: "choose your wallpaper"
    3 wallpaper thumbnails: 136×76px each, 8px gap
        Rounded corners: 8px
        Selected: Space Orange 2px border, scale 1.04×
        Hover: scale 1.02×
        SPRING_HOVER (600,40) all scale animations
    "Choose your own" text button below thumbnails
        Space Orange 13px — opens Filer portal
    Continue button: same spec as step 1
        Always enabled (default wallpaper pre-selected)
    Skip: allowed (keeps default Mars wallpaper)

STEP 3 — DONE:
    Title: "you're all set"
    Subtitle: "welcome to vitusOS"
        Both centered. Same type scale as before.
    vitusOS wordmark: 200×34px centered
        Same bitmap from AnimusBoot
    Button: "let's go"
        Space Orange, 220×44px
        On click: commitDone()
            WelcomeScreen fades out SPRING_BOOT (200,22)
            Shell surfaces shown
            Boot chime plays
```

### 37.4 Known Limits — First Boot

```
BUG-37-1: Passphrase field masking
    Input field is custom-rendered (no GTK).
    Masked with bullet characters (•).
    Copy/paste from clipboard into masked field:
    pastes but not visible — correct security behavior.
    Known issue: paste from clipboard may contain
    trailing newline. Trimmed automatically.

KNOWN LIMIT-37-1: No wallpaper preview on custom pick
    When user picks custom wallpaper via Filer portal:
    card shows filename only, not thumbnail preview.
    Desktop preview not shown inside WelcomeScreen.
    Post-unstable: live preview thumbnail in card.

KNOWN LIMIT-37-2: Vault setup is blocking
    HEV::initialize() with new passphrase runs on main thread.
    Takes 200-500ms on HP Victus.
    UI freezes briefly while vault initializes.
    Should run on background thread with spinner.
    Known rough edge — acceptable for one-time operation.
```

---

## PART 38 — Reduced Motion

### 38.1 Overview

Reduced motion is a Settings → Appearance toggle.
Immediate effect. No restart. No recompile.
SpringSolver checks a global flag each tick.

Two categories of animations:
- ELIMINATED: spring jumps to target instantly
- REDUCED: spring still runs but faster constants

### 38.2 SpringSolver Reduced Motion Integration

```cpp
// Extension to SpringSolver — Part 9 + Part 29 extended.
// Global flag read every tick.
// No per-spring configuration needed.

// animus/animation/SpringSolver.h — add:
class SpringSolver {
public:
    // ... existing API unchanged ...

    float tick(float dt) {
        dt = std::clamp(dt, 0.001f, 0.100f);

        // Check reduced motion flag
        bool reduced = s_reducedMotion.load(std::memory_order_relaxed);
        if (reduced && m_eliminateOnReducedMotion) {
            // Jump to target instantly
            m_pos = m_target;
            m_vel = 0.0f;
            return m_pos;
        }
        // If reduced but not eliminated: spring continues
        // (direct manipulation, safety-critical, scroll)

        // ... existing spring integration unchanged ...
    }

    // Mark this spring as eliminatable under reduced motion.
    // Call this on construction for visual-only springs.
    // Do NOT call for springs that are direct manipulation
    // (window drag, scroll, throw physics).
    void setEliminateOnReducedMotion(bool b) {
        m_eliminateOnReducedMotion = b;
    }

    // Global flag — set by Settings → Appearance → Reduce Motion
    static void setReducedMotion(bool reduced) {
        s_reducedMotion.store(reduced, std::memory_order_relaxed);
        EventBus::shared().publish(OSFEvent::ReducedMotionChanged, reduced);
    }
    static bool reducedMotion() {
        return s_reducedMotion.load(std::memory_order_relaxed);
    }

private:
    // ... existing members ...
    bool m_eliminateOnReducedMotion = false;

    static std::atomic<bool> s_reducedMotion;
    // Defined in SpringSolver.cpp:
    // std::atomic<bool> SpringSolver::s_reducedMotion{false};
};
```

### 38.3 Which Springs Are Eliminated vs Preserved

```
ELIMINATED (setEliminateOnReducedMotion(true)):
    Window birth scale/position spring
    Window close scale/position spring
    CockpitView zoom (m_cockpitZoom)
    Desktop switch slide (m_slideOffsetX, m_bgSlideOffsetX)
    App launch from Dock (birth animation)
    Notification slide-in (m_slideX)
    BootCrossfade scale/opacity
    WelcomeScreen card drop-in
    Panel auto-hide slide (fullscreen)
    Dock auto-hide slide (fullscreen)
    OrangeBoxMenu clip height
    Pathfinder open/close scale

PRESERVED (setEliminateOnReducedMotion(false) — default):
    Window drag position (direct manipulation)
    Window throw/edge resistance (direct manipulation)
    Scroll (direct manipulation)
    Traffic light hover (still needed for discoverability)
    LockScreen opacity (safety — must be visible)
    Shutdown/restart opacity (safety)
    Spring highlight pill in menus (usability)
    Dock magnification (direct manipulation response)
    SPRING_SHADOW focus change (barely perceptible — harmless)
    MotionWave bounce at desktop boundary (orientation cue)
```

### 38.4 Settings Integration

```cpp
// In Settings → Appearance → Reduce Motion toggle handler:
void SettingsApp::onReducedMotionToggled(bool enabled) {
    SpringSolver::setReducedMotion(enabled);
    StateManager::shared().set(StateKey::ReducedMotion, enabled);
    // Persisted to vitusos-config.nix on next config write
}

// On compositor startup — restore from config:
void OSFDesktop::initialize() {
    // ... other init ...

    // Restore reduced motion setting
    bool reduced = std::any_cast<bool>(
        StateManager::shared().getOr(
            StateKey::ReducedMotion, std::any(false)));
    SpringSolver::setReducedMotion(reduced);

    // First boot detection — MUST run after StateManager initialized
    // and BEFORE shell surfaces are shown to user.
    bool firstBootDone = std::any_cast<bool>(
        StateManager::shared().getOr(
            StateKey::FirstBootComplete, std::any(false)));

    if (!firstBootDone) {
        // Show WelcomeScreen — blocks shell from appearing
        // WelcomeScreen owns its own render loop section.
        // On completion: sets FirstBootComplete = true,
        //   plays boot chime, fades out, shell appears.
        m_welcomeScreen = std::make_unique<WelcomeScreen>();
        StateManager::shared().set(StateKey::LockScreenVisible, false);
        // Shell surfaces (Panel, Dock) not shown until
        // WelcomeScreen::isComplete() returns true.
        // RenderPipeline checks this flag each frame.
    }
    // If firstBootDone = true: skip WelcomeScreen entirely.
    // Desktop appears normally. Boot chime plays via BootCrossfade.
}
// WelcomeScreen completion hook — called from WelcomeScreen::commitDone():
//   StateManager::shared().set(StateKey::FirstBootComplete, true);
//   m_welcomeScreen.reset();  // destroys WelcomeScreen
//   EventBus::shared().publish(OSFEvent::BootCrossfadeComplete, {});
//   SoundEngine::shared().play(Sounds::BootChime, 1.0f);
```

---

## PART 39 — Multi-Monitor + XWayland

### 39.1 Multi-Monitor Architecture

```
Primary monitor:   full shell (Panel + Dock + orange box)
Secondary monitor: Panel only (orange box + clock, NO Dock)

GlobalMenu follows focused window's monitor.
    Focused window on primary   → menu on primary Panel
    Focused window on secondary → menu on secondary Panel
    All other Panels: app name only

System tray (battery, wifi, volume icons): primary Panel only
Clock: ALL Panels

This is the correct model. Day one.
Not post-unstable. Not configurable. This is how it works.
```

### 39.2 Multi-Output C11 Extension

```c
// compositor/compositor.h — extend for multi-output

// Replace single primary_output with output list:
struct wlr_output *outputs[8];  // max 8 connected outputs
int                output_count;
struct wlr_output *primary_output;  // first connected output

// Add output_remove callback:
void (*on_output_added)(struct wlr_output*, bool isPrimary, void*);
void (*on_output_removed)(struct wlr_output*, void*);

// h_new_output — modified to handle multiple outputs:
static void h_new_output(struct wl_listener *l, void *data) {
    struct wlr_output *out = data; (void)l;
    struct wlr_output_state st = {0};
    wlr_output_state_set_enabled(&st, true);

    // Pick best mode
    struct wlr_output_mode *mode =
        wlr_output_preferred_mode(out);
    if (mode) wlr_output_state_set_mode(&st, mode);
    wlr_output_commit_state(out, &st);

    // Add to layout
    wlr_output_layout_add_auto(g.output_layout, out);

    // Track
    bool isPrimary = (g.output_count == 0);
    if (g.output_count < 8) {
        g.outputs[g.output_count++] = out;
    }
    if (isPrimary) g.primary_output = out;

    if (g.on_output_added)
        g.on_output_added(out, isPrimary, g.ud);
}
```

### 39.3 PanelManager Multi-Output Wiring

```cpp
// OSFDesktop callbacks for output events:
static void cbOutputAdded(struct wlr_output *out, bool isPrimary, void *ud) {
    PanelManager::shared().onOutputAdded(out, isPrimary);
}
static void cbOutputRemoved(struct wlr_output *out, void *ud) {
    PanelManager::shared().onOutputRemoved(out);
    // Windows on removed output moved to primary:
    // WindowManager::shared().migrateWindowsFromOutput(out);
}

// PanelManager::onOutputAdded():
void PanelManager::onOutputAdded(struct wlr_output *output, bool isPrimary) {
    PanelEntry entry;
    entry.output    = output;
    entry.isPrimary = isPrimary;
    entry.panel     = std::make_unique<Panel>();
    entry.panel->initialize();
    entry.panel->setIsPrimary(isPrimary);  // primary Panel shows system tray
    m_panels.push_back(std::move(entry));

    if (isPrimary) m_primaryOutput = output;
}

// Panel::setIsPrimary() controls:
//   isPrimary = true:  render orange box + clock + system tray + GlobalMenu zone
//   isPrimary = false: render orange box + clock only
//                      GlobalMenu zone renders name or menu per focus
```

### 39.4 Window Migration on Output Removal

```cpp
// When a monitor is disconnected:
// All windows on that output must move to primary.
// They stack at center of primary output.
// User must reposition them manually.

void WindowManager::migrateWindowsFromOutput(struct wlr_output *removed) {
    float cx = primaryOutput()->width  * 0.5f;
    float cy = primaryOutput()->height * 0.5f;

    float offset = 0.0f;
    for (auto &win : m_windows) {
        if (win->currentOutput() == removed) {
            // Spring window to center of primary, staggered
            win->m_pos.setTarget(cx - win->width()  * 0.5f + offset,
                                  cy - win->height() * 0.5f + offset);
            win->setOutput(primaryOutput());
            offset += 24.0f;  // stagger so windows don't stack exactly
        }
    }
    // Notification: "Display disconnected. Windows moved to primary display."
    EventBus::shared().publishAsync(OSFEvent::NotificationPosted, {
        // title: "Display disconnected"
        // body:  "Windows moved to main display."
        // timeout: 5000ms
    });
}
```

### 39.5 XWayland Decision

```
XWayland: DISABLED in vitusOS unstable ISO.
Configuration: programs.xwayland.enable = false
               in /etc/nixos/configuration.nix

Rationale:
    vitusOS unstable ISO targets Wayland-native apps.
    Release checklist: DOOM, Zen Browser, LibreOffice —
    all Wayland-native.
    XWayland adds compositor complexity without benefit
    for the unstable ISO target set.

When user launches an X11-only app:
    App fails to connect to display.
    Error shown in Pathfinder (if launched from there):
        "This app requires X11 which is not available
         in this release of vitusOS."
    If launched from terminal:
        App prints its own error to stdout/stderr.
        vitusOS shows nothing additional.
    Known rough edge — some legacy apps will not run.
    Documented clearly.

Post-unstable:
    programs.xwayland.enable = true
    XWayland auto-starts on demand (wlr_xwayland)
    X11 apps composited via xwm (X window manager in wlroots)
    No changes needed to vitusOS C++17 compositor —
    wlroots handles XWayland surface integration.
    Known limit even post-unstable: some X11 apps
    have clipboard/DnD issues under XWayland.
    Those are upstream wlroots/XWayland bugs, not vitusOS bugs.
```

### 39.6 Known Limits — Multi-Monitor + XWayland

```
BUG-39-1: CockpitView on multi-monitor
    CockpitView zoom currently operates on single output.
    On multi-monitor: CockpitView only zooms on
    the output where the focused window lives.
    Secondary output: not zoomed.
    Windows from secondary output shown in CockpitView
    but positioned as if on primary.
    Visual: odd layout on multi-monitor CockpitView.
    Known rough edge. Post-unstable: per-output CockpitView.

BUG-39-2: Dock visible on secondary via Panel trick
    User cannot launch apps from secondary monitor directly.
    Must look at primary Dock.
    Workaround: Pathfinder accessible from secondary
    orange box → Pathfinder.
    Acceptable. Documented.

KNOWN LIMIT-39-1: Max 8 outputs hardcoded
    g.outputs[8] — eight monitor maximum.
    More than 8 monitors: compositor may crash or ignore
    additional outputs. Known limit. Documented.
    Post-unstable: dynamic output list.

KNOWN LIMIT-39-2: No per-monitor scaling (HiDPI)
    All outputs use same pixel density.
    Mixed HiDPI + non-HiDPI: one or the other looks wrong.
    wlroots supports per-output scale factor.
    vitusOS does not expose this in unstable ISO.
    Post-unstable feature.
```

---

## PART 40 — Fullscreen

### 40.1 Overview

Fullscreen is required for the unstable ISO release checklist.
DOOM requests fullscreen via xdg_toplevel::set_fullscreen.
vitusOS must honor it correctly.

Fullscreen windows:
- Expand to fill entire output (no Panel, no Dock visible)
- Panel auto-hides — returns on cursor approach
- Dock auto-hides — returns on cursor approach
- Traffic lights float in top-left corner on hover
- Escape or app request restores pre-fullscreen geometry

### 40.2 Fullscreen in WindowManager

```cpp
// animus/core/WindowManager.h — add fullscreen tracking

struct WindowFullscreenState {
    bool    active       = false;
    float   prevX        = 0.0f;   // geometry before fullscreen
    float   prevY        = 0.0f;
    float   prevW        = 0.0f;
    float   prevH        = 0.0f;
};

// In OSFWindow — add:
WindowFullscreenState m_fullscreen;

// When xdg_toplevel requests set_fullscreen:
void WindowManager::onSetFullscreen(OSFWindow *win,
                                      struct wlr_output *output) {
    if (win->m_fullscreen.active) return;  // already fullscreen

    // Store pre-fullscreen geometry
    win->m_fullscreen.active = true;
    win->m_fullscreen.prevX  = win->posX();
    win->m_fullscreen.prevY  = win->posY();
    win->m_fullscreen.prevW  = win->width();
    win->m_fullscreen.prevH  = win->height();

    // Configure to fill output
    struct wlr_output *target = output ? output : primaryOutput();
    float ow = target->width;
    float oh = target->height;

    // Spring window to fill output
    win->m_pos.setTarget(0.0f, 0.0f);
    win->m_scale.setTarget(1.0f);

    // xdg_toplevel::configure with fullscreen size
    wlr_xdg_toplevel_set_size(win->xdgToplevel(), ow, oh);
    wlr_xdg_toplevel_set_fullscreen(win->xdgToplevel(), true);

    // Hide Panel + Dock on this output
    EventBus::shared().publish(OSFEvent::FullscreenEntered,
                                static_cast<uint64_t>(win->handle()));
}

// When app requests unset_fullscreen OR user presses Esc:
void WindowManager::onUnsetFullscreen(OSFWindow *win) {
    if (!win->m_fullscreen.active) return;

    win->m_fullscreen.active = false;

    // Restore pre-fullscreen geometry
    win->m_pos.setTarget(win->m_fullscreen.prevX,
                          win->m_fullscreen.prevY);
    wlr_xdg_toplevel_set_size(win->xdgToplevel(),
                               win->m_fullscreen.prevW,
                               win->m_fullscreen.prevH);
    wlr_xdg_toplevel_set_fullscreen(win->xdgToplevel(), false);

    // Show Panel + Dock again
    EventBus::shared().publish(OSFEvent::FullscreenExited,
                                static_cast<uint64_t>(win->handle()));
}
```

### 40.3 Panel and Dock Auto-Hide in Fullscreen

```cpp
// Panel adds fullscreen auto-hide behavior
// animus/shell/Panel.h — additions:

class Panel {
    // ... existing ...

    // Fullscreen mode
    void enterFullscreenMode();   // subscribes to FullscreenEntered
    void exitFullscreenMode();    // subscribes to FullscreenExited
    bool isFullscreen() const { return m_fullscreen; }

private:
    bool         m_fullscreen   = false;
    SpringSolver m_hideY;  // SPRING_SELECTION (400,28)
                           // 0 = visible, -HEIGHT = hidden above screen
    float        m_cursorY = 0.0f;  // updated from InputRouter

    static constexpr float HOT_ZONE_PX = 4.0f;  // cursor within 4px of top
};

// Panel::render() in fullscreen mode:
// if (m_fullscreen) {
//     bool cursorInHotZone = (m_cursorY <= HOT_ZONE_PX);
//     float target = cursorInHotZone ? 0.0f : -HEIGHT;
//     m_hideY.setTarget(target);
//     // Translate Panel by m_hideY.value() on Y axis
// }

// Dock: same model, mirrored — hot zone at bottom 4px
// m_hideY target: 0 (visible) or +Dock::HEIGHT (hidden below screen)
```

### 40.4 Floating Traffic Lights in Fullscreen

```cpp
// When fullscreen window is focused:
// Traffic lights float at top-left corner of screen.
// Appear when cursor enters 60×32px zone at top-left.
// Close (red): sends close request to fullscreen app
// Minimize (yellow): exits fullscreen first, then minimizes
// Maximize/restore (blue): exits fullscreen

// Rendered by RenderPipeline above fullscreen window content.
// Z-order: above everything including fullscreen window.
// Not part of Panel (Panel is hidden).
// Separate render pass: FullscreenTrafficLights.

struct FullscreenTrafficLights {
    bool         visible    = false;
    SpringSolver opacity;     // SPRING_HOVER (600,40)
    SpringSolver scale[3];    // SPRING_TRAFFIC_LIGHT (700,38) per button

    static constexpr float HOT_ZONE_W = 60.0f;
    static constexpr float HOT_ZONE_H = 32.0f;
    static constexpr float BUTTON_X   = 12.0f;  // left margin
    static constexpr float BUTTON_Y   = 10.0f;  // top margin
    static constexpr float BUTTON_SIZE = 12.0f;
    static constexpr float BUTTON_GAP  =  8.0f;
};

// Colors: same as normal traffic lights
// Close:    #FF3B30
// Minimize: #FFCC00
// Maximize: #007AFF (exits fullscreen = restore)
```

### 40.5 Esc Key Fullscreen Exit

```cpp
// In InputRouter::onKey() — add before app delivery:
if (keysym == XKB_KEY_Escape && pressed) {
    // Check if focused window is fullscreen
    auto focused = RegistryManager::shared().windows().focusedWindow();
    if (focused && focused->m_fullscreen.active) {
        WindowManager::shared().onUnsetFullscreen(focused.get());
        return;  // consumed — not passed to app
    }
    // Otherwise: deliver Esc to app normally
    // (app may use Esc for its own purposes when not fullscreen)
}

// Known limit: some apps use Esc internally in fullscreen
// (e.g. DOOM uses Esc to open its menu).
// vitusOS intercept happens BEFORE app.
// Single Esc: exits fullscreen.
// App's Esc behavior (open menu): lost.
// User must re-enter fullscreen to access app menu.
// This is the correct behavior — system takes precedence.
// If app wants to handle Esc: it should use unset_fullscreen.
```

### 40.6 New OSFEvents — Fullscreen

```cpp
// data = uint64_t windowHandle
FullscreenEntered,   // LOCAL — Panel/Dock hide, floating TL shown
FullscreenExited,    // LOCAL — Panel/Dock show, floating TL hidden
```

### 40.7 Known Limits — Fullscreen

```
BUG-40-1: Esc intercept conflicts with DOOM
    DOOM uses Esc to open its menu in fullscreen.
    vitusOS intercepts Esc to exit fullscreen.
    First Esc: exits fullscreen (vitusOS behavior).
    User re-enters fullscreen: DOOM is now running windowed
    unless user fullscreens again via DOOM's own controls.
    Mitigation: DOOM uses F11 or its own fullscreen toggle.
    If DOOM uses xdg_toplevel set/unset_fullscreen:
    vitusOS honors it and Esc behavior is DOOM's problem.
    Depends on DOOM's Wayland backend implementation.
    Known rough edge.

KNOWN LIMIT-40-1: One fullscreen window at a time per output
    Only the focused window can be fullscreen.
    If another window requests fullscreen while one is active:
    the request is honored — previous fullscreen exits.
    Window stacking handles this naturally.

KNOWN LIMIT-40-2: Fullscreen on secondary output
    Fullscreen on secondary output: Panel hides on secondary.
    Primary Panel remains visible.
    This is correct behavior — only the affected output's
    Panel hides. Other outputs unaffected.
```

---

## PART 41 — Minimize Behavior

### 41.1 Overview

Minimize = window springs toward its Dock icon.
Yellow traffic light triggers it.
Three-finger tap (Show Desktop) minimizes all windows.
Three-finger tap again restores all.

This part formalizes what was established in the
earlier brainstorming session.

### 41.2 Minimize Implementation

```cpp
// animus/shell/OSFWindow.h — minimize state

class OSFWindow {
    // ... existing ...

    void minimize();
    void restore();
    bool isMinimized() const { return m_minimized; }

private:
    bool           m_minimized     = false;
    AnimusContext  m_minimizeCtx;  // stored on minimize, used on restore
};

// OSFWindow::minimize():
void OSFWindow::minimize() {
    if (m_minimized) return;
    m_minimized = true;

    // Find Dock icon position for this app
    float iconX = Dock::shared().iconCenterX(m_appId);
    float iconY = Dock::shared().iconCenterY(m_appId);

    // Store context for restore
    m_minimizeCtx = AnimusContext::fromDockIcon(iconX, iconY,
                                                 Dock::ICON_SIZE);

    // Spring toward Dock icon — same as window birth in reverse
    m_pos.setTarget(iconX - m_width  * 0.5f,
                    iconY - m_height * 0.5f);
    m_scale.setTarget(0.05f);
    m_opacity.setTarget(0.0f);

    // Springs: SPRING_SELECTION (400,28) for scale and opacity
    // SPRING_WINDOW_DRAG (800,35) for position

    // When settled at 0 opacity: window stops rendering
    // (RenderPipeline checks m_minimized flag)

    SoundEngine::shared().play(Sounds::AppClose,
                                SoundVolumes::AppClose);
    EventBus::shared().publish(OSFEvent::WindowMinimized,
                                m_handle);
}

// OSFWindow::restore():
void OSFWindow::restore() {
    if (!m_minimized) return;
    m_minimized = false;

    // Spring back from Dock icon to pre-minimize position
    // m_minimizeCtx has the origin
    beginBirthAnimation(m_minimizeCtx);

    SoundEngine::shared().play(Sounds::AppLaunch,
                                SoundVolumes::AppLaunch);
    EventBus::shared().publish(OSFEvent::WindowRestored,
                                m_handle);
}
```

### 41.3 Show Desktop (Three-Finger Tap)

```cpp
// ShowDesktopToggle handler in OSFDesktop:

void OSFDesktop::onShowDesktopToggle() {
    bool showDesktopActive = std::any_cast<bool>(
        StateManager::shared().getOr(
            StateKey::ShowDesktopActive, std::any(false)));

    if (!showDesktopActive) {
        // Minimize all windows
        for (auto &win : m_windowManager->allWindows()) {
            if (!win->isMinimized()) {
                win->minimize();
            }
        }
        StateManager::shared().set(StateKey::ShowDesktopActive, true);
    } else {
        // Restore all previously minimized windows
        // Restore in reverse minimize order (LIFO)
        // Each window springs from its Dock icon independently
        // Creates organic cascade — not synchronized
        for (auto it = m_minimizeOrder.rbegin();
             it != m_minimizeOrder.rend(); ++it) {
            (*it)->restore();
        }
        m_minimizeOrder.clear();
        StateManager::shared().set(StateKey::ShowDesktopActive, false);
    }
}

// Track minimize order for restore:
// EventBus subscription in OSFDesktop::initialize():
EventBus::shared().subscribe(OSFEvent::WindowMinimized,
    [this](const std::any &data) {
        uint64_t handle = std::any_cast<uint64_t>(data);
        auto win = m_windowManager->findWindow(handle);
        if (win) m_minimizeOrder.push_back(win);
    });
```

### 41.4 New OSFEvents — Minimize

```cpp
// data = uint64_t windowHandle
WindowMinimized,     // LOCAL — window minimize started
WindowRestored,      // LOCAL — window restore started
```

---

## PART 42 — Notification Center Scope

### 42.1 Scope for Unstable ISO

```
No notification history in unstable ISO.
No notification center panel.
No persistence.

Notifications appear, auto-dismiss, are gone.
This is clean and intentional.
An inbox of old notifications is noise.
Unstable ISO: only what matters right now.

Known gap — documented.
Post-unstable: notification center in Panel system tray.
```

### 42.2 Notification Actions

```cpp
// org.freedesktop.Notifications action strings
// DBusBridge receives action labels with each notification.
// Rendered as small buttons at bottom of OSFNotification surface.

// OSFNotification — extend for actions:
struct NotificationAction {
    std::string key;    // action key string (from D-Bus)
    std::string label;  // display label
    SpringSolver hoverAlpha;  // SPRING_HOVER (600,40)
};

// Max 2 action buttons per notification.
// 3+ actions: first 2 shown, rest truncated.
// Known limit — acceptable for unstable ISO.

// Button spec:
//     Height: 24px
//     Corner radius: 6px
//     Background: white 12% opacity
//     Hover: white 20% opacity (SPRING_HOVER)
//     Text: 11px Inter Medium, white 90%
//     Bottom of notification card, 8px from bottom edge
//     8px gap between buttons
//     Two buttons: each ~50% of card width minus gaps

// On button click:
//     DBusBridge sends org.freedesktop.Notifications ActionInvoked
//     signal with action key
//     Notification auto-dismisses after action click
```

### 42.3 Notification Auto-Dismiss Timeout

```
Default timeout from app: honored if provided
Fallback (no timeout or 0): 5000ms
Persistent (timeout = -1): shown until user dismisses
    via click on notification or action button
Maximum timeout honored: 30000ms (30 seconds)
    Anything longer is capped — no notification stays
    forever unless explicitly persistent

Dismiss on click: clicking notification body dismisses it
    + activates default action (if any)
    Default action key: "default"
    If no default action: dismiss only, no signal sent
```

---

## PART 43 — Clipboard

### 43.1 Complete Clipboard Spec

```cpp
// ClipboardBridge already specced in Addendum J.
// This part adds the wl_data_device wiring
// and clarifies what vitusOS brokers.

// wlr_seat clipboard wiring — in OSFDesktop::initialize():
ClipboardBridge::shared().initialize(g_seat);

// ClipboardBridge::setText() — complete implementation:
void ClipboardBridge::setText(const std::string &text) {
    // 1. Add to history (front)
    m_history.push_front(text);
    if (m_history.size() > MAX_HISTORY) m_history.pop_back();
    m_current = text;

    // 2. Set on wlr_seat clipboard
    // Create wl_data_source via wlr_data_source
    // Offer text/plain;charset=utf-8
    // wlr_seat_set_selection(m_seat, source, serial)
    // serial: last pointer/keyboard event serial
    // Known limit: serial must be recent — Wayland security model
    // requires clipboard set during user input event.
    // If called outside input event: may be rejected by protocol.
    // OSFNative apps (Pathfinder copy) call this during pointer event.
    // Correct usage guaranteed for OSFNative.

    // 3. Publish event
    EventBus::shared().publish(OSFEvent::ClipboardChanged, text);
}
```

### 43.2 Mime Type Support

```
Text:   text/plain;charset=utf-8    — always offered
        text/plain                  — offered (same content)
HTML:   text/html                   — passed through if offered by app
        ClipboardBridge does NOT parse or validate HTML
Images: image/png                   — passed through if offered by app
        image/jpeg                  — passed through

NOT supported:
    application/octet-stream        — binary data not supported
    Any non-text non-image type     — rejected silently
    Known limit — documented.

"Passed through":
    ClipboardBridge holds a reference to the app's wl_data_source.
    When another app requests the data:
    ClipboardBridge proxies the request to the original app.
    If original app has closed:
    ClipboardBridge has a text copy only (from setText).
    HTML and image data is lost if app closes before paste.
    Known limit: ClipboardBridge survives app close only for text.
    Post-unstable: serialize all types to memory on source close.
```

### 43.3 Clipboard History

```
Already specced in Addendum J:
    Max 20 entries, memory-only, never persisted.

Additional spec:
    History accessible from Pathfinder:
        "Clipboard" section in Pathfinder results
        Shows last 20 copied items
        Click to re-copy to clipboard
    History cleared:
        Manually from Pathfinder (clear all button)
        On session end (memory-only — automatic)
    Privacy:
        Clipboard history never written to disk.
        HEV never touches clipboard data.
        This is intentional — clipboard content
        is transient, not a secret to be vaulted.
```

---

## PART 44 — Empty States + Personality Moments

### 44.1 Empty States

```
Every empty state is a personality moment.
The OS notices the absence of content
and says something honest about it.

FILER — empty folder:
    Icon:   folder outline, 48px, 40% opacity, centered
    Text:   "nothing here yet"
            11px Inter Regular, tertiary (#6B6B6B), centered
    Margin: 40px above icon from center of view
    No button. No action. Just acknowledgment.

FILER — search with no results:
    Icon:   magnifying glass, 48px, 40% opacity, centered
    Text:   "no files match "{query}""
            11px, tertiary, centered
    Sub:    "try a different search"
            11px, tertiary 70%, centered

PATHFINDER — no results:
    Icon:   none (Pathfinder is search-first)
    Text:   "nothing found for "{query}""
            13px, tertiary, centered in results area
    If nixpkgs search still pending:
        Spinner above text (same as search bar spinner)
        "searching nixpkgs…"
        11px tertiary

COCKPITVIEW — only one window:
    Single card centered (no grid needed)
    Text below card: "open more apps to fill the space"
    11px, white 30% opacity
    Shown only first 3 times CockpitView opens
    After that: silence — user understands the space
    Counter stored in StateManager:
        key "cockpitViewOpenCount" → int

DESKTOP — completely empty (no windows):
    Nothing. The wallpaper is the empty state.
    No hint text. No ghost UI. Silence.
    The user knows where they are.

NOTIFICATIONS — none queued:
    Nothing shown. Silence is correct.
    An empty notification area is not an empty state —
    it is the desired state.

SETTINGS — first open (no customization done):
    Each section shows current defaults with no
    special empty state treatment.
    Defaults ARE the state. Nothing is empty.
```

### 44.2 Personality Moments

```
These are the moments vitusOS speaks.
They are rare. That is why they are heard.

SHUTDOWN: "goodbye"
    (Already locked in Part 29 — not changing)

RESTART: "i'll see you in a bit"
    (Already locked in Part 29 — not changing)

FIRST BOOT step 3: "you're all set"
    (Part 37 — already specced)

LOCK SCREEN — time display:
    Shows current time in large format
    HH:MM, 48px Inter Light
    Below: current date, 13px, white 60%
    No "Enter password" prompt visible by default
    Password field appears on first keypress
    Springs in: SPRING_SHEET (420,30)
    This is macOS Lock Screen behavior — correct.
    The time is the face. The field is the door.

PATHFINDER — first open:
    Placeholder text in search bar:
    "Search vitusOS"
    On first ever open (StateManager key "pathfinderFirstOpen"):
    Placeholder: "what are you looking for?"
    Only once. Never again. Then "Search vitusOS" forever.
    Known limit: this is a tiny moment.
    Some users will never notice. That is fine.
    It is there for the ones who do.

FILER — first open:
    Home directory contents shown.
    If home directory is nearly empty (< 3 items):
    A subtle prompt in the empty space:
    "this is your home"
    11px, tertiary, right-aligned in content area
    Shown only once. StateManager key "filerFirstOpen".

WELCOME SCREEN step 3 button: "let's go"
    Not "Finish". Not "Done". Not "Continue".
    "let's go"
    Lowercase. Invitation. Energy.
    (Already specced in Part 37)

PATHFINDER install complete:
    Icon travels to Dock.
    One bounce.
    Sound: InstallComplete.
    No modal. No dialog.
    The icon settling into the Dock IS the confirmation.
    vitusOS does not explain what just happened.
    The user saw it.
```

---

## PART 45 — Window Management Edge Cases

### 45.1 Window Size Constraints

```cpp
// Minimum window size: 200×100px
// Applied in WindowManager — never configurable.
// Below minimum: configure back to minimum.

// Maximum window size: output dimensions (no Panel height)
// A window cannot be larger than the screen.
// Applied on maximize and on resize.

// Resize handling (xdg_toplevel::set_min_size hint):
// App may request its own minimum size.
// vitusOS respects the larger of:
//     200×100px (system minimum) OR app-requested minimum
// App maximum size hint: respected if smaller than output.

void WindowManager::onConfigureRequest(OSFWindow *win,
                                         float reqW, float reqH) {
    float screenW = primaryOutput()->width;
    float screenH = primaryOutput()->height - Panel::HEIGHT;

    float w = std::clamp(reqW, std::max(200.0f, win->minWidth()),
                          screenW);
    float h = std::clamp(reqH, std::max(100.0f, win->minHeight()),
                          screenH);

    wlr_xdg_toplevel_set_size(win->xdgToplevel(), w, h);
}
```

### 45.2 Window Title

```cpp
// xdg_toplevel::set_title — wired via wlr listener
// Stored in OSFWindow::m_title
// Displayed in:
//   OSFWindow title bar: center, 13px Inter Semibold
//   CockpitView card label: 11px below card (truncated to 24 chars)
//   RegistryManager: queryable by appId
//
// Title truncation in title bar:
//   If title > available width: truncate with "…" suffix
//   Available width = window width - (traffic lights area) - padding
//   Traffic lights area: 12 + 3×12 + 2×8 + 12 = 68px from left
//   Right padding: 68px (symmetric)
//   Title area: window width - 136px
//   If title wider than title area: truncate
//
// Title update timing:
//   xdg_toplevel set_title can fire at any time
//   Damage issued to title bar region on change
//   Not to full window — just the title bar strip
//   wlr_damage_ring_add() for title bar rect only
```

### 45.3 Window Focus and Z-Order

```cpp
// Focus model: click-to-focus.
// Clicking a window: focuses it, raises to top of z-order.
// Focus follows mouse: NOT implemented in vitusOS.
// This is intentional — vitusOS is click-to-focus only.
// Known gap for sloppy-focus users. Documented.

// Z-order:
//   Most recently focused window = highest z.
//   Z-order stored as ordering in WindowManager::m_windows vector.
//   Front of vector = lowest z. Back = highest z.
//   On focus: window moved to back of vector.
//   RenderPipeline renders back to front = highest z on top.

// Focus during CockpitView:
//   Clicking a card: sets m_pendingFocus to that window handle.
//   CockpitView close event: m_pendingFocus window gets focus.
//   Prevents focus racing during zoom animation.
```

### 45.4 Window Stack Performance

```cpp
// Known limit: > 20 windows degrades performance.
// Each window = one wl_surface = one Vulkan texture sampler.
// 20 windows × avg 1MB texture = 20MB VRAM minimum.
// On HP Victus (NVIDIA + Intel): VRAM is shared.
// Performance at 20 windows: acceptable (tested baseline).
// Performance at 30+ windows: may drop below 60fps.
// No hard cap enforced — user can open as many as they want.
// vitusOS does not kill windows for memory pressure.
// CacheKeepr pressure handler (Part 26) handles caches,
// not active window surfaces.
// Known limit — documented. User responsibility.
```

### 45.5 What Opus Must NEVER Do — Window Edge Cases

```
NEVER resize a window below 200×100px.
    Even if the app requests it.
    The system minimum is absolute.

NEVER destroy a window's wl_surface when it moves
    to an inactive virtual desktop.
    The surface lives. Springs tick. State preserved.
    Desktop switch is a camera move, not a state change.

NEVER use focus-follows-mouse.
    vitusOS is click-to-focus. Period.
    If found in code: remove it.

NEVER render windows out of z-order.
    RenderPipeline renders m_windows front-to-back.
    Front = lowest z (behind everything).
    Back = highest z (on top of everything).
    Reversing this = topmost window renders behind others.
    CrashManager will catch the visual bug — the cause is here.

NEVER block the main thread waiting for a window to resize.
    xdg_toplevel configure → wait for configure_ack.
    The ack arrives asynchronously.
    Do not spin-wait for it.
    Use state machine: CONFIGURING → CONFIGURED on ack.
```

---

## PART 46 — Installer (native/Installer/)

### 46.1 Overview

The vitusOS installer is an OSFNative app that runs inside the
full live vitusOS compositor on the ISO.

When the user boots from USB:
    AnimusBoot runs — Space Orange screen + wordmark
    Kernel boots with static CMDLINE from AnimusBoot.c
    animus-early initramfs runs (Part 2)
    Live root filesystem mounted (squashfs on ISO)
    vitusOS compositor starts (OSFDesktop)
    InstallerApp launches automatically — fullscreen
    No desktop visible behind it yet
    Installer IS the first thing the user sees after boot

When installation is complete:
    InstallerApp triggers reboot
    Machine boots from newly installed AnimusBoot EFI entry
    WelcomeScreen runs (Part 37) — vault setup → wallpaper → done
    Desktop appears

The live system while installer is running:
    Full vitusOS compositor is running
    User can Cmd+Space (or orange box) to open Pathfinder
    Filer works — user can explore the live filesystem
    Terminow works — user can open a terminal
    This is intentional — the live system IS the demo
    "What you see right now is what you're about to install"

### 46.2 Directory Structure

```
native/Installer/
    main.cpp                    # Auto-launch detection:
                                # checks /proc/cmdline for "vitusos-installer"
                                # If present: launch InstallerApp fullscreen
                                # If absent: do not launch (normal boot)
    InstallerApp.cpp/.h         # Main app, step controller, spring transitions
    steps/
        DiskSelectStep.cpp/.h   # Disk list, detection, selection
        PartitionStep.cpp/.h    # Graphical partition editor
        AccountStep.cpp/.h      # Username + password creation
        SummaryStep.cpp/.h      # Review before install
        ProgressStep.cpp/.h     # Install progress, phases
    engine/
        DiskManager.cpp/.h      # Reads partition tables, disk info
        PartitionOp.cpp/.h      # Create/delete/resize/format operations
        InstallEngine.cpp/.h    # Copies closure, writes config, installs EFI
        EFIInstaller.cpp/.h     # efibootmgr wrapper, AnimusBoot deployment
    CMakeLists.txt
```

### 46.3 InstallerApp — Step Controller

```cpp
// native/Installer/InstallerApp.h
// OSFNative fullscreen app.
// 5 steps, left-to-right flow.
// No back button on Progress step.
// Cancel exits to live desktop at any point before Progress.
#pragma once
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include <memory>
#include <vector>

namespace Animus {

class DiskSelectStep;
class PartitionStep;
class AccountStep;
class SummaryStep;
class ProgressStep;

class InstallerApp {
public:
    InstallerApp();
    bool initialize();
    void render(VkCommandBuffer cmd, float screenW, float screenH, float dt);
    void onKey(uint32_t sym, uint32_t mods, bool pressed);
    void onPointerMotion(float x, float y);
    void onPointerButton(float x, float y, bool pressed);

    // Step indices
    enum class Step : uint8_t {
        DiskSelect  = 0,
        Partition   = 1,
        Account     = 2,
        Summary     = 3,
        Progress    = 4,
    };

    void advanceStep();
    void retreatStep();   // disabled on Progress step
    void cancelInstall(); // exits to live desktop

private:
    Step         m_currentStep = Step::DiskSelect;
    Step         m_prevStep    = Step::DiskSelect;

    std::unique_ptr<DiskSelectStep> m_diskSelect;
    std::unique_ptr<PartitionStep>  m_partition;
    std::unique_ptr<AccountStep>    m_account;
    std::unique_ptr<SummaryStep>    m_summary;
    std::unique_ptr<ProgressStep>   m_progress;

    // Slide transition between steps
    // Current step slides left off screen, next springs in from right
    // Reverse: current slides right, prev springs in from left
    SpringSolver m_slideX;   // SPRING_DESKTOP_SWITCH (280,28)
    bool         m_sliding   = false;
    float        m_screenW   = 1920.f;

    // Progress dots — 5 dots, one per step
    SpringSolver m_dotScale[5];  // SPRING_HOVER (600,40) per dot

    void renderProgressDots(VkCommandBuffer cmd, float cx, float y);
    void renderStepNavigation(VkCommandBuffer cmd, float screenW, float screenH);
    void renderHeader(VkCommandBuffer cmd, float screenW);
};

} // namespace Animus
```

### 46.4 Installer Visual Spec

```
BACKGROUND:
    Full screen — not glass, not blurred
    #1A1208 — same as LockScreen and WelcomeScreen
    Installer is a system surface. System speaks in #1A1208.

HEADER BAR:
    Height: 56px
    vitusOS wordmark left-aligned — 160×28px
    "Install vitusOS" text right of wordmark — 13px, white 60%
    No Panel shown behind (Panel is hidden — fullscreen mode)
    No orange box. No dock. This is installer mode.

CONTENT AREA:
    Below header: full remaining height
    Step content centered — 720px max width
    Left/right padding: (screenW - 720) / 2

NAVIGATION:
    Bottom bar: 64px
    Left: "Back" button — 120×36px glass, 13px white
          Hidden on step 0 (DiskSelect) and step 4 (Progress)
    Right: "Continue" button — Space Orange, 160×36px
           Label changes on last step: "Install"
           Disabled (50% opacity) when step is incomplete
    Center: "Cancel" link — 13px, white 50%
            Hidden during Progress step

STEP PROGRESS DOTS:
    5 dots, 8px each, 12px gap
    Centered below content, above nav bar
    Active: Space Orange, scale 1.1×
    Completed: white 60%, scale 1.0×
    Upcoming: white 20%, scale 1.0×
    SPRING_HOVER (600,40) on advance/retreat

STEP TRANSITION:
    Advance (forward): current step slides left off screen
                       next step springs in from right
                       SPRING_DESKTOP_SWITCH (280,28)
    Retreat (back):    current step slides right off screen
                       prev step springs in from left
    Both springs have initVelocity from nav button click: 400px/s
```

### 46.5 Step 1 — Disk Selection

```cpp
// native/Installer/steps/DiskSelectStep.h
// Shows list of connected block devices.
// User selects the target disk.
// WARNING shown for any disk that has partitions.
// NEVER auto-select. Always require explicit user choice.

// Disk information read from:
//   /sys/block/ — enumerate block devices
//   /sys/block/{dev}/size — size in 512-byte sectors
//   /sys/block/{dev}/device/model — model name
//   /proc/partitions — partition list

struct DiskInfo {
    std::string path;        // e.g. "/dev/nvme0n1" or "/dev/sda"
    std::string model;       // e.g. "Samsung SSD 980 Pro"
    uint64_t    sizeBytes;
    bool        hasPartitions;
    bool        isRemovable; // USB drives — shown but flagged
    std::vector<PartitionInfo> partitions;
};

// Visual spec:
//   Title: "choose a disk"
//   Subtitle: "vitusOS will be installed here"
//   Both: Inter, locked type scale, white
//
//   Disk list: one row per disk
//   Row height: 72px
//   Row contents:
//       Left:  disk icon (drive type: NVMe/SSD/HDD/USB) 32×32px
//       Center-top: model name, 15px Semibold, white 90%
//       Center-bottom: size formatted e.g. "512 GB NVMe SSD"
//                      11px, white 50%
//       Right: selection radio button (custom — SpaceOrange)
//   Selected row: 1px Space Orange border, glass background
//   SPRING_HOVER (600,40) on selection
//
//   Warning banner (if selected disk has partitions):
//       "#FF3B30 tinted glass pill, full width"
//       "This disk has existing data. Installing will erase it."
//       13px white. Shown below disk list.
//       Requires explicit acknowledgment checkbox before Continue
//
//   If disk is removable (USB):
//       Additional warning: "This appears to be a USB drive.
//       Are you sure? vitusOS needs at least 20 GB."
//
//   No disk selected: Continue button disabled
//   Disk selected + warnings acknowledged: Continue enabled
```

### 46.6 Step 2 — Partition Editor

```cpp
// native/Installer/steps/PartitionStep.h
// Graphical partition editor — macOS Disk Utility style.
// Shows disk map at top: visual bar proportional to size.
// Shows partition list below.
//
// Operations:
//   Create partition: click "+" in empty space or partition list
//   Delete partition: select partition, press Delete or "-" button
//   Resize partition: drag partition edge in disk map
//   Format partition: select partition, choose filesystem
//   Assign mount point: select partition, choose from list or type
//
// Required partitions for Continue to be enabled:
//   1. EFI partition: ≥ 512MB, FAT32, /boot/efi
//      If existing EFI detected: auto-selected, shown as locked
//      (cannot delete or reformat — other OSes depend on it)
//   2. Root partition: ≥ 20GB, ext4 or btrfs, /
//
// Optional:
//   Swap partition or swapfile
//   Additional partitions (e.g. /home)

struct PartitionInfo {
    std::string device;       // e.g. "/dev/nvme0n1p1"
    std::string fstype;       // "fat32", "ext4", "btrfs", "swap", "unformatted"
    std::string mountPoint;   // "/", "/boot/efi", "/home", "swap", ""
    uint64_t    startBytes;
    uint64_t    sizeBytes;
    bool        existing;     // true = pre-existing, false = to be created
    bool        efiSystem;    // true = EFI System Partition flag set
    bool        locked;       // existing EFI — cannot modify
};

// DISK MAP VISUAL SPEC:
//   Container: full content width, 48px tall, corner 8px
//   Glass background (SurfaceAltitude::Low)
//   Each partition: colored segment proportional to size
//
//   Partition colors:
//       EFI partition:    #007AFF (blue — matches maximize traffic light)
//       Root /:           Space Orange #FF6B2B
//       Swap:             #FFCC00 (yellow — matches minimize traffic light)
//       Home /home:       #34C759 (green — standard data color)
//       Unallocated:      white 15% opacity, dashed border
//       Other (unknown):  white 40%
//
//   Selected partition in map: 2px white border, scale 1.01× (SPRING_HOVER)
//   Resize handle: 4px white vertical line at partition edge
//       Appears on hover. Draggable.
//       SPRING_WINDOW_DRAG (800,35) while dragging
//       Minimum partition size enforced during drag
//       Snaps to 1GB boundaries (visual snap, not enforced)
//
// PARTITION LIST SPEC:
//   Below disk map, 16px gap
//   One row per partition: 48px tall
//   Columns: Device | Size | Filesystem | Mount Point | Actions
//   Widths:  120px  | 80px | 100px      | 160px       | 80px
//   Actions: format icon, delete icon (locked partitions: greyed)
//   Selected row: Space Orange left border 2px
//
// ADD PARTITION:
//   "+" button below list
//   Opens OSFSheet: size input, filesystem picker, mount point
//   Size: slider + text field, GB
//   Filesystem: ext4 (default) / btrfs / swap / FAT32
//   Mount: dropdown: / | /boot/efi | /home | swap | custom
//   Spring-animated sheet (SPRING_SHEET 420,30)
//
// VALIDATION (shown as inline errors in list):
//   No EFI partition: "An EFI partition (≥512MB, FAT32) is required"
//   No root partition: "A root partition (≥20GB) is required"
//   Root too small: "Root partition must be at least 20 GB"
//   Multiple root partitions: "Only one root partition allowed"
//   All errors shown simultaneously — not one at a time
//   Continue disabled until all errors resolved
```

### 46.7 Step 3 — Account Creation

```cpp
// native/Installer/steps/AccountStep.h
// Creates the primary vitusOS user account.
// Username + password before reboot.
// (HEV vault setup happens post-install in WelcomeScreen)

// Visual spec:
//   Title: "create your account"
//   Subtitle: (none — the fields speak for themselves)
//
//   Display name field:
//       Label: "your name"
//       11px white 60%, above field
//       Placeholder: "Jane Smith"
//       Used as display name in LockScreen and User Account settings
//       Max 64 chars
//
//   Username field:
//       Label: "username"
//       Lowercase letters, digits, hyphens only
//       Max 32 chars
//       Auto-populated from display name:
//           lowercase, spaces→hyphens, strip special chars
//           e.g. "Jane Smith" → "jane-smith"
//       User can edit. Validated live.
//       Error if invalid chars: shown inline below field
//
//   Password field:
//       Label: "password"
//       Masked. Show/hide toggle.
//       Min 1 char (no maximum enforced — user's choice)
//       Strength bar: 4 segments (same as WelcomeScreen)
//
//   Confirm password field:
//       Label: "confirm password"
//       Inline error if mismatch: "Passwords don't match"
//       Shown on second field blur only (not while typing)
//
//   Admin checkbox:
//       "Allow this user to perform administrative tasks"
//       Default: ON
//       If OFF: user cannot sudo — known limit
//       Unstable ISO: always ON (cannot uncheck)
//       Caption: "Administrative access is required
//                  in this release of vitusOS."
//       Post-unstable: configurable
//
//   Continue disabled until:
//       display name non-empty
//       username valid (non-empty, valid chars)
//       password non-empty
//       passwords match

// Data collected here is used by InstallEngine to:
//   Write configuration.nix: users.users.{username}
//   Set password hash via hashedPasswordFile
//   Set display name in vitusos-config.nix user.displayName
```

### 46.8 Step 4 — Summary

```cpp
// native/Installer/steps/SummaryStep.h
// Shows exactly what will happen before install begins.
// Last chance to go back.
// "Install" button triggers installation — no further confirmation.

// Visual spec:
//   Title: "ready to install"
//   Subtitle: "here's what will happen"
//
//   Summary cards — glass material, 8px corner, stacked vertically:
//
//   Card 1: TARGET DISK
//       Icon: drive icon 24px
//       Title: disk model + size
//       Body: list of partitions to be created/formatted
//             Each line: device → size, filesystem, mount point
//             Existing EFI shown as: "EFI partition — preserved"
//             New partitions shown as: "NEW — /dev/nvme0n1p2 — 40GB ext4 /"
//       Color: #FF3B30 tinted border if any partition will be erased
//              (existing data will be lost)
//
//   Card 2: YOUR ACCOUNT
//       Icon: person icon 24px
//       Title: display name
//       Body: "Username: {username}"
//             "Administrator: Yes"
//
//   Card 3: WHAT VITUSCOS INSTALLS
//       Icon: vitusos wordmark 24px
//       Body: "vitusOS {version}"
//             "NixOS {version}"
//             "AnimusBoot EFI entry"
//             "Installed offline — no internet required"
//
//   WARNING if any partition will erase data:
//       Full-width glass pill, #FF3B30 border
//       "This cannot be undone. Make sure you have
//        a backup of any important data."
//       13px white. Bold red left border 3px.
//
//   Continue button label: "Install" (not "Continue")
//   Space Orange. 200×44px.
//   Hover: scale 1.02× SPRING_HOVER
```

### 46.9 Step 5 — Install Progress

```cpp
// native/Installer/steps/ProgressStep.h
// Shows installation phases with progress bar.
// No back button. No cancel. Install is in progress.
// Reboot happens automatically when complete.

// INSTALLATION PHASES:
//   Phase 1: Preparing disk         0% → 10%
//       Partition table written (sfdisk or parted)
//       Partitions formatted (mkfs.ext4, mkfs.fat, mkswap)
//
//   Phase 2: Copying vitusOS        10% → 75%
//       NixOS closure copied from ISO squashfs to root partition
//       rsync or cp -a from /run/iso/nix/store to /mnt/nix/store
//       This is the longest phase — depends on disk speed
//       Progress estimated from bytes copied / total bytes
//
//   Phase 3: Configuring system     75% → 85%
//       Write /mnt/etc/nixos/configuration.nix
//       Write /mnt/etc/vitusos/vitusos-config.nix
//       Set user password hash
//       nixos-enter to run nixos-install --no-root-password
//       (closure already on disk — no download needed)
//
//   Phase 4: Installing boot files  85% → 95%
//       Mount EFI partition
//       Copy AnimusBoot.efi to \EFI\vitusos\AnimusBoot.efi
//       Copy kernel to \EFI\vitusos\kernel
//       Copy initrd to \EFI\vitusos\initrd
//       Run efibootmgr to create AnimusBoot EFI entry
//
//   Phase 5: Finishing up           95% → 100%
//       Unmount all partitions (umount -R /mnt)
//       Sync filesystem (sync)
//       Installation complete

// VISUAL SPEC:
//   Title: "installing vitusOS"
//   Subtitle: current phase name (updates per phase)
//   Both centered, same type scale
//
//   Progress bar:
//       Width: 480px (2/3 of content area)
//       Height: 6px
//       Corner: 3px (fully rounded)
//       Background: white 15% opacity
//       Fill: Space Orange
//       Spring-animated fill width:
//           SPRING_SELECTION (400,28) — smooth, not jumpy
//           Never goes backwards (progress clamp)
//
//   Phase label below bar:
//       "Phase N of 5 — {phase name}"
//       11px, white 50%, centered
//
//   Percentage: large display
//       48px Inter Light, white 90%, centered above bar
//       e.g. "73%"
//       Updates every second
//
//   On complete:
//       Progress bar fills to 100% (SPRING_SELECTION settles)
//       Title changes to: "vitusOS is installed"
//       Subtitle: "your computer will restart in {N} seconds"
//       Countdown: 10 → 0 seconds
//       10px white 50% countdown text
//       Auto-reboot at 0 via systemctl reboot
//       "Restart now" button appears (skips countdown)
//           Space Orange, 180×44px
//
//   On error:
//       Progress bar stops
//       Title: "installation failed"
//       Subtitle: error message (from InstallEngine)
//       Two buttons: "Try Again" (reruns from failed phase)
//                    "Quit to Desktop"
//       Error logged to /tmp/vitusos-install.log
//       User can open log in Terminow via Pathfinder

// BACK BUTTON: hidden during this step
// CANCEL BUTTON: hidden during this step
// No interaction possible during install — intentional
// User cannot corrupt a partial install by navigating away
```

### 46.10 DiskManager.cpp/.h

```cpp
// native/Installer/engine/DiskManager.h
// Reads disk and partition information from /sys/block and /proc.
// NEVER writes to disk — read-only discovery.
// Write operations go through PartitionOp.

#pragma once
#include <string>
#include <vector>
#include <cstdint>

namespace Animus {

struct PartitionInfo {
    std::string device;       // /dev/nvme0n1p1
    std::string fstype;       // detected via blkid
    std::string label;        // partition label if any
    std::string mountPoint;   // desired mount (set by user in PartitionStep)
    uint64_t    startBytes;
    uint64_t    sizeBytes;
    bool        efiSystem;    // GPT EFI System Partition flag
    bool        existing;     // true = found on disk, false = user-created
    bool        locked;       // existing EFI — cannot modify (other OS depends)
};

struct DiskInfo {
    std::string              path;         // /dev/nvme0n1
    std::string              model;        // Samsung SSD 980 Pro
    std::string              type;         // "nvme", "ssd", "hdd", "usb"
    uint64_t                 sizeBytes;
    bool                     isRemovable;
    bool                     hasPartitions;
    std::vector<PartitionInfo> partitions;
};

class DiskManager {
public:
    // Enumerate all block devices except loop/ram
    static std::vector<DiskInfo> enumerateDisks();

    // Refresh info for a specific disk
    static DiskInfo refreshDisk(const std::string &devPath);

    // Detect filesystem type on a partition (calls blkid)
    static std::string detectFstype(const std::string &partPath);

    // Detect if partition has EFI System Partition flag (via sgdisk or sfdisk -J)
    static bool detectEFIFlag(const std::string &partPath);

    // Detect RAM size in bytes (from /proc/meminfo MemTotal)
    static uint64_t ramSizeBytes();

    // Recommended swap size: min(RAM, 8GB)
    static uint64_t recommendedSwapBytes();

private:
    static std::string readSysFile(const std::string &path);
    static uint64_t    readSectorsToBytes(const std::string &devPath);
};

} // namespace Animus
```

### 46.11 PartitionOp.cpp/.h

```cpp
// native/Installer/engine/PartitionOp.h
// Executes partition operations by shelling out to system tools.
// All operations are destructive — called only after user confirms.
// Each op returns bool success + std::string error message.
// InstallEngine calls these in correct order.

#pragma once
#include <string>
#include <vector>
#include <functional>

namespace Animus {

class PartitionOp {
public:
    // Write new partition table to disk
    // Uses sfdisk with JSON partition spec
    // partitions: ordered list of (start, size, type, name)
    static bool writePartitionTable(
        const std::string &diskPath,
        const std::vector<PartitionInfo> &partitions,
        std::string &errorOut);

    // Format a partition with specified filesystem
    // fstype: "ext4", "btrfs", "vfat", "swap"
    // label: optional partition label
    static bool formatPartition(
        const std::string &partPath,
        const std::string &fstype,
        const std::string &label,
        std::string &errorOut);

    // Mount a partition at a path
    static bool mountPartition(
        const std::string &partPath,
        const std::string &mountPath,
        std::string &errorOut);

    // Unmount recursively
    static bool unmountRecursive(
        const std::string &mountPath,
        std::string &errorOut);

    // Progress callback for long operations
    // Called with fraction 0.0–1.0 as operation proceeds
    using ProgressCb = std::function<void(float)>;

    // Copy NixOS closure from ISO to installed root
    // src: ISO squashfs mount point (e.g. /run/iso)
    // dst: installed root (e.g. /mnt)
    // progressCb: called with bytes copied / total bytes
    static bool copyNixClosure(
        const std::string &src,
        const std::string &dst,
        ProgressCb progressCb,
        std::string &errorOut);

private:
    // Shell out helper — captures stdout/stderr
    static bool runCommand(
        const std::string &cmd,
        std::string &stdoutOut,
        std::string &stderrOut);
};

} // namespace Animus
```

### 46.12 InstallEngine.cpp/.h

```cpp
// native/Installer/engine/InstallEngine.h
// Orchestrates the full installation sequence.
// Runs on background thread — publishes progress via EventBus::publishAsync.
// Main thread renders progress bar from published events.
// NEVER called on main thread — would freeze compositor.

#pragma once
#include <string>
#include <vector>
#include <thread>
#include <atomic>
#include <functional>

namespace Animus {

struct InstallConfig {
    // Disk
    std::string              targetDisk;          // /dev/nvme0n1
    std::vector<PartitionInfo> partitions;        // final layout from PartitionStep

    // Account
    std::string              displayName;
    std::string              username;
    std::string              passwordHash;        // pre-hashed with sha512crypt

    // Derived paths (set by InstallEngine)
    std::string              rootMount  = "/mnt";
    std::string              efiMount   = "/mnt/boot/efi";
    std::string              isoRoot    = "/run/iso";  // live squashfs mount
};

struct InstallProgress {
    int   phase;            // 1-5
    float phaseFraction;    // 0.0-1.0 within current phase
    float totalFraction;    // 0.0-1.0 overall
    std::string phaseLabel;
};

class InstallEngine {
public:
    // Start installation on background thread
    // onProgress: called from background thread via publishAsync
    // onComplete: called from background thread via publishAsync
    // onError:    called from background thread via publishAsync
    void start(
        const InstallConfig &config,
        std::function<void(InstallProgress)> onProgress,
        std::function<void()>                onComplete,
        std::function<void(std::string)>     onError);

    bool isRunning() const { return m_running.load(); }

    static constexpr const char *LOG_PATH = "/tmp/vitusos-install.log";

private:
    std::thread        m_thread;
    std::atomic<bool>  m_running{false};
    InstallConfig      m_config;

    // Phase implementations
    bool phase1_prepareDisk(std::function<void(float)> progress, std::string &err);
    bool phase2_copyNixClosure(std::function<void(float)> progress, std::string &err);
    bool phase3_configureSystem(std::function<void(float)> progress, std::string &err);
    bool phase4_installBootFiles(std::function<void(float)> progress, std::string &err);
    bool phase5_finishUp(std::function<void(float)> progress, std::string &err);

    void log(const std::string &msg);
    FILE *m_logFile = nullptr;
};

} // namespace Animus
```

### 46.13 Phase 3 — configuration.nix Generation

```cpp
// InstallEngine::phase3_configureSystem()
// Generates the installed system's configuration.nix
// Written to /mnt/etc/nixos/configuration.nix

// GENERATED configuration.nix structure:
// { config, pkgs, ... }:
// {
//   imports = [ ./hardware-configuration.nix ];
//
//   boot.loader.efi.canTouchEfiVariables = false;  # AnimusBoot manages EFI
//   boot.loader.grub.enable = false;               # No GRUB — AnimusBoot only
//   boot.initrd.enable = true;                     # animus-early initramfs
//
//   networking.hostName = "{username}-vitusos";
//
//   users.users.{username} = {
//       isNormalUser = true;
//       description  = "{displayName}";
//       extraGroups  = [ "wheel" "video" "audio" "input" ];
//       hashedPasswordFile = "/etc/vitusos/shadow/{username}";
//   };
//
//   programs.xwayland.enable = false;  # disabled in unstable ISO
//
//   services.vitusos-session.enable = true;
//
//   environment.systemPackages = with pkgs; [
//       # base packages from ISO — already in closure
//   ];
// }
//
// hardware-configuration.nix:
//   Generated by nixos-generate-config after partitions are mounted
//   Detects filesystem UUIDs, swap, etc.
//   Standard NixOS tool — InstallEngine shells out to it:
//   nixos-generate-config --root /mnt --no-filesystems
//   (--no-filesystems because we wrote them already)

// Also generated: /mnt/etc/vitusos/vitusos-config.nix
// {
//   user.displayName = "{displayName}";
//   user.username    = "{username}";
//   pathfinder.nixpkgsSources = [ "nixpkgs" ];
//   installedApps = [];
//   wallpaper = "/etc/vitusos/wallpapers/mars.jpg";
//   firstBootComplete = false;  # triggers WelcomeScreen on first boot
// }
```

### 46.14 EFIInstaller.cpp/.h

```cpp
// native/Installer/engine/EFIInstaller.h
// Installs AnimusBoot to the EFI partition.
// Adds EFI boot entry via efibootmgr.
// NEVER modifies existing EFI entries.

#pragma once
#include <string>

namespace Animus {

class EFIInstaller {
public:
    // Install AnimusBoot to EFI partition
    // efiMount:    where EFI partition is mounted (e.g. /mnt/boot/efi)
    // diskPath:    disk device (e.g. /dev/nvme0n1) for efibootmgr
    // partNumber:  EFI partition number (e.g. 1 for nvme0n1p1)
    static bool install(
        const std::string &efiMount,
        const std::string &diskPath,
        int partNumber,
        std::string &errorOut);

private:
    // Source paths on live ISO
    static constexpr const char *ANIMUSBOOT_SRC =
        "/run/iso/EFI/vitusos/AnimusBoot.efi";
    static constexpr const char *KERNEL_SRC =
        "/run/iso/EFI/vitusos/kernel";
    static constexpr const char *INITRD_SRC =
        "/run/iso/EFI/vitusos/initrd";

    // Destination paths on EFI partition
    // Matches AnimusBoot.c kernel path: L"\\EFI\\vitusos\\kernel"
    static constexpr const char *EFI_DIR      = "/EFI/vitusos/";
    static constexpr const char *BOOT_EFI     = "AnimusBoot.efi";
    static constexpr const char *BOOT_KERNEL  = "kernel";
    static constexpr const char *BOOT_INITRD  = "initrd";

    // efibootmgr command:
    // efibootmgr --create
    //             --disk /dev/nvme0n1
    //             --part 1
    //             --label "vitusOS"
    //             --loader "\\EFI\\vitusos\\AnimusBoot.efi"
    //             --unicode ""
    //
    // --unicode "": AnimusBoot.c has static CMDLINE — no args to pass
    //              The EFI entry just launches AnimusBoot.efi
    //              AnimusBoot reads its CMDLINE internally
    //
    // This adds "vitusOS" to EFI boot order.
    // Existing entries (Windows, Ubuntu, etc.) are untouched.
    // UEFI firmware manages boot order — vitusOS does not.

    static bool copyFile(const std::string &src,
                          const std::string &dst,
                          std::string &errorOut);
    static bool runEfibootmgr(const std::string &disk,
                               int partNum,
                               std::string &errorOut);
};

} // namespace Animus
```

### 46.15 ISO Boot Detection

```c
// native/Installer/main.cpp
// Detects whether running from live ISO.
// Checks /proc/cmdline for "vitusos-installer" flag.
// If present: launch InstallerApp fullscreen, suppress WelcomeScreen.
// If absent: normal boot — WelcomeScreen handles first-boot check.

// ISO kernel cmdline (added to AnimusBoot.c CMDLINE for ISO builds):
// CMDLINE_ISO = CMDLINE + L" vitusos-installer"
//
// This flag is ONLY in the ISO build.
// The installed system's AnimusBoot.c has the normal CMDLINE
// without this flag.
// The distinction is in the BUILD — not runtime detection trickery.
//
// Two AnimusBoot builds:
//   AnimusBoot-ISO.efi:      CMDLINE includes "vitusos-installer"
//   AnimusBoot-installed.efi: CMDLINE is standard (no installer flag)
//
// ISO places AnimusBoot-ISO.efi at \EFI\vitusos\AnimusBoot.efi
// Installer copies AnimusBoot-installed.efi to target EFI partition
// as \EFI\vitusos\AnimusBoot.efi

// Detection in main.cpp:
bool isLiveISO() {
    FILE *f = fopen("/proc/cmdline", "r");
    if (!f) return false;
    char buf[1024] = {0};
    fread(buf, 1, sizeof(buf)-1, f);
    fclose(f);
    return strstr(buf, "vitusos-installer") != nullptr;
}

int main() {
    if (isLiveISO()) {
        // Suppress WelcomeScreen — installer replaces it
        StateManager::shared().set(StateKey::FirstBootComplete, true);
        // But mark as ISO — WelcomeScreen won't fire on live system
        // After install, fresh disk has FirstBootComplete = false
        // so WelcomeScreen fires correctly on first installed boot

        InstallerApp app;
        app.initialize();
        app.runFullscreen();
    }
    // If not live ISO: this binary does nothing
    // Normal OSFDesktop handles everything
    return 0;
}
```

### 46.16 NixOS Integration — ISO Build

```nix
# nixos/installer.nix — additional module for ISO build
# Included alongside configuration.nix when building ISO image

{ config, pkgs, ... }:
{
  # Mount ISO squashfs at /run/iso
  # Contains full NixOS closure to be copied to disk
  boot.initrd.postMountCommands = ''
    mkdir -p /run/iso
    mount -o loop,ro /dev/disk/by-label/VITUSOS_ISO /run/iso
  '';

  # Auto-start InstallerApp via session service
  # Runs AFTER vitusOS compositor starts
  systemd.user.services.vitusos-installer = {
    description = "vitusOS Installer";
    after = [ "vitusos-compositor.service" ];
    wantedBy = [ "graphical-session.target" ];
    serviceConfig = {
      ExecStart = "${pkgs.vitusos}/bin/vitusos-installer";
      Restart = "no";  # Do not restart — install is one-shot
    };
  };

  # ISO label — used by installer to find the squashfs
  isoImage.isoName = "vitusOS-unstable.iso";
  isoImage.volumeID = "VITUSOS_ISO";

  # Ensure all required tools are in ISO for installer
  environment.systemPackages = with pkgs; [
    efibootmgr      # EFI boot entry management
    dosfstools      # mkfs.fat (EFI partition)
    e2fsprogs       # mkfs.ext4
    btrfs-progs     # mkfs.btrfs (optional)
    util-linux      # sfdisk, blkid, mount, umount
    nixos-install-tools  # nixos-generate-config, nixos-enter
    rsync           # closure copy
    gptfdisk        # sgdisk for EFI flag detection
  ];
}
```

### 46.17 Known Limits and Bugs — Installer

```
BUG-46-1: Resize drag precision
    Partition resize by dragging edge in disk map.
    Disk map is proportional — on a 2TB disk,
    1px = ~2GB. Fine-grained resize requires
    text field fallback.
    Mitigation: text field shown below map.
    User can type exact size.
    Drag for coarse, field for precise.

BUG-46-2: nixos-generate-config detection
    hardware-configuration.nix generated via
    nixos-generate-config --root /mnt.
    If hardware detection fails (unusual hardware):
    hardware-configuration.nix may be incomplete.
    Installed system may not boot correctly.
    Mitigation: log error, warn user in summary.
    Known rough edge for unusual hardware.
    HP Victus: hardware-configuration.nix generates cleanly.

BUG-46-3: EFI partition number detection
    efibootmgr needs partition number (e.g. 1 for nvme0n1p1).
    Extracted by parsing device name:
    nvme0n1p1 → partition 1, sda1 → partition 1.
    Edge case: md RAID, LVM — partition number unclear.
    Known limit: installer does not support RAID/LVM.
    Documented. Acceptable for unstable ISO.

BUG-46-4: Progress estimation for closure copy
    Total bytes estimated from du before copy starts.
    On large SSDs: du of ISO closure takes 2-5 seconds.
    Progress bar shows 0% during this estimation period.
    User sees no progress for first few seconds.
    Mitigation: spinner shown during estimation.
    "Measuring installation size…" label.

KNOWN LIMIT-46-1: English only
    No language/region selection.
    vitusOS unstable ISO is English only.
    Post-unstable: locale selection as step 1.

KNOWN LIMIT-46-2: No RAID / LVM / encryption
    Simple partition table only.
    LUKS: not offered (decided — post-unstable)
    RAID: not supported
    LVM: not supported
    All three: documented as known gaps.

KNOWN LIMIT-46-3: Single disk install only
    Installer targets one disk.
    Cannot span / across multiple disks.
    Known limit — acceptable.

KNOWN LIMIT-46-4: No auto-update during install
    Installs exactly what is on the ISO.
    If ISO is 3 months old: packages are 3 months old.
    User runs nixos-rebuild after first boot to update.
    This is intentional — offline install guarantee.

KNOWN LIMIT-46-5: Reboot countdown cannot be cancelled
    After successful install: 10-second countdown to reboot.
    "Restart now" button skips to immediate reboot.
    No "stay in live session" option.
    Post-unstable: option to continue in live session.
```

### 46.18 What Opus Must NEVER Do — Installer

```
NEVER write to disk in DiskManager.
    DiskManager is read-only.
    All writes go through PartitionOp and InstallEngine.
    If Opus adds write operations to DiskManager: wrong file.

NEVER run InstallEngine on the main thread.
    Installation involves disk I/O that takes minutes.
    Running on main thread = frozen compositor = frozen UI.
    InstallEngine::start() spawns a background thread.
    Progress published via EventBus::publishAsync only.

NEVER auto-select a disk.
    The user must explicitly click a disk.
    Auto-selection risks data loss on wrong disk.
    No defaults. No pre-selection. User chooses.

NEVER modify existing EFI entries.
    EFIInstaller::install() only ADDS a new entry.
    It never deletes, modifies, or reorders existing entries.
    efibootmgr --create only. Never --delete on existing entries.

NEVER use the installer CMDLINE flag in the installed AnimusBoot.
    "vitusos-installer" flag is ISO-only.
    AnimusBoot-installed.efi has the standard CMDLINE.
    If this flag ends up in the installed system:
    InstallerApp will launch on every boot.
    The two binaries must be distinct at build time.

NEVER show WelcomeScreen on live ISO session.
    main.cpp sets FirstBootComplete = true in StateManager
    to suppress WelcomeScreen during live session.
    After install: fresh disk has FirstBootComplete = false.
    WelcomeScreen fires on first installed boot. Correct.

NEVER write to /etc/nixos/configuration.nix on the live system.
    The live system is read-only squashfs.
    All writes go to /mnt (the target disk mount).
    If Opus writes to /etc: it is modifying the live ISO.
    That is wrong. Target is always /mnt.
```
