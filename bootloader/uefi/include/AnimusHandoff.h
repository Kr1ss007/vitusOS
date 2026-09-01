#ifndef ANIMUS_HANDOFF_H
#define ANIMUS_HANDOFF_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ANIMUS_HANDOFF_GUID { 0xe4b8e798, 0xa5f4, 0x4b2c, { 0xb9, 0xab, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab } }
#define ANIMUS_HANDOFF_VAR_NAME L"AnimusGpuHandoff"

typedef enum {
    GPU_VENDOR_UNKNOWN     = 0,
    GPU_VENDOR_NVIDIA      = 1,
    GPU_VENDOR_AMD         = 2,
    GPU_VENDOR_INTEL_LEGACY = 3, // i915 driver
    GPU_VENDOR_INTEL_ARC   = 4   // xe driver
} ANIMUS_GPU_VENDOR;

typedef enum {
    GPU_TYPE_UNKNOWN    = 0,
    GPU_TYPE_DISCRETE   = 1,
    GPU_TYPE_INTEGRATED = 2
} ANIMUS_GPU_TYPE;

#pragma pack(push, 1)
typedef struct {
    uint32_t vendor;
    uint32_t gpu_type;
    uint16_t device_id;
    uint8_t  bus_number;
    uint8_t  padding;
    uint64_t framebuffer_base;
    uint32_t framebuffer_size;
    uint32_t horizontal_resolution;
    uint32_t vertical_resolution;
    uint32_t pixels_per_scanline;
    uint32_t pixel_format;
} ANIMUS_GPU_HANDOFF;
#pragma pack(pop)

#ifdef __cplusplus
}
#endif

#endif // ANIMUS_HANDOFF_H
