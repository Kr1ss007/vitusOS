//! ShellController — production integration layer that wires the EventBus
//! to all shell components, manages interaction state, and routes input
//! events to the correct component based on the current shell state.
//!
//! This is the SINGLE point where EventBus events are dispatched to shell
//! components. No component subscribes to the EventBus directly — the
//! ShellController receives events and calls the appropriate methods.
//!
//! State machine priority (highest wins):
//!   1. ShutdownScreen / SystemScreen — blocks everything
//!   2. LockScreen — blocks all input except unlock
//!   3. WelcomeScreen — blocks shell from appearing
//!   4. CockpitView (open) — intercepts pointer, routes to cards
//!   5. ControlCenter (open) — intercepts pointer
//!   6. GlobalMenu (active) — intercepts keyboard
//!   7. Normal desktop — Panel, Dock, windows

use crate::shell::{
    CockpitView, CockpitCard,
    ControlCenter,
    Dock,
    LockScreen,
    NotificationCenter,
    PanelManager,
    PowerAction,
    SystemScreenMode,
    WelcomeScreen,
    ShutdownScreen,
    SystemScreen,
};
use crate::window_manager::WindowManager;
use crate::sound_manager::SoundManager;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;

/// The current shell interaction mode — determines which component
/// receives input events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMode {
    /// Normal desktop operation — windows, panel, dock all interactive.
    Desktop,
    /// CockpitView is open — cards receive pointer, Esc closes.
    CockpitView,
    /// Lock screen is active — only password input matters.
    LockScreen,
    /// Welcome screen (first boot) — blocks shell.
    WelcomeScreen,
    /// Control center popover is open.
    ControlCenter,
    /// Shutdown/restart screen is showing.
    ShutdownScreen,
    /// System screen (blackout/sleep transition).
    SystemScreen,
}

/// The shell controller — owns all shell components and routes events.
pub struct ShellController {
    pub mode: ShellMode,
    pub panel_manager: PanelManager,
    pub dock: Dock,
    pub cockpit_view: CockpitView,
    pub lock_screen: LockScreen,
    pub welcome_screen: WelcomeScreen,
    pub control_center: ControlCenter,
    pub notification_center: NotificationCenter,
    pub shutdown_screen: ShutdownScreen,
    pub system_screen: SystemScreen,
    pub window_manager: WindowManager,
    pub sounds: SoundManager,

    /// Current screen dimensions.
    screen_w: f32,
    screen_h: f32,
    /// Panel height (28px).
    panel_h: f32,
    /// Whether first boot is complete.
    first_boot_done: bool,

    #[allow(dead_code)]
    bus: EventBus,
}

impl ShellController {
    pub fn new(bus: EventBus, screen_w: f32, screen_h: f32) -> Self {
        Self {
            mode: ShellMode::WelcomeScreen,
            panel_manager: PanelManager::new(),
            dock: Dock::new(),
            cockpit_view: CockpitView::new(bus.clone()),
            lock_screen: LockScreen::new(bus.clone()),
            welcome_screen: WelcomeScreen::new(bus.clone()),
            control_center: ControlCenter::new(bus.clone()),
            notification_center: NotificationCenter::new(bus.clone()),
            shutdown_screen: ShutdownScreen::new(bus.clone()),
            system_screen: SystemScreen::new(bus.clone()),
            window_manager: WindowManager::new(screen_w, screen_h, 28.0),
            sounds: SoundManager::new(),
            screen_w,
            screen_h,
            panel_h: 28.0,
            first_boot_done: false,
            bus,
        }
    }

    /// Set screen geometry (on output change/add).
    pub fn set_screen_geometry(&mut self, w: f32, h: f32) {
        self.screen_w = w;
        self.screen_h = h;
        self.window_manager.set_screen_geometry(w, h, self.panel_h);
    }

    /// Mark first boot as complete — transitions from WelcomeScreen to Desktop.
    pub fn complete_first_boot(&mut self) {
        self.first_boot_done = true;
        self.mode = ShellMode::Desktop;
        self.sounds.play_boot_chime();
    }

    /// Returns the current shell mode.
    pub fn mode(&self) -> ShellMode {
        self.mode
    }

    // -- Event Dispatch --

    /// Handle an AEEvent by routing it to the appropriate shell component.
    /// This is the main integration point — called from the compositor tick loop.
    pub fn dispatch_event(&mut self, event: &AEEvent) {
        match event {
            // -- Boot / Lifecycle --
            AEEvent::BootCrossfadeComplete => {
                if !self.first_boot_done {
                    self.welcome_screen.activate();
                    self.mode = ShellMode::WelcomeScreen;
                } else {
                    self.mode = ShellMode::Desktop;
                }
            }
            AEEvent::WelcomeScreenCompleted => {
                self.complete_first_boot();
            }
            AEEvent::ShutdownRequested => {
                self.mode = ShellMode::ShutdownScreen;
                self.shutdown_screen.open(PowerAction::Shutdown);
            }
            AEEvent::SystemShutdown => {
                self.mode = ShellMode::ShutdownScreen;
                self.shutdown_screen.open(PowerAction::Shutdown);
            }
            AEEvent::SystemRestart => {
                self.mode = ShellMode::ShutdownScreen;
                self.shutdown_screen.open(PowerAction::Restart);
            }
            AEEvent::SystemSleep => {
                self.mode = ShellMode::SystemScreen;
                self.system_screen.show(SystemScreenMode::Shutdown);
            }
            AEEvent::DisplaySleep => {
                self.mode = ShellMode::SystemScreen;
                self.system_screen.show(SystemScreenMode::Shutdown);
            }
            AEEvent::DisplayWake => {
                // system_screen doesn't have a wake method — just transition mode
                if self.mode == ShellMode::SystemScreen {
                    self.mode = ShellMode::Desktop;
                }
            }

            // -- Lock Screen --
            AEEvent::LockScreenActivate | AEEvent::LidClosed => {
                self.lock_screen.activate();
                self.mode = ShellMode::LockScreen;
                self.sounds.play_lock_screen();
            }
            AEEvent::LockScreenUnlocked => {
                self.lock_screen.deactivate();
                self.mode = ShellMode::Desktop;
                self.sounds.play_unlock_screen();
            }

            // -- CockpitView --
            AEEvent::CockpitViewOpen { ctx: _ } => {
                if self.mode == ShellMode::Desktop {
                    self.open_cockpit_view();
                }
            }
            AEEvent::CockpitViewClose => {
                if self.mode == ShellMode::CockpitView {
                    self.cockpit_view.close();
                    self.mode = ShellMode::Desktop;
                }
            }
            AEEvent::CockpitViewOpened => {
                self.mode = ShellMode::CockpitView;
                self.sounds.play_cockpit_open();
            }
            AEEvent::CockpitViewClosed => {
                if self.mode == ShellMode::CockpitView {
                    self.mode = ShellMode::Desktop;
                }
            }

            // -- Global Menu --
            AEEvent::GlobalMenuActivated => {
                if let Some(panel) = self.panel_manager.focused_panel_mut() {
                    panel.global_menu.activate();
                }
            }
            AEEvent::GlobalMenuDeactivated => {
                if let Some(panel) = self.panel_manager.focused_panel_mut() {
                    panel.global_menu.deactivate();
                }
            }
            AEEvent::DBusMenuRegistered { app_id, menu_json: _ } => {
                // Parse menu JSON and route to focused panel's GlobalMenu
                // In production, this would parse the dbusmenu JSON
                tracing::info!("ShellController: D-Bus menu registered for {}", app_id);
            }

            // -- Window Management --
            AEEvent::WindowOpened { handle, app_id } => {
                tracing::info!("ShellController: Window opened handle={} app={}", handle, app_id);
                self.sounds.play_window_open();
            }
            AEEvent::WindowClosed { handle } => {
                self.window_manager.remove_window(*handle);
            }
            AEEvent::WindowFocused { handle, app_id } => {
                self.window_manager.focus(*handle);
                if let Some(entry) = self.panel_manager.focused_panel_mut() {
                    let name = app_id.clone();
                    entry.global_menu.set_app_name_only(name);
                }
            }
            AEEvent::WindowMinimized { handle } => {
                if let Some(win) = self.window_manager.get_window_mut(*handle) {
                    let dock_x = self.screen_w * 0.5;
                    let dock_y = self.screen_h - Dock::HEIGHT;
                    win.minimize(dock_x, dock_y);
                }
                self.sounds.play_app_close();
            }
            AEEvent::WindowDeminimized { handle } => {
                if let Some(win) = self.window_manager.get_window_mut(*handle) {
                    win.restore();
                }
                self.sounds.play_app_launch();
            }
            AEEvent::FullscreenEntered { handle } => {
                if let Some(win) = self.window_manager.get_window_mut(*handle) {
                    win.enter_fullscreen(self.screen_w, self.screen_h);
                }
                self.panel_manager.panels.iter_mut().for_each(|e| {
                    e.panel.enter_fullscreen_mode();
                });
                self.dock.enter_fullscreen_mode();
            }
            AEEvent::FullscreenExited { handle } => {
                if let Some(win) = self.window_manager.get_window_mut(*handle) {
                    win.exit_fullscreen();
                }
                self.panel_manager.panels.iter_mut().for_each(|e| {
                    e.panel.exit_fullscreen_mode();
                });
                self.dock.exit_fullscreen_mode();
            }

            // -- Desktop / Workspace --
            AEEvent::DesktopNext => {
                self.sounds.play_desktop_switch();
            }
            AEEvent::DesktopPrev => {
                self.sounds.play_desktop_switch();
            }
            AEEvent::ShowDesktopToggle => {
                let any_visible = self.window_manager.visible_window_count() > 0;
                if any_visible {
                    let dock_x = self.screen_w * 0.5;
                    let dock_y = self.screen_h - Dock::HEIGHT;
                    self.window_manager.minimize_all(dock_x, dock_y);
                } else {
                    self.window_manager.restore_all();
                }
            }

            // -- Sound --
            AEEvent::VolumeChanged { volume, muted } => {
                self.sounds.set_master_volume(if *muted { 0.0 } else { *volume });
            }
            AEEvent::BrightnessChanged { brightness } => {
                tracing::info!("ShellController: Brightness set to {:.0}%", brightness * 100.0);
            }

            // -- Notifications --
            AEEvent::NotificationPosted(payload) => {
                self.notification_center.post(payload.clone());
                self.sounds.play_notification();
            }
            AEEvent::NotificationDismissed { id } => {
                self.notification_center.dismiss(*id);
            }

            // -- Accessibility --
            AEEvent::ReducedMotionChanged { enabled } => {
                animus_physics::set_reduced_motion(*enabled);
                tracing::info!("ShellController: Reduced motion {}", if *enabled { "enabled" } else { "disabled" });
            }

            // -- Clipboard --
            AEEvent::ClipboardChanged => {
                tracing::debug!("ShellController: Clipboard changed");
            }

            _ => {}
        }
    }

    // -- Input Routing --

    /// Route a pointer motion event to the correct shell component.
    /// Called from the compositor tick loop when InputRouter fires MouseMoved.
    pub fn on_pointer_motion(&mut self, x: f32, y: f32) {
        match self.mode {
            ShellMode::ShutdownScreen | ShellMode::SystemScreen => return,
            ShellMode::LockScreen => return,
            ShellMode::WelcomeScreen => return,
            ShellMode::CockpitView => {
                self.cockpit_view.on_pointer_motion(x, y);
                return;
            }
            ShellMode::ControlCenter => {
                // Control center handles its own pointer
                return;
            }
            ShellMode::Desktop => {}
        }

        // Panel hot zone (fullscreen auto-hide)
        let _panel_w = self.screen_w;
        for entry in &mut self.panel_manager.panels {
            entry.panel.on_pointer_motion(y);
        }

        // Dock magnification — Gaussian magnification based on cursor X
        let dock_y = self.screen_h - Dock::HEIGHT;
        let dock_x = (self.screen_w - self.dock.items.len() as f32 * 56.0 - 32.0) * 0.5;
        if y > dock_y - 20.0 {
            self.dock.handle_pointer_motion(x, dock_x);
        } else {
            self.dock.reset_magnification();
        }

        // Dock auto-hide hot zone (fullscreen)
        self.dock.on_pointer_motion(x, y, self.screen_h);

        // GlobalMenu pointer motion
        if let Some(entry) = self.panel_manager.focused_panel_mut() {
            if entry.global_menu.is_active() {
                entry.global_menu.on_pointer_motion(x, y, self.screen_w, self.panel_h);
            }
        }
    }

    /// Route a pointer button event. Returns true if consumed.
    pub fn on_pointer_button(&mut self, _button: u32, x: f32, y: f32, pressed: bool) -> bool {
        match self.mode {
            ShellMode::ShutdownScreen | ShellMode::SystemScreen => return false,
            ShellMode::LockScreen => {
                // Lock screen doesn't respond to pointer clicks
                return false;
            }
            ShellMode::WelcomeScreen => return false,
            ShellMode::CockpitView => {
                if pressed {
                    let handle = self.cockpit_view.on_pointer_button(x, y, true);
                    if let Some(h) = handle {
                        self.window_manager.focus(h);
                        self.mode = ShellMode::Desktop;
                    }
                    return true;
                }
                return false;
            }
            ShellMode::ControlCenter => {
                self.control_center.update(0.0);
                return true;
            }
            ShellMode::Desktop => {}
        }

        let mut consumed = false;

        // GlobalMenu click
        if let Some(entry) = self.panel_manager.focused_panel_mut() {
            if entry.global_menu.is_active() {
                let panel_w = self.screen_w;
                consumed |= entry.global_menu.on_pointer_button(x, y, pressed, panel_w, self.panel_h);
                if consumed {
                    return true;
                }
            }
        }

        // Panel click (orange box, traffic lights, tray)
        if y < self.panel_h + 4.0 {
            // Panel area
            consumed = true;
            return consumed;
        }

        // Dock click
        let dock_y = self.screen_h - Dock::HEIGHT;
        if y > dock_y && pressed {
            let dock_x = (self.screen_w - self.dock.items.len() as f32 * 56.0 - 32.0) * 0.5;
            let item_width = 56.0f32;
            for (i, _item) in self.dock.items.iter().enumerate() {
                let item_x = dock_x + 16.0 + i as f32 * item_width;
                if x >= item_x && x <= item_x + item_width {
                    self.dock.launch_item(i);
                    return true;
                }
            }
        }

        // Window click — focus the window under the cursor
        if pressed {
            for win in self.window_manager.windows() {
                if win.is_renderable() && win.pos.x.value <= x
                    && x <= win.pos.x.value + win.width
                    && win.pos.y.value <= y
                    && y <= win.pos.y.value + win.height
                {
                    self.window_manager.focus(win.handle);
                    return true;
                }
            }
        }

        consumed
    }

    /// Route a keyboard event. Returns true if consumed.
    pub fn on_key(&mut self, keycode: u32, modifiers: u32, pressed: bool) -> bool {
        match self.mode {
            ShellMode::ShutdownScreen | ShellMode::SystemScreen => return false,
            ShellMode::LockScreen => {
                // Route key to lock screen password input
                if pressed {
                    match keycode {
                        14 => { // Backspace
                            self.lock_screen.backspace();
                            return true;
                        }
                        28 => { // Enter
                            self.lock_screen.submit_password();
                            return true;
                        }
                        _ => {
                            // Character input handled by compositor text input
                            return false;
                        }
                    }
                }
                return false;
            }
            ShellMode::WelcomeScreen => return false,
            ShellMode::CockpitView => {
                if pressed && keycode == 1 { // Esc
                    self.cockpit_view.close();
                    self.mode = ShellMode::Desktop;
                    return true;
                }
                return false;
            }
            ShellMode::Desktop => {}
            ShellMode::ControlCenter => {
                if pressed && keycode == 1 { // Esc
                    self.control_center.toggle();
                    self.mode = ShellMode::Desktop;
                    return true;
                }
                return false;
            }
        }

        if !pressed {
            return false;
        }

        // Alt+Tab — cycle window focus
        if keycode == 15 && (modifiers & 0x08 != 0) { // Tab + Alt
            self.window_manager.cycle_focus_next();
            return true;
        }

        // F10 — activate GlobalMenu
        if keycode == 68 {
            if let Some(entry) = self.panel_manager.focused_panel_mut() {
                if entry.global_menu.is_active() {
                    entry.global_menu.deactivate();
                } else {
                    entry.global_menu.activate();
                }
            }
            return true;
        }

        // Esc — close CockpitView or exit fullscreen
        if keycode == 1 {
            if self.cockpit_view.is_open {
                self.cockpit_view.close();
                self.mode = ShellMode::Desktop;
                return true;
            }
            // Exit fullscreen on focused window
            if let Some(win) = self.window_manager.focused_mut() {
                if win.is_fullscreen() {
                    let _handle = win.handle;
                    win.exit_fullscreen();
                    self.panel_manager.panels.iter_mut().for_each(|e| {
                        e.panel.exit_fullscreen_mode();
                    });
                    self.dock.exit_fullscreen_mode();
                    return true;
                }
            }
        }

        // Show Desktop toggle (Super+D or F11)
        if keycode == 41 && (modifiers & 0x40 != 0) { // F11 + Super
            let any_visible = self.window_manager.visible_window_count() > 0;
            if any_visible {
                let dock_x = self.screen_w * 0.5;
                let dock_y = self.screen_h - Dock::HEIGHT;
                self.window_manager.minimize_all(dock_x, dock_y);
            } else {
                self.window_manager.restore_all();
            }
            return true;
        }

        // Alt (alone) — GlobalMenu activation
        if keycode == 56 && (modifiers & 0x08 != 0) {
            // Check if Alt is pressed alone (no other key)
            // This is handled by the input router with a timer
            // For now, just pass through
        }

        false
    }

    /// Open CockpitView — captures window positions and creates cards.
    fn open_cockpit_view(&mut self) {
        let cards: Vec<CockpitCard> = self.window_manager.windows()
            .iter()
            .filter(|w| w.is_renderable())
            .map(|w| CockpitCard::new(
                w.handle,
                w.title.clone(),
                w.app_id.clone(),
                w.pos.x.value,
                w.pos.y.value,
                w.width,
                w.height,
            ))
            .collect();
        self.cockpit_view.set_cards(cards);
        self.cockpit_view.open(None, self.screen_w, self.screen_h);
    }

    // -- Update Loop --

    /// Update all shell components by dt seconds.
    /// Called from the compositor tick loop every frame.
    pub fn update(&mut self, dt: f32) {
        self.panel_manager.tick(dt);
        self.dock.update(dt);
        self.cockpit_view.update(dt);
        self.lock_screen.update(dt);
        self.welcome_screen.update(dt);
        self.control_center.update(dt);
        self.notification_center.update(dt);
        self.shutdown_screen.update(dt);
        self.system_screen.update(dt);
        self.window_manager.update(dt);
    }

    /// Returns true if the shell is in a state that blocks window interaction.
    pub fn blocks_window_input(&self) -> bool {
        matches!(
            self.mode,
            ShellMode::LockScreen
                | ShellMode::WelcomeScreen
                | ShellMode::ShutdownScreen
                | ShellMode::SystemScreen
        )
    }

    /// Returns the currently focused window handle, if any.
    pub fn focused_window(&self) -> Option<u64> {
        if self.window_manager.focused_handle() == 0 {
            None
        } else {
            Some(self.window_manager.focused_handle())
        }
    }

    /// Returns the number of visible windows.
    pub fn visible_window_count(&self) -> usize {
        self.window_manager.visible_window_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_controller() -> ShellController {
        ShellController::new(EventBus::new(), 1920.0, 1080.0)
    }

    #[test]
    fn shell_controller_starts_in_welcome_mode() {
        let ctrl = make_controller();
        assert_eq!(ctrl.mode(), ShellMode::WelcomeScreen);
    }

    #[test]
    fn complete_first_boot_transitions_to_desktop() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();
        assert_eq!(ctrl.mode(), ShellMode::Desktop);
        assert!(ctrl.first_boot_done);
    }

    #[test]
    fn boot_crossfade_complete_starts_welcome_screen() {
        let mut ctrl = make_controller();
        ctrl.dispatch_event(&AEEvent::BootCrossfadeComplete);
        assert_eq!(ctrl.mode, ShellMode::WelcomeScreen);
        assert!(ctrl.welcome_screen.is_active);
    }

    #[test]
    fn welcome_screen_completed_transitions_to_desktop() {
        let mut ctrl = make_controller();
        ctrl.dispatch_event(&AEEvent::BootCrossfadeComplete);
        ctrl.dispatch_event(&AEEvent::WelcomeScreenCompleted);
        assert_eq!(ctrl.mode(), ShellMode::Desktop);
    }

    #[test]
    fn lock_screen_activation() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();
        ctrl.dispatch_event(&AEEvent::LockScreenActivate);
        assert_eq!(ctrl.mode(), ShellMode::LockScreen);
        assert!(*ctrl.lock_screen.is_active.read());
    }

    #[test]
    fn lock_screen_unlock_returns_to_desktop() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();
        ctrl.dispatch_event(&AEEvent::LockScreenActivate);
        ctrl.dispatch_event(&AEEvent::LockScreenUnlocked);
        assert_eq!(ctrl.mode(), ShellMode::Desktop);
    }

    #[test]
    fn cockpit_view_open_close() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();

        // Create a window so cockpit has cards
        ctrl.window_manager.create_window("App", "app", 100.0, 100.0, 800.0, 600.0);

        ctrl.dispatch_event(&AEEvent::CockpitViewOpen {
            ctx: animus_core::events::AnimusContext::default(),
        });
        // CockpitViewOpened event fires and sets mode
        ctrl.dispatch_event(&AEEvent::CockpitViewOpened);
        assert_eq!(ctrl.mode(), ShellMode::CockpitView);

        // Esc closes it
        ctrl.on_key(1, 0, true); // Esc
        assert_eq!(ctrl.mode(), ShellMode::Desktop);
    }

    #[test]
    fn alt_tab_cycles_focus() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();

        let h1 = ctrl.window_manager.create_window("A", "a", 0.0, 0.0, 400.0, 300.0);
        let h2 = ctrl.window_manager.create_window("B", "b", 0.0, 0.0, 400.0, 300.0);

        assert_eq!(ctrl.focused_window(), Some(h2));

        // Alt+Tab
        ctrl.on_key(15, 0x08, true); // Tab + Alt modifier
        assert_eq!(ctrl.focused_window(), Some(h1));
    }

    #[test]
    fn show_desktop_toggle() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();

        ctrl.window_manager.create_window("A", "a", 0.0, 0.0, 400.0, 300.0);
        assert_eq!(ctrl.visible_window_count(), 1);

        // F11 + Super
        ctrl.on_key(41, 0x40, true);
        assert_eq!(ctrl.visible_window_count(), 0);

        // Toggle back
        ctrl.on_key(41, 0x40, true);
        assert_eq!(ctrl.visible_window_count(), 1);
    }

    #[test]
    fn fullscreen_enter_exit_cascades() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();

        ctrl.panel_manager.on_output_added(1, true);
        let h = ctrl.window_manager.create_window("App", "app", 100.0, 100.0, 800.0, 600.0);

        ctrl.dispatch_event(&AEEvent::FullscreenEntered { handle: h });
        assert!(ctrl.window_manager.get_window(h).unwrap().is_fullscreen());
        assert!(ctrl.dock.is_fullscreen);

        ctrl.dispatch_event(&AEEvent::FullscreenExited { handle: h });
        assert!(!ctrl.window_manager.get_window(h).unwrap().is_fullscreen());
        assert!(!ctrl.dock.is_fullscreen);
    }

    #[test]
    fn pointer_motion_updates_dock_magnification() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();
        ctrl.dock.add_item(crate::shell::DockItem::new("filer", "Files", "icon.svg"));

        let dock_y = 1080.0 - Dock::HEIGHT;
        // Cursor near the dock item center (centered on screen)
        let dock_x = (1920.0 - 1.0 * 56.0 - 32.0) * 0.5;
        let item_center = dock_x + 16.0 + 28.0;
        ctrl.on_pointer_motion(item_center, dock_y - 10.0);

        // Dock magnification should have been applied
        assert!(ctrl.dock.items[0].magnify.target > Dock::ICON_SIZE);
    }

    #[test]
    fn pointer_motion_in_cockpit_mode_routes_to_cards() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();
        ctrl.window_manager.create_window("App", "app", 100.0, 100.0, 800.0, 600.0);
        ctrl.open_cockpit_view();
        ctrl.dispatch_event(&AEEvent::CockpitViewOpened);
        assert_eq!(ctrl.mode(), ShellMode::CockpitView);

        // Pointer motion should go to cockpit, not dock
        ctrl.on_pointer_motion(960.0, 540.0);
        // No crash = success, cards received the event
    }

    #[test]
    fn shutdown_blocks_all_input() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();
        ctrl.dispatch_event(&AEEvent::SystemShutdown);
        assert_eq!(ctrl.mode(), ShellMode::ShutdownScreen);

        // Pointer should be blocked
        assert!(!ctrl.on_pointer_button(1, 100.0, 100.0, true));
        // Key should be blocked
        assert!(!ctrl.on_key(15, 0x08, true));
    }

    #[test]
    fn window_focus_routes_global_menu_name() {
        let mut ctrl = make_controller();
        ctrl.complete_first_boot();
        ctrl.panel_manager.on_output_added(1, true);

        let h = ctrl.window_manager.create_window("TestApp", "testapp", 100.0, 100.0, 800.0, 600.0);
        ctrl.dispatch_event(&AEEvent::WindowFocused { handle: h, app_id: "testapp".to_string() });

        let panel = ctrl.panel_manager.focused_panel().unwrap();
        assert_eq!(panel.global_menu.app_name, "testapp");
    }

    #[test]
    fn reduced_motion_propagates() {
        let mut ctrl = make_controller();
        ctrl.dispatch_event(&AEEvent::ReducedMotionChanged { enabled: true });
        assert!(animus_physics::is_reduced_motion());

        ctrl.dispatch_event(&AEEvent::ReducedMotionChanged { enabled: false });
        assert!(!animus_physics::is_reduced_motion());
    }
}
