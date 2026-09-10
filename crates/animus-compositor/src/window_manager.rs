//! WindowManager — z-order, focus chain, raise/lower, dock overlap detection (Part G.2).
//!
//! Z-order: most recently focused = highest z (back of vector).
//! Focus model (Part 45.3): click-to-focus ONLY. No focus-follows-mouse.
//! Dock auto-hide: checks anyWindowOverlapsDockArea() each frame.

use crate::window::AEWindow;

/// Window snap zones for edge snapping (macOS-style window tiling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapZone {
    None,
    Left,
    Right,
    Top,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Maximize,
}

/// Window snap target geometry.
#[derive(Debug, Clone, Copy)]
pub struct SnapGeometry {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl SnapGeometry {
    pub fn for_zone(zone: SnapZone, screen_w: f32, screen_h: f32, panel_h: f32) -> Option<Self> {
        let usable_h = screen_h - panel_h;
        let usable_y = panel_h;
        match zone {
            SnapZone::None => None,
            SnapZone::Left => Some(Self { x: 0.0, y: usable_y, w: screen_w * 0.5, h: usable_h }),
            SnapZone::Right => Some(Self { x: screen_w * 0.5, y: usable_y, w: screen_w * 0.5, h: usable_h }),
            SnapZone::Top | SnapZone::Maximize => Some(Self { x: 0.0, y: usable_y, w: screen_w, h: usable_h }),
            SnapZone::TopLeft => Some(Self { x: 0.0, y: usable_y, w: screen_w * 0.5, h: usable_h * 0.5 }),
            SnapZone::TopRight => Some(Self { x: screen_w * 0.5, y: usable_y, w: screen_w * 0.5, h: usable_h * 0.5 }),
            SnapZone::BottomLeft => Some(Self { x: 0.0, y: usable_y + usable_h * 0.5, w: screen_w * 0.5, h: usable_h * 0.5 }),
            SnapZone::BottomRight => Some(Self { x: screen_w * 0.5, y: usable_y + usable_h * 0.5, w: screen_w * 0.5, h: usable_h * 0.5 }),
        }
    }
}

/// The window manager.
pub struct WindowManager {
    /// Windows in z-order (back = bottom, front = top).
    windows: Vec<AEWindow>,
    /// Focused window handle (0 = none).
    focused_handle: u64,
    /// Next window handle counter.
    next_handle: u64,
    /// Screen geometry for snap calculations.
    screen_w: f32,
    screen_h: f32,
    panel_h: f32,
}

impl WindowManager {
    pub const SNAP_THRESHOLD: f32 = 20.0; // px from edge to trigger snap
    pub const DOCK_OVERLAP_MARGIN: f32 = 4.0;

    pub fn new(screen_w: f32, screen_h: f32, panel_h: f32) -> Self {
        Self {
            windows: Vec::new(),
            focused_handle: 0,
            next_handle: 1,
            screen_w,
            screen_h,
            panel_h,
        }
    }

    /// Set screen geometry (on output change).
    pub fn set_screen_geometry(&mut self, w: f32, h: f32, panel_h: f32) {
        self.screen_w = w;
        self.screen_h = h;
        self.panel_h = panel_h;
    }

    /// Create and register a new window. Returns its handle.
    pub fn create_window(
        &mut self,
        title: impl Into<String>,
        app_id: impl Into<String>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> u64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        let mut win = AEWindow::new(handle, title, app_id, x, y, w, h);
        // New window gets focus
        for w in &mut self.windows {
            w.is_focused = false;
        }
        win.is_focused = true;
        self.focused_handle = handle;
        self.windows.push(win);
        handle
    }

    /// Remove a window by handle.
    pub fn remove_window(&mut self, handle: u64) {
        self.windows.retain(|w| w.handle != handle);
        if self.focused_handle == handle {
            // Focus the next window (top of z-order)
            self.focused_handle = self.windows.last().map(|w| w.handle).unwrap_or(0);
            if let Some(top) = self.windows.last_mut() {
                top.is_focused = true;
            }
        }
    }

    /// Get a window by handle.
    pub fn get_window(&self, handle: u64) -> Option<&AEWindow> {
        self.windows.iter().find(|w| w.handle == handle)
    }

    pub fn get_window_mut(&mut self, handle: u64) -> Option<&mut AEWindow> {
        self.windows.iter_mut().find(|w| w.handle == handle)
    }

    /// Get all windows in z-order.
    pub fn windows(&self) -> &[AEWindow] {
        &self.windows
    }

    pub fn windows_mut(&mut self) -> &mut Vec<AEWindow> {
        &mut self.windows
    }

    /// Get the focused window.
    pub fn focused(&self) -> Option<&AEWindow> {
        self.windows.iter().find(|w| w.handle == self.focused_handle)
    }

    pub fn focused_mut(&mut self) -> Option<&mut AEWindow> {
        self.windows.iter_mut().find(|w| w.handle == self.focused_handle)
    }

    pub fn focused_handle(&self) -> u64 {
        self.focused_handle
    }

    /// Focus a window by handle (click-to-focus).
    pub fn focus(&mut self, handle: u64) {
        // Unfocus current
        if let Some(win) = self.windows.iter_mut().find(|w| w.handle == self.focused_handle) {
            win.is_focused = false;
        }
        // Focus new
        if let Some(win) = self.windows.iter_mut().find(|w| w.handle == handle) {
            win.is_focused = true;
            self.focused_handle = handle;
            // Move to end of vector (top of z-order)
            if let Some(idx) = self.windows.iter().position(|w| w.handle == handle) {
                let win = self.windows.remove(idx);
                self.windows.push(win);
            }
        }
    }

    /// Raise a window to the top of z-order.
    pub fn raise(&mut self, handle: u64) {
        if let Some(idx) = self.windows.iter().position(|w| w.handle == handle) {
            let win = self.windows.remove(idx);
            self.windows.push(win);
        }
    }

    /// Lower a window to the bottom of z-order.
    pub fn lower(&mut self, handle: u64) {
        if let Some(idx) = self.windows.iter().position(|w| w.handle == handle) {
            let win = self.windows.remove(idx);
            self.windows.insert(0, win);
        }
    }

    /// Cycle focus to the next window (Alt+Tab).
    pub fn cycle_focus_next(&mut self) {
        if self.windows.len() < 2 {
            return;
        }
        let current_idx = self
            .windows
            .iter()
            .position(|w| w.handle == self.focused_handle)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.windows.len();
        let next_handle = self.windows[next_idx].handle;
        self.focus(next_handle);
    }

    /// Cycle focus to the previous window (Shift+Alt+Tab).
    pub fn cycle_focus_prev(&mut self) {
        if self.windows.len() < 2 {
            return;
        }
        let current_idx = self
            .windows
            .iter()
            .position(|w| w.handle == self.focused_handle)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.windows.len() - 1
        } else {
            current_idx - 1
        };
        let prev_handle = self.windows[prev_idx].handle;
        self.focus(prev_handle);
    }

    /// Check if any window overlaps the Dock area (for auto-hide).
    pub fn any_window_overlaps_dock(&self, dock_y: f32, screen_w: f32) -> bool {
        let dock_top = dock_y - Self::DOCK_OVERLAP_MARGIN;
        for win in &self.windows {
            if !win.is_visible || win.is_minimized || win.is_fullscreen() {
                continue;
            }
            let win_bottom = win.pos.y.value + win.height;
            if win_bottom > dock_top {
                // Check horizontal overlap
                let win_left = win.pos.x.value;
                let win_right = win.pos.x.value + win.width;
                if win_right > 0.0 && win_left < screen_w {
                    return true;
                }
            }
        }
        false
    }

    /// Detect snap zone based on window position during drag.
    pub fn detect_snap_zone(&self, drag_x: f32, drag_y: f32) -> SnapZone {
        let threshold = Self::SNAP_THRESHOLD;

        let near_left = drag_x < threshold;
        let near_right = drag_x > self.screen_w - threshold;
        let near_top = drag_y < self.panel_h + threshold;
        let near_bottom = drag_y > self.screen_h - threshold;

        if near_top && near_left {
            SnapZone::TopLeft
        } else if near_top && near_right {
            SnapZone::TopRight
        } else if near_bottom && near_left {
            SnapZone::BottomLeft
        } else if near_bottom && near_right {
            SnapZone::BottomRight
        } else if near_top {
            SnapZone::Maximize
        } else if near_left {
            SnapZone::Left
        } else if near_right {
            SnapZone::Right
        } else {
            SnapZone::None
        }
    }

    /// Apply snap zone to a window.
    pub fn snap_window(&mut self, handle: u64, zone: SnapZone) -> bool {
        if zone == SnapZone::None {
            return false;
        }
        let geom = match SnapGeometry::for_zone(zone, self.screen_w, self.screen_h, self.panel_h) {
            Some(g) => g,
            None => return false,
        };
        if let Some(win) = self.get_window_mut(handle) {
            win.set_target_position(geom.x, geom.y);
            win.set_size(geom.w, geom.h);
            return true;
        }
        false
    }

    /// Minimize all windows (Show Desktop).
    pub fn minimize_all(&mut self, dock_icon_x: f32, dock_icon_y: f32) {
        for win in &mut self.windows {
            if !win.is_minimized {
                win.minimize(dock_icon_x, dock_icon_y);
            }
        }
    }

    /// Restore all minimized windows (Show Desktop toggle off).
    pub fn restore_all(&mut self) {
        for win in &mut self.windows {
            if win.is_minimized {
                win.restore();
            }
        }
    }

    /// Get the number of visible (non-minimized) windows.
    pub fn visible_window_count(&self) -> usize {
        self.windows
            .iter()
            .filter(|w| w.is_visible && !w.is_minimized)
            .count()
    }

    /// Total window count.
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Update all window springs.
    pub fn update(&mut self, dt: f32) {
        for win in &mut self.windows {
            win.update(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wm() -> WindowManager {
        WindowManager::new(1920.0, 1080.0, 28.0)
    }

    #[test]
    fn window_creation_and_focus() {
        let mut wm = make_wm();
        let h1 = wm.create_window("App 1", "app1", 100.0, 100.0, 800.0, 600.0);
        let h2 = wm.create_window("App 2", "app2", 200.0, 200.0, 800.0, 600.0);

        assert_eq!(wm.window_count(), 2);
        assert_eq!(wm.focused_handle(), h2); // newest gets focus

        // Focus first window
        wm.focus(h1);
        assert_eq!(wm.focused_handle(), h1);
        assert!(wm.get_window(h1).unwrap().is_focused);
        assert!(!wm.get_window(h2).unwrap().is_focused);
    }

    #[test]
    fn raise_and_lower() {
        let mut wm = make_wm();
        let h1 = wm.create_window("A", "a", 0.0, 0.0, 400.0, 300.0);
        let _h2 = wm.create_window("B", "b", 0.0, 0.0, 400.0, 300.0);
        let h3 = wm.create_window("C", "c", 0.0, 0.0, 400.0, 300.0);

        // Z-order: [A, B, C] (C on top)
        assert_eq!(wm.windows()[2].handle, h3);

        // Raise A to top
        wm.raise(h1);
        assert_eq!(wm.windows()[2].handle, h1);

        // Lower A to bottom
        wm.lower(h1);
        assert_eq!(wm.windows()[0].handle, h1);
    }

    #[test]
    fn focus_moves_to_top_z() {
        let mut wm = make_wm();
        let h1 = wm.create_window("A", "a", 0.0, 0.0, 400.0, 300.0);
        let _h2 = wm.create_window("B", "b", 0.0, 0.0, 400.0, 300.0);

        // Focus h1 — should move to top of z-order
        wm.focus(h1);
        assert_eq!(wm.windows()[1].handle, h1);
        assert_eq!(wm.focused_handle(), h1);
    }

    #[test]
    fn cycle_focus_next_prev() {
        let mut wm = make_wm();
        let h1 = wm.create_window("A", "a", 0.0, 0.0, 400.0, 300.0);
        let h2 = wm.create_window("B", "b", 0.0, 0.0, 400.0, 300.0);
        let h3 = wm.create_window("C", "c", 0.0, 0.0, 400.0, 300.0);

        // Current focus: h3 (last created)
        assert_eq!(wm.focused_handle(), h3);

        // Alt+Tab → should cycle to h1 (wraps)
        wm.cycle_focus_next();
        assert_eq!(wm.focused_handle(), h1);

        // Alt+Tab → h2
        wm.cycle_focus_next();
        assert_eq!(wm.focused_handle(), h2);

        // Shift+Alt+Tab → back to h1
        wm.cycle_focus_prev();
        assert_eq!(wm.focused_handle(), h1);
    }

    #[test]
    fn remove_window_refocuses() {
        let mut wm = make_wm();
        let h1 = wm.create_window("A", "a", 0.0, 0.0, 400.0, 300.0);
        let h2 = wm.create_window("B", "b", 0.0, 0.0, 400.0, 300.0);

        // Remove focused window h2
        wm.remove_window(h2);
        assert_eq!(wm.window_count(), 1);
        assert_eq!(wm.focused_handle(), h1);
    }

    #[test]
    fn dock_overlap_detection() {
        let mut wm = make_wm();
        let h1 = wm.create_window("A", "a", 0.0, 0.0, 1920.0, 800.0);

        // Window at y=0, height=800, bottom=800. Dock at y=1060.
        // 800 < 1060 → no overlap
        assert!(!wm.any_window_overlaps_dock(1060.0, 1920.0));

        // Move window down to overlap dock
        wm.get_window_mut(h1).unwrap().set_target_position(0.0, 1000.0);
        // Snap position for test
        wm.get_window_mut(h1).unwrap().pos.snap(0.0, 1000.0);
        // bottom = 1000 + 800 = 1800 > 1060 → overlap
        assert!(wm.any_window_overlaps_dock(1060.0, 1920.0));
    }

    #[test]
    fn snap_zone_detection() {
        let wm = make_wm();

        // Drag near left edge
        assert_eq!(wm.detect_snap_zone(5.0, 500.0), SnapZone::Left);

        // Drag near right edge
        assert_eq!(wm.detect_snap_zone(1915.0, 500.0), SnapZone::Right);

        // Drag near top
        assert_eq!(wm.detect_snap_zone(960.0, 30.0), SnapZone::Maximize);

        // Drag near top-left
        assert_eq!(wm.detect_snap_zone(5.0, 30.0), SnapZone::TopLeft);

        // Drag near top-right
        assert_eq!(wm.detect_snap_zone(1915.0, 30.0), SnapZone::TopRight);

        // Center — no snap
        assert_eq!(wm.detect_snap_zone(960.0, 540.0), SnapZone::None);
    }

    #[test]
    fn snap_window_to_left() {
        let mut wm = make_wm();
        let h = wm.create_window("A", "a", 100.0, 100.0, 800.0, 600.0);

        let applied = wm.snap_window(h, SnapZone::Left);
        assert!(applied);

        let win = wm.get_window(h).unwrap();
        assert_eq!(win.pos.x.target, 0.0);
        assert!((win.width - 960.0).abs() < 1.0); // half of 1920
    }

    #[test]
    fn snap_window_maximize() {
        let mut wm = make_wm();
        let h = wm.create_window("A", "a", 100.0, 100.0, 800.0, 600.0);

        wm.snap_window(h, SnapZone::Maximize);
        let win = wm.get_window(h).unwrap();
        assert_eq!(win.pos.x.target, 0.0);
        assert!((win.width - 1920.0).abs() < 1.0);
        assert!((win.height - 1052.0).abs() < 1.0); // 1080 - 28 panel
    }

    #[test]
    fn minimize_all_and_restore_all() {
        let mut wm = make_wm();
        let h1 = wm.create_window("A", "a", 0.0, 0.0, 400.0, 300.0);
        let h2 = wm.create_window("B", "b", 0.0, 0.0, 400.0, 300.0);

        assert_eq!(wm.visible_window_count(), 2);

        wm.minimize_all(960.0, 1060.0);
        assert_eq!(wm.visible_window_count(), 0);
        assert!(wm.get_window(h1).unwrap().is_minimized);
        assert!(wm.get_window(h2).unwrap().is_minimized);

        wm.restore_all();
        assert_eq!(wm.visible_window_count(), 2);
        assert!(!wm.get_window(h1).unwrap().is_minimized);
    }

    #[test]
    fn fullscreen_window_skips_dock_overlap() {
        let mut wm = make_wm();
        let h = wm.create_window("A", "a", 0.0, 0.0, 1920.0, 1080.0);

        // Fullscreen window should not trigger dock overlap
        wm.get_window_mut(h).unwrap().enter_fullscreen(1920.0, 1080.0);
        assert!(!wm.any_window_overlaps_dock(1060.0, 1920.0));
    }
}
