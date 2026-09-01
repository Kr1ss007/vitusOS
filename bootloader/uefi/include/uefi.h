/**
 * @file uefi.h
 * @brief Self-Contained Canonical UEFI 2.8+ Specification Type Definitions.
 *
 * Provides cross-platform, zero-dependency UEFI types, data structures, and protocol
 * definitions for bare-metal compiling and IDE static analysis.
 */

#ifndef VITUSOS_UEFI_H
#define VITUSOS_UEFI_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef NULL
#define NULL ((void*)0)
#endif

#ifndef EFIAPI
#if defined(_MSC_VER) || defined(__x86_64__)
#define EFIAPI __attribute__((ms_abi))
#else
#define EFIAPI
#endif
#endif

#define IN
#define OUT
#define OPTIONAL
#define CONST const

typedef uint8_t   UINT8;
typedef uint16_t  UINT16;
typedef uint32_t  UINT32;
typedef uint64_t  UINT64;
typedef int8_t    INT8;
typedef int16_t   INT16;
typedef int32_t   INT32;
typedef int64_t   INT64;
typedef size_t    UINTN;
typedef intptr_t  INTN;
typedef uint8_t   BOOLEAN;
typedef void      VOID;
typedef uint16_t  CHAR16;
typedef char      CHAR8;

#define TRUE  1
#define FALSE 0

typedef UINTN EFI_STATUS;
typedef VOID* EFI_HANDLE;
typedef VOID* EFI_EVENT;

#define EFI_SUCCESS               0x0000000000000000ULL
#define EFI_LOAD_ERROR            0x8000000000000001ULL
#define EFI_INVALID_PARAMETER     0x8000000000000002ULL
#define EFI_UNSUPPORTED           0x8000000000000003ULL
#define EFI_BAD_BUFFER_SIZE       0x8000000000000004ULL
#define EFI_BUFFER_TOO_SMALL      0x8000000000000005ULL
#define EFI_NOT_READY             0x8000000000000006ULL
#define EFI_DEVICE_ERROR          0x8000000000000007ULL
#define EFI_WRITE_PROTECTED       0x8000000000000008ULL
#define EFI_OUT_OF_RESOURCES      0x8000000000000009ULL
#define EFI_NOT_FOUND             0x800000000000000EULL

#define EFI_ERROR(status) (((INTN)(status)) < 0)

typedef struct {
    UINT32 Data1;
    UINT16 Data2;
    UINT16 Data3;
    UINT8  Data4[8];
} EFI_GUID;

#define EFI_VARIABLE_NON_VOLATILE                          0x00000001
#define EFI_VARIABLE_BOOTSERVICE_ACCESS                    0x00000002
#define EFI_VARIABLE_RUNTIME_ACCESS                        0x00000004
#define EFI_VARIABLE_HARDWARE_ERROR_RECORD                 0x00000008
#define EFI_VARIABLE_AUTHENTICATED_WRITE_ACCESS            0x00000010
#define EFI_VARIABLE_TIME_BASED_AUTHENTICATED_WRITE_ACCESS 0x00000020
#define EFI_VARIABLE_APPEND_WRITE                          0x00000040

typedef struct _EFI_TABLE_HEADER {
    UINT64 Signature;
    UINT32 Revision;
    UINT32 HeaderSize;
    UINT32 CRC32;
    UINT32 Reserved;
} EFI_TABLE_HEADER;

/* Forward Declarations */
typedef struct _EFI_SYSTEM_TABLE EFI_SYSTEM_TABLE;
typedef struct _EFI_BOOT_SERVICES EFI_BOOT_SERVICES;
typedef struct _EFI_RUNTIME_SERVICES EFI_RUNTIME_SERVICES;

/* Protocol GUIDs */
#define EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID \
    { 0x9042a9de, 0x23dc, 0x4a38, { 0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a } }

#define EFI_PCI_IO_PROTOCOL_GUID \
    { 0x4cf5b200, 0x68b8, 0x4106, { 0x88, 0x86, 0x45, 0x67, 0x03, 0x23, 0x44, 0x13 } }

#define EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID \
    { 0x0964e5b22, 0x6459, 0x11d2, { 0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b } }

#define EFI_LOADED_IMAGE_PROTOCOL_GUID \
    { 0x5B1B31A1, 0x9562, 0x11d2, { 0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B } }

#define EFI_DEVICE_PATH_PROTOCOL_GUID \
    { 0x09576e91, 0x6d3f, 0x11d2, { 0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b } }

/* Device Path Protocol */
typedef struct _EFI_DEVICE_PATH_PROTOCOL {
    UINT8 Type;
    UINT8 SubType;
    UINT8 Length[2];
} EFI_DEVICE_PATH_PROTOCOL;

/* Loaded Image Protocol */
typedef struct _EFI_LOADED_IMAGE_PROTOCOL {
    UINT32 Revision;
    EFI_HANDLE ParentHandle;
    EFI_SYSTEM_TABLE *SystemTable;
    EFI_HANDLE DeviceHandle;
    EFI_DEVICE_PATH_PROTOCOL *FilePath;
    VOID *Reserved;
    UINT32 ImageCodeType;
    UINT32 ImageDataType;
    EFI_STATUS (EFIAPI *Unload)(EFI_HANDLE ImageHandle);
    VOID *ImageBase;
    UINT64 ImageSize;
    UINT32 ImageCodeType2;
    UINT32 ImageDataType2;
    VOID *LoadOptions;
    UINT32 LoadOptionsSize;
} EFI_LOADED_IMAGE_PROTOCOL;

/* Graphics Output Protocol */
typedef enum {
    PixelRedGreenBlueReserved8BitPerColor,
    PixelBlueGreenRedReserved8BitPerColor,
    PixelBitMask,
    PixelBltOnly,
    PixelFormatMax
} EFI_GRAPHICS_PIXEL_FORMAT;

typedef struct {
    UINT32 Version;
    UINT32 HorizontalResolution;
    UINT32 VerticalResolution;
    EFI_GRAPHICS_PIXEL_FORMAT PixelFormat;
    UINT32 RedMask;
    UINT32 GreenMask;
    UINT32 BlueMask;
    UINT32 ReservedMask;
    UINT32 PixelsPerScanLine;
} EFI_GRAPHICS_OUTPUT_MODE_INFORMATION;

typedef struct {
    UINT32 MaxMode;
    UINT32 Mode;
    EFI_GRAPHICS_OUTPUT_MODE_INFORMATION *Info;
    UINTN SizeOfInfo;
    UINT64 FrameBufferBase;
    UINTN FrameBufferSize;
} EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE;

typedef struct _EFI_GRAPHICS_OUTPUT_PROTOCOL {
    EFI_STATUS (EFIAPI *QueryMode)(
        struct _EFI_GRAPHICS_OUTPUT_PROTOCOL *This,
        UINT32 ModeNumber,
        UINTN *SizeOfInfo,
        EFI_GRAPHICS_OUTPUT_MODE_INFORMATION **Info
    );
    EFI_STATUS (EFIAPI *SetMode)(
        struct _EFI_GRAPHICS_OUTPUT_PROTOCOL *This,
        UINT32 ModeNumber
    );
    VOID *Blt;
    EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE *Mode;
} EFI_GRAPHICS_OUTPUT_PROTOCOL;

/* File Protocols */
#define EFI_FILE_MODE_READ   0x0000000000000001ULL
#define EFI_FILE_MODE_WRITE  0x0000000000000002ULL
#define EFI_FILE_MODE_CREATE 0x8000000000000000ULL

typedef struct _EFI_FILE_PROTOCOL {
    UINT64 Revision;
    EFI_STATUS (EFIAPI *Open)(
        struct _EFI_FILE_PROTOCOL *This,
        struct _EFI_FILE_PROTOCOL **NewHandle,
        CHAR16 *FileName,
        UINT64 OpenMode,
        UINT64 Attributes
    );
    EFI_STATUS (EFIAPI *Close)(struct _EFI_FILE_PROTOCOL *This);
    EFI_STATUS (EFIAPI *Delete)(struct _EFI_FILE_PROTOCOL *This);
    EFI_STATUS (EFIAPI *Read)(struct _EFI_FILE_PROTOCOL *This, UINTN *BufferSize, VOID *Buffer);
    EFI_STATUS (EFIAPI *Write)(struct _EFI_FILE_PROTOCOL *This, UINTN *BufferSize, VOID *Buffer);
    VOID *GetPosition;
    VOID *SetPosition;
    VOID *GetInfo;
    VOID *SetInfo;
    VOID *Flush;
} EFI_FILE_PROTOCOL;

typedef struct _EFI_SIMPLE_FILE_SYSTEM_PROTOCOL {
    UINT64 Revision;
    EFI_STATUS (EFIAPI *OpenVolume)(
        struct _EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *This,
        EFI_FILE_PROTOCOL **Root
    );
} EFI_SIMPLE_FILE_SYSTEM_PROTOCOL;

/* PCI I/O Protocol */
typedef enum {
    EfiPciIoWidthUint8,
    EfiPciIoWidthUint16,
    EfiPciIoWidthUint32,
    EfiPciIoWidthUint64,
    EfiPciIoWidthMaximum
} EFI_PCI_IO_PROTOCOL_WIDTH;

typedef struct _EFI_PCI_IO_PROTOCOL EFI_PCI_IO_PROTOCOL;

typedef struct {
    EFI_STATUS (EFIAPI *Read)(
        EFI_PCI_IO_PROTOCOL *This,
        EFI_PCI_IO_PROTOCOL_WIDTH Width,
        UINT32 Offset,
        UINTN Count,
        VOID *Buffer
    );
    EFI_STATUS (EFIAPI *Write)(
        EFI_PCI_IO_PROTOCOL *This,
        EFI_PCI_IO_PROTOCOL_WIDTH Width,
        UINT32 Offset,
        UINTN Count,
        VOID *Buffer
    );
} EFI_PCI_IO_PROTOCOL_ACCESS;

struct _EFI_PCI_IO_PROTOCOL {
    VOID *PollMem;
    VOID *PollIo;
    VOID *Mem;
    VOID *Io;
    EFI_PCI_IO_PROTOCOL_ACCESS Pci;
    VOID *CopyMem;
    VOID *Map;
    VOID *Unmap;
    VOID *AllocateBuffer;
    VOID *FreeBuffer;
    VOID *Flush;
    EFI_STATUS (EFIAPI *GetLocation)(
        EFI_PCI_IO_PROTOCOL *This,
        UINTN *SegmentNumber,
        UINTN *BusNumber,
        UINTN *DeviceNumber,
        UINTN *FunctionNumber
    );
};

typedef enum {
    AllHandles,
    ByRegisterNotify,
    ByProtocol
} EFI_LOCATE_SEARCH_TYPE;

/* Boot & Runtime Services */
struct _EFI_BOOT_SERVICES {
    EFI_TABLE_HEADER Hdr;
    VOID *RaiseTPL;
    VOID *RestoreTPL;
    VOID *AllocatePages;
    VOID *FreePages;
    VOID *GetMemoryMap;
    EFI_STATUS (EFIAPI *AllocatePool)(UINT32 PoolType, UINTN Size, VOID **Buffer);
    EFI_STATUS (EFIAPI *FreePool)(VOID *Buffer);
    VOID *CreateEvent;
    VOID *SetTimer;
    VOID *WaitForEvent;
    VOID *SignalEvent;
    VOID *CloseEvent;
    VOID *CheckEvent;
    VOID *InstallProtocolInterface;
    VOID *ReinstallProtocolInterface;
    VOID *UninstallProtocolInterface;
    EFI_STATUS (EFIAPI *HandleProtocol)(EFI_HANDLE Handle, EFI_GUID *Protocol, VOID **Interface);
    VOID *Reserved;
    VOID *RegisterProtocolNotify;
    EFI_STATUS (EFIAPI *LocateHandle)(EFI_LOCATE_SEARCH_TYPE SearchType, EFI_GUID *Protocol, VOID *SearchKey, UINTN *BufferSize, EFI_HANDLE *Buffer);
    VOID *LocateDevicePath;
    VOID *InstallConfigurationTable;
    EFI_STATUS (EFIAPI *LoadImage)(BOOLEAN BootPolicy, EFI_HANDLE ParentImageHandle, EFI_DEVICE_PATH_PROTOCOL *DevicePath, VOID *SourceBuffer, UINTN SourceSize, EFI_HANDLE *ImageHandle);
    EFI_STATUS (EFIAPI *StartImage)(EFI_HANDLE ImageHandle, UINTN *ExitDataSize, CHAR16 **ExitData);
    VOID *Exit;
    VOID *UnloadImage;
    VOID *ExitBootServices;
    VOID *GetNextMonotonicCount;
    VOID *Stall;
    VOID *SetWatchdogTimer;
    VOID *ConnectController;
    VOID *DisconnectController;
    VOID *OpenProtocol;
    VOID *CloseProtocol;
    VOID *OpenProtocolInformation;
    VOID *ProtocolsPerHandle;
    EFI_STATUS (EFIAPI *LocateHandleBuffer)(EFI_LOCATE_SEARCH_TYPE SearchType, EFI_GUID *Protocol, VOID *SearchKey, UINTN *NoHandles, EFI_HANDLE **Buffer);
    EFI_STATUS (EFIAPI *LocateProtocol)(EFI_GUID *Protocol, VOID *Registration, VOID **Interface);
};

struct _EFI_RUNTIME_SERVICES {
    EFI_TABLE_HEADER Hdr;
    VOID *GetTime;
    VOID *SetTime;
    VOID *GetWakeupTime;
    VOID *SetWakeupTime;
    VOID *SetVirtualAddressMap;
    VOID *ConvertPointer;
    EFI_STATUS (EFIAPI *GetVariable)(CHAR16 *VariableName, EFI_GUID *VendorGuid, UINT32 *Attributes, UINTN *DataSize, VOID *Data);
    VOID *GetNextVariableName;
    EFI_STATUS (EFIAPI *SetVariable)(CHAR16 *VariableName, EFI_GUID *VendorGuid, UINT32 Attributes, UINTN DataSize, VOID *Data);
    VOID *GetNextHighMonotonicCount;
    VOID *ResetSystem;
};

/* System Table */
struct _EFI_SYSTEM_TABLE {
    EFI_TABLE_HEADER Hdr;
    CHAR16 *FirmwareVendor;
    UINT32 FirmwareRevision;
    EFI_HANDLE ConsoleInHandle;
    VOID *ConIn;
    EFI_HANDLE ConsoleOutHandle;
    VOID *ConOut;
    EFI_HANDLE StandardErrorHandle;
    VOID *StdErr;
    EFI_RUNTIME_SERVICES *RuntimeServices;
    EFI_BOOT_SERVICES *BootServices;
    UINTN NumberOfTableEntries;
    VOID *ConfigurationTable;
};

/* Helper Functions */
static inline UINTN StrLen(const CHAR16 *s) {
    UINTN len = 0;
    while (s && s[len] != 0) { len++; }
    return len;
}

#ifdef __cplusplus
}
#endif

#endif /* VITUSOS_UEFI_H */
