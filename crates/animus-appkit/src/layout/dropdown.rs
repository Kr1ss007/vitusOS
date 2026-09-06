//! AEDropdown — appears below trigger element.
//! SurfaceAltitude::High — 32px blur, 72% opacity.
//! SPRING_SHEET (420,30) on open — drops with weight.
//! Item hover via SPRING_HOVER (600,40) per item.

use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;

pub struct DropdownItem {
    pub label: String,
    pub is_enabled: bool,
    pub on_select: Option<Box<dyn FnMut() + Send>>,
}

impl DropdownItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            is_enabled: true,
            on_select: None,
        }
    }

    pub fn with_action(mut self, action: impl FnMut() + Send + 'static) -> Self {
        self.on_select = Some(Box::new(action));
        self
    }
}

pub struct AEDropdown {
    pub items: Vec<DropdownItem>,
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub anchor_w: f32,
    pub width: f32,
    pub height: f32,
    pub is_visible: bool,
    pub hovered_index: Option<usize>,
    on_dismiss: Option<Box<dyn FnMut() + Send>>,

    pub slide_y: SpringSolver,
    pub opacity: SpringSolver,
    pub item_hover: Vec<SpringSolver>,
}

impl AEDropdown {
    pub const ITEM_HEIGHT: f32 = 32.0;
    pub const MIN_WIDTH: f32 = 160.0;
    pub const MAX_WIDTH: f32 = 320.0;
    pub const CORNER_RADIUS: f32 = 10.0;
    pub const ALTITUDE: SurfaceAltitude = SurfaceAltitude::High;

    pub fn new(
        items: Vec<DropdownItem>,
        anchor_x: f32,
        anchor_y: f32,
        anchor_w: f32,
        on_dismiss: impl FnMut() + Send + 'static,
    ) -> Self {
        let n = items.len();
        let width = (anchor_w.max(Self::MIN_WIDTH)).min(Self::MAX_WIDTH);
        let height = n as f32 * Self::ITEM_HEIGHT;
        let item_hover = (0..n)
            .map(|_| SpringSolver::new(0.0, SpringProfile::Hover))
            .collect();
        Self {
            items,
            anchor_x,
            anchor_y,
            anchor_w,
            width,
            height,
            is_visible: false,
            hovered_index: None,
            on_dismiss: Some(Box::new(on_dismiss)),
            slide_y: SpringSolver::new(anchor_y - 8.0, SpringProfile::Sheet),
            opacity: SpringSolver::new(0.0, SpringProfile::Hover),
            item_hover,
        }
    }

    pub fn open(&mut self) {
        self.is_visible = true;
        self.slide_y.snap(self.anchor_y - 8.0);
        self.slide_y.set_target(self.anchor_y);
        self.opacity.snap(0.0);
        self.opacity.set_target(1.0);
    }

    pub fn dismiss(&mut self) {
        self.is_visible = false;
        self.opacity.set_target(0.0);
        self.hovered_index = None;
        if let Some(cb) = &mut self.on_dismiss {
            cb();
        }
    }

    pub fn current_y(&self) -> f32 {
        self.slide_y.value
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let y = self.slide_y.value;
        mx >= self.anchor_x
            && mx <= self.anchor_x + self.width
            && my >= y
            && my <= y + self.height
    }

    pub fn on_pointer_motion(&mut self, mx: f32, my: f32) {
        if !self.is_visible {
            return;
        }
        let y = self.slide_y.value;
        if mx >= self.anchor_x
            && mx <= self.anchor_x + self.width
            && my >= y
            && my <= y + self.height
        {
            let idx = ((my - y) / Self::ITEM_HEIGHT) as usize;
            if idx < self.items.len() {
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
        let y = self.slide_y.value;
        let idx = ((my - y) / Self::ITEM_HEIGHT) as usize;
        if idx < self.items.len() && self.items[idx].is_enabled {
            let mut action_taken = false;
            if let Some(action) = self.items[idx].on_select.as_mut() {
                action();
                action_taken = true;
            }
            self.dismiss();
            return action_taken || true;
        }
        false
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_visible && self.opacity.value < 0.01 && self.opacity.is_settled() {
            return;
        }
        self.slide_y.update(dt);
        self.opacity.update(dt);
        for spring in &mut self.item_hover {
            spring.update(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_items() -> Vec<DropdownItem> {
        vec![
            DropdownItem::new("Cut"),
            DropdownItem::new("Copy"),
            DropdownItem::new("Paste"),
        ]
    }

    #[test]
    fn dropdown_open_dismiss() {
        let mut d = AEDropdown::new(make_items(), 100.0, 200.0, 120.0, || {});
        assert!(!d.is_visible);

        d.open();
        assert!(d.is_visible);
        assert!((d.opacity.value - 0.0).abs() < 0.01);

        for _ in 0..120 {
            d.update(1.0 / 60.0);
        }
        assert!((d.slide_y.value - 200.0).abs() < 2.0);
        assert!((d.opacity.value - 1.0).abs() < 0.05);

        d.dismiss();
        assert!(!d.is_visible);
    }

    #[test]
    fn dropdown_item_hover_and_select() {
        let mut d = AEDropdown::new(make_items(), 100.0, 200.0, 120.0, || {});
        d.open();
        for _ in 0..120 {
            d.update(1.0 / 60.0);
        }

        let item_y = d.slide_y.value + 10.0;
        d.on_pointer_motion(110.0, item_y);
        assert_eq!(d.hovered_index, Some(0));

        let item2_y = d.slide_y.value + AEDropdown::ITEM_HEIGHT + 10.0;
        d.on_pointer_motion(110.0, item2_y);
        assert_eq!(d.hovered_index, Some(1));

        let triggered = d.on_pointer_button(110.0, item_y, true);
        assert!(triggered);
        assert!(!d.is_visible);
    }

    #[test]
    fn dropdown_dismiss_on_outside_click() {
        let mut d = AEDropdown::new(make_items(), 100.0, 200.0, 120.0, || {});
        d.open();
        for _ in 0..120 {
            d.update(1.0 / 60.0);
        }

        let triggered = d.on_pointer_button(50.0, 50.0, true);
        assert!(triggered);
        assert!(!d.is_visible);
    }
}
