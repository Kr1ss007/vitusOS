# AnimusEngine — Architecture Gap Fixes, Volume 3
## vitusOS ARES · Upstream Color + Upstream One
## From complete read of Parts 29–32 and cross-section analysis

**Status:** Supplements Vol.1 and Vol.2
**Sections read this pass:** 29 (AnimusContext, CockpitView, OrangeBox,
SpringSolver extensions, throw physics, shutdown screen, Alt-Tab),
30 (MotionWave), 31 (DesktopManager), 32 (PanelManager, GlobalMenu)

---

## INDEX OF FIXES

| ID | Bug / Gap | Severity | Location |
|----|-----------|----------|----------|
| FIX3-01 | `StateManager::get()` null dereference in `DesktopManager` | **CRASH** | DesktopManager.cpp |
| FIX3-02 | `StateManager::screenWidth/Height()` methods don't exist | **COMPILE ERROR** | DesktopManager, OSFWindow |
| FIX3-03 | `SpringSolver2D::setVelocity()` not declared | **COMPILE ERROR** | SpringSolver.h |
| FIX3-04 | `SpringSolver2D::enableEdgeResistanceY()` not declared | **COMPILE ERROR** | SpringSolver.h |
| FIX3-05 | GestureRecognizer never formally deleted — InputRouter still references it | **COMPILE ERROR** | InputRouter.h |
| FIX3-06 | `animus_compositor_get_backend()` not declared in bridge header | **COMPILE ERROR** | animus_compositor.h |
| FIX3-07 | `MotionWave::initialize()` never called in OSFDesktop init sequence | **SILENT BUG** | OSFDesktop.cpp |
| FIX3-08 | `DesktopManager` missing from Vessels DAG | **MISSING** | Vessels.cpp |
| FIX3-09 | OSFEvent enum missing all events added in Parts 26A–32 | **COMPILE ERROR** | OSFEvent.h |
| FIX3-10 | CockpitView Part 15 model (overlay surface) vs Part 29 model (zoom) conflict | **ARCH** | CockpitView.h |
| FIX3-11 | OSFBridge serialization protocol unspecified | **GAP** | OSFBridge.h |
| FIX3-12 | `DesktopManager::switchPrev/Next` resets spring to 0 mid-transition | **BUG** | DesktopManager.cpp |
| FIX3-13 | `MotionWave` tap timing: `m_tapStartMs` never set | **BUG** | MotionWave.cpp |
| FIX3-14 | `CockpitView` close button fires wlr_surface destroy without RegistryManager | **CRASH** | CockpitView spec |
| FIX3-15 | `SpringSolver` conflicting reduced motion implementations between Part 22.5 and Part 38 | **CONFLICT** | SpringSolver.h |

---

## FIX3-01 — `StateManager::get()` Null Dereference in `DesktopManager`

### Problem

`DesktopManager::desktopForWindow()` at line 15563:

```cpp
int DesktopManager::desktopForWindow(uint64_t handle) const {
    std::string key = "windowDesktop:" + std::to_string(handle);
    try {
        return std::any_cast<int>(StateManager::shared().get(key));
    } catch (...) {
        return 0;
    }
}
```

`StateManager::get()` returns `const std::any*` — a pointer, nullable when
the key doesn't exist. `std::any_cast<int>` on a pointer performs a pointer
cast, not a value cast. This is a different overload from what was intended.

If the key exists: `std::any_cast<const std::any*>` gives a pointer, but
`std::any_cast<int>(ptr)` where ptr is `const std::any*` is not the value
cast — it's casting the pointer itself as data, which is undefined behavior
that will silently produce garbage.

If the key doesn't exist: `get()` returns `nullptr`. `std::any_cast<int>(nullptr)`
on the pointer overload returns `nullptr` cast to `int*`, which is null —
undefined behavior again.

The `try/catch` will not help because neither case throws.

### Fix

```cpp
// animus/shell/DesktopManager.cpp — corrected desktopForWindow()

int DesktopManager::desktopForWindow(uint64_t handle) const {
    std::string key = "windowDesktop:" + std::to_string(handle);

    // Use getAs<int>() which handles missing keys and bad_any_cast safely
    return StateManager::shared().getAs<int>(key, 0);
    // Returns 0 (Desktop 1) if key missing — correct default behavior
}

// Same pattern for windowVisibleOnCurrent():
bool DesktopManager::windowVisibleOnCurrent(uint64_t handle) const {
    return desktopForWindow(handle) == m_currentIndex;
}
```

`StateManager::getAs<T>()` (already declared in the document at line 3046)
handles both missing keys and wrong types by returning the default value.
Use it everywhere StateManager values are read with known types.

---

## FIX3-02 — `StateManager::screenWidth/Height()` Do Not Exist

### Problem

`DesktopManager::switchPrev()`, `switchNext()`, and
`OSFWindow::onPointerButtonRelease()` call:

```cpp
float screenW = StateManager::shared().screenWidth();
float screenH = StateManager::shared().screenHeight();
```

`StateManager` is a key-value store. It has no `screenWidth()` or
`screenHeight()` methods. These do not exist in the `StateManager.h`
specification. The code will not compile.

### Fix

Add screen geometry to `StateKey` and update all call sites:

```cpp
// animus/core/StateManager.h — add to StateKey namespace

namespace StateKey {
    // ... existing keys ...

    // Screen geometry — written by OSFDesktop on first output attach
    // and on OutputResized event. Read by DesktopManager, OSFWindow, etc.
    constexpr char ScreenWidth[]    = "screen_width";   // float
    constexpr char ScreenHeight[]   = "screen_height";  // float
    constexpr char ScreenDpiScale[] = "screen_dpi_scale"; // float, default 1.0
}
```

```cpp
// animus/core/OSFDesktop.cpp — write screen dimensions on output attach

void OSFDesktop::cbOutputAdded(struct wlr_output *output,
                                bool isPrimary, void *ud) {
    if (isPrimary) {
        StateManager::shared().set(StateKey::ScreenWidth,
            static_cast<float>(output->width));
        StateManager::shared().set(StateKey::ScreenHeight,
            static_cast<float>(output->height));
        // Also update on OutputResized via EventBus subscription
    }
    // ... rest of cbOutputAdded ...
}
```

```cpp
// animus/shell/DesktopManager.cpp — corrected switchPrev/Next

void DesktopManager::switchPrev(float velocity) {
    float screenW = StateManager::shared()
                        .getAs<float>(StateKey::ScreenWidth, 1920.0f);

    if (m_currentIndex == 0) {
        triggerBounce(1.0f);
        return;
    }
    m_slideOffsetX.reset(0.0f);
    m_slideOffsetX.setTarget(screenW);
    m_slideOffsetX.setVelocity(velocity);
    m_bgSlideOffsetX.reset(0.0f);
    m_bgSlideOffsetX.setTarget(screenW * PARALLAX_FACTOR);
    m_bgSlideOffsetX.setVelocity(velocity * PARALLAX_FACTOR);
    // ... rest unchanged ...
}

// animus/shell/OSFWindow.cpp — corrected onPointerButtonRelease

void OSFWindow::onPointerButtonRelease(float vx, float vy) {
    if (m_dragging) {
        m_dragging = false;

        vx = std::clamp(vx, -2000.0f, 2000.0f);
        vy = std::clamp(vy, -2000.0f, 2000.0f);

        m_pos.setVelocity(vx, vy);

        float screenW = StateManager::shared()
                            .getAs<float>(StateKey::ScreenWidth,  1920.0f);
        float screenH = StateManager::shared()
                            .getAs<float>(StateKey::ScreenHeight, 1080.0f);

        m_pos.enableEdgeResistance(
            -m_width  * 0.5f,
            screenW - m_width * 0.5f,
            32.0f);
        m_pos.enableEdgeResistanceY(
            Panel::HEIGHT,
            screenH - 32.0f,
            32.0f);
    }
}
```

---

## FIX3-03 — `SpringSolver2D::setVelocity()` Not Declared

### Problem

Part 29 adds `setVelocity(float vel)` to `SpringSolver` (1D), but
`OSFWindow::onPointerButtonRelease()` calls:

```cpp
m_pos.setVelocity(vx, vy);  // m_pos is SpringSolver2D
```

`SpringSolver2D` delegates to two `SpringSolver x, y` members.
`SpringSolver2D::setVelocity(float, float)` is never declared.

### Fix

```cpp
// animus/animation/SpringSolver.h — add to SpringSolver2D

class SpringSolver2D {
public:
    // ... existing API unchanged ...

    void setTarget(float tx, float ty) { x.setTarget(tx); y.setTarget(ty); }
    void snap(float sx, float sy)      { x.snap(sx); y.snap(sy); }
    void reset(float rx, float ry)     { x.reset(rx); y.reset(ry); }
    bool isResting() const             { return x.isResting() && y.isResting(); }
    void tick(float dt)                { x.tick(dt); y.tick(dt); }

    // ── NEW: throw velocity ───────────────────────────────────────
    void setVelocity(float vx, float vy) {
        x.setVelocity(vx);
        y.setVelocity(vy);
    }

    // ── NEW: edge resistance — per-axis ──────────────────────────
    // Call separately for X and Y axes because bounds differ:
    // X: left/right screen edges
    // Y: Panel top / bottom of screen
    void enableEdgeResistance(float minX, float maxX,
                               float resistZone = 20.0f) {
        x.enableEdgeResistance(minX, maxX, resistZone);
    }
    void enableEdgeResistanceY(float minY, float maxY,
                                float resistZone = 20.0f) {
        y.enableEdgeResistance(minY, maxY, resistZone);
    }
    void disableEdgeResistance() {
        x.disableEdgeResistance();
        y.disableEdgeResistance();
    }

    SpringSolver x, y;
};
```

---

## FIX3-04 — `SpringSolver::enableEdgeResistanceY()` Not Declared

Covered by FIX3-03. `enableEdgeResistanceY()` is the Y-axis-specific version
of `enableEdgeResistance()` on `SpringSolver2D`. Added in the fix above.
The 1D `SpringSolver` already has `enableEdgeResistance(min, max, zone)` from
Part 29.10. No additional change needed there.

---

## FIX3-05 — GestureRecognizer Never Formally Deleted

### Problem

Part 30 states:
> "GestureRecognizer.cpp/.h is renamed to MotionWave.cpp/.h.
> InputRouter's m_gestures member changes type to std::unique_ptr\<MotionWave\>."

But `InputRouter.h` from Part 12 still declares:
```cpp
class GestureRecognizer;
// ...
// InputRouter delegates swipe/pinch events to MotionWave::shared()
```

There is no `m_gestures` member shown — the comment says to delegate to the
singleton — but the forward declaration of `GestureRecognizer` is still there.
If any compilation unit includes both `InputRouter.h` and `GestureRecognizer.h`,
the old class is still in scope.

### Fix

```cpp
// animus/input/InputRouter.h — complete replacement of gesture section

#pragma once
#include <cstdint>

namespace Animus {

// InputRouter: routes raw compositor events to MotionWave and focused surfaces.
// MotionWave is the sole gesture recognizer — GestureRecognizer is deleted.
// DO NOT include GestureRecognizer.h anywhere. That file no longer exists.
class InputRouter {
public:
    static InputRouter& shared();
    void initialize();

    // Raw compositor callbacks (main thread)
    void onKey(uint32_t sym, uint32_t mods, bool pressed);
    void onPointerMotion(double x, double y);
    void onPointerButton(uint32_t button, bool pressed);
    void onPointerAxis(double dx, double dy);

    // Delegated to MotionWave::shared()
    void onSwipeBegin(uint32_t fingers);
    void onSwipeUpdate(uint32_t fingers, double dx, double dy);
    void onSwipeEnd(bool cancelled);
    void onPinchBegin(uint32_t fingers);
    void onPinchUpdate(uint32_t fingers, double dx, double dy,
                        double scale, double rotation);
    void onPinchEnd(bool cancelled);

    double pointerX() const { return m_px; }
    double pointerY() const { return m_py; }

private:
    InputRouter() = default;

    // NOTE: NO GestureRecognizer member. MotionWave is a singleton.
    // Access via MotionWave::shared() — no ownership here.

    double m_px           = 0;
    double m_py           = 0;
    bool   m_altDownAlone = false;
    float  m_screenW      = 1920.f;  // updated from ScreenWidth StateKey
    float  m_screenH      = 1080.f;
};

} // namespace Animus
```

Also: `animus/input/GestureRecognizer.h` and `GestureRecognizer.cpp` must be
deleted from the repository. Any `#include "GestureRecognizer.h"` remaining
is a compile error.

---

## FIX3-06 — `animus_compositor_get_backend()` Not Declared

### Problem

`OSFDesktop::cbOutputAdded()` (in Vol.2 FIX2-06) calls:
```cpp
int drm_fd = wlr_backend_get_drm_fd(animus_compositor_get_backend());
```

`animus_compositor_get_backend()` was never declared in `animus_compositor.h`.

### Fix

```c
// compositor/animus_compositor.h — add to public API

// Returns the wlr_backend created during init.
// Required by VulkanContext to obtain the DRM fd for device selection.
struct wlr_backend* animus_compositor_get_backend(void);

// Returns the primary output (first connected monitor).
// May be NULL before any monitor connects.
struct wlr_output*  animus_compositor_get_primary_output(void);

// Returns the wlr_renderer (Vulkan renderer).
// Required by VulkanContext to share device/instance with wlroots.
struct wlr_renderer* animus_compositor_get_renderer(void);
```

```c
// compositor/animus_compositor.c — implementations

struct wlr_backend* animus_compositor_get_backend(void) {
    return g.backend;
}

struct wlr_output* animus_compositor_get_primary_output(void) {
    return g.primary_output;
}

struct wlr_renderer* animus_compositor_get_renderer(void) {
    return g.renderer;
}
```

---

## FIX3-07 — `MotionWave::initialize()` Never Called

### Problem

`MotionWave::initialize()` reads sensitivity, natural scroll, and per-gesture
enable states from `StateManager`. If not called, all settings default to
hardcoded values ignoring user preferences from `vitusos-config.nix`.
No component calls it in any init sequence.

### Fix

```cpp
// animus/core/OSFDesktop.cpp — add MotionWave init to sequence

void OSFDesktop::initShell(struct wlr_output *output) {
    // ... existing shell init ...

    // ── MotionWave: MUST be called after StateManager is ready
    //    and BEFORE compositor starts receiving input events.
    //    StateManager is ready from Step 3.
    //    Input events arrive after animus_compositor_run() —
    //    so any time before run() is safe. Put it here.
    MotionWave::shared().initialize();

    // ── DesktopManager
    DesktopManager::shared().initialize();

    // ... Panel, Dock, CockpitView, LockScreen init ...
}
```

---

## FIX3-08 — `DesktopManager` Missing from Vessels DAG

### Problem

`Vessels::initialize()` (Part 21.8) registers all subsystems. `DesktopManager`,
`MotionWave`, `PanelManager`, and `OrangeBoxMenu` are all absent.
If `DesktopManager` fails (e.g. StateManager dies), Vessels can't propagate
the blast radius to DesktopManager and its dependents.

### Fix

```cpp
// animus/crash/Vessels.cpp — add to initialize()

// After the existing registrations:

registerVessel({ "DesktopManager", {"StateManager", "AnimationEngine"},
    []{ /* isolated: desktop switching disabled, stays on current */ },
    []{ /* restored: resume normal desktop switching */ }
});

registerVessel({ "MotionWave", {"EventBus"},
    []{ /* isolated: all gestures disabled — input still works for pointer */ },
    []{ /* restored: gestures re-enabled */ }
});

registerVessel({ "PanelManager", {"RenderPipeline", "EventBus"},
    []{ /* isolated: panels hidden — compositor still runs */ },
    []{ /* restored: panels re-shown */ }
});

registerVessel({ "OrangeBoxMenu", {"PanelManager"},
    []{ /* isolated: orange box click does nothing */ },
    []{ /* restored: orange box functional again */ }
});

registerVessel({ "SystemScreen", {"RenderPipeline"},
    []{ /* isolated: shutdown/restart shows black but no message */ },
    []{ /* restored: shutdown screen functional */ }
});
```

---

## FIX3-09 — OSFEvent Enum Missing All Events Added in Parts 26A–32

### Problem

The canonical `OSFEvent.h` (Part 4.5, lines 1281–1382) was written before
Parts 26A through 32 existed. Each later part says "Add to OSFEvent.h enum"
but the original enum listing is never updated to show the final consolidated
state. An implementer building from the canonical OSFEvent.h will miss all
of these. None of them compile until added.

Missing events by part:

| Event | Part | Notes |
|-------|------|-------|
| `CockpitViewOpen` | 29 | replaces `CockpitViewToggle` for open |
| `CockpitViewClose` | 29 | replaces `CockpitViewToggle` for close |
| `CockpitViewCycleNext` | 29 | Alt+Tab cycle forward |
| `CockpitViewCyclePrev` | 29 | Shift+Alt+Tab cycle backward |
| `OrangeBoxMenuOpen` | 29 | orange box single-click |
| `OrangeBoxMenuClose` | 29 | menu dismissed |
| `SystemShutdown` | 29 | BRIDGED |
| `SystemRestart` | 29 | BRIDGED |
| `AboutVitusOS` | 29 | LOCAL |
| `AppIndexReady` | 26A | AppIndexCache rebuild done |
| `InstallProgress` | 26A | progress update |
| `InstallComplete` | 26A | BRIDGED |
| `RemoveComplete` | 26A | BRIDGED |
| `RemoveFailed` | 26A | LOCAL |
| `CacheEvicted` | 26 | data = PressureLevel |
| `CacheInvalidated` | 26 | data = std::string component |
| `ResourcePressure` | 21 | data = ResourceSnapshot |
| `SubsystemHealthChanged` | 21 | data = HandshakeResult |
| `ClientCrashed` | 21 | data = std::string appId |
| `BlastRadius` | 21 | data = vector\<string\> |
| `InstallFailed` | 21 | data = std::string stderr |
| `MemoryPressure` | 21 | data = PressureLevel |
| `ShutdownRequested` | 24 | fatal error, controlled exit |
| `ConfigReload` | 24 | SIGHUP received |
| `StateChanged` | StateManager | data = std::string key |
| `HEVUnlocked` | 25 | data = std::string deviceId |
| `HEVLocked` | 25 | |
| `HEVSealed` | 25 | |
| `HEVAccessDenied` | 25 | |
| `HEVAuthorizationNeeded` | 25 | |
| `ProximityUnlockReady` | 25 | |
| `ProximityLockWarning` | 25 | |
| `DBusMenuChanged` | 22 | |
| `StatusNotifierChanged` | 22 | |
| `AccessibilityTreeChanged` | 22 | |
| `ReducedMotionChanged` | 22 | data = bool |
| `OpenURI` | 22 | data = std::string uri |
| `PortalFileChosen` | 22 | |
| `PortalScreenCastStarted` | 22 | |

### Fix

The canonical `OSFEvent.h` must be replaced with the consolidated version
from FIX2-01 (Volume 2), extended with all the above. The Vol.2 OSFEvent.h
already includes most of them. Verify every event listed above appears in
the final enum before compilation begins.

**Rule: There is ONE OSFEvent.h. It is the canonical list. Every part that
says "Add to OSFEvent.h" is an amendment to that one file. The final file
must contain all amendments.**

---

## FIX3-10 — CockpitView Part 15 vs Part 29 Architectural Conflict

### Problem

Part 15 specifies `CockpitView` as a separate `SurfaceAltitude::High`
overlay surface with its own `Card` struct containing:
```cpp
struct Card {
    VkImage        thumbImage;
    VkImageView    thumbView;
    VkDeviceMemory thumbMem;
    SpringSolver2D pos;
    SpringSolver   scale;
    SpringSolver   opacity;
};
```

Part 29 (section 29.4) supersedes this entirely:
> "There is no CockpitView surface. There is one desktop. The camera zooms out."

These two models are structurally incompatible. Part 29 is canonical.
But the Part 15 `CockpitView.h` with its `Card` struct and full overlay
approach is still present in the document and has not been marked deleted.

### Fix — CockpitView.h for the Zoom Model

```cpp
// animus/shell/CockpitView.h — REPLACES Part 15 entirely.
// CockpitView is NOT a separate surface. It is a zoom level.
// RenderPipeline applies m_cockpitZoom transform to the window layer.
// This class owns the zoom spring and sidebar spring.
// All window card rendering happens inside RenderPipeline at reduced scale.
#pragma once
#include "animation/SpringSolver.h"
#include "core/EventBus.h"
#include "core/AnimusContext.h"
#include "registry/RegistryManager.h"
#include <string>
#include <vector>

struct wlr_output;

namespace Animus {

class CockpitView {
public:
    static CockpitView& shared();

    bool initialize();

    // Open from AnimusContext — animates zoom out
    void open(const AnimusContext &ctx);

    // Close — animates zoom back in
    // If handle != REG_INVALID: zoom into that window's position
    // Otherwise: zoom into previously focused window
    void close(RegHandle focusWindowHandle = REG_INVALID);

    bool isOpen() const { return m_open; }

    // Cycle highlighted window (Alt+Tab while open)
    void cycleNext();
    void cyclePrev();

    // Called by Dock click while CockpitView is open
    // Assigns clicked Dock app window to target desktop, closes CockpitView
    void onDockLaunch(const std::string &appId);

    // Called by CockpitView sidebar + button click
    void addDesktop();

    // Called by drag-to-desktop gesture
    void assignWindowToDesktop(RegHandle windowHandle, int desktopIndex);

    // Read by RenderPipeline every frame — drives the zoom transform
    float zoomLevel()    const { return m_cockpitZoom.value(); }
    float zoomOffsetY()  const { return m_cockpitOffsetY.value(); }
    float zoomOffsetX()  const { return m_cockpitOffsetX.value(); }
    float bgDarken()     const { return m_cockpitBgDarken.value(); }
    float sidebarX()     const { return m_sidebarX.value(); }

    // Highlighted window for Alt+Tab cycling
    RegHandle highlightedWindow() const { return m_highlightedHandle; }

    static constexpr float ZOOM_DESKTOP    = 1.00f;  // normal desktop
    static constexpr float ZOOM_COCKPIT    = 0.45f;  // cockpit view
    static constexpr float SIDEBAR_WIDTH   = 80.0f;  // virtual desktop sidebar
    static constexpr float COCKPIT_OFFSET_Y= 60.0f;  // px, shifts windows up

    // Close button on window card — appears on hover
    static constexpr float CLOSE_BTN_SIZE  = 20.0f;  // px
    static constexpr uint32_t CLOSE_BTN_COLOR = 0xFFFF3B30; // #FF3B30

private:
    CockpitView() = default;

    bool         m_open             = false;
    RegHandle    m_highlightedHandle= REG_INVALID;
    RegHandle    m_prevFocused      = REG_INVALID;  // restore on close
    int          m_highlightedIdx   = -1;

    // The zoom springs — read by RenderPipeline each frame
    SpringSolver m_cockpitZoom;       // SPRING_SELECTION (400,28): 1.0 ↔ 0.45
    SpringSolver m_cockpitOffsetY;    // SPRING_SELECTION: 0 ↔ +60px
    SpringSolver m_cockpitOffsetX;    // SPRING_SELECTION: 0 ↔ +40px (sidebar)
    SpringSolver m_cockpitBgDarken;   // SPRING_SELECTION: 0.0 ↔ 0.5
    SpringSolver m_sidebarX;          // SPRING_SELECTION: -80 ↔ 0

    // Per-window close button hover springs
    // Key: RegHandle, Value: hover alpha spring
    // Created lazily when CockpitView opens
    std::unordered_map<RegHandle, SpringSolver> m_closeBtnHover;

    uint64_t m_openHandle  = 0;
    uint64_t m_closeHandle = 0;
    uint64_t m_tickHandle  = 0;

    std::vector<RegHandle> m_windowOrder;  // z-order at time of open
    void buildWindowOrder();
};

} // namespace Animus
```

### RenderPipeline zoom transform addition

```cpp
// animus/render/RenderPipeline.cpp — add to renderFrame() Layer 1+2+3

// Layer 1+2+3: Windows with CockpitView zoom transform
{
    float zoom    = CockpitView::shared().zoomLevel();
    float offsetY = CockpitView::shared().zoomOffsetY();
    float offsetX = CockpitView::shared().zoomOffsetX();
    float darken  = CockpitView::shared().bgDarken();

    // Apply uniform scale transform centered on screen for window layer
    // All window positions and sizes multiplied by zoom factor
    // Translate by offsets to center the zoomed view

    RegistryManager::shared().windows().forEach(
        [&](RegHandle handle, OSFWindow *win) {
            if (!win->isVisible()) return;

            // Apply CockpitView transform
            float wx = (win->x() + offsetX) * zoom
                       + m_ctx->width  * (1.0f - zoom) * 0.5f;
            float wy = (win->y() + offsetY) * zoom
                       + m_ctx->height * (1.0f - zoom) * 0.5f;
            float ww = win->width()  * zoom;
            float wh = win->height() * zoom;

            // Layer 1: Shadow (at zoomed position)
            float swx = (win->shadowX() + offsetX) * zoom
                        + m_ctx->width  * (1.0f - zoom) * 0.5f;
            float swy = (win->shadowY() + offsetY) * zoom
                        + m_ctx->height * (1.0f - zoom) * 0.5f;
            m_shadow->drawWindowShadow(cmd, swx, swy, ww, wh,
                win->cornerRadius() * zoom);

            // Layer 2: Glass background
            m_material->drawGlassSurface(cmd, wx, wy, ww, wh,
                win->cornerRadius() * zoom, win->altitude());

            // Layer 3: Content
            m_material->drawWindowSurface(cmd, win, wx, wy, ww, wh);

            // CockpitView: draw close button on hover
            if (CockpitView::shared().isOpen()) {
                // ... close button render using m_closeBtnHover[handle] ...
            }
        });

    // Wallpaper darkening during CockpitView
    if (darken > 0.01f) {
        // Draw semi-transparent black over wallpaper layer
        // Color: 0x000000 at darken opacity
        m_material->drawRoundRect(cmd, 0, 0,
            (float)m_ctx->width, (float)m_ctx->height,
            0.0f,
            /* fill */ static_cast<uint32_t>(darken * 255) << 24,
            /* border */ 0, 0.0f, 1.0f);
    }
}
```

---

## FIX3-11 — OSFBridge Serialization Protocol Unspecified

### Problem

`OSFBridge` is referenced in Part 28 (systemd service relationship), Part 22
(EO-Bus description), and multiple "BRIDGED" event annotations throughout.
But the serialization format on `/run/vitusos/osf-ipc.sock` is never defined.
An implementer cannot build this component.

### Fix — Define minimum viable wire protocol

```cpp
// animus/core/OSFBridge.h — extended with wire protocol

// Wire format for OSFBridge Unix domain socket:
//
// Each message is a length-prefixed binary packet:
//   [4 bytes] uint32_t message length (little-endian, not including this field)
//   [4 bytes] uint32_t event id (OSFEvent cast to uint32_t)
//   [1 byte]  uint8_t  payload type:
//                0x00 = no payload (std::any empty)
//                0x01 = bool (1 byte follows)
//                0x02 = int32 (4 bytes follow, little-endian)
//                0x03 = float (4 bytes follow, IEEE 754)
//                0x04 = string (4-byte length + UTF-8 bytes, no null term)
//                0x05 = AnimusContext (packed struct, 28 bytes fixed)
//                0x06 = uint64 (8 bytes follow, little-endian)
//   [variable] payload bytes per type above
//
// Socket: SOCK_STREAM, SOCK_SEQPACKET not required.
// Session process binds and listens. Compositor process connects.
// Reconnect: if connection drops, compositor attempts reconnect every 500ms.
// Max message size: 4096 bytes. Larger payloads are errors — drop and log.
//
// Thread safety:
//   OSFBridge has a dedicated rx thread reading from the socket.
//   Received events are dispatched via EventBus::publishAsync()
//   which safely delivers them to the compositor main thread.
//   Write path (publishToSession): protected by mutex.

// Supported BRIDGED events and their payload types:
//
// compositor → session:
//   ClientConnected        → uint64 (windowHandle)
//   ClientCrashed          → string (appId)
//   WindowFocusChanged     → uint64 (windowHandle)
//   CompositorReady        → no payload
//   FatalError             → string (description)
//   SystemShutdown         → no payload
//   SystemRestart          → no payload
//
// session → compositor:
//   HEVUnlocked            → string (deviceId)
//   HEVLocked              → no payload
//   WallpaperChanged       → string (path)
//   StateChanged           → string (key) [compositor reads new value from its own StateManager]
//   InstallComplete        → string (appId)
//   AppIndexReady          → no payload
//   ConfigReload           → no payload
//   DesktopSwitched        → int32 (desktopIndex) [BRIDGED for multi-monitor sync]

// AnimusContext wire layout (payload type 0x05, 28 bytes):
//   [1 byte]  uint8_t  type (AnimusContext::Type enum)
//   [4 bytes] float    originX
//   [4 bytes] float    originY
//   [4 bytes] float    originW
//   [4 bytes] float    originH
//   [8 bytes] double   triggeredAtS
//   [3 bytes] padding (align to 28 bytes total)
```

---

## FIX3-12 — `DesktopManager::switchPrev/Next` Spring Reset Mid-Transition

### Problem (documented as BUG-31-1, but the root cause is a real bug)

`switchPrev()` and `switchNext()` both call `m_slideOffsetX.reset(0.0f)`
before setting the new target. If the user swipes rapidly while a transition
is in progress, the spring position jumps to 0 (the reset) before springing
to the new target. This creates a visual discontinuity — the desktop
teleports to center then slides again.

The document acknowledges this as BUG-31-1 ("Brief visual discontinuity on
rapid swipes") and defers it. For Upstream One (stable channel), this
should be fixed before promotion.

### Fix (for Upstream One — mark as post-unstable for Upstream Color)

```cpp
// DesktopManager.cpp — accumulating offset model
// Instead of reset + new target, accumulate from current spring value

void DesktopManager::switchPrev(float velocity) {
    if (m_currentIndex == 0) {
        triggerBounce(1.0f);
        return;
    }
    float screenW = StateManager::shared()
                        .getAs<float>(StateKey::ScreenWidth, 1920.0f);

    // KEY CHANGE: do NOT reset the spring.
    // The spring is currently somewhere between 0 and previous target.
    // Set a new target that represents the accumulated displacement
    // from the current position.
    float currentOffset = m_slideOffsetX.value();
    m_slideOffsetX.setTarget(currentOffset + screenW);
    // Do not call reset() — position continuity preserved
    m_slideOffsetX.setVelocity(velocity);

    float currentBgOffset = m_bgSlideOffsetX.value();
    m_bgSlideOffsetX.setTarget(currentBgOffset + screenW * PARALLAX_FACTOR);
    m_bgSlideOffsetX.setVelocity(velocity * PARALLAX_FACTOR);

    m_currentIndex--;
    StateManager::shared().set(StateKey::CurrentDesktopIndex, m_currentIndex);
    SoundEngine::shared().play(Sounds::DesktopSwitch, 0.5f);
    EventBus::shared().publish(OSFEvent::DesktopSwitched, m_currentIndex);
    persistState();
}

// Same pattern for switchNext() — negate the offsets.
// RenderPipeline must also handle the case where slideOffsetX
// is at a non-zero settled value — normalization needed on settle:

void DesktopManager::tick(float dt) {
    m_slideOffsetX.tick(dt);
    m_bgSlideOffsetX.tick(dt);

    // When both springs settle: normalize to 0.
    // This prevents accumulated floating point drift over many switches.
    if (m_slideOffsetX.isResting() && m_bgSlideOffsetX.isResting()) {
        m_slideOffsetX.reset(0.0f);
        m_bgSlideOffsetX.reset(0.0f);
    }

    // ... bounce handling unchanged ...
}
```

---

## FIX3-13 — MotionWave Tap Timing: `m_tapStartMs` Never Set

### Problem (documented as BUG-30-2, but fixable)

`m_tapStartMs` is initialized to 0 and never updated. Tap duration is
effectively not measured — only travel distance (≤10px) is checked.
This means a very slow deliberate three-finger press-and-hold-then-release
with minimal movement fires `ShowDesktopToggle` unintentionally.

The document defers this as "acceptable for unstable ISO" but the fix
is straightforward: thread `time_msec` from the wlroots swipe events.

### Fix

```c
// compositor/animus_compositor.c — already passes time_msec in swipe events
// wlr_pointer_swipe_begin_event has time_msec: uint32_t
// wlr_pointer_swipe_update_event has time_msec: uint32_t
// Already forwarded via the callbacks. Need to thread it through.

// C11 compositor h_swipe_begin — already: g.on_swipe_begin(ev->fingers, g.ud)
// Change signature to include time_msec:
void (*on_swipe_begin)(uint32_t fingers, uint32_t time_msec, void*);
void (*on_swipe_end)(bool cancelled, uint32_t time_msec, void*);
```

```cpp
// animus/input/MotionWave.h — update callback signatures
void onSwipeBegin(uint32_t fingers, uint32_t time_msec);
void onSwipeEnd(bool cancelled, uint32_t time_msec);

// animus/input/MotionWave.cpp — store tap start time

void MotionWave::onSwipeBegin(uint32_t fingers, uint32_t time_msec) {
    resetSwipe();
    m_swipeFingers = fingers;
    if (fingers == 3) {
        m_swipeState = SwipeState::Tracking;
        m_tapState   = TapState::Waiting;
        m_tapStartMs = time_msec;  // ← NOW SET CORRECTLY
    }
}

void MotionWave::onSwipeEnd(bool cancelled, uint32_t time_msec) {
    if (cancelled) { resetSwipe(); return; }

    if (m_swipeFingers == 3) {
        if (m_tapState == TapState::Waiting) {
            double totalTravel = m_tapTravelX + m_tapTravelY;
            uint32_t duration  = time_msec - m_tapStartMs;

            // Both travel AND duration must be within tap limits
            if (totalTravel <= TAP_MAX_TRAVEL_PX &&
                duration    <= (uint32_t)TAP_MAX_MS) {
                fireTapResult();
                resetSwipe();
                return;
            }
        }
        // ... rest unchanged ...
    }
    resetSwipe();
}
```

---

## FIX3-14 — CockpitView Close Button Bypasses RegistryManager

### Problem

Part 29 specifies that the close button (×) on each window card in CockpitView:
> "Click: wl_surface destroy → WindowManager::removeSurface()"

This is wrong — it fires `wl_surface destroy` directly, which is what the
Wayland client does to indicate it wants to close. The compositor cannot
unilaterally call `wl_surface destroy`. The correct action is to send a
`wl_surface::close()` request to the Wayland client via
`wlr_xdg_toplevel_send_close()`, then wait for the client to respond.

Additionally, the close button must go through RegistryManager to safely
retrieve the window pointer from a handle.

### Fix

```cpp
// CockpitView close button click handler:

void CockpitView::onCloseButtonClicked(RegHandle windowHandle) {
    // Resolve safely — window may have died between click and handler
    OSFWindow *win = RegistryManager::shared()
                        .windows()
                        .resolve(windowHandle);
    if (!win) return;  // already gone — no action needed

    // Send close request to the Wayland client (not force-kill)
    // The client decides when to actually destroy its surface.
    // wlroots will fire on_surface_destroy when the client complies.
    struct wlr_xdg_toplevel *toplevel = win->xdgToplevel();
    if (toplevel) {
        wlr_xdg_toplevel_send_close(toplevel);
    }

    // Remove from CockpitView close button hover springs
    m_closeBtnHover.erase(windowHandle);

    // CockpitView remains open — reflows on WindowClosed event
    // EventBus::subscribe(OSFEvent::WindowClosed) → rebuild window order
}
```

---

## FIX3-15 — Conflicting Reduced Motion Implementations

### Problem

Two separate, incompatible reduced motion implementations exist:

**Part 22.5 (AccessibilityProvider integration):**
```cpp
namespace Animus {
inline bool& reducedMotionEnabled() {
    static bool g = false;
    return g;
}
}
// SpringSolver tick(): if (reducedMotionEnabled()) { snap; return; }
```

**Part 38 (Reduced Motion complete spec):**
```cpp
class SpringSolver {
    static void setReducedMotion(bool reduced) {
        s_reducedMotion.store(reduced, std::memory_order_relaxed);
    }
    static std::atomic<bool> s_reducedMotion;

    // Plus: per-spring m_eliminateOnReducedMotion flag
    // Allows selective elimination (some springs preserved)
};
```

Part 22.5 snaps ALL springs globally. Part 38 snaps only springs marked
with `setEliminateOnReducedMotion(true)`, preserving direct manipulation
springs. Part 38 is the complete, correct specification.
Part 22.5 is an incomplete earlier draft. They cannot both exist.

### Fix

**Part 22.5's implementation is superseded by Part 38. Delete it.**

```cpp
// animus/animation/SpringSolver.h — canonical reduced motion (Part 38 only)

// DELETE this from Part 22.5:
// inline bool& reducedMotionEnabled() { static bool g = false; return g; }
// This function must not exist. It conflicts with Part 38's atomic approach.

// USE ONLY the Part 38 model:
class SpringSolver {
public:
    // Global: set by Settings → Appearance → Reduce Motion
    static void setReducedMotion(bool reduced);
    static bool reducedMotion();

    // Per-spring: mark which springs are eliminated vs preserved
    // Default: false (not eliminated = preserved during reduced motion)
    void setEliminateOnReducedMotion(bool b) {
        m_eliminateOnReducedMotion = b;
    }

    void tick(float dt) {
        dt = std::clamp(dt, 0.001f, 0.100f);

        bool reduced = s_reducedMotion.load(std::memory_order_relaxed);
        if (reduced && m_eliminateOnReducedMotion) {
            m_pos = m_target;
            m_vel = 0.0f;
            return;
        }
        // If reduced but NOT eliminatable: spring runs normally
        // (direct manipulation, safety-critical, scroll)

        if (isResting()) { m_pos = m_target; m_vel = 0.0f; return; }
        float accel = -m_cfg.stiffness * (m_pos - m_target)
                    - m_cfg.damping    * m_vel;
        m_vel += accel * dt;
        m_pos += m_vel * dt;
    }

private:
    // ...
    bool                    m_eliminateOnReducedMotion = false;
    static std::atomic<bool> s_reducedMotion;
};
```

`AccessibilityProvider` subscribes to `OSFEvent::ReducedMotionChanged` and
calls `SpringSolver::setReducedMotion(bool)` — not the inline function.
No other change to `AccessibilityProvider` is needed.

---

## ADDITIONAL GAPS FOUND — LESS SEVERE BUT REAL

### GAP-A: `ConfigWriter` is referenced but never specced

`DesktopManager::persistState()` references a `ConfigWriter` component
that "writes vitusos-config.nix, background thread, non-blocking."
BUG-31-3 documents this as a known gap. For Upstream Color this is acceptable.
For Upstream One: `ConfigWriter` must be fully specced. At minimum:

- Atomic write: write to `.tmp` file, then `rename()` (same pattern as `InstallManager`)
- Serialization: Nix expression syntax for user preferences section
- Background thread: `std::async` or dedicated thread; never blocks compositor
- Debounce: coalesce multiple rapid changes into one write (500ms window)

### GAP-B: `StateManager::get()` return type inconsistency

`StateManager::get()` returns `const std::any*`. Throughout the codebase, call
sites do either:
```cpp
auto *v = sm.get(key);        // correct: null-check before use
auto val = sm.get(key);       // wrong: stores pointer, not value
std::any_cast<T>(sm.get(key)); // wrong: pointer cast, not value cast
```

The document uses all three patterns in different places. The correct pattern
everywhere is `getAs<T>(key, default)` for typed access, or `get(key)` with
explicit null-check followed by `*v` dereference. Audit all call sites.

### GAP-C: `SpringSolver::reset(float)` vs `snap(float)` naming inconsistency

Part 9 declares `snap(float v)` as the API to snap position without animation.
Part 29.3 uses `m_scale.reset(0.95f)` and `m_pos.reset(ctx.originX, ctx.originY)`.
Part 31 uses `m_slideOffsetX.reset(0.0f)`.

`reset()` is not declared in the canonical Part 9 `SpringSolver`. Only `snap()`
is. Either `reset` is an alias for `snap` and should be declared, or all call
sites using `reset()` must be changed to `snap()`.

Recommendation: Add `void reset(float v) { snap(v); }` to `SpringSolver`
and `void reset(float x, float y) { this->x.snap(x); this->y.snap(y); }`
to `SpringSolver2D`. This satisfies both naming conventions.

### GAP-D: `CockpitView::m_previousIndex` for adjacent desktop rendering during transition

Part 31.5 notes:
> "During transition, adjacent desktop windows must be temporarily visible.
> This requires DesktopManager to expose 'previousIndex' during transition.
> m_previousIndex: set on switchTo(), cleared when spring settles."

`DesktopManager::m_previousIndex` is never declared in `DesktopManager.h`.
`RenderPipeline` needs it to decide which two desktops to render during
a transition. Add:

```cpp
// animus/shell/DesktopManager.h — add:
int  previousIndex() const { return m_previousIndex; }
bool isTransitioning() const {
    return !m_slideOffsetX.isResting();
}

private:
int m_previousIndex = 0;  // set on switchTo(), used by RenderPipeline
```

---

## SUMMARY — Volume 3

After reading Parts 29–32 in full:

**15 confirmed bugs** ranging from compile errors to architectural conflicts.

The most impactful findings:

**`StateManager::get()` null dereference** (FIX3-01) — affects
`DesktopManager` and any other call site that dereferences the pointer
without null-checking. Will produce UB silently, not a compile error.

**`StateManager::screenWidth/Height()` don't exist** (FIX3-02) — used in
`DesktopManager` and `OSFWindow` throw physics. Both files fail to compile.

**`SpringSolver2D::setVelocity()` and `enableEdgeResistanceY()` missing**
(FIX3-03) — Part 29 window throw physics cannot be implemented without these.

**GestureRecognizer never deleted** (FIX3-05) — Part 30 says delete it,
Part 12 still has it. Compilation depends on include order.

**OSFEvent enum missing ~30 events** (FIX3-09) — the most pervasive single
issue in the document. Every subscriber to any event added after Part 22
will silently fail to compile or link.

**CockpitView Part 15 vs Part 29 conflict** (FIX3-10) — the old overlay
surface model and the new zoom model are structurally incompatible.
Part 29 wins. Part 15's `CockpitView.h` is deleted.

**Two conflicting reduced motion implementations** (FIX3-15) — Part 22.5
and Part 38 cannot coexist. Part 38 wins.

---

## Updated Absolute Rules (additions to Vol.1 and Vol.2)

```
26. StateManager::get() returns const std::any* — a nullable pointer.
    NEVER pass it directly to std::any_cast<T>().
    ALWAYS use getAs<T>(key, defaultVal) for typed reads.
    Or: null-check the pointer first, then dereference: *sm.get(key).

27. DesktopManager exposes m_slideOffsetX.value() and
    m_bgSlideOffsetX.value() to RenderPipeline. RenderPipeline NEVER
    stores screen width or height directly — always reads from
    StateKey::ScreenWidth and StateKey::ScreenHeight.

28. CockpitView is a zoom level, not a surface. Never create a
    SurfaceAltitude::High CockpitView overlay. Part 29 is canonical.
    Part 15 CockpitView.h is deleted.

29. The close button on a CockpitView window card sends
    wlr_xdg_toplevel_send_close() — it does NOT call removeSurface()
    directly. The client controls when its surface is destroyed.
    The compositor requests, the client decides.

30. All OSFEvent additions across all parts belong in ONE OSFEvent.h.
    The file has one canonical version. There is no "add to enum" that
    lives anywhere other than that file. Consolidate before building.
```
