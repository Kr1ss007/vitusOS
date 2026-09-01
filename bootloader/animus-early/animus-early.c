// bootloader/animus-early/animus-early.c
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
#include "../uefi/include/AnimusHandoff.h"

// ── EFI variable ─────────────────────────────────────────────────
// /sys/firmware/efi/efivars/<n>-<GUID>
// Format: 4-byte EFI attributes + data
#define EFIVARS "/sys/firmware/efi/efivars/"

static bool read_handoff(ANIMUS_GPU_HANDOFF *out) {
    char p[256];
    snprintf(p, sizeof(p), EFIVARS "AnimusGpuHandoff-" ANIMUS_HANDOFF_GUID_STR);
    int fd = open(p, O_RDONLY);
    if (fd < 0) return false;
    uint32_t attrs;
    if (read(fd, &attrs, 4) != 4) { close(fd); return false; }
    ssize_t n = read(fd, out, sizeof(*out));
    close(fd);
    return n == (ssize_t)sizeof(*out);
}

// ── simpledrm ────────────────────────────────────────────────────
static int open_simpledrm(void) {
    for (int i = 0; i < 8; i++) {
        char p[32]; snprintf(p, sizeof(p), "/dev/dri/card%d", i);
        int fd = open(p, O_RDWR | O_CLOEXEC);
        if (fd < 0) continue;
        drmVersionPtr v = drmGetVersion(fd);
        if (!v) { close(fd); continue; }
        bool ok = strcmp(v->name, "simple") == 0;
        drmFreeVersion(v);
        if (ok) return fd;
        close(fd);
    }
    return -1;
}

// ── DRM dumb buffer ───────────────────────────────────────────────
typedef struct {
    int fd; uint32_t handle, pitch, fb_id; uint64_t size;
    uint32_t *map; uint32_t w, h;
} DumbBuf;

static bool make_dumb(int fd, DumbBuf *db, uint32_t w, uint32_t h) {
    struct drm_mode_create_dumb c = {.width = w, .height = h, .bpp = 32};
    if (ioctl(fd, DRM_IOCTL_MODE_CREATE_DUMB, &c)) return false;
    db->fd = fd; db->handle = c.handle; db->pitch = c.pitch;
    db->size = c.size; db->w = w; db->h = h;
    if (drmModeAddFB(fd, w, h, 24, 32, c.pitch, c.handle, &db->fb_id)) return false;
    struct drm_mode_map_dumb m = {.handle = c.handle};
    if (ioctl(fd, DRM_IOCTL_MODE_MAP_DUMB, &m)) return false;
    db->map = mmap(NULL, c.size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, m.offset);
    return db->map != MAP_FAILED;
}

// ── Boot splash ───────────────────────────────────────────────────
// #E85D00 = 0xFFE85D00 (ARGB). Matches AnimusBoot exactly.
// pitch/4 = pixels-per-row (pitch includes alignment padding).
static void render_splash(const DumbBuf *db) {
    uint32_t stride = db->pitch / 4;
    for (uint32_t y = 0; y < db->h; y++)
        for (uint32_t x = 0; x < db->w; x++)
            db->map[y * stride + x] = 0xFFE85D00u;
    uint32_t wx = (db->w - 280) / 2, wy = (db->h - 48) / 2;
    for (uint32_t y = wy; y < wy + 48; y++)
        for (uint32_t x = wx; x < wx + 280; x++)
            db->map[y * stride + x] = 0xFFFFFFFFu;
}

// ── Boot chime (child process) ────────────────────────────────────
typedef struct {
    struct pw_main_loop *loop; struct pw_stream *stream;
    const uint8_t *pcm; size_t sz, pos;
} ChimeS;

static void chime_proc(void *ud) {
    ChimeS *s = ud;
    struct pw_buffer *b = pw_stream_dequeue_buffer(s->stream);
    if (!b) return;
    uint8_t *dst = b->buffer->datas[0].data;
    uint32_t cap = b->buffer->datas[0].maxsize;
    size_t rem = s->sz - s->pos;
    if (!rem) {
        b->buffer->datas[0].chunk->size = 0;
        pw_stream_queue_buffer(s->stream, b);
        pw_main_loop_quit(s->loop); return;
    }
    uint32_t cp = (uint32_t)(rem < cap ? rem : cap);
    memcpy(dst, s->pcm + s->pos, cp); s->pos += cp;
    b->buffer->datas[0].chunk->size = cp;
    pw_stream_queue_buffer(s->stream, b);
}
static const struct pw_stream_events CHIME_EVT = {PW_VERSION_STREAM_EVENTS, .process = chime_proc};

static void play_chime_child(void) {
    int fd = open("/etc/vitusos/sounds/boot_chime.wav", O_RDONLY);
    if (fd < 0) return;
    off_t sz = lseek(fd, 0, SEEK_END); lseek(fd, 0, SEEK_SET);
    if (sz <= 44) { close(fd); return; }
    uint8_t *wav = malloc((size_t)sz);
    if (read(fd, wav, (size_t)sz) != sz) { free(wav); close(fd); return; }
    close(fd);

    pw_init(NULL, NULL);
    ChimeS s = {.pcm = wav + 44, .sz = (size_t)(sz - 44), .pos = 0};
    s.loop = pw_main_loop_new(NULL);
    struct pw_properties *p = pw_properties_new(
        PW_KEY_MEDIA_TYPE, "Audio", PW_KEY_MEDIA_CATEGORY, "Playback",
        PW_KEY_MEDIA_ROLE, "Music", NULL);
    // pw_stream_new: verified 5-arg signature (PipeWire 1.0.5)
    s.stream = pw_stream_new_simple(pw_main_loop_get_loop(s.loop),
        "animus-chime", p, &CHIME_EVT, &s);
    uint8_t buf[1024];
    struct spa_pod_builder b = SPA_POD_BUILDER_INIT(buf, sizeof(buf));
    const struct spa_pod *params[1];
    params[0] = spa_format_audio_raw_build(&b, SPA_PARAM_EnumFormat,
        &SPA_AUDIO_INFO_RAW_INIT(.format = SPA_AUDIO_FORMAT_S16,
                                  .rate = 44100, .channels = 2));
    pw_stream_connect(s.stream, PW_DIRECTION_OUTPUT, PW_ID_ANY,
        PW_STREAM_FLAG_AUTOCONNECT | PW_STREAM_FLAG_MAP_BUFFERS, params, 1);
    pw_main_loop_run(s.loop);
    pw_stream_destroy(s.stream); pw_main_loop_destroy(s.loop);
    pw_deinit(); free(wav);
}

// ── GPU driver load ───────────────────────────────────────────────
// NVIDIA order NON-NEGOTIABLE: nvidia → nvidia_modeset → nvidia_uvm → nvidia_drm
static bool do_modprobe(const char *name) {
    char cmd[128]; snprintf(cmd, sizeof(cmd), "modprobe %s 2>/dev/null", name);
    return system(cmd) == 0;
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
    ANIMUS_GPU_HANDOFF h = {.vendor = GPU_VENDOR_INTEL_LEGACY};
    read_handoff(&h);

    int drm = open_simpledrm();
    if (drm < 0) return 1;

    drmModeRes *res = drmModeGetResources(drm);
    if (!res) return 1;
    drmModeConnector *conn = NULL;
    for (int i = 0; i < res->count_connectors && !conn; i++) {
        drmModeConnector *c = drmModeGetConnector(drm, res->connectors[i]);
        if (c && c->connection == DRM_MODE_CONNECTED && c->count_modes > 0) conn = c;
        else if (c) drmModeFreeConnector(c);
    }
    if (!conn) return 1;

    DumbBuf db = {0};
    if (!make_dumb(drm, &db, conn->modes[0].hdisplay, conn->modes[0].vdisplay))
        return 1;

    drmModeEncoder *enc = drmModeGetEncoder(drm, conn->encoder_id);
    if (!enc) return 1;
    uint32_t crtc = enc->crtc_id; drmModeFreeEncoder(enc);
    drmModeSetCrtc(drm, crtc, db.fb_id, 0, 0, &conn->connector_id, 1, &conn->modes[0]);

    render_splash(&db);

    // Fork chime — main thread proceeds to driver load immediately
    pid_t pid = fork();
    if (pid == 0) { play_chime_child(); _exit(0); }

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
