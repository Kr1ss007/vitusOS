# AnimusEngine — Architecture Gap Fixes, Volume 4
## vitusOS ARES · Final volume — Addenda A–G, Part 17/18 coverage

**This is the last volume. All 20,130 lines have now been read.**

---

## INDEX OF FIXES

| ID | Bug | Severity | Location |
|----|-----|----------|----------|
| FIX4-01 | `TL_ZOOM = 0xFF28C840` (green) — Part 29 explicitly forbids green | **VISUAL/RULE VIOLATION** | Addendum H |
| FIX4-02 | `AnimationEngine::onSettle()` publishes callback via `OSFEvent::Tick` — bad_any_cast on every active spring subscriber | **CRASH** | Addendum B.2 |
| FIX4-03 | `OSFDesktop::initSubsystems()` subscribes to `CockpitViewToggle` — Part 29 forbids it | **RULE VIOLATION** | Part 17 |
| FIX4-04 | `StateManager::observeState()` passes key string, not new value, to callback | **LOGIC BUG** | Addendum C |
| FIX4-05 | `osf-shell-v1.xml` has two versions with different `menu_activated` arg names | **PROTOCOL CONFLICT** | Part 16 vs Addendum F |
| FIX4-06 | `TextColor::Primary` (#1A1A1A, near-black) used on dark glass shell surfaces — invisible text | **VISUAL BUG** | Addendum E |
| FIX4-07 | `cbPointerMotion` only forwards to Dock, skips CockpitView/menus/notifications | **INPUT BUG** | Part 17 |
| FIX4-08 | `RenderPipeline.h` (Addendum G) still uses `std::vector<std::shared_ptr<Window>>` — conflicts with RegistryManager (Part 27) | **ARCH CONFLICT** | Addendum G |
| FIX4-09 | `InputRouter::onKey()` checks `m_cockpitView->isOpen()` but `m_cockpitView` is never declared | **COMPILE ERROR** | Part 29.8 |
| FIX4-10 | `initSubsystems(1920, 1080)` hardcodes dimensions before output is available | **INIT ORDER BUG** | Part 17 |
| FIX4-11 | `AnimusBoot.c` Part 1 stub calls `gBS->LoadImage(FALSE, Img, NULL, NULL, 0, &KH)` — NULL device path never works | **BOOT FAILURE** | Part 1 vs Addendum A.2 |

---

## FIX4-01 — Traffic Light Green vs Blue

### Problem

Addendum H line 5272:
```cpp
static constexpr uint32_t TL_ZOOM = 0xFF28C840;  // #28C840 — GREEN
```

Part 29.12 explicitly:
```
Maximize: #007AFF   (blue — vitusOS accent, not green)
NEVER use green (#28C840 or similar) for the maximize button.
MAXIMIZE_COLOR = #007AFF. Always.
```

Part 29 is canonical. Addendum H is an earlier draft that was not updated
when Part 29 locked the colors.

### Fix

```cpp
// osf/surfaces/OSFWindow.h — corrected traffic light constants

static constexpr uint32_t TL_CLOSE    = 0xFFFF3B30;  // #FF3B30  red
static constexpr uint32_t TL_MINIMIZE = 0xFFFFCC00;  // #FFCC00  yellow
static constexpr uint32_t TL_ZOOM     = 0xFF007AFF;  // #007AFF  BLUE — not green
static constexpr float    TL_SIZE     = 12.0f;
static constexpr float    TL_SPACING  = 8.0f;
static constexpr float    TL_MARGIN_X = 12.0f;
static constexpr float    TL_MARGIN_Y = 8.0f;
```

Note: The FullscreenTrafficLights spec in Part 40 also defines colors:
```
Close:    #FF3B30
Minimize: #FFCC00
Maximize: #007AFF (exits fullscreen = restore)
```
Consistent with Part 29. Addendum H is the only divergent definition.
Delete the Addendum H constants and use only the Part 29/40 values.

---

## FIX4-02 — `AnimationEngine::onSettle()` Crashes All Tick Subscribers

### Problem

Addendum B.2 line 4620:
```cpp
for (auto& s : fired)
    EventBus::shared().publishAsync(OSFEvent::Tick, s.callback);
    // Note: publishAsync with callback wrapped in std::any
    // Subscriber unwraps and calls. Pattern used by BootCrossfade.
```

`OSFEvent::Tick` carries `float dt` as payload. Every `Tick` subscriber does:
```cpp
float dt = std::any_cast<float>(data);
```

If a settle fires and publishes a `std::function<void()>` via `OSFEvent::Tick`,
every single active Tick subscriber throws `std::bad_any_cast` simultaneously
— the entire compositor crashes.

The comment ("Subscriber unwraps and calls") implies a different subscriber
pattern, but the document never specifies such a subscriber. No component
subscribes to `OSFEvent::Tick` to receive callbacks.

### Fix

Use a dedicated event, not `OSFEvent::Tick`:

```cpp
// animus/core/OSFEvent.h — add:
SpringSettled,   // data = uint64_t settleId — fired when spring reaches rest
                 // AnimationEngine::onSettle() fires this when predicate returns true
```

```cpp
// animus/animation/AnimationEngine.cpp — corrected settle mechanism

void AnimationEngine::tick(float dt) {
    if (!m_running) return;
    EventBus::shared().publish(OSFEvent::Tick, dt);

    // Check settlers
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

    // Fire settle callbacks directly — do NOT route through EventBus::Tick
    // These are one-shot callbacks, called on main thread (we are in tick()).
    for (auto& s : fired) {
        s.callback();  // call directly — already on main thread
    }
}

// AnimationEngine.h — updated onSettle signature:
uint64_t onSettle(std::function<bool()> isSettled,
                  std::function<void()> callback);
void     cancelSettle(uint64_t id);

struct Settler {
    uint64_t               id;
    std::function<bool()>  isSettled;
    std::function<void()>  callback;
};

std::vector<Settler>  m_settlers;
std::mutex            m_settleMu;
uint64_t              m_nextSettleId = 1;
```

Usage by BootCrossfade and CockpitView:
```cpp
// BootCrossfade: fire OSFEvent::BootCrossfadeComplete when opacity settles at 0
AnimationEngine::shared().onSettle(
    [this]{ return m_opacity.isResting() && m_opacity.value() < 0.01f; },
    []{ EventBus::shared().publish(OSFEvent::BootCrossfadeComplete, {}); }
);
```

---

## FIX4-03 — `OSFDesktop::initSubsystems()` Uses Forbidden `CockpitViewToggle`

### Problem

Part 17 `OSFDesktop::initSubsystems()` at line 4109:
```cpp
EventBus::shared().subscribe(OSFEvent::CockpitViewToggle,
    [this](const std::any&) {
        if (m_cockpit->isOpen()) m_cockpit->close();
        else m_cockpit->open(m_wm->windows());
    });
```

Part 29.15 is explicit:
> "NEVER use CockpitViewToggle in new code."
> "CockpitViewToggle remains in the enum but is never published."

Part 17 is the original OSFDesktop written before Part 29.
Part 29 supersedes it. This subscription must be replaced.

### Fix

```cpp
// animus/core/OSFDesktop.cpp — replace CockpitViewToggle subscriptions

// REMOVE:
// EventBus::shared().subscribe(OSFEvent::CockpitViewToggle, ...)

// REPLACE WITH:
EventBus::shared().subscribe(OSFEvent::CockpitViewOpen,
    [this](const std::any &data) {
        AnimusContext ctx = std::any_cast<AnimusContext>(data);
        CockpitView::shared().open(ctx);
    });

EventBus::shared().subscribe(OSFEvent::CockpitViewClose,
    [this](const std::any &data) {
        // data may be AnimusContext or empty
        CockpitView::shared().close();
    });
```

---

## FIX4-04 — `StateManager::observeState()` Passes Wrong Data to Callback

### Problem

Addendum C:
```cpp
uint64_t StateManager::observeState(const std::string &key,
                                     std::function<void(const std::any&)> cb)
{
    return EventBus::shared().subscribe(OSFEvent::StateChanged,
        [key, cb](const std::any &data) {
            auto changedKey = std::any_cast<std::string>(data);
            if (changedKey == key) cb(data);  // ← passes key string, not value
        });
}
```

`data` at the point of `cb(data)` is the changed *key* (a `std::string`),
not the new value for that key. Any callback that tries to read the value
with `std::any_cast<bool>(data)` or similar gets a `std::bad_any_cast`
because it's actually receiving a `std::string`.

### Fix

```cpp
// animus/core/StateManager.cpp — observeState passes the actual new value

uint64_t StateManager::observeState(const std::string &key,
                                     std::function<void(const std::any&)> cb)
{
    return EventBus::shared().subscribe(OSFEvent::StateChanged,
        [this, key, cb](const std::any &data) {
            auto changedKey = std::any_cast<std::string>(data);
            if (changedKey == key) {
                // Fetch the current value for this key and pass it
                const std::any *val = this->get(key);
                if (val) cb(*val);  // pass the VALUE, not the key string
            }
        });
}
```

Usage pattern (now correct):
```cpp
StateManager::shared().observeState("lock_screen_visible",
    [](const std::any &val) {
        bool locked = std::any_cast<bool>(val);  // now works correctly
        if (locked) HEV::shared().onScreenLocked();
    });
```

---

## FIX4-05 — `osf-shell-v1.xml` Protocol Version Conflict

### Problem

Part 16 defines the event as:
```xml
<event name="menu_activated">
    <arg name="menu_item_id" type="string"/>
</event>
```

Addendum F replaces it with:
```xml
<event name="menu_activated">
    <arg name="item_path" type="string"/>  <!-- e.g. "File/Save" -->
</event>
```

Different argument name (`menu_item_id` vs `item_path`). Wayland protocol
scanners generate C code from XML. If the compositor uses Addendum F's XML
(correct) but an OSFNative app was compiled against Part 16's XML (wrong),
the argument names mismatch — the generated proxy/stub code disagrees.
Since these are the same interface version (v1), there's no version negotiation.

**Addendum F is canonical.** Part 16's protocol XML is superseded entirely.

### Fix

There is exactly **one** `osf-shell-v1.xml` in the repository at
`protocol/osf-shell-v1.xml`. It must match Addendum F exactly.
Part 16's XML is documentation of the intermediate state — it is not
the file that ships. Delete the Part 16 version. Use Addendum F.

The complete final protocol XML is Addendum F's version. Additionally,
Addendum F adds `update_menu_item` and `set_shadow_style` requests that
Part 16 does not have. These must be in the final XML.

---

## FIX4-06 — `TextColor::Primary` Is Near-Black On Dark Glass Surfaces

### Problem

Addendum E defines:
```cpp
enum class TextColor { Primary, Secondary, Muted, Accent, OnAccent, OnDark };
static constexpr uint32_t TEXT_COLORS[] = {
    0xFF1A1A1A,  // Primary  — near BLACK
    0xFF808080,  // Secondary
    0xFF3D3D3D,  // Muted    — near BLACK
    0xFFE85D00,  // Accent
    0xFFFFFFFF,  // OnAccent
    0xFFF0F0F0,  // OnDark   — near WHITE
};
```

`TextColor::Primary` (`#1A1A1A`) is near-black. It is correct for text on
`OSFContent` (`#FEFEFE` background). It is **invisible** on dark glass
surfaces — Panel (`SurfaceAltitude::Low`), Dock (`Mid`), sidebars, menus.

Throughout the shell spec, components that render text on dark glass should
use `TextColor::OnDark` (`#F0F0F0`). But nowhere is this mapping explicitly
stated, and some places in the spec (OSFSidebar section headers use
`TextColor::Muted` = `#3D3D3D`) would produce invisible dark text on a
dark glass surface.

Part 29.13 defines the actual on-dark colors:
```
Primary text:   #F2F2F2  (white, 95%)
Secondary text: #ABABAB  (white, 67%)
Tertiary text:  #6B6B6B  (white, 42%)
```

### Fix — Expand TextColor for dark surfaces

```cpp
// animus/render/TextRenderer.h — expanded TextColor

enum class TextColor {
    // ── For use on LIGHT backgrounds (OSFContent #FEFEFE) ──────────
    Primary,        // #1A1A1A — body text on light content area
    Secondary,      // #808080 — supporting text on light content area
    Muted,          // #3D3D3D — subdued labels on light content area

    // ── For use on DARK glass surfaces (Panel, Dock, menus, etc.) ──
    OnDark,         // #F2F2F2 — primary text on dark glass (95% white)
    OnDarkSecondary,// #ABABAB — secondary text on dark glass (67% white)
    OnDarkTertiary, // #6B6B6B — tertiary/sidebar headers on dark glass

    // ── Context-independent ─────────────────────────────────────────
    Accent,         // #E85D00 — Space Orange links and highlights
    OnAccent,       // #FFFFFF — text on Space Orange backgrounds
};

static constexpr uint32_t TEXT_COLORS[] = {
    0xFF1A1A1A,  // Primary
    0xFF808080,  // Secondary
    0xFF3D3D3D,  // Muted
    0xFFF2F2F2,  // OnDark           (was 0xFFF0F0F0 — update to match Part 29.13)
    0xFFABABAB,  // OnDarkSecondary  (NEW)
    0xFF6B6B6B,  // OnDarkTertiary   (NEW)
    0xFFE85D00,  // Accent
    0xFFFFFFFF,  // OnAccent
};
```

### Usage Rule — New Absolute Rule

```
31. Text on any glass surface (Panel, Dock, Sidebar, menus, Pathfinder,
    notifications, OrangeBoxMenu) MUST use TextColor::OnDark,
    TextColor::OnDarkSecondary, or TextColor::OnDarkTertiary.
    TextColor::Primary/Secondary/Muted are ONLY for OSFContent
    (the #FEFEFE light background content area).
    Never use near-black text colors on dark glass. It is invisible.
```

### Specific corrections:

```
OSFSidebar section headers:     TextColor::Muted     → TextColor::OnDarkTertiary
Panel clock:                     TextColor::OnDark    ✓ (already correct)
Panel app name / GlobalMenu:     TextColor::OnDark    ✓
Dock tooltip text:               TextColor::OnDark    ✓
OrangeBoxMenu items:             TextColor::OnDark    (verify in implementation)
Notification title:              TextColor::OnDark    (verify in implementation)
Notification body:               TextColor::OnDarkSecondary
CockpitView window labels:       TextColor::OnDark    (Part 29 confirms white text)
```

---

## FIX4-07 — `cbPointerMotion` Only Forwards to Dock

### Problem

Part 17 `OSFDesktop::cbPointerMotion()`:
```cpp
void OSFDesktop::cbPointerMotion(double x, double y, void *ud) {
    InputRouter::shared().onPointerMotion(x, y);
    auto *self = static_cast<OSFDesktop*>(ud);
    if (self->m_dock) self->m_dock->onPointerMotion((float)x, (float)y);
    if (self->m_cockpit && self->m_cockpit->isOpen())
        ; // handled by cockpit directly — DOES NOTHING
}
```

Only the Dock receives pointer motion. Missing:
- `CockpitView` (close button hover, window card hover)
- `OrangeBoxMenu` (item hover)
- `GlobalMenu` (top bar hover, submenu hover)
- All active `OSFNotification` instances (hover to pause dismiss)
- `OSFContextMenu` (item hover)
- `OSFTooltip` (dwell timer update)
- `DragManager` (cursor tracking during drag)

None of these work correctly without pointer motion. Hover springs never fire.
Context menus are non-interactive. Tooltips never appear.

### Fix — Route all pointer motion through InputRouter

```cpp
// animus/input/InputRouter.cpp — onPointerMotion is the single dispatch point

void InputRouter::onPointerMotion(double x, double y) {
    m_px = x;
    m_py = y;
    float fx = (float)x;
    float fy = (float)y;

    // Update PowerManager idle timer
    PowerManager::shared().onInputEvent();

    // Route to all interactive shell surfaces
    // Each component checks if it's relevant (hit test or always-on)

    // Dock: magnify effect
    Dock::shared().onPointerMotion(fx, fy);

    // Panel: traffic light hover, orange box hover, global menu hover
    PanelManager::shared().onPointerMotion(fx, fy);

    // CockpitView: window card hover, close button hover, sidebar hover
    if (CockpitView::shared().isOpen())
        CockpitView::shared().onPointerMotion(fx, fy);

    // OrangeBoxMenu: item hover
    if (OrangeBoxMenu::shared().isOpen())
        OrangeBoxMenu::shared().onPointerMotion(fx, fy);

    // Drag ghost tracking
    if (DragManager::shared().isDragging())
        DragManager::shared().onCursorMove(fx, fy);

    // Active overlays (notifications, context menus, tooltips)
    // These are in RegistryManager::notifications()
    RegistryManager::shared().notifications().forEach(
        [fx, fy](RegHandle, OSFNotification *n) {
            if (n->isVisible()) n->onPointerMotion(fx, fy);
        });

    // Active context menu (at most one at a time)
    // Stored in WindowManager or OSFDesktop as m_activeContextMenu
    if (m_activeContextMenu)
        m_activeContextMenu->onPointerMotion(fx, fy);

    // Tooltip dwell update
    OSFTooltip::shared().update(AnimationClock::shared().dt(), fx, fy);

    // Publish for any other subscriber
    EventBus::shared().publish(OSFEvent::MouseMoved,
        std::make_pair(fx, fy));
}
```

```cpp
// animus/core/OSFDesktop.cpp — cbPointerMotion is now a thin forwarder

void OSFDesktop::cbPointerMotion(double x, double y, void *ud) {
    // ALL routing happens inside InputRouter — not here.
    InputRouter::shared().onPointerMotion(x, y);
}
```

---

## FIX4-08 — `RenderPipeline.h` (Addendum G) Conflicts with RegistryManager

### Problem

Addendum G's `RenderPipeline.h`:
```cpp
std::vector<std::shared_ptr<Window>> m_windows;
std::vector<class Surface*>          m_overlays;
```

Part 27 `RegistryManager` was written specifically to eliminate this pattern.
The `Vol.2 FIX2-04` replaced these with `RegistryManager::forEach()` calls.
But Addendum G, which is an authoritative document section, still has the old
API including `addWindow(std::shared_ptr<Window>)` and `removeWindow()`.

These methods must not exist in the final `RenderPipeline`.

### Fix

```cpp
// animus/render/RenderPipeline.h — authoritative final version
// REMOVE: addWindow(), removeWindow(), addOverlay(), removeOverlay()
// REMOVE: m_windows vector
// REMOVE: m_overlays vector
// Windows and notifications accessed via RegistryManager in renderFrame()

class RenderPipeline {
public:
    RenderPipeline() = default;
    bool initialize();
    void destroy();
    void renderFrame(float dt);

    void setOutput(struct wlr_output *output);
    void setPanel(Panel *p)              { m_panel = p; }
    void setDock(Dock *d)                { m_dock = d; }
    void setCrossfade(BootCrossfade *c)  { m_crossfade = c; }
    void setWallpaperView(VkImageView v) { m_wallpaperView = v; }

    MaterialRenderer* material() const { return m_material.get(); }
    ShadowRenderer*   shadow()   const { return m_shadow.get(); }
    TextRenderer*     text()     const { return m_text.get(); }
    VulkanContext*    vk()       const { return m_ctx.get(); }

private:
    std::unique_ptr<VulkanContext>     m_ctx;
    std::unique_ptr<MaterialRenderer>  m_material;
    std::unique_ptr<ShadowRenderer>    m_shadow;
    std::unique_ptr<GlyphAtlas>        m_atlas;
    std::unique_ptr<TextRenderer>      m_text;

    struct wlr_output *m_output       = nullptr;
    VkImageView        m_wallpaperView = VK_NULL_HANDLE;
    Panel             *m_panel         = nullptr;
    Dock              *m_dock          = nullptr;
    BootCrossfade     *m_crossfade     = nullptr;

    // NO m_windows. NO m_overlays.
    // Windows: RegistryManager::shared().windows().forEach()
    // Notifications: RegistryManager::shared().notifications().forEach()
};
```

Addendum G's `WindowManager.h` also has the same conflict:
```cpp
// REMOVE from WindowManager.h:
// const std::vector<std::shared_ptr<OSFWindow>>& windows() const
// OSFWindow* focused() const { return m_focused; }
// std::vector<std::shared_ptr<OSFWindow>> m_windows;
// OSFWindow *m_focused;

// REPLACE WITH:
// WindowManager holds unique_ptr ownership only.
// All read access goes through RegistryManager.
// focused() → RegistryManager::shared().windows().focusedWindow()
```

---

## FIX4-09 — `InputRouter::onKey()` References Undeclared `m_cockpitView`

### Problem

Part 29.8 `InputRouter::onKey()`:
```cpp
if (!m_cockpitView->isOpen()) {
    // ...
    EventBus::shared().publish(OSFEvent::CockpitViewOpen, ctx);
} else {
    EventBus::shared().publish(OSFEvent::CockpitViewClose, ...);
}
```

`InputRouter` has no `m_cockpitView` member. `CockpitView` is a singleton
(`CockpitView::shared()`). `InputRouter` should not hold a pointer to it.

### Fix

```cpp
// animus/input/InputRouter.cpp — use CockpitView::shared() directly

void InputRouter::onKey(uint32_t keysym, uint32_t mods, bool pressed) {
    if ((mods & MOD_ALT) && keysym == XKB_KEY_Tab && pressed) {
        // Use the singleton — no member pointer needed
        if (!CockpitView::shared().isOpen()) {
            float cx = m_screenW * 0.5f;
            float cy = m_screenH * 0.5f;
            auto focused = RegistryManager::shared().windows().focusedWindow();
            if (focused) {
                cx = focused->posX() + focused->width()  * 0.5f;
                cy = focused->posY() + focused->height() * 0.5f;
            }
            AnimusContext ctx = AnimusContext::fromKeyboardShortcut(cx, cy);
            EventBus::shared().publish(OSFEvent::CockpitViewOpen, ctx);
        } else {
            EventBus::shared().publish(OSFEvent::CockpitViewClose,
                                        AnimusContext::none());
        }
        return;
    }
    // ... rest of onKey unchanged, replacing m_cockpitView with CockpitView::shared()
}
```

---

## FIX4-10 — `initSubsystems(1920, 1080)` Hardcoded Before Output Available

### Problem

Part 17 `OSFDesktop::run()`:
```cpp
int OSFDesktop::run() {
    if (animus_compositor_init() < 0) return 1;
    animus_compositor_register_callbacks(...);
    initSubsystems(1920, 1080);  // hardcoded dimensions
    // ...
}
```

The output dimensions are not known at this point.
`animus_compositor_init()` starts the backend but `h_new_output` hasn't
fired yet — `g.primary_output` is still NULL. The VulkanContext is
initialized with wrong dimensions that "OutputResized event corrects"
— but `OutputResized` is only published if someone subscribes and
publishes it, which nothing does in the current spec.

### Fix

```cpp
// animus/core/OSFDesktop.cpp — defer subsystem init to first output

int OSFDesktop::run() {
    if (animus_compositor_init() < 0) return 1;

    animus_compositor_register_callbacks(
        cbPresent, cbNewSurface, cbSurfaceDestroy,
        cbKey, cbPointerMotion, cbPointerButton, cbPointerAxis,
        cbSwipeBegin, cbSwipeUpdate, cbSwipeEnd,
        this
    );

    // DO NOT call initSubsystems() here.
    // initSubsystems() requires a valid wlr_output for correct dimensions.
    // It is called from cbOutputAdded() when the first output connects.
    // This is guaranteed to happen before animus_compositor_run() returns.

    AnimationEngine::shared().start();
    SoundEngine::shared().initialize();

    animus_compositor_run();  // blocks; cbOutputAdded fires before first frame
    return 0;
}

// In cbOutputAdded — called from h_new_output in C11 compositor:
void OSFDesktop::cbOutputAdded(struct wlr_output *output,
                                bool isPrimary, void *ud)
{
    auto *self = static_cast<OSFDesktop*>(ud);
    if (isPrimary && !self->m_initialized) {
        // Write screen dimensions to StateManager immediately
        StateManager::shared().set(StateKey::ScreenWidth,
            static_cast<float>(output->width));
        StateManager::shared().set(StateKey::ScreenHeight,
            static_cast<float>(output->height));

        self->initSubsystems((uint32_t)output->width,
                             (uint32_t)output->height);
    }
    PanelManager::shared().onOutputAdded(output, isPrimary);
}
```

---

## FIX4-11 — `AnimusBoot.c` Stub `gBS->LoadImage` with NULL Device Path

### Problem

Part 1 `AnimusBoot.c`:
```c
EFI_HANDLE KH;
// Real path: L"\\EFI\\vitusos\\kernel" — populated by NixOS install
gBS->LoadImage(FALSE, Img, NULL, NULL, 0, &KH);
```

Passing `NULL` for both the device path and file buffer to `LoadImage` does
nothing useful — UEFI has no way to locate the kernel. This always returns
`EFI_INVALID_PARAMETER` or `EFI_NOT_FOUND`. The boot fails silently with
the compositor at "AnimusBoot loaded" and never reaching the kernel.

Addendum A.2 provides the correct implementation using
`EFI_SIMPLE_FILE_SYSTEM_PROTOCOL` to locate `\EFI\vitusos\bzImage`.

### Fix — Part 1 stub is deleted, Addendum A.2 is canonical

Addendum A.2 provides `LoadKernelFromFilesystem()`. The `AnimusBoot.c`
entry point must call it:

```c
// AnimusBoot/AnimusBoot.c — corrected UefiMain

EFI_STATUS EFIAPI UefiMain(EFI_HANDLE Img, EFI_SYSTEM_TABLE *ST) {
    ANIMUS_GPU_HANDOFF H = {0};
    if (EFI_ERROR(DetectGpu(&H)))         return EFI_NOT_FOUND;
    if (EFI_ERROR(SetupGopAndRender(&H))) return EFI_DEVICE_ERROR;

    // Write GPU handoff to EFI variable
    gRT->SetVariable(HANDOFF_VAR, &HandoffGuid,
        EFI_VARIABLE_BOOTSERVICE_ACCESS |
        EFI_VARIABLE_RUNTIME_ACCESS     |
        EFI_VARIABLE_NON_VOLATILE,
        sizeof(H), &H);

    // Load kernel via filesystem (Addendum A.2 — not NULL device path)
    EFI_HANDLE KH;
    EFI_STATUS S = LoadKernelFromFilesystem(Img, &KH);
    if (EFI_ERROR(S)) {
        AsciiPrint("AnimusBoot: kernel not found at \\EFI\\vitusos\\bzImage: %r\n", S);
        return S;
    }

    // Set kernel command line
    EFI_LOADED_IMAGE_PROTOCOL *Li;
    gBS->HandleProtocol(KH, &gEfiLoadedImageProtocolGuid, (VOID**)&Li);
    Li->LoadOptions     = (VOID*)CMDLINE;
    Li->LoadOptionsSize = (UINT32)((StrLen(CMDLINE)+1)*sizeof(CHAR16));

    UINTN ExSz; CHAR16 *Ex;
    return gBS->StartImage(KH, &ExSz, &Ex);
}
```

Also: the wordmark placeholder `FillRect(Fb, St, (W-280)/2, (Ht-48)/2, 280, 48, White)`
from Part 1 must be replaced with `RenderWordmark()` from Addendum A.1.
The placeholder renders a white rectangle, not the actual "vitusos" wordmark.

---

## COMPLETE ISSUES INDEX — ALL FOUR VOLUMES

### Critical (compile error or boot failure)
- FIX-01: VulkanContext → DRM buffer path (Vol.1) — blank screen
- FIX-07: Vulkan fallback broken (Vol.1) — GPU fault
- FIX2-01: WindowRestored duplicate (Vol.2) — compile error
- FIX2-03: ShaderCache missing member (Vol.2) — compile error
- FIX3-02: StateManager::screenWidth() doesn't exist (Vol.3) — compile error
- FIX3-03: SpringSolver2D::setVelocity() missing (Vol.3) — compile error
- FIX3-05: GestureRecognizer not deleted (Vol.3) — compile error
- FIX3-06: animus_compositor_get_backend() not declared (Vol.3) — compile error
- FIX3-09: OSFEvent enum missing ~30 events (Vol.3) — compile errors everywhere
- FIX3-10: CockpitView Part 15 vs Part 29 conflict (Vol.3) — arch conflict
- FIX4-02: AnimationEngine::onSettle() crashes all Tick subscribers (Vol.4) — crash
- FIX4-09: InputRouter m_cockpitView not declared (Vol.4) — compile error
- FIX4-11: AnimusBoot NULL kernel load (Vol.4) — boot failure

### Security
- FIX-02: WelcomeScreen passphrase not zeroed (Vol.1)
- Rule 22: sodium_memzero before clear() for all credentials

### Crash (runtime)
- FIX-03: Battery notification bad_any_cast (Vol.1)
- FIX-05: CockpitView sound on frame 1 (Vol.1)
- FIX2-07: CrashSite wrong notification type (Vol.2)
- FIX3-01: StateManager::get() null dereference (Vol.3)
- FIX3-04: RenderPipeline raw pointer iteration (Vol.3) — use-after-free

### Architecture conflicts
- FIX3-15: Two conflicting reduced motion implementations (Vol.3) — Part 22.5 vs Part 38
- FIX4-01: TL_ZOOM green (Addendum H) vs blue (Part 29) — Part 29 wins
- FIX4-03: CockpitViewToggle subscription — Part 29 forbids it
- FIX4-05: osf-shell-v1.xml two versions — Addendum F wins
- FIX4-08: RenderPipeline raw windows vector vs RegistryManager — Part 27 wins

### Logic bugs
- FIX-04: DragManager compositor wiring missing (Vol.1)
- FIX-06: onSetFullscreen float→uint32 (Vol.1)
- FIX2-02: pw_stream_new_simple rule ambiguity (Vol.2)
- FIX2-04: RegistryManager not used in RenderPipeline (Vol.2)
- FIX2-05: VulkanContext::commitFrame() no wlr_output* (Vol.2)
- FIX2-06: Two-process model not reflected in init order (Vol.2)
- FIX3-07: MotionWave::initialize() never called (Vol.3)
- FIX3-08: DesktopManager missing from Vessels DAG (Vol.3)
- FIX3-12: DesktopManager spring reset mid-transition (Vol.3)
- FIX3-13: MotionWave tap timing never set (Vol.3)
- FIX3-14: CockpitView close button bypasses wlr_xdg_toplevel_send_close (Vol.3)
- FIX4-04: StateManager::observeState() passes key not value (Vol.4)
- FIX4-06: TextColor::Primary invisible on dark glass (Vol.4)
- FIX4-07: cbPointerMotion only forwards to Dock (Vol.4)
- FIX4-10: initSubsystems(1920,1080) hardcoded (Vol.4)

### Gaps / Missing specification
- FIX3-11: OSFBridge wire protocol unspecified (Vol.3)
- GAP-A: ConfigWriter never specced (Vol.3)
- GAP-B: StateManager::get() usage patterns inconsistent (Vol.3)
- GAP-C: SpringSolver::reset() vs snap() naming (Vol.3)
- GAP-D: DesktopManager::m_previousIndex not declared (Vol.3)

---

**Total confirmed issues across all volumes: 46**
**All 20,130 lines of the document have been read.**
