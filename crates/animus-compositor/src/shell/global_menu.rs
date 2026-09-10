//! Global Menu System with Keyboard Navigation (F10 / Alt) — Part 32.
//!
//! GlobalMenu moves the application menu bar out of the window and into the Panel.
//! Reads menu layout from DBusBridge (com.canonical.dbusmenu).
//! Falls back to app-name-only when no D-Bus menu available.
//! Keyboard navigable: F10 or Alt activates, arrows navigate.
//!
//! State machine (Part 32.5):
//!   INACTIVE → F10/Alt → ACTIVE (first top item highlighted)
//!   ACTIVE + ArrowRight → next top item
//!   ACTIVE + ArrowLeft  → prev top item
//!   ACTIVE + ArrowDown  → open highlighted submenu
//!   ACTIVE + Enter      → open highlighted submenu
//!   OPEN_SUBMENU + ArrowDown  → next submenu item
//!   OPEN_SUBMENU + ArrowUp    → prev submenu item
//!   OPEN_SUBMENU + ArrowRight → open nested submenu OR move to next top item
//!   OPEN_SUBMENU + ArrowLeft  → close submenu → ACTIVE OR move to prev top item
//!   OPEN_SUBMENU + Enter      → activate highlighted item
//!   OPEN_SUBMENU + Esc        → close submenu → ACTIVE
//!   ACTIVE + Esc              → deactivate → INACTIVE
//!   OPEN_SUBMENU + letter     → jump to item starting with letter

use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

/// Menu item types matching the D-Bus menu protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub is_separator: bool,
    pub is_enabled: bool,
    pub is_checked: bool,
    pub is_radio: bool,
    pub sub_items: Vec<MenuItem>,
    pub hover_alpha: SpringSolver, // SPRING_HOVER (600, 40)
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            is_separator: false,
            is_enabled: true,
            is_checked: false,
            is_radio: false,
            sub_items: Vec::new(),
            hover_alpha: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            is_separator: true,
            is_enabled: false,
            is_checked: false,
            is_radio: false,
            sub_items: Vec::new(),
            hover_alpha: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_sub_items(mut self, items: Vec<MenuItem>) -> Self {
        self.sub_items = items;
        self
    }
}

/// Keyboard navigation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuNavState {
    /// Menu bar not active, all keys pass through to app.
    Inactive,
    /// Keyboard navigation active, top bar highlighted.
    Active,
    /// Submenu open, keyboard navigating items.
    OpenSubmenu,
}

/// Open submenu runtime state.
#[derive(Debug, Clone)]
pub struct OpenSubmenu {
    pub parent_index: usize,
    pub items: Vec<MenuItem>,
    pub x: f32,
    pub y: f32,
    pub hover_index: Option<usize>,
    pub clip_h: SpringSolver,   // SPRING_SHEET (420, 30)
    pub opacity: SpringSolver,  // SPRING_HOVER (600, 40)
}

pub struct GlobalMenu {
    pub items: Vec<MenuItem>,
    pub app_name: String,
    pub has_menu: bool,
    pub nav_state: MenuNavState,
    pub kb_top_index: Option<usize>,
    pub hover_top_index: Option<usize>,
    pub open_submenu: Option<OpenSubmenu>,
    pub app_name_opacity: SpringSolver, // SPRING_HOVER cross-fade
    pub menu_items_opacity: SpringSolver,
}

impl Default for GlobalMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalMenu {
    pub const ITEM_PADDING_H: f32 = 10.0;
    pub const SUBMENU_ITEM_H: f32 = 28.0;
    pub const SUBMENU_MIN_W: f32 = 180.0;
    pub const SUBMENU_MAX_W: f32 = 320.0;

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            app_name: String::new(),
            has_menu: false,
            nav_state: MenuNavState::Inactive,
            kb_top_index: None,
            hover_top_index: None,
            open_submenu: None,
            app_name_opacity: SpringSolver::new(1.0, SpringProfile::Hover),
            menu_items_opacity: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    /// Set menu from D-Bus menu JSON (from DBusBridge).
    pub fn set_menu_from_json(&mut self, app_name: impl Into<String>, items: Vec<MenuItem>) {
        self.app_name = app_name.into();
        self.items = items;
        self.has_menu = true;
        // Cross-fade: app name fades out, menu items fade in
        self.app_name_opacity.set_target(0.0);
        self.menu_items_opacity.set_target(1.0);
    }

    /// No D-Bus menu — show app name only.
    pub fn set_app_name_only(&mut self, app_name: impl Into<String>) {
        self.app_name = app_name.into();
        self.items.clear();
        self.has_menu = false;
        self.close_submenu();
        self.app_name_opacity.set_target(1.0);
        self.menu_items_opacity.set_target(0.0);
    }

    /// Activate keyboard navigation (F10 or Alt press).
    pub fn activate(&mut self) {
        if self.nav_state == MenuNavState::Inactive {
            self.nav_state = MenuNavState::Active;
            // Highlight first non-separator item
            self.kb_top_index = self.first_enabled_item();
            self.hover_top_index = self.kb_top_index;
            self.update_hover_springs();
        }
    }

    /// Deactivate keyboard navigation (Esc or Alt release).
    pub fn deactivate(&mut self) {
        self.nav_state = MenuNavState::Inactive;
        self.kb_top_index = None;
        self.hover_top_index = None;
        self.close_submenu();
        self.update_hover_springs();
    }

    pub fn is_active(&self) -> bool {
        self.nav_state != MenuNavState::Inactive
    }

    /// Open submenu at a top-level index.
    fn open_submenu_at(&mut self, index: usize, x: f32, y: f32) {
        if index >= self.items.len() || self.items[index].sub_items.is_empty() {
            return;
        }
        let sub_items = self.items[index].sub_items.clone();
        let first_enabled = sub_items
            .iter()
            .position(|i| !i.is_separator && i.is_enabled);
        self.open_submenu = Some(OpenSubmenu {
            parent_index: index,
            items: sub_items,
            x,
            y,
            hover_index: first_enabled,
            clip_h: SpringSolver::new(0.0, SpringProfile::Sheet),
            opacity: SpringSolver::new(0.0, SpringProfile::Hover),
        });
        self.open_submenu.as_mut().unwrap().clip_h.set_target(1.0);
        self.open_submenu.as_mut().unwrap().opacity.set_target(1.0);
        self.nav_state = MenuNavState::OpenSubmenu;
    }

    /// Close the open submenu.
    fn close_submenu(&mut self) {
        self.open_submenu = None;
        if self.nav_state == MenuNavState::OpenSubmenu {
            self.nav_state = MenuNavState::Active;
        }
    }

    fn first_enabled_item(&self) -> Option<usize> {
        self.items
            .iter()
            .position(|i| !i.is_separator && i.is_enabled)
    }

    fn next_enabled_top(&self, from: usize) -> Option<usize> {
        let n = self.items.len();
        for i in 1..=n {
            let idx = (from + i) % n;
            if !self.items[idx].is_separator && self.items[idx].is_enabled {
                return Some(idx);
            }
        }
        None
    }

    fn prev_enabled_top(&self, from: usize) -> Option<usize> {
        let n = self.items.len();
        for i in 1..=n {
            let idx = (from + n - i) % n;
            if !self.items[idx].is_separator && self.items[idx].is_enabled {
                return Some(idx);
            }
        }
        None
    }

    fn next_enabled_sub(&self, from: usize) -> Option<usize> {
        let sm = self.open_submenu.as_ref()?;
        let n = sm.items.len();
        for i in 1..=n {
            let idx = (from + i) % n;
            if !sm.items[idx].is_separator && sm.items[idx].is_enabled {
                return Some(idx);
            }
        }
        None
    }

    fn prev_enabled_sub(&self, from: usize) -> Option<usize> {
        let sm = self.open_submenu.as_ref()?;
        let n = sm.items.len();
        for i in 1..=n {
            let idx = (from + n - i) % n;
            if !sm.items[idx].is_separator && sm.items[idx].is_enabled {
                return Some(idx);
            }
        }
        None
    }

    fn update_hover_springs(&mut self) {
        for (i, item) in self.items.iter_mut().enumerate() {
            let target = if Some(i) == self.hover_top_index || Some(i) == self.kb_top_index {
                1.0
            } else {
                0.0
            };
            item.hover_alpha.set_target(target);
        }
    }

    /// Handle keyboard input. Returns true if the key was consumed.
    pub fn on_key(&mut self, sym: u32, pressed: bool, panel_w: f32, submenu_y: f32) -> bool {
        if !pressed {
            return false;
        }
        match self.nav_state {
            MenuNavState::Inactive => false,
            MenuNavState::Active => match sym {
                0xff1b => {
                    // Esc — deactivate
                    self.deactivate();
                    true
                }
                0xff53 => {
                    // ArrowRight — next top item
                    if let Some(idx) = self.kb_top_index {
                        self.kb_top_index = self.next_enabled_top(idx);
                        self.hover_top_index = self.kb_top_index;
                        self.update_hover_springs();
                    }
                    true
                }
                0xff51 => {
                    // ArrowLeft — prev top item
                    if let Some(idx) = self.kb_top_index {
                        self.kb_top_index = self.prev_enabled_top(idx);
                        self.hover_top_index = self.kb_top_index;
                        self.update_hover_springs();
                    }
                    true
                }
                0xff54 => {
                    // ArrowDown — open submenu
                    if let Some(idx) = self.kb_top_index {
                        let x = self.submenu_x_for(idx, panel_w);
                        self.open_submenu_at(idx, x, submenu_y);
                    }
                    true
                }
                0xff0d => {
                    // Enter — open submenu
                    if let Some(idx) = self.kb_top_index {
                        let x = self.submenu_x_for(idx, panel_w);
                        self.open_submenu_at(idx, x, submenu_y);
                    }
                    true
                }
                _ => false,
            },
            MenuNavState::OpenSubmenu => match sym {
                0xff1b => {
                    // Esc — close submenu → ACTIVE
                    self.close_submenu();
                    true
                }
                0xff54 => {
                    // ArrowDown — next submenu item
                    if let Some(sm) = &self.open_submenu {
                        if let Some(idx) = sm.hover_index {
                            let next = self.next_enabled_sub(idx);
                            if let Some(n) = next {
                                self.open_submenu.as_mut().unwrap().hover_index = Some(n);
                            }
                        }
                    }
                    true
                }
                0xff52 => {
                    // ArrowUp — prev submenu item
                    if let Some(sm) = &self.open_submenu {
                        if let Some(idx) = sm.hover_index {
                            let prev = self.prev_enabled_sub(idx);
                            if let Some(p) = prev {
                                self.open_submenu.as_mut().unwrap().hover_index = Some(p);
                            }
                        }
                    }
                    true
                }
                0xff53 => {
                    // ArrowRight — open nested submenu OR move to next top item
                    if let Some(sm) = &self.open_submenu {
                        if let Some(idx) = sm.hover_index {
                            if idx < sm.items.len() && !sm.items[idx].sub_items.is_empty() {
                                // Open nested submenu (known limit: only one level deep)
                                return true;
                            }
                        }
                    }
                    // Move to next top item
                    if let Some(idx) = self.kb_top_index {
                        self.close_submenu();
                        self.kb_top_index = self.next_enabled_top(idx);
                        self.hover_top_index = self.kb_top_index;
                        self.update_hover_springs();
                        let new_idx = self.kb_top_index.unwrap();
                        let x = self.submenu_x_for(new_idx, panel_w);
                        self.open_submenu_at(new_idx, x, submenu_y);
                    }
                    true
                }
                0xff51 => {
                    // ArrowLeft — close submenu → ACTIVE OR move to prev top item
                    if let Some(idx) = self.kb_top_index {
                        self.close_submenu();
                        self.kb_top_index = self.prev_enabled_top(idx);
                        self.hover_top_index = self.kb_top_index;
                        self.update_hover_springs();
                        let new_idx = self.kb_top_index.unwrap();
                        let x = self.submenu_x_for(new_idx, panel_w);
                        self.open_submenu_at(new_idx, x, submenu_y);
                    }
                    true
                }
                0xff0d => {
                    // Enter — activate highlighted item
                    if let Some(sm) = &self.open_submenu {
                        if let Some(idx) = sm.hover_index {
                            if idx < sm.items.len() && sm.items[idx].is_enabled {
                                self.deactivate();
                                return true;
                            }
                        }
                    }
                    true
                }
                _ => {
                    // Letter — jump to item starting with letter
                    if let Some(ch) = char::from_u32(sym) {
                        if ch.is_ascii_alphabetic() {
                            let lower = ch.to_ascii_lowercase();
                            if let Some(sm) = &self.open_submenu {
                                let start = sm.hover_index.unwrap_or(0);
                                let n = sm.items.len();
                                for i in 1..=n {
                                    let idx = (start + i) % n;
                                    if !sm.items[idx].is_separator
                                        && sm.items[idx].is_enabled
                                        && sm.items[idx]
                                            .label
                                            .to_lowercase()
                                            .starts_with(lower)
                                    {
                                        self.open_submenu.as_mut().unwrap().hover_index = Some(idx);
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                    false
                }
            },
        }
    }

    /// Handle pointer motion.
    pub fn on_pointer_motion(&mut self, x: f32, y: f32, panel_w: f32, submenu_y: f32) {
        if self.nav_state == MenuNavState::Inactive {
            return;
        }

        // Check if hovering over top bar items
        for (i, item) in self.items.iter().enumerate() {
            if item.is_separator || !item.is_enabled {
                continue;
            }
            let item_x = self.top_item_x(i, panel_w);
            let item_w = self.top_item_width(i);
            if x >= item_x && x <= item_x + item_w {
                self.hover_top_index = Some(i);
                self.kb_top_index = Some(i);
                self.update_hover_springs();

                // If a submenu is open and we hover a different top item, switch
                if self.nav_state == MenuNavState::OpenSubmenu {
                    if let Some(sm) = &self.open_submenu {
                        if sm.parent_index != i {
                            let sx = self.submenu_x_for(i, panel_w);
                            self.open_submenu_at(i, sx, submenu_y);
                        }
                    }
                }
                return;
            }
        }

        // Check if hovering over open submenu items
        if let Some(sm) = &self.open_submenu {
            let item_h = Self::SUBMENU_ITEM_H;
            if x >= sm.x && x <= sm.x + Self::SUBMENU_MIN_W && y >= sm.y {
                let rel_y = y - sm.y;
                let idx = (rel_y / item_h) as usize;
                if idx < sm.items.len() && !sm.items[idx].is_separator && sm.items[idx].is_enabled {
                    self.open_submenu.as_mut().unwrap().hover_index = Some(idx);
                }
            }
        }
    }

    /// Handle pointer button. Returns true if consumed.
    pub fn on_pointer_button(&mut self, x: f32, y: f32, pressed: bool, panel_w: f32, submenu_y: f32) -> bool {
        if !pressed || self.nav_state == MenuNavState::Inactive {
            return false;
        }

        // Check top bar items
        for (i, item) in self.items.iter().enumerate() {
            if item.is_separator || !item.is_enabled {
                continue;
            }
            let item_x = self.top_item_x(i, panel_w);
            let item_w = self.top_item_width(i);
            if x >= item_x && x <= item_x + item_w {
                if self.nav_state == MenuNavState::OpenSubmenu {
                    if let Some(sm) = &self.open_submenu {
                        if sm.parent_index == i {
                            self.close_submenu();
                            return true;
                        }
                    }
                }
                let sx = self.submenu_x_for(i, panel_w);
                self.open_submenu_at(i, sx, submenu_y);
                return true;
            }
        }

        // Check submenu items
        if let Some(sm) = &self.open_submenu {
            let item_h = Self::SUBMENU_ITEM_H;
            if x >= sm.x && x <= sm.x + Self::SUBMENU_MIN_W && y >= sm.y {
                let rel_y = y - sm.y;
                let idx = (rel_y / item_h) as usize;
                if idx < sm.items.len() && !sm.items[idx].is_separator && sm.items[idx].is_enabled {
                    self.deactivate();
                    return true;
                }
            }
        }

        // Click outside — deactivate
        self.deactivate();
        true
    }

    /// Calculate X position for submenu (aligns right edge to screen if needed).
    #[allow(dead_code)]
    fn submenu_x_for_top_index(&self, index: usize, panel_w: f32) -> f32 {
        self.submenu_x_for(index, panel_w)
    }

    fn submenu_x_for(&self, index: usize, panel_w: f32) -> f32 {
        let item_x = self.top_item_x(index, panel_w);
        let submenu_w = Self::SUBMENU_MIN_W;
        if item_x + submenu_w > panel_w {
            panel_w - submenu_w - 4.0
        } else {
            item_x
        }
    }

    fn top_item_x(&self, index: usize, _panel_w: f32) -> f32 {
        let mut x = 12.0; // Left margin
        for i in 0..index {
            x += self.top_item_width(i) + Self::ITEM_PADDING_H * 2.0;
        }
        x
    }

    fn top_item_width(&self, index: usize) -> f32 {
        if index >= self.items.len() {
            return 0.0;
        }
        // Approximate text width: ~7px per char at 13px font
        self.items[index].label.len() as f32 * 7.0 + Self::ITEM_PADDING_H * 2.0
    }

    /// Update all springs.
    pub fn update(&mut self, dt: f32) {
        self.app_name_opacity.update(dt);
        self.menu_items_opacity.update(dt);
        for item in &mut self.items {
            item.hover_alpha.update(dt);
        }
        if let Some(sm) = &mut self.open_submenu {
            sm.clip_h.update(dt);
            sm.opacity.update(dt);
            for item in &mut sm.items {
                item.hover_alpha.update(dt);
            }
        }
    }
}

/// PanelManager — owns one Panel per connected monitor (Part 32.2).
/// GlobalMenu follows focused window's output. Clock on all Panels.
/// Orange box on all Panels. System tray on primary Panel only.
pub struct PanelManager {
    pub panels: Vec<PanelEntry>,
    pub primary_output_id: Option<u32>,
    pub focused_output_id: Option<u32>,
    pub focus_handle: u64,
}

pub struct PanelEntry {
    pub output_id: u32,
    pub is_primary: bool,
    pub panel: super::panel::Panel,
    pub global_menu: GlobalMenu,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            primary_output_id: None,
            focused_output_id: None,
            focus_handle: 0,
        }
    }

    /// Called when a new output is connected.
    pub fn on_output_added(&mut self, output_id: u32, is_primary: bool) {
        if is_primary {
            self.primary_output_id = Some(output_id);
            // Demote previous primary
            for entry in &mut self.panels {
                entry.is_primary = false;
            }
        }
        self.panels.push(PanelEntry {
            output_id,
            is_primary,
            panel: super::panel::Panel::new(),
            global_menu: GlobalMenu::new(),
        });
    }

    /// Called when an output is removed.
    pub fn on_output_removed(&mut self, output_id: u32) {
        self.panels.retain(|e| e.output_id != output_id);
        if self.primary_output_id == Some(output_id) {
            self.primary_output_id = self.panels.first().map(|e| e.output_id);
            if let Some(new_primary) = self.primary_output_id {
                if let Some(entry) = self.panels.iter_mut().find(|e| e.output_id == new_primary) {
                    entry.is_primary = true;
                }
            }
        }
        if self.focused_output_id == Some(output_id) {
            self.focused_output_id = self.primary_output_id;
        }
    }

    /// Route GlobalMenu to correct Panel based on focused window output.
    pub fn on_window_focused(&mut self, window_handle: u64, output_id: u32) {
        self.focus_handle = window_handle;
        self.focused_output_id = Some(output_id);
    }

    /// Called when app registers D-Bus menu.
    pub fn on_menu_registered(&mut self, app_name: &str, items: Vec<MenuItem>) {
        let target = self.focused_output_id.or(self.primary_output_id);
        if let Some(oid) = target {
            if let Some(entry) = self.panels.iter_mut().find(|e| e.output_id == oid) {
                entry.global_menu.set_menu_from_json(app_name, items);
            }
        }
    }

    /// Called when focused app changes but has no D-Bus menu.
    pub fn on_no_menu(&mut self, app_name: &str) {
        let target = self.focused_output_id.or(self.primary_output_id);
        if let Some(oid) = target {
            if let Some(entry) = self.panels.iter_mut().find(|e| e.output_id == oid) {
                entry.global_menu.set_app_name_only(app_name);
            }
        }
    }

    /// Get the panel for a specific output.
    pub fn panel_for_output(&self, output_id: u32) -> Option<&PanelEntry> {
        self.panels.iter().find(|e| e.output_id == output_id)
    }

    pub fn panel_for_output_mut(&mut self, output_id: u32) -> Option<&mut PanelEntry> {
        self.panels.iter_mut().find(|e| e.output_id == output_id)
    }

    /// Get the primary panel.
    pub fn primary_panel(&self) -> Option<&PanelEntry> {
        self.panels.iter().find(|e| e.is_primary)
    }

    pub fn primary_panel_mut(&mut self) -> Option<&mut PanelEntry> {
        self.panels.iter_mut().find(|e| e.is_primary)
    }

    /// Get the panel for the focused output (where GlobalMenu is shown).
    pub fn focused_panel(&self) -> Option<&PanelEntry> {
        let oid = self.focused_output_id.or(self.primary_output_id)?;
        self.panel_for_output(oid)
    }

    pub fn focused_panel_mut(&mut self) -> Option<&mut PanelEntry> {
        let oid = self.focused_output_id.or(self.primary_output_id)?;
        self.panel_for_output_mut(oid)
    }

    /// Update all panels and their GlobalMenu springs.
    pub fn tick(&mut self, dt: f32) {
        for entry in &mut self.panels {
            entry.panel.update(dt);
            entry.global_menu.update(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_menu() -> Vec<MenuItem> {
        vec![
            MenuItem::new("File").with_sub_items(vec![
                MenuItem::new("New").with_shortcut("Ctrl+N"),
                MenuItem::new("Open").with_shortcut("Ctrl+O"),
                MenuItem::separator(),
                MenuItem::new("Save").with_shortcut("Ctrl+S"),
            ]),
            MenuItem::new("Edit").with_sub_items(vec![
                MenuItem::new("Cut").with_shortcut("Ctrl+X"),
                MenuItem::new("Copy").with_shortcut("Ctrl+C"),
                MenuItem::new("Paste").with_shortcut("Ctrl+V"),
            ]),
            MenuItem::new("View").with_sub_items(vec![
                MenuItem::new("Zoom In").with_shortcut("Ctrl++"),
                MenuItem::new("Zoom Out").with_shortcut("Ctrl+-"),
            ]),
        ]
    }

    #[test]
    fn global_menu_activate_deactivate() {
        let mut menu = GlobalMenu::new();
        assert_eq!(menu.nav_state, MenuNavState::Inactive);
        assert!(!menu.is_active());

        menu.set_menu_from_json("TestApp", make_test_menu());
        menu.activate();
        assert_eq!(menu.nav_state, MenuNavState::Active);
        assert!(menu.is_active());
        assert_eq!(menu.kb_top_index, Some(0));

        menu.deactivate();
        assert_eq!(menu.nav_state, MenuNavState::Inactive);
        assert!(!menu.is_active());
    }

    #[test]
    fn global_menu_keyboard_arrow_navigation() {
        let mut menu = GlobalMenu::new();
        menu.set_menu_from_json("TestApp", make_test_menu());
        menu.activate();
        assert_eq!(menu.kb_top_index, Some(0)); // File

        // ArrowRight → Edit
        menu.on_key(0xff53, true, 1920.0, 28.0); // Right
        assert_eq!(menu.kb_top_index, Some(1));

        // ArrowRight → View
        menu.on_key(0xff53, true, 1920.0, 28.0);
        assert_eq!(menu.kb_top_index, Some(2));

        // ArrowRight → wraps to File
        menu.on_key(0xff53, true, 1920.0, 28.0);
        assert_eq!(menu.kb_top_index, Some(0));

        // ArrowLeft → wraps to View
        menu.on_key(0xff51, true, 1920.0, 28.0);
        assert_eq!(menu.kb_top_index, Some(2));
    }

    #[test]
    fn global_menu_open_submenu_and_navigate() {
        let mut menu = GlobalMenu::new();
        menu.set_menu_from_json("TestApp", make_test_menu());
        menu.activate();
        assert_eq!(menu.kb_top_index, Some(0)); // File

        // ArrowDown → open File submenu
        menu.on_key(0xff54, true, 1920.0, 28.0); // Down
        assert_eq!(menu.nav_state, MenuNavState::OpenSubmenu);
        assert!(menu.open_submenu.is_some());
        let sm = menu.open_submenu.as_ref().unwrap();
        assert_eq!(sm.parent_index, 0);
        assert_eq!(sm.items.len(), 4); // New, Open, separator, Save
        assert_eq!(sm.hover_index, Some(0)); // First enabled = "New"

        // ArrowDown → "Open" (index 1)
        menu.on_key(0xff54, true, 1920.0, 28.0);
        let sm = menu.open_submenu.as_ref().unwrap();
        assert_eq!(sm.hover_index, Some(1));

        // ArrowDown → skip separator → "Save" (index 3)
        menu.on_key(0xff54, true, 1920.0, 28.0);
        let sm = menu.open_submenu.as_ref().unwrap();
        assert_eq!(sm.hover_index, Some(3));

        // ArrowUp → back to "Open" (index 1, skip separator)
        menu.on_key(0xff52, true, 1920.0, 28.0);
        let sm = menu.open_submenu.as_ref().unwrap();
        assert_eq!(sm.hover_index, Some(1));

        // Esc → close submenu → ACTIVE
        menu.on_key(0xff1b, true, 1920.0, 28.0);
        assert_eq!(menu.nav_state, MenuNavState::Active);
        assert!(menu.open_submenu.is_none());
    }

    #[test]
    fn global_menu_letter_jump() {
        let mut menu = GlobalMenu::new();
        menu.set_menu_from_json("TestApp", make_test_menu());
        menu.activate();
        menu.on_key(0xff54, true, 1920.0, 28.0); // Down → open File submenu

        // Press 'o' → should jump to "Open" (index 1)
        menu.on_key(b'o' as u32, true, 1920.0, 28.0);
        let sm = menu.open_submenu.as_ref().unwrap();
        assert_eq!(sm.hover_index, Some(1));

        // Press 's' → should jump to "Save" (index 3)
        menu.on_key(b's' as u32, true, 1920.0, 28.0);
        let sm = menu.open_submenu.as_ref().unwrap();
        assert_eq!(sm.hover_index, Some(3));
    }

    #[test]
    fn global_menu_app_name_only_fallback() {
        let mut menu = GlobalMenu::new();
        menu.set_app_name_only("SimpleApp");
        assert!(!menu.has_menu);
        assert_eq!(menu.app_name, "SimpleApp");
        assert_eq!(menu.items.len(), 0);

        // Activating with no menu still highlights nothing
        menu.activate();
        assert_eq!(menu.kb_top_index, None);
    }

    #[test]
    fn global_menu_arrow_right_in_submenu_moves_to_next_top() {
        let mut menu = GlobalMenu::new();
        menu.set_menu_from_json("TestApp", make_test_menu());
        menu.activate();
        menu.on_key(0xff54, true, 1920.0, 28.0); // Down → open File submenu
        assert_eq!(menu.nav_state, MenuNavState::OpenSubmenu);

        // ArrowRight → move to Edit submenu
        menu.on_key(0xff53, true, 1920.0, 28.0);
        assert_eq!(menu.nav_state, MenuNavState::OpenSubmenu);
        let sm = menu.open_submenu.as_ref().unwrap();
        assert_eq!(sm.parent_index, 1); // Edit
    }

    #[test]
    fn panel_manager_multi_monitor() {
        let mut pm = PanelManager::new();

        // Add primary monitor
        pm.on_output_added(1, true);
        assert_eq!(pm.panels.len(), 1);
        assert!(pm.panels[0].is_primary);
        assert_eq!(pm.primary_output_id, Some(1));

        // Add secondary monitor
        pm.on_output_added(2, false);
        assert_eq!(pm.panels.len(), 2);

        // Focus window on secondary
        pm.on_window_focused(42, 2);
        assert_eq!(pm.focused_output_id, Some(2));
        assert_eq!(pm.focus_handle, 42);

        // Register menu on focused panel
        pm.on_menu_registered("TestApp", make_test_menu());
        let focused = pm.focused_panel().unwrap();
        assert!(focused.global_menu.has_menu);

        // Primary panel should not have menu
        let primary = pm.primary_panel().unwrap();
        assert!(!primary.global_menu.has_menu);

        // Remove secondary — focus should fall back to primary
        pm.on_output_removed(2);
        assert_eq!(pm.panels.len(), 1);
        assert_eq!(pm.focused_output_id, Some(1));
    }

    #[test]
    fn panel_manager_primary_promotion() {
        let mut pm = PanelManager::new();
        pm.on_output_added(1, true);
        pm.on_output_added(2, false);

        // Remove primary → secondary promoted
        pm.on_output_removed(1);
        assert_eq!(pm.panels.len(), 1);
        assert_eq!(pm.primary_output_id, Some(2));
        assert!(pm.panels[0].is_primary);
    }
}
