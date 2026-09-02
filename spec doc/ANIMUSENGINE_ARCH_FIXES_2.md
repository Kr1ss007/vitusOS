# AnimusEngine — Architecture Gap Fixes, Volume 2
## vitusOS ARES · Upstream Color + Upstream One
## Dari pembacaan penuh semua 20,130 baris

**Status:** Tambahan terhadap ANIMUSENGINE_ARCH_FIXES.md (Volume 1)
**Ditemukan dari:** Pembacaan lengkap semua seksi yang sebelumnya ter-truncate

---

## INDEX OF FIXES

| ID | Bug | Severity | Lokasi |
|----|-----|----------|--------|
| FIX2-01 | `WindowRestored` duplikat di `OSFEvent` enum | **COMPILE ERROR** | OSFEvent.h |
| FIX2-02 | `pw_stream_new_simple` vs `pw_stream_new` — inkonsistensi Rule 7 | **RULE AMBIGUITY** | animus-early.c |
| FIX2-03 | `ShaderCache::saveToDisk()` referensi member yang tidak ada | **COMPILE ERROR** | CacheKeepr.h/.cpp |
| FIX2-04 | `RenderPipeline` iterasi `OSFWindow*` raw, bukan `RegHandle` | **CRASH** | RenderPipeline.cpp |
| FIX2-05 | `VulkanContext::commitFrame()` tidak menyimpan `wlr_output*` | **CRASH** | VulkanContext.h/.cpp |
| FIX2-06 | Two-process model tidak direfleksikan di OSFDesktop init order | **ARCH** | OSFDesktop.cpp |
| FIX2-07 | `CrashSite::onClientCrash` kirim `NotifData` bukan `NotificationPayload` | **CRASH** | CrashSite.cpp |

---

## FIX2-01 — `WindowRestored` Duplikat di OSFEvent Enum

### Masalah

`OSFEvent.h` mendefinisikan `WindowRestored` dua kali:

```cpp
// Baris 1308 — Window lifecycle section:
WindowRestored,      // window un-maximized

// Baris 1376 — Part 41 Minimize section:
WindowRestored,      // LOCAL: data = uint64_t windowHandle
```

C++ tidak akan compile. Dua enum member dengan nama sama dalam satu enum
adalah ill-formed. Error: `error: 'WindowRestored' conflicts with a previous
declaration`.

### Fix

Event kedua (dari Part 41) adalah untuk restore dari minimize state,
bukan dari maximize. Namanya harus dibedakan.

```cpp
// animus/core/OSFEvent.h — complete corrected enum (relevant sections)

enum class OSFEvent : uint32_t {
    Tick = 0,

    // ── Window lifecycle ──────────────────────────────────────────
    WindowOpened,
    WindowClosed,
    WindowFocused,
    WindowBlurred,
    WindowMoved,
    WindowResized,
    WindowMaximized,
    WindowUnmaximized,      // ← was WindowRestored (baris 1308)
                            //   renamed: unmaximize ≠ un-minimize

    // ── Input ─────────────────────────────────────────────────────
    KeyDown, KeyUp,
    MouseMoved, MouseButtonDown, MouseButtonUp,
    ScrollDelta,
    SwipeBegin, SwipeUpdate, SwipeEnd,

    // ── Shell ─────────────────────────────────────────────────────
    DockBounce,
    PanelMenuActivated,
    CockpitViewToggle,
    LockScreenActivate,
    LockScreenUnlocked,
    NotificationPosted,
    NotificationDismissed,

    // ── Render ────────────────────────────────────────────────────
    WallpaperChanged,
    WallpaperTintChanged,
    OutputResized,
    DamageRegion,
    DamageWhole,

    // ── Filesystem ────────────────────────────────────────────────
    DirectoryChanged,
    DirectoryLoaded,
    ThumbnailReady,

    // ── Audio ─────────────────────────────────────────────────────
    SoundPlay, SoundStop, VolumeChanged,

    // ── Pathfinder ────────────────────────────────────────────────
    PathfinderResultsReady,
    PathfinderQueryChanged,
    PathfinderClosed,

    // ── Boot ──────────────────────────────────────────────────────
    BootCrossfadeComplete,

    // ── App lifecycle ─────────────────────────────────────────────
    AppLaunched, AppTerminated, AppMenuChanged,
    AppIndexReady,

    // ── Clipboard ─────────────────────────────────────────────────
    ClipboardChanged,

    // ── MotionWave ────────────────────────────────────────────────
    DesktopPrev, DesktopNext,
    ShowDesktop, ShowDesktopToggle,
    PinchIn, PinchOut,

    // ── Virtual Desktops ──────────────────────────────────────────
    DesktopSwitched, DesktopAdded,
    DesktopRemoving, DesktopRemoved, DesktopRenamed,

    // ── Power ─────────────────────────────────────────────────────
    SystemSleep, DisplaySleep, DisplayWake,
    BatteryLevelChanged, LidClosed,

    // ── Fullscreen ────────────────────────────────────────────────
    FullscreenEntered,   // data = uint64_t windowHandle
    FullscreenExited,    // data = uint64_t windowHandle

    // ── Minimize (Part 41) ────────────────────────────────────────
    WindowMinimized,     // data = uint64_t windowHandle
    WindowDeminimized,   // ← RENAMED dari WindowRestored (baris 1376)
                         //   "deminimize" = keluar dari minimize state
                         //   "unmaximize" sudah dicover oleh WindowUnmaximized

    // ── CrashManager ──────────────────────────────────────────────
    ResourcePressure,
    SubsystemHealthChanged,
    ClientCrashed,
    BlastRadius,
    InstallFailed,
    MemoryPressure,

    // ── EO-Bus ────────────────────────────────────────────────────
    DBusMenuChanged,
    StatusNotifierChanged,
    AccessibilityTreeChanged,
    ReducedMotionChanged,
    OpenURI,
    PortalFileChosen,
    PortalScreenCastStarted,

    // ── HEV ───────────────────────────────────────────────────────
    HEVUnlocked, HEVLocked, HEVSealed,
    HEVAccessDenied, HEVAuthorizationNeeded,
    ProximityUnlockReady, ProximityLockWarning,

    // ── CacheKeepr ────────────────────────────────────────────────
    CacheEvicted, CacheInvalidated,

    // ── Install lifecycle ─────────────────────────────────────────
    InstallProgress, InstallComplete,
    RemoveComplete, RemoveFailed,

    // ── System ────────────────────────────────────────────────────
    ShutdownRequested,
    ConfigReload,
    StateChanged,

    _Count
};
```

### Semua call site yang harus diupdate

```cpp
// Part 41: OSFWindow::restore() — ganti WindowRestored → WindowDeminimized
EventBus::shared().publish(OSFEvent::WindowDeminimized, m_handle);

// Part 41: OSFDesktop::initialize() subscription
EventBus::shared().subscribe(OSFEvent::WindowDeminimized, [...]);

// Part 40: WindowManager::onUnsetFullscreen() — sudah pakai FullscreenExited
// Tidak perlu diubah — event ini independen dari WindowUnmaximized.

// Anywhere WindowManager::onUnmaximize() fires — ganti ke WindowUnmaximized
EventBus::shared().publish(OSFEvent::WindowUnmaximized,
    static_cast<uint64_t>(win->handle()));
```

---

## FIX2-02 — Rule 7 Klarifikasi: `pw_stream_new_simple` vs `pw_stream_new`

### Masalah

**Absolute Rule 7** menyatakan:
```
pw_stream_new(core, name, props, &events, userdata) — 5-arg signature, verified.
```

Tapi `animus-early.c` baris 710 menggunakan:
```c
s.stream = pw_stream_new_simple(pw_main_loop_get_loop(s.loop),
    "animus-chime", p, &CHIME_EVT, &s);
```

Ini bukan kontradiksi — keduanya valid, tapi untuk konteks berbeda.
`pw_stream_new_simple` menerima `pw_loop*` langsung dan membuat stream yang
lebih ringan, sesuai untuk `animus-early` yang hanya butuh single-shot chime.
`pw_stream_new` (5-arg dengan `pw_core*`) digunakan oleh `SoundEngine` di
AnimusEngine yang membutuhkan shared `pw_core` untuk multi-stream management.

### Fix — Klarifikasi Rule 7

Tambahkan keterangan ke Absolute Rule 7:

```
7. pw_stream_new(core, name, props, &events, userdata) — 5-arg signature.
   Berlaku untuk SoundEngine.cpp (multi-stream, shared pw_core).
   PENGECUALIAN: animus-early.c menggunakan pw_stream_new_simple()
   karena ia membuat pw_main_loop sendiri dan tidak punya shared pw_core.
   pw_stream_new_simple adalah API yang benar untuk single-shot use case.
   Jangan pernah mencampur keduanya dalam file yang sama.
```

Tidak ada perubahan kode yang diperlukan. Rule 7 hanya perlu klarifikasi
agar implementer tidak bingung melihat dua API berbeda di dua file berbeda.

---

## FIX2-03 — `ShaderCache::saveToDisk()` Member Tidak Ada

### Masalah

`ShaderCache::saveToDisk()` di baris 10921:
```cpp
std::ofstream sp(m_storepathFile);
sp << m_storePaths_animusengine;  // ← member ini TIDAK ADA di ShaderCache
```

`m_storePaths` adalah `std::unordered_map` milik `CacheKeepr`, bukan
`ShaderCache`. `ShaderCache` tidak tahu store path mana yang sedang digunakan
karena ia tidak menyimpannya.

### Fix — Tambahkan `m_currentStorePath` ke `ShaderCache`

```cpp
// animus/cache/CacheKeepr.h — ShaderCache class, tambahkan private member

class ShaderCache {
public:
    bool initialize(VulkanContext *ctx,
                    const std::string &cachePath,
                    const std::string &currentStorePath);
    void destroy();
    bool saveToDisk();
    void invalidate();
    VkPipelineCache handle() const { return m_cache; }
    size_t byteSize() const { return m_blobSize; }

private:
    VkDevice        m_device           = VK_NULL_HANDLE;
    VkPipelineCache m_cache            = VK_NULL_HANDLE;
    std::string     m_cachePath;
    std::string     m_storepathFile;
    std::string     m_currentStorePath;  // ← TAMBAHKAN INI
    size_t          m_blobSize          = 0;

    bool loadFromDisk(const std::string &currentStorePath);
    bool createEmpty();
};
```

```cpp
// animus/cache/ShaderCache.cpp — initialize() menyimpan store path

bool ShaderCache::initialize(VulkanContext *ctx,
                               const std::string &cachePath,
                               const std::string &currentStorePath)
{
    m_device           = ctx->device;
    m_cachePath        = cachePath;
    m_storepathFile    = cachePath + ".storepath";
    m_currentStorePath = currentStorePath;  // ← simpan di sini

    if (!loadFromDisk(currentStorePath)) {
        return createEmpty();
    }
    return true;
}
```

```cpp
// animus/cache/ShaderCache.cpp — saveToDisk() yang benar

bool ShaderCache::saveToDisk() {
    if (m_cache == VK_NULL_HANDLE) return false;

    size_t dataSize = 0;
    VkResult r = vkGetPipelineCacheData(m_device, m_cache,
                                         &dataSize, nullptr);
    if (r != VK_SUCCESS || dataSize == 0) return false;

    std::vector<char> data(dataSize);
    r = vkGetPipelineCacheData(m_device, m_cache, &dataSize, data.data());
    if (r != VK_SUCCESS) return false;

    // Atomic write: tulis ke .tmp dulu, baru rename
    std::string tmp = m_cachePath + ".tmp";
    {
        std::ofstream f(tmp, std::ios::binary);
        if (!f.is_open()) return false;
        f.write(data.data(), dataSize);
    }
    ::rename(tmp.c_str(), m_cachePath.c_str());

    // Tulis store path — ini yang sebelumnya salah
    {
        std::ofstream sp(m_storepathFile);
        if (!sp.is_open()) return false;
        sp << m_currentStorePath;  // ← gunakan member yang disimpan saat init
    }

    m_blobSize = dataSize;
    return true;
}
```

```cpp
// animus/cache/ShaderCache.cpp — invalidate() juga update m_currentStorePath

void ShaderCache::invalidate() {
    if (m_cache != VK_NULL_HANDLE) {
        vkDestroyPipelineCache(m_device, m_cache, nullptr);
        m_cache = VK_NULL_HANDLE;
    }
    ::unlink(m_cachePath.c_str());
    ::unlink(m_storepathFile.c_str());
    m_blobSize = 0;
    // m_currentStorePath tetap valid — akan diupdate oleh CacheKeepr
    // setelah store path baru diketahui
    createEmpty();
}

// CacheKeepr::onStorePathChanged() harus update ShaderCache store path:
void CacheKeepr::onStorePathChanged(const std::string &component,
                                     const std::string &newStorePath)
{
    // ... existing logic ...
    if (component == "animusengine") {
        m_shaders.setCurrentStorePath(newStorePath);  // ← tambahkan setter
        m_shaders.invalidate();
    }
    // ...
}

// Tambahkan setter ke ShaderCache:
void ShaderCache::setCurrentStorePath(const std::string &path) {
    m_currentStorePath = path;
}
```

---

## FIX2-04 — RenderPipeline Iterasi Raw `OSFWindow*`, Bukan `RegHandle`

### Masalah

`RenderPipeline::renderFrame()` (dari Volume 1 fix dan dokumen asli) iterasi
`m_windows` sebagai vector of raw pointers atau shared_ptr:

```cpp
for (auto& win : m_windows)
    if (win->isVisible())
        m_shadow->drawWindowShadow(cmd, ...);
```

`RegistryManager` (Part 27) ada justru untuk mencegah dereference pointer
yang sudah di-destroy. Setelah `WindowManager::removeSurface()` memanggil
`RegistryManager::windows().unregisterWindow(handle)`, pointer tersebut
tidak valid lagi. Jika `RenderPipeline` masih memegang pointer lama,
itu adalah use-after-free — SIGSEGV.

Dokumen sendiri menyebut ini di Part 27.1:
> "OSFWindow::renderTrafficLights() called after WindowManager already destroyed
> the window following a client disconnect. RenderPipeline held a stale
> OSFWindow* reference. SIGSEGV at 0xFFFFFFFFFFFFFFFF."

### Fix — RenderPipeline menggunakan RegHandle via RegistryManager

```cpp
// animus/render/RenderPipeline.h — ganti m_windows

#include "registry/RegistryManager.h"

class RenderPipeline {
    // HAPUS: std::vector<std::shared_ptr<OSFWindow>> m_windows;
    // HAPUS: std::vector<std::shared_ptr<OSFWindow>> m_overlays;

    // Tidak perlu menyimpan window list sama sekali.
    // RegistryManager adalah sumber kebenaran tunggal untuk semua window hidup.
    // RenderPipeline query RegistryManager setiap frame.

    // TETAP ADA (non-window render state):
    VulkanContext         *m_ctx      = nullptr;
    struct wlr_output     *m_output   = nullptr;
    MaterialRenderer      *m_material = nullptr;
    ShadowRenderer        *m_shadow   = nullptr;
    Panel                 *m_panel    = nullptr;
    Dock                  *m_dock     = nullptr;
    BootCrossfade         *m_crossfade= nullptr;
    VkImageView            m_wallpaperView = VK_NULL_HANDLE;
    // Overlays: notifikasi, tooltip, context menu — juga via RegistryManager
};
```

```cpp
// animus/render/RenderPipeline.cpp — renderFrame() yang benar

void RenderPipeline::renderFrame(float dt) {
    // ... damage check dan acquireFrame() tidak berubah ...

    VkFramebuffer fb = m_ctx->acquireFrame(m_output);
    if (fb == VK_NULL_HANDLE) return;

    int f = m_ctx->frame;
    VkCommandBuffer cmd = m_ctx->cmdBuf[f];
    // ... begin command buffer, barriers, begin render pass ...

    // Layer 0: Wallpaper
    if (m_wallpaperView != VK_NULL_HANDLE)
        m_material->drawTextureQuad(cmd, 0, 0,
            (float)m_ctx->width, (float)m_ctx->height,
            m_wallpaperView, 1.0f, 0.0f);

    // Layer 1+2+3: Windows — via RegistryManager, bukan raw pointer
    // forEach() memegang mutex selama iterasi — tidak ada window yang
    // bisa di-unregister di tengah iterasi (main thread single-threaded).
    RegistryManager::shared().windows().forEach(
        [&](RegHandle handle, OSFWindow *win) {
            // win dijamin valid di sini — RegistryManager menjamin ini.
            // Tidak perlu null-check tambahan.
            if (!win->isVisible()) return;

            // Layer 1: shadow
            m_shadow->drawWindowShadow(cmd,
                win->shadowX(), win->shadowY(),
                win->width(),   win->height(),
                win->cornerRadius());

            // Layer 2: glass background
            m_material->drawGlassSurface(cmd,
                win->x(), win->y(), win->width(), win->height(),
                win->cornerRadius(), win->altitude());

            // Layer 3: window content (wlr_surface texture)
            m_material->drawWindowSurface(cmd, win);
        });

    // Layer 4: Shell
    if (m_panel) m_panel->render(cmd, dt);
    if (m_dock)  m_dock->render(cmd, dt);

    // Layer 5: Boot crossfade
    if (m_crossfade && !m_crossfade->isComplete())
        m_crossfade->render(cmd,
            (float)m_ctx->width, (float)m_ctx->height);

    // Layer 6: Floating overlays (notifikasi) — via NotificationRegistry
    RegistryManager::shared().notifications().forEach(
        [&](RegHandle handle, OSFNotification *notif) {
            if (notif->isVisible())
                notif->render(cmd,
                    (float)m_ctx->width,
                    (float)m_ctx->height, dt);
        });

    // ... end render pass, barriers, submit, commitFrame() ...
}
```

### WindowManager harus update RegistryManager, bukan RenderPipeline

```cpp
// animus/core/WindowManager.cpp

void WindowManager::addSurface(struct wlr_surface *surface,
                                 const AnimusContext &ctx)
{
    // Buat OSFWindow
    auto win = std::make_unique<OSFWindow>(surface,
        ctx.x, ctx.y, ctx.w, ctx.h);

    // Daftarkan ke RegistryManager — ini yang dipakai semua komponen
    RegHandle handle = RegistryManager::shared()
                            .windows()
                            .registerWindow(win.get());
    win->setHandle(handle);

    // Daftarkan surface ke SurfaceRegistry
    RegHandle surfHandle = RegistryManager::shared()
                                .surfaces()
                                .registerSurface(surface);

    // WindowManager menyimpan unique_ptr untuk ownership
    // Komponen lain (RenderPipeline, Dock, InputRouter) pakai RegHandle
    m_windowsByHandle[handle] = std::move(win);

    // Suara + EventBus
    SoundEngine::shared().play(Sounds::AppLaunch, SoundVolumes::AppLaunch);
    EventBus::shared().publish(OSFEvent::WindowOpened,
        static_cast<uint64_t>(handle));
}

void WindowManager::removeSurface(struct wlr_surface *surface)
{
    // Cari handle dari SurfaceRegistry
    RegHandle surfHandle = RegistryManager::shared()
                                .surfaces()
                                .handleFor(surface);

    // Cari window handle dari surface handle
    // (WindowManager menyimpan mapping ini)
    RegHandle winHandle = m_surfaceToWindow[surfHandle];

    // Unregister DULU dari RegistryManager
    // Setelah ini, semua resolve(winHandle) return nullptr
    // RenderPipeline tidak akan menyentuh window ini lagi
    RegistryManager::shared().windows().unregisterWindow(winHandle);
    RegistryManager::shared().surfaces().unregisterSurface(surfHandle);

    // BARU hapus ownership — OSFWindow dihancurkan di sini
    m_windowsByHandle.erase(winHandle);
    m_surfaceToWindow.erase(surfHandle);

    SoundEngine::shared().play(Sounds::AppClose, SoundVolumes::AppClose);
    EventBus::shared().publish(OSFEvent::WindowClosed,
        static_cast<uint64_t>(winHandle));
}
```

---

## FIX2-05 — `VulkanContext::commitFrame()` Tidak Menyimpan `wlr_output*`

### Masalah

`VulkanContext::acquireFrame(struct wlr_output *output)` menerima `output`
sebagai parameter, menyimpan `m_currentBuffer`, tapi tidak menyimpan `output`
itu sendiri. `commitFrame()` dipanggil tanpa parameter — ia tidak punya
`wlr_output*` untuk memanggil `wlr_output_commit_state()`.

Dari Volume 1 fix, `commitFrame(struct wlr_output *output)` menerima output
sebagai parameter — tapi `RenderPipeline::renderFrame()` memanggil
`m_ctx->commitFrame(m_output)` dimana `m_output` belum ditunjukkan sebagai
member `RenderPipeline`. Kedua sisi perlu diperjelas.

### Fix — Perjelas ownership `wlr_output*`

```cpp
// animus/render/VulkanContext.h — tambahkan m_currentOutput

class VulkanContext {
public:
    // ... existing API ...

    VkFramebuffer acquireFrame(struct wlr_output *output);
    // commitFrame tidak butuh parameter — output disimpan saat acquireFrame
    bool commitFrame();

private:
    // ...
    struct wlr_buffer *m_currentBuffer = nullptr;
    struct wlr_output *m_currentOutput = nullptr;  // ← TAMBAHKAN
    int                m_currentSlot   = 0;
    // ...
};
```

```cpp
// animus/render/VulkanContext.cpp — acquireFrame menyimpan output

VkFramebuffer VulkanContext::acquireFrame(struct wlr_output *output) {
    m_currentOutput = output;  // ← simpan sebelum attach_render
    // ... rest of acquireFrame unchanged ...
}

bool VulkanContext::commitFrame() {
    if (!m_currentBuffer || !m_currentOutput) {
        wlr_log(WLR_ERROR,
            "AnimusEngine: commitFrame called with no buffer/output");
        return false;
    }

    vkWaitForFences(device, 1, &fence[m_currentSlot],
                    VK_TRUE, UINT64_MAX);
    vkResetFences(device, 1, &fence[m_currentSlot]);

    struct wlr_output_state state;
    wlr_output_state_init(&state);
    wlr_output_state_set_buffer(&state, m_currentBuffer);
    bool ok = wlr_output_commit_state(m_currentOutput, &state);
    wlr_output_state_finish(&state);

    if (!ok) wlr_log(WLR_ERROR,
        "AnimusEngine: wlr_output_commit_state failed");

    m_currentBuffer = nullptr;
    m_currentOutput = nullptr;
    frame = (m_currentSlot + 1) % FRAMES;
    return ok;
}
```

```cpp
// animus/render/RenderPipeline.h — m_output sebagai member

class RenderPipeline {
public:
    void setOutput(struct wlr_output *output) { m_output = output; }

private:
    struct wlr_output *m_output = nullptr;
    VulkanContext     *m_ctx    = nullptr;
    // ...
};

// RenderPipeline::renderFrame() — panggil commitFrame tanpa parameter
void RenderPipeline::renderFrame(float dt) {
    VkFramebuffer fb = m_ctx->acquireFrame(m_output);
    // ... render ...
    m_ctx->commitFrame();  // ← no parameter needed
}
```

```cpp
// animus/core/OSFDesktop.cpp — wire output ke RenderPipeline
// In OSFDesktop::initSubsystems():

// Setelah compositor init, output tersedia via g.primary_output
m_render->setOutput(animus_compositor_get_primary_output());

// Tambahkan ke animus_compositor.h:
struct wlr_output* animus_compositor_get_primary_output(void);

// Implementasi di animus_compositor.c:
struct wlr_output* animus_compositor_get_primary_output(void) {
    return g.primary_output;
}
```

---

## FIX2-06 — Two-Process Model: OSFDesktop Init Order

### Masalah

Part 28 mengungkap bahwa vitusOS adalah **two-process architecture**:
- `vitusos-session` — memiliki HEV, RegistryManager (session-side),
  StateManager, EventBus (session-side), EO-Bus, SeaDrop
- `vitusos-compositor` — memiliki AnimusEngine, CrashManager,
  CacheKeepr, RegistryManager (compositor-side), Shell, OSFBridge

Keduanya berkomunikasi via `OSFBridge` di `/run/vitusos/osf-ipc.sock`.

Dari `BRIDGED` event classification (Part 28.4): beberapa OSFEvent
harus di-bridge antar proses. Ini punya implikasi pada `OSFDesktop::run()`.

Volume 1 fix document menulis `OSFDesktop::run()` sebagai single-process.
Ini adalah simplifikasi yang salah.

### Fix — Init order yang benar untuk compositor process

```cpp
// animus/core/OSFDesktop.cpp — compositor process init order

int OSFDesktop::run() {
    // ══════════════════════════════════════════════════════
    // COMPOSITOR PROCESS — hanya komponen compositor di sini
    // Session process (vitusos-session) sudah jalan lebih dulu
    // via systemd: After=vitusos-session.service
    // ══════════════════════════════════════════════════════

    // ── Step 0: CrashManager — SELALU PERTAMA ─────────────────────
    CrashManager::shared().initialize();
    // Signal handlers terpasang. Crash pipe fd tersedia.

    // ── Step 0.5: RegistryManager ────────────────────────────────
    // Harus ada sebelum komponen apapun yang membuat registered objects.
    RegistryManager::shared().initialize();

    // ── Step 1: Compositor C11 core ──────────────────────────────
    if (animus_compositor_init() != 0) {
        wlr_log(WLR_ERROR, "Compositor init failed");
        return 1;
    }

    // ── Step 2: EventHandler — wlr_log bridge ────────────────────
    CrashManager::shared().eventHandler().initialize();

    // ── Step 3: AnimusEngine core subsystems ─────────────────────
    // VulkanContext, GlyphAtlas, TextRenderer, AnimationClock,
    // SpringSolver (static), EventBus (compositor-side),
    // StateManager (compositor-side), WallpaperTintSampler

    // Dapatkan output dimensions dari compositor
    struct wlr_output *output = animus_compositor_get_primary_output();
    // Output mungkin NULL di sini jika belum ada monitor terhubung.
    // Tunggu di event loop — on_new_output akan fire.
    // VulkanContext init ditunda sampai output tersedia.

    // AnimationClock dan SpringSolver bisa init tanpa output.
    AnimationClock::shared();  // initialize singleton
    AnimationEngine::shared().start();

    // EventBus compositor-side — tidak perlu explicit init
    // StateManager compositor-side
    StateManager::shared();

    // SoundEngine
    SoundEngine::shared().initialize();

    // ── Step 4: OSFBridge — koneksi ke session process ───────────
    // OSFBridge menghubungkan compositor EventBus ke session EventBus
    // via /run/vitusos/osf-ipc.sock
    // Session process sudah bind socket ini sebelum compositor start
    // (dijamin oleh systemd After=vitusos-session.service)
    OSFBridge::shared().connectToSession("/run/vitusos/osf-ipc.sock");

    // ── Step 5: CacheKeepr — setelah Vulkan ada ──────────────────
    // VulkanContext diinit saat on_new_output fires (lihat Step 7).
    // CacheKeepr.initialize() dipanggil dari on_new_output callback,
    // bukan di sini langsung.

    // ── Step 6: Background monitors ──────────────────────────────
    // PSI fd dari FirstResponder ke GlobalFeed
    CrashManager::shared().globalFeed().setPsiFd(
        CrashManager::shared().firstResponder().psiFd());
    CrashManager::shared().globalFeed().start();
    CrashManager::shared().handshakes().start();

    // ── Step 7: Wire crash pipe ke Wayland event loop ─────────────
    wl_event_loop_add_fd(
        animus_compositor_get_event_loop(),
        CrashManager::shared().firstResponder().pipeReadFd(),
        WL_EVENT_READABLE,
        onCrashPipe,
        nullptr);

    // ── Step 8: Register compositor callbacks ────────────────────
    animus_compositor_register_callbacks(
        cbPresent, cbNewSurface, cbSurfaceDestroy,
        cbKey, cbPointerMotion, cbPointerButton, cbPointerAxis,
        cbSwipeBegin, cbSwipeUpdate, cbSwipeEnd,
        this);

    // ── Step 9: Wayland event loop ────────────────────────────────
    // Shell (Panel, Dock, CockpitView, LockScreen) diinit dari
    // onNewOutput() ketika monitor pertama terhubung.
    animus_compositor_run();

    // ── Shutdown ──────────────────────────────────────────────────
    if (m_vulkan) {
        CacheKeepr::shared().shaders().saveToDisk();
        CacheKeepr::shared().apps().saveToDisk();
        CacheKeepr::shared().destroy();
    }
    CrashManager::shared().handshakes().stop();
    CrashManager::shared().globalFeed().stop();
    CrashManager::shared().destroy();
    RegistryManager::shared().destroy();
    return 0;
}

// ── onNewOutput — dipanggil saat monitor pertama terhubung ───────────
// Output event datang dari h_new_output di compositor C11 core.
void OSFDesktop::cbOutputAdded(struct wlr_output *output,
                                bool isPrimary, void *ud)
{
    auto *desk = static_cast<OSFDesktop*>(ud);

    if (isPrimary && !desk->m_vulkan) {
        // Inisialisasi VulkanContext dengan output dimensions
        desk->m_vulkan = std::make_unique<VulkanContext>();
        int drm_fd = wlr_backend_get_drm_fd(
            animus_compositor_get_backend());
        desk->m_vulkan->initialize(drm_fd,
            animus_compositor_get_vk_renderer());

        // RenderPipeline — butuh VulkanContext + output
        desk->m_render = std::make_unique<RenderPipeline>();
        desk->m_render->initialize(desk->m_vulkan.get());
        desk->m_render->setOutput(output);

        // CacheKeepr — butuh VulkanContext
        std::string cacheDir = userHomeDir() + "/.vitusOS/cache";
        std::string animusPath = resolveNixStorePath("animusengine");
        CacheKeepr::shared().initialize(
            desk->m_vulkan.get(), cacheDir, animusPath);

        // Shell
        desk->initShell(output);

        // Notify session process: compositor ready
        OSFBridge::shared().publishToSession(
            OSFEvent::CompositorReady, {});
    }

    // PanelManager: tambahkan Panel untuk output baru
    PanelManager::shared().onOutputAdded(output, isPrimary);
}
```

### OSFBridge — Interface Minimal

```cpp
// animus/core/OSFBridge.h
// Jembatan EventBus compositor ↔ EventBus session via Unix domain socket.
// Hanya event yang ditandai BRIDGED (Part 28.4) yang di-forward.
// LOCAL events tidak pernah melewati bridge.
#pragma once
#include "core/EventBus.h"
#include "core/OSFEvent.h"
#include <string>
#include <thread>
#include <atomic>

namespace Animus {

class OSFBridge {
public:
    static OSFBridge& shared();

    // Compositor side: koneksi ke session
    bool connectToSession(const std::string &socketPath);

    // Session side: bind dan tunggu compositor
    bool bindForCompositor(const std::string &socketPath);

    void destroy();

    // Forward event ke session (compositor → session)
    void publishToSession(OSFEvent event, std::any data);

    // Forward event ke compositor (session → compositor)
    // Hanya dipanggil dari session process
    void publishToCompositor(OSFEvent event, std::any data);

    // BRIDGED event list — hanya event ini yang dikirim via socket
    static bool isBridged(OSFEvent event);

private:
    OSFBridge() = default;

    int              m_sockFd   = -1;
    std::thread      m_rxThread;
    std::atomic<bool>m_running  = false;

    void receiveLoop();
    void serialize(OSFEvent event, const std::any &data,
                   std::vector<uint8_t> &out);
    bool deserialize(const uint8_t *buf, size_t len,
                     OSFEvent &event, std::any &data);
};

} // namespace Animus
```

---

## FIX2-07 — `CrashSite::onClientCrash()` Kirim `NotifData` Bukan `NotificationPayload`

### Masalah

`CrashSite.cpp` baris 8719–8730 mendefinisikan struct lokal `CrashNotifData`:

```cpp
struct CrashNotifData {
    std::string title;
    std::string body;
    int timeoutMs;
};
CrashNotifData nd { appId + " quit unexpectedly", "...", 7000 };
EventBus::shared().publishAsync(OSFEvent::NotificationPosted,
    std::move(nd));
```

Tapi FIX-03 dari Volume 1 menetapkan bahwa semua `NotificationPosted` events
harus menggunakan `NotificationPayload` (didefinisikan di `OSFEvent.h`).
Subscriber yang menerima `NotificationPosted` akan melakukan
`std::any_cast<NotificationPayload>` — dan `CrashNotifData` bukan
`NotificationPayload`. Crash: `std::bad_any_cast`.

### Fix

```cpp
// animus/crash/CrashSite.cpp — ganti CrashNotifData dengan NotificationPayload

#include "core/OSFEvent.h"  // untuk NotificationPayload

void CrashSite::onClientCrash(struct wl_client *client,
                                const std::string &appId)
{
    recordRespawn(appId);

    if (shouldRespawn(appId)) {
        respawnApp(appId);
    } else {
        // Unregister client dari RegistryManager
        RegistryManager::shared().clients().unregisterClientByPid(
            getPidForClient(client));

        EventBus::shared().publishAsync(OSFEvent::ClientCrashed,
            std::string(appId));

        // Gunakan NotificationPayload, bukan struct lokal
        EventBus::shared().publishAsync(OSFEvent::NotificationPosted,
            NotificationPayload{
                .title     = appId + " quit unexpectedly",
                .body      = "It could not be reopened automatically.",
                .timeoutMs = 7000
            });
    }
}
```

---

## AUDIT: Semua call site `NotificationPosted` yang harus pakai `NotificationPayload`

Ini adalah daftar lengkap berdasarkan pembacaan dokumen. Semua harus menggunakan
`NotificationPayload` struct:

```
File                          Baris     Status
──────────────────────────────────────────────────────────
PowerManager.cpp              ~8580     ✓ Fixed di Vol.1 FIX-03
PowerManager.cpp              ~8589     ✓ Fixed di Vol.1 FIX-03
CrashSite.cpp                 ~8729     ✓ Fixed di Vol.2 FIX2-07
WindowManager.cpp (migrate)   ~17491    ✗ BELUM — pakai anonymous struct {}
DBusBridge.cpp (onNotify)     ~7113     ✗ BELUM — pakai NotifData struct lokal
```

### DBusBridge::onNotify() fix

```cpp
// animus/eobus/DBusBridge.cpp — onNotify() yang benar

uint32_t DBusBridge::onNotify(const std::string &appName,
                               uint32_t replacesId,
                               const std::string &appIcon,
                               const std::string &summary,
                               const std::string &body,
                               int32_t timeout)
{
    if (!validateMessage(appName, "org.freedesktop.Notifications", "Notify"))
        return 0;

    // Clamp timeout: -1 = persistent, 0 = use default (5000ms), max 30000ms
    int timeoutMs;
    if (timeout < 0)     timeoutMs = -1;       // persistent
    else if (timeout == 0) timeoutMs = 5000;   // default
    else timeoutMs = std::min(timeout, 30000); // max 30s

    EventBus::shared().publishAsync(OSFEvent::NotificationPosted,
        NotificationPayload{
            .title       = summary,
            .body        = body,
            .timeoutMs   = timeoutMs,
            .isPersistent= (timeout < 0)
        });

    static uint32_t nextId = 1;
    return nextId++;
}
```

### WindowManager::migrateWindowsFromOutput() fix

```cpp
// animus/core/WindowManager.cpp — migrateWindowsFromOutput() yang benar

void WindowManager::migrateWindowsFromOutput(struct wlr_output *removed) {
    float cx = primaryOutput()->width  * 0.5f;
    float cy = primaryOutput()->height * 0.5f;

    float offset = 0.0f;
    RegistryManager::shared().windows().forEach(
        [&](RegHandle handle, OSFWindow *win) {
            if (win->currentOutput() == removed) {
                win->m_pos.setTarget(
                    cx - win->width()  * 0.5f + offset,
                    cy - win->height() * 0.5f + offset);
                win->setOutput(primaryOutput());
                offset += 24.0f;
            }
        });

    EventBus::shared().publishAsync(OSFEvent::NotificationPosted,
        NotificationPayload{
            .title     = "Display disconnected",
            .body      = "Windows moved to main display.",
            .timeoutMs = 5000
        });
}
```

---

## SUMMARY — Volume 2

Setelah membaca dokumen secara penuh dan jujur dari atas ke bawah:

**5 bugs compile/crash baru** ditemukan yang tidak ada di Volume 1:
1. Duplicate `WindowRestored` — tidak akan compile
2. `ShaderCache` referensi member tidak ada — tidak akan compile
3. `RenderPipeline` raw pointer iteration — use-after-free, SIGSEGV
4. `VulkanContext::commitFrame()` tidak punya `wlr_output*` — null dereference
5. `CrashSite` + `DBusBridge` + `WindowManager` kirim wrong notification type — `bad_any_cast`

**1 architectural clarification:**
- Two-process model (session + compositor) harus direfleksikan di `OSFDesktop::run()`
- `OSFBridge` wiring antara dua proses perlu didefinisikan

**1 rule clarification:**
- `pw_stream_new_simple` di `animus-early` adalah benar untuk konteksnya
- Rule 7 butuh catatan pengecualian yang eksplisit

---

## Absolute Rules Baru (tambahan ke 20 rules yang ada)

```
21. wlroots owns the scanout buffer. [dari Vol.1]

22. sodium_memzero() sebelum clear() untuk semua credential strings. [dari Vol.1]

23. Semua NotificationPosted events HARUS menggunakan NotificationPayload struct.
    Jangan pernah kirim anonymous struct {} atau local struct ke publishAsync
    untuk event ini. bad_any_cast adalah crash yang tidak terdeteksi saat compile.

24. RenderPipeline tidak boleh menyimpan OSFWindow* atau shared_ptr<OSFWindow>.
    Semua akses window melalui RegistryManager::shared().windows().forEach()
    atau .resolve(handle). Tidak ada pengecualian.

25. Setelah RegistryManager::unregisterWindow(handle), pointer yang tadinya
    valid menjadi dangling. Tidak ada komponen yang boleh menyimpan OSFWindow*
    setelah unregister. Selalu gunakan RegHandle.
```
