/**
 * @file AnimusBoot.c
 * @brief Stage 0 UEFI Bootloader for vitusOS.
 *
 * Implements GOP display initialization, GPU vendor detection, EFI variable handoff
 * (ANIMUS_GPU_HANDOFF), zero-flicker splash rendering, and direct Linux kernel execution.
 */

#include <efi.h>
#include <efilib.h>
#include "../include/AnimusHandoff.h"

static EFI_GUID gHandoffGuid = ANIMUS_HANDOFF_GUID;
static const CHAR16 *gKernelPaths[] = {
    L"\\EFI\\vitusos\\kernel",
    L"\\EFI\\vitusos\\bzImage",
    L"\\casper\\vmlinuz",
    NULL
};

static const CHAR16 *gDefaultCmdline =
    L"BOOT_IMAGE=/EFI/vitusos/kernel "
    L"quiet splash "
    L"vt.global_cursor_default=0 "
    L"loglevel=0 "
    L"nvidia-drm.modeset=1 "
    L"nvidia-drm.fbdev=1 "
    L"amdgpu.modeset=1 "
    L"amdgpu.seamless=1 "
    L"i915.fastboot=1 "
    L"drm.modeset=1 "
    L"console=ttyS0,115200 "
    L"console=tty1 "
    L"systemd.show_status=0";

/**
 * @brief Detects Primary Display GPU via PCI I/O Protocol.
 */
static EFI_STATUS DetectGpu(ANIMUS_GPU_HANDOFF *Handoff) {
    EFI_STATUS Status;
    UINTN HandleCount = 0;
    EFI_HANDLE *HandleBuffer = NULL;

    Status = uefi_call_wrapper(BS->LocateHandleBuffer, 5,
        ByProtocol, &PciIoProtocol, NULL, &HandleCount, &HandleBuffer);

    if (EFI_ERROR(Status) || HandleCount == 0) {
        // Fallback default
        Handoff->vendor = GPU_VENDOR_INTEL_LEGACY;
        Handoff->gpu_type = GPU_TYPE_INTEGRATED;
        Handoff->device_id = 0x46A6;
        Handoff->bus_number = 0;
        return EFI_SUCCESS;
    }

    for (UINTN i = 0; i < HandleCount; i++) {
        EFI_PCI_IO_PROTOCOL *PciIo;
        Status = uefi_call_wrapper(BS->HandleProtocol, 3,
            HandleBuffer[i], &PciIoProtocol, (VOID**)&PciIo);

        if (EFI_ERROR(Status)) continue;

        UINT16 VendorId = 0, DeviceId = 0;
        UINT8 ClassCode[3] = {0};

        uefi_call_wrapper(PciIo->Pci.Read, 5, PciIo, EfiPciIoWidthUint16, 0x00, 1, &VendorId);
        uefi_call_wrapper(PciIo->Pci.Read, 5, PciIo, EfiPciIoWidthUint16, 0x02, 1, &DeviceId);
        uefi_call_wrapper(PciIo->Pci.Read, 5, PciIo, EfiPciIoWidthUint8, 0x09, 3, &ClassCode);

        // Class 0x03 = Display Controller
        if (ClassCode[2] == 0x03) {
            UINTN Segment, Bus, Device, Function;
            uefi_call_wrapper(PciIo->GetLocation, 5, PciIo, &Segment, &Bus, &Device, &Function);

            Handoff->device_id = DeviceId;
            Handoff->bus_number = (UINT8)Bus;

            if (VendorId == 0x10DE) {
                Handoff->vendor = GPU_VENDOR_NVIDIA;
                Handoff->gpu_type = GPU_TYPE_DISCRETE;
                break; // Prioritize discrete NVIDIA
            } else if (VendorId == 0x1002) {
                Handoff->vendor = GPU_VENDOR_AMD;
                Handoff->gpu_type = (Bus > 0) ? GPU_TYPE_DISCRETE : GPU_TYPE_INTEGRATED;
                break;
            } else if (VendorId == 0x8086) {
                if (DeviceId >= 0x5690 && DeviceId <= 0x57FF) {
                    Handoff->vendor = GPU_VENDOR_INTEL_ARC;
                    Handoff->gpu_type = GPU_TYPE_DISCRETE;
                } else {
                    Handoff->vendor = GPU_VENDOR_INTEL_LEGACY;
                    Handoff->gpu_type = GPU_TYPE_INTEGRATED;
                }
            }
        }
    }

    if (HandleBuffer) {
        uefi_call_wrapper(BS->FreePool, 1, HandleBuffer);
    }

    return EFI_SUCCESS;
}

/**
 * @brief Renders the canonical vitusOS splash logo centered on GOP framebuffer.
 */
static VOID RenderWordmark(UINT32 *Fb, UINT32 Stride, UINT32 Width, UINT32 Height) {
    UINT32 Color = 0xFFFFFFFF; // Clean White
    UINT32 Accent = 0xFFFF6B00; // Space Orange #FF6B00

    // Draw central brand mark crest
    UINT32 CenterX = Width / 2;
    UINT32 CenterY = Height / 2;

    // Outer glow square
    for (INT32 dy = -16; dy <= 16; dy++) {
        for (INT32 dx = -16; dx <= 16; dx++) {
            UINT32 px = (UINT32)(CenterX + dx);
            UINT32 py = (UINT32)(CenterY + dy);
            if (px < Width && py < Height) {
                if ((dx * dx + dy * dy) <= 16 * 16) {
                    Fb[py * Stride + px] = Accent;
                }
            }
        }
    }

    // Inner core
    for (INT32 dy = -8; dy <= 8; dy++) {
        for (INT32 dx = -8; dx <= 8; dx++) {
            UINT32 px = (UINT32)(CenterX + dx);
            UINT32 py = (UINT32)(CenterY + dy);
            if (px < Width && py < Height) {
                if ((dx * dx + dy * dy) <= 8 * 8) {
                    Fb[py * Stride + px] = Color;
                }
            }
        }
    }
}

/**
 * @brief Configures GOP mode and prepares zero-flicker background.
 */
static EFI_STATUS SetupGopAndRender(ANIMUS_GPU_HANDOFF *Handoff) {
    EFI_STATUS Status;
    EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop;

    Status = uefi_call_wrapper(BS->LocateProtocol, 3,
        &GraphicsOutputProtocol, NULL, (VOID**)&Gop);

    if (EFI_ERROR(Status) || !Gop) {
        return Status;
    }

    Handoff->framebuffer_base = (UINT64)Gop->Mode->FrameBufferBase;
    Handoff->framebuffer_size = (UINT32)Gop->Mode->FrameBufferSize;
    Handoff->horizontal_resolution = Gop->Mode->Info->HorizontalResolution;
    Handoff->vertical_resolution = Gop->Mode->Info->VerticalResolution;
    Handoff->pixels_per_scanline = Gop->Mode->Info->PixelsPerScanLine;
    Handoff->pixel_format = (UINT32)Gop->Mode->Info->PixelFormat;

    // Fill screen with #1A1208 (Warm Black)
    UINT32 *Fb = (UINT32*)Gop->Mode->FrameBufferBase;
    UINT32 Stride = Gop->Mode->Info->PixelsPerScanLine;
    UINT32 W = Gop->Mode->Info->HorizontalResolution;
    UINT32 H = Gop->Mode->Info->VerticalResolution;
    UINT32 WarmBlack = 0xFF1A1208;

    for (UINT32 y = 0; y < H; y++) {
        for (UINT32 x = 0; x < W; x++) {
            Fb[y * Stride + x] = WarmBlack;
        }
    }

    RenderWordmark(Fb, Stride, W, H);
    return EFI_SUCCESS;
}

/**
 * @brief Locates the Linux kernel image using EFI_SIMPLE_FILE_SYSTEM_PROTOCOL.
 */
static EFI_STATUS LoadKernelFromFilesystem(EFI_HANDLE ImageHandle, EFI_HANDLE *KernelHandle) {
    EFI_STATUS Status;
    UINTN Count = 0;
    EFI_HANDLE *Handles = NULL;

    Status = uefi_call_wrapper(BS->LocateHandleBuffer, 5,
        ByProtocol, &FileSystemProtocol, NULL, &Count, &Handles);

    if (EFI_ERROR(Status) || Count == 0) {
        return EFI_NOT_FOUND;
    }

    for (UINTN i = 0; i < Count; i++) {
        EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *Fs;
        Status = uefi_call_wrapper(BS->HandleProtocol, 3,
            Handles[i], &FileSystemProtocol, (VOID**)&Fs);

        if (EFI_ERROR(Status)) continue;

        EFI_FILE_PROTOCOL *Root = NULL;
        Status = uefi_call_wrapper(Fs->OpenVolume, 2, Fs, &Root);
        if (EFI_ERROR(Status) || !Root) continue;

        for (UINTN k = 0; gKernelPaths[k] != NULL; k++) {
            EFI_FILE_PROTOCOL *KernelFile = NULL;
            Status = uefi_call_wrapper(Root->Open, 5,
                Root, &KernelFile, (CHAR16*)gKernelPaths[k], EFI_FILE_MODE_READ, 0);

            if (!EFI_ERROR(Status) && KernelFile != NULL) {
                uefi_call_wrapper(KernelFile->Close, 1, KernelFile);
                uefi_call_wrapper(Root->Close, 1, Root);

                EFI_DEVICE_PATH_PROTOCOL *DevPath;
                uefi_call_wrapper(BS->HandleProtocol, 3,
                    Handles[i], &DevicePathProtocol, (VOID**)&DevPath);

                Status = uefi_call_wrapper(BS->LoadImage, 6,
                    FALSE, ImageHandle, DevPath, NULL, 0, KernelHandle);

                uefi_call_wrapper(BS->FreePool, 1, Handles);
                return Status;
            }
        }

        uefi_call_wrapper(Root->Close, 1, Root);
    }

    if (Handles) {
        uefi_call_wrapper(BS->FreePool, 1, Handles);
    }

    return EFI_NOT_FOUND;
}

/**
 * @brief Canonical UEFI Application Entry Point.
 */
EFI_STATUS EFIAPI efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable) {
    InitializeLib(ImageHandle, SystemTable);

    ANIMUS_GPU_HANDOFF Handoff = {0};
    DetectGpu(&Handoff);
    SetupGopAndRender(&Handoff);

    // Save EFI Runtime Variable
    uefi_call_wrapper(RT->SetVariable, 5,
        (CHAR16*)ANIMUS_HANDOFF_VAR_NAME,
        &gHandoffGuid,
        EFI_VARIABLE_BOOTSERVICE_ACCESS | EFI_VARIABLE_RUNTIME_ACCESS | EFI_VARIABLE_NON_VOLATILE,
        sizeof(Handoff),
        &Handoff);

    EFI_HANDLE KernelHandle = NULL;
    EFI_STATUS Status = LoadKernelFromFilesystem(ImageHandle, &KernelHandle);

    if (EFI_ERROR(Status)) {
        Print(L"AnimusBoot: Kernel not found on EFI filesystem (%r)\n", Status);
        return Status;
    }

    // Set Kernel Command Line Options
    EFI_LOADED_IMAGE_PROTOCOL *LoadedImage;
    Status = uefi_call_wrapper(BS->HandleProtocol, 3,
        KernelHandle, &LoadedImageProtocol, (VOID**)&LoadedImage);

    if (!EFI_ERROR(Status) && LoadedImage) {
        LoadedImage->LoadOptions = (VOID*)gDefaultCmdline;
        LoadedImage->LoadOptionsSize = (UINT32)((StrLen(gDefaultCmdline) + 1) * sizeof(CHAR16));
    }

    UINTN ExitDataSize = 0;
    CHAR16 *ExitData = NULL;
    return uefi_call_wrapper(BS->StartImage, 3, KernelHandle, &ExitDataSize, &ExitData);
}
