//! AEContextMenu — right-click context menu.
//! SurfaceAltitude::Floating — 48px blur, 64% opacity.
//! Springs from cursor position, dismissed by clicking outside or Esc.

use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};
use animus_render::altitude::SurfaceAltitude;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuItemType {
    Normal,
    Separator,
    Checkbox { checked: bool },
    Submenu,
}

pub struct ContextMenuItem {
    pub label: String,
    pub item_type: ContextMenuItemType,
    pub is_enabled: bool,
    pub shortcut: Option<String>,
    pub on_select: Option<Box<dyn FnMut() + Send>>,
    pub children: Vec<ContextMenuItem>,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            item_type: ContextMenuItemType::Normal,
            is_enabled: true,
            shortcut: None,
            on_select: None,
            children: Vec::new(),
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            item_type: ContextMenuItemType::Separator,
            is_enabled: false,
            shortcut: None,
            on_select: None,
            children: Vec::new(),
        }
    }

    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            item_type: ContextMenuItemType::Checkbox { checked },
            is_enabled: true,
            shortcut: None,
            on_select: None,
            children: Vec::new(),
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_action(mut self, action: impl FnMut() + Send + 'static) -> Self {
        self.on_select = Some(Box::new(action));
        self
    }
}

pub struct AEContextMenu {
    pub items: Vec<ContextMenuItem>,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub width: f32,
    pub height: f32,
    pub is_visible: bool,
    pub hovered_index: Option<usize>,
    pub screen_w: f32,
    pub screen_h: f32,

    pub pos: SpringSolver2D,
    pub scale: SpringSolver,
    pub opacity: SpringSolver,
    pub item_hover: Vec<SpringSolver>,
}

impl AEContextMenu {
    pub const ITEM_HEIGHT: f32 = 30.0;
    pub const SEPARATOR_HEIGHT: f32 = 9.0;
    pub const MIN_WIDTH: f32 = 180.0;
    pub const MAX_WIDTH: f32 = 280.0;
    pub const CORNER_RADIUS: f32 = 8.0;
    pub const PADDING_V: f32 = 4.0;
    pub const ALTITUDE: SurfaceAltitude = SurfaceAltitude::Floating;

    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        Self {
            items: Vec::new(),
            cursor_x: 0.0,
            cursor_y: 0.0,
            width: Self::MIN_WIDTH,
            height: 0.0,
            is_visible: false,
            hovered_index: None,
            screen_w,
            screen_h,
            pos: SpringSolver2D::new(0.0, 0.0, SpringProfile::Selection),
            scale: SpringSolver::new(0.92, SpringProfile::Selection),
            opacity: SpringSolver::new(0.0, SpringProfile::Hover),
            item_hover: Vec::new(),
        }
    }

    pub fn show(&mut self, items: Vec<ContextMenuItem>, cursor_x: f32, cursor_y: f32) {
        let n = items.len();
        let separators = items.iter().filter(|i| i.item_type == ContextMenuItemType::Separator).count();
        let non_sep = n - separators;
        self.height = non_sep as f32 * Self::ITEM_HEIGHT
            + separators as f32 * Self::SEPARATOR_HEIGHT
            + Self::PADDING_V * 2.0;
        self.width = Self::MIN_WIDTH;
        self.items = items;
        self.item_hover = (0..n)
            .map(|_| SpringSolver::new(0.0, SpringProfile::Hover))
            .collect();

        let mut target_x = cursor_x;
        let mut target_y = cursor_y;
        if target_x + self.width > self.screen_w {
            target_x = self.screen_w - self.width - 4.0;
        }
        if target_y + self.height > self.screen_h {
            target_y = self.screen_h - self.height - 4.0;
        }
        if target_x < 0.0 {
            target_x = 0.0;
        }
        if target_y < 0.0 {
            target_y = 0.0;
        }

        self.cursor_x = cursor_x;
        self.cursor_y = cursor_y;
        self.is_visible = true;
        self.hovered_index = None;
        self.pos.snap(cursor_x, cursor_y);
        self.pos.set_target(target_x, target_y);
        self.scale.snap(0.92);
        self.scale.set_target(1.0);
        self.opacity.snap(0.0);
        self.opacity.set_target(1.0);
    }

    pub fn dismiss(&mut self) {
        self.is_visible = false;
        self.opacity.set_target(0.0);
        self.scale.set_target(0.92);
        self.hovered_index = None;
        for spring in &mut self.item_hover {
            spring.set_target(0.0);
        }
    }

    fn item_y_at(&self, my: f32) -> Option<usize> {
        let y = self.pos.y.value + Self::PADDING_V;
        let mut current_y = y;
        for (i, item) in self.items.iter().enumerate() {
            let h = if item.item_type == ContextMenuItemType::Separator {
                Self::SEPARATOR_HEIGHT
            } else {
                Self::ITEM_HEIGHT
            };
            if my >= current_y && my < current_y + h {
                return Some(i);
            }
            current_y += h;
        }
        None
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let px = self.pos.x.value;
        let py = self.pos.y.value;
        mx >= px && mx <= px + self.width && my >= py && my <= py + self.height
    }

    pub fn on_pointer_motion(&mut self, mx: f32, my: f32) {
        if !self.is_visible {
            return;
        }
        if !self.hit_test(mx, my) {
            self.hovered_index = None;
            for spring in &mut self.item_hover {
                spring.set_target(0.0);
            }
            return;
        }
        if let Some(idx) = self.item_y_at(my) {
            if self.items[idx].item_type != ContextMenuItemType::Separator
                && self.items[idx].is_enabled
            {
                self.hovered_index = Some(idx);
                for (i, spring) in self.item_hover.iter_mut().enumerate() {
                    spring.set_target(if i == idx { 1.0 } else { 0.0 });
                }
                return;
            }
        }
        self.hovered_index = None;
        for spring in &mut self.item_hover {
            spring.set_target(0.0);
        }
    }

    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) -> bool {
        if !self.is_visible {
            return false;
        }
        if !pressed {
            return false;
        }
        if !self.hit_test(mx, my) {
            self.dismiss();
            return true;
        }
        if let Some(idx) = self.item_y_at(my) {
            if self.items[idx].item_type != ContextMenuItemType::Separator
                && self.items[idx].is_enabled
            {
                let action_taken = self.items[idx].on_select.is_some();
                if let Some(action) = self.items[idx].on_select.as_mut() {
                    action();
                }
                self.dismiss();
                return action_taken || true;
            }
        }
        false
    }

    pub fn on_key(&mut self, sym: u32, _mods: u32, pressed: bool) -> bool {
        if !self.is_visible || !pressed {
            return false;
        }
        match sym {
            0xff1b => {
                self.dismiss();
                true
            }
            0xff0d => {
                if let Some(idx) = self.hovered_index {
                    let action_taken = self.items[idx].on_select.is_some();
                    if let Some(action) = self.items[idx].on_select.as_mut() {
                        action();
                    }
                    self.dismiss();
                    return action_taken || true;
                }
                false
            }
            0xff54 => {
                if let Some(idx) = self.hovered_index {
                    let mut next = idx + 1;
                    while next < self.items.len()
                        && (self.items[next].item_type == ContextMenuItemType::Separator
                            || !self.items[next].is_enabled)
                    {
                        next += 1;
                    }
                    if next < self.items.len() {
                        self.hovered_index = Some(next);
                        for (i, spring) in self.item_hover.iter_mut().enumerate() {
                            spring.set_target(if i == next { 1.0 } else { 0.0 });
                        }
                    }
                } else if let Some(first) = self
                    .items
                    .iter()
                    .position(|i| i.item_type != ContextMenuItemType::Separator && i.is_enabled)
                {
                    self.hovered_index = Some(first);
                    for (i, spring) in self.item_hover.iter_mut().enumerate() {
                        spring.set_target(if i == first { 1.0 } else { 0.0 });
                    }
                }
                true
            }
            0xff52 => {
                if let Some(idx) = self.hovered_index {
                    if idx > 0 {
                        let mut prev = idx - 1;
                        while prev > 0
                            && (self.items[prev].item_type == ContextMenuItemType::Separator
                                || !self.items[prev].is_enabled)
                        {
                            prev -= 1;
                        }
                        if self.items[prev].item_type != ContextMenuItemType::Separator
                            && self.items[prev].is_enabled
                        {
                            self.hovered_index = Some(prev);
                            for (i, spring) in self.item_hover.iter_mut().enumerate() {
                                spring.set_target(if i == prev { 1.0 } else { 0.0 });
                            }
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_visible && self.opacity.value < 0.01 && self.opacity.is_settled() {
            return;
        }
        self.pos.update(dt);
        self.scale.update(dt);
        self.opacity.update(dt);
        for spring in &mut self.item_hover {
            spring.update(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_items() -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::new("Cut").with_shortcut("Ctrl+X"),
            ContextMenuItem::new("Copy").with_shortcut("Ctrl+C"),
            ContextMenuItem::separator(),
            ContextMenuItem::new("Paste").with_shortcut("Ctrl+V"),
        ]
    }

    #[test]
    fn context_menu_show_dismiss() {
        let mut m = AEContextMenu::new(1920.0, 1080.0);
        assert!(!m.is_visible);

        m.show(make_items(), 500.0, 500.0);
        assert!(m.is_visible);
        assert_eq!(m.items.len(), 4);
        assert!(m.height > 0.0);

        for _ in 0..120 {
            m.update(1.0 / 60.0);
        }
        assert!((m.opacity.value - 1.0).abs() < 0.05);
        assert!((m.scale.value - 1.0).abs() < 0.05);

        m.dismiss();
        assert!(!m.is_visible);
    }

    #[test]
    fn context_menu_clamps_to_screen() {
        let mut m = AEContextMenu::new(1920.0, 1080.0);
        m.show(make_items(), 1900.0, 1050.0);

        for _ in 0..120 {
            m.update(1.0 / 60.0);
        }
        assert!(m.pos.x.value + m.width <= 1920.0 + 1.0);
        assert!(m.pos.y.value + m.height <= 1080.0 + 1.0);
    }

    #[test]
    fn context_menu_keyboard_navigation() {
        let mut m = AEContextMenu::new(1920.0, 1080.0);
        m.show(make_items(), 500.0, 500.0);
        for _ in 0..120 {
            m.update(1.0 / 60.0);
        }

        // Down arrow selects first item (index 0 = "Cut")
        m.on_key(0xff54, 0, true); // Down
        assert_eq!(m.hovered_index, Some(0));

        // Down arrow again selects index 1 ("Copy"), skips separator
        m.on_key(0xff54, 0, true); // Down
        assert_eq!(m.hovered_index, Some(1));

        // Down arrow skips separator, selects index 3 ("Paste")
        m.on_key(0xff54, 0, true); // Down
        assert_eq!(m.hovered_index, Some(3));

        // Up arrow goes back to index 1 ("Copy"), skips separator
        m.on_key(0xff52, 0, true); // Up
        assert_eq!(m.hovered_index, Some(1));
    }

    #[test]
    fn context_menu_esc_dismisses() {
        let mut m = AEContextMenu::new(1920.0, 1080.0);
        m.show(make_items(), 500.0, 500.0);
        m.update(1.0 / 60.0);
        assert!(m.is_visible);

        m.on_key(0xff1b, 0, true); // Esc
        assert!(!m.is_visible);
    }
}
