//! CockpitView Camera Altitude Zoom Model (Part 29.4, FIX3-10).
//!
//! CockpitView is NOT a separate surface. It is a zoom level.
//! RenderPipeline reads these springs to scale the window layer.
//!
//! Zoom: 1.0 (desktop) <-> 0.45 (cockpit)
//! Offset Y: 0.0 <-> +60px (shifts windows up)
//! Offset X: 0.0 <-> +80px (sidebar space)
//! Sidebar X: -80px <-> 0px (desktop strip slides in)
//! Background Darken: 0.0 <-> 0.5 (wallpaper darkens)
//!
//! Window cards spring FROM their actual screen positions (no teleport).
//! Clicking a card focuses that window and closes CockpitView.
//!
//! Sentinel (FIX-05): prev_zoom = -1.0 prevents spurious cockpit sound on frame 1.
//! Reduced Motion (Part 38): All springs are marked for elimination.

use animus_core::context::AnimusContext;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};

/// A window card in the CockpitView (Mission Control equivalent).
/// Springs from the window's actual screen position to its card position.
#[derive(Clone)]
pub struct CockpitCard {
    pub window_handle: u64,
    pub title: String,
    pub app_id: String,
    /// Card position — springs from real window pos to card pos.
    pub pos: SpringSolver2D,
    /// Card scale — springs from 1.0 to card scale.
    pub scale: SpringSolver,
    /// Card width.
    pub width: f32,
    /// Card height.
    pub height: f32,
    /// Hover alpha (SPRING_HOVER).
    pub hover_alpha: SpringSolver,
    /// Original window position (for springing back on close).
    pub origin_x: f32,
    pub origin_y: f32,
    /// Original window size.
    pub origin_w: f32,
    pub origin_h: f32,
}

impl CockpitCard {
    pub const CARD_SPACING: f32 = 20.0;
    pub const CARD_CORNER: f32 = 10.0;
    pub const CARD_SCALE: f32 = 0.45;

    pub fn new(
        handle: u64,
        title: impl Into<String>,
        app_id: impl Into<String>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Self {
        Self {
            window_handle: handle,
            title: title.into(),
            app_id: app_id.into(),
            pos: SpringSolver2D::new(x, y, SpringProfile::Selection),
            scale: SpringSolver::new(1.0, SpringProfile::Selection),
            width: w,
            height: h,
            hover_alpha: SpringSolver::new(0.0, SpringProfile::Hover),
            origin_x: x,
            origin_y: y,
            origin_w: w,
            origin_h: h,
        }
    }

    /// Set the target card position in the cockpit grid.
    pub fn set_card_target(&mut self, x: f32, y: f32) {
        self.pos.set_target(x, y);
        self.scale.set_target(Self::CARD_SCALE);
    }

    /// Spring back to original window position (on close).
    pub fn spring_back(&mut self) {
        self.pos.set_target(self.origin_x, self.origin_y);
        self.scale.set_target(1.0);
    }

    /// Hit test for pointer clicks.
    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let s = self.scale.value;
        let w = self.width * s;
        let h = self.height * s;
        let cx = self.pos.x.value;
        let cy = self.pos.y.value;
        mx >= cx && mx <= cx + w && my >= cy && my <= cy + h
    }

    pub fn update(&mut self, dt: f32) {
        self.pos.update(dt);
        self.scale.update(dt);
        self.hover_alpha.update(dt);
    }
}

pub struct CockpitView {
    pub is_open: bool,
    pub zoom: SpringSolver,
    pub offset_y: SpringSolver,
    pub offset_x: SpringSolver,
    pub sidebar_x: SpringSolver,
    pub bg_darken: SpringSolver,
    pub prev_zoom: f32,
    pub active_desktop: usize,
    pub desktop_count: usize,
    /// Window cards in the cockpit view.
    pub cards: Vec<CockpitCard>,
    bus: EventBus,
}

impl CockpitView {
    pub fn new(bus: EventBus) -> Self {
        Self {
            is_open: false,
            zoom: SpringSolver::new(1.0, SpringProfile::Selection).eliminate_on_reduced_motion(true),
            offset_y: SpringSolver::new(0.0, SpringProfile::Selection).eliminate_on_reduced_motion(true),
            offset_x: SpringSolver::new(0.0, SpringProfile::Selection).eliminate_on_reduced_motion(true),
            sidebar_x: SpringSolver::new(-80.0, SpringProfile::Selection).eliminate_on_reduced_motion(true),
            bg_darken: SpringSolver::new(0.0, SpringProfile::Selection).eliminate_on_reduced_motion(true),
            prev_zoom: -1.0,
            active_desktop: 0,
            desktop_count: 1,
            cards: Vec::new(),
            bus,
        }
    }

    /// Open CockpitView with the given windows.
    /// Each window springs from its real position to a card grid position.
    pub fn open(&mut self, _ctx: Option<AnimusContext>, screen_w: f32, screen_h: f32) {
        self.is_open = true;
        self.zoom.set_target(0.45);
        self.offset_y.set_target(60.0);
        self.offset_x.set_target(80.0);
        self.sidebar_x.set_target(0.0);
        self.bg_darken.set_target(0.5);
        self.layout_cards(screen_w, screen_h);
        self.bus.publish(AEEvent::CockpitViewOpened);
    }

    /// Open without screen geometry (legacy API — uses default layout).
    pub fn open_simple(&mut self, ctx: Option<AnimusContext>) {
        self.open(ctx, 1920.0, 1080.0);
    }

    /// Lay out cards in a centered grid.
    fn layout_cards(&mut self, screen_w: f32, screen_h: f32) {
        let n = self.cards.len();
        if n == 0 {
            return;
        }

        let card_w = (screen_w * 0.5).min(320.0);
        let card_h = card_w * 0.625; // 16:10 aspect
        let spacing = CockpitCard::CARD_SPACING;
        let max_per_row = ((screen_w - 160.0) / (card_w + spacing)) as usize;
        let max_per_row = max_per_row.max(1);
        let cols = max_per_row.min(n);
        let rows = (n + cols - 1) / cols;

        let grid_w = cols as f32 * (card_w + spacing) - spacing;
        let grid_h = rows as f32 * (card_h + spacing) - spacing;
        let start_x = (screen_w - grid_w) * 0.5;
        let start_y = (screen_h - grid_h) * 0.5 + 30.0;

        for (i, card) in self.cards.iter_mut().enumerate() {
            let row = i / cols;
            let col = i % cols;
            let target_x = start_x + col as f32 * (card_w + spacing);
            let target_y = start_y + row as f32 * (card_h + spacing);
            card.width = card_w / CockpitCard::CARD_SCALE;
            card.height = card_h / CockpitCard::CARD_SCALE;
            card.set_card_target(target_x, target_y);
        }
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.zoom.set_target(1.0);
        self.offset_y.set_target(0.0);
        self.offset_x.set_target(0.0);
        self.sidebar_x.set_target(-80.0);
        self.bg_darken.set_target(0.0);

        for card in &mut self.cards {
            card.spring_back();
        }

        self.bus.publish(AEEvent::CockpitViewClosed);
    }

    /// Set the window cards from the window manager.
    pub fn set_cards(&mut self, cards: Vec<CockpitCard>) {
        self.cards = cards;
    }

    /// Handle pointer motion over cards.
    pub fn on_pointer_motion(&mut self, mx: f32, my: f32) {
        for card in &mut self.cards {
            if card.hit_test(mx, my) {
                card.hover_alpha.set_target(1.0);
            } else {
                card.hover_alpha.set_target(0.0);
            }
        }
    }

    /// Handle pointer button — click a card to focus and close.
    /// Returns the window handle of the clicked card, if any.
    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) -> Option<u64> {
        if !self.is_open || !pressed {
            return None;
        }
        for card in &self.cards {
            if card.hit_test(mx, my) {
                let handle = card.window_handle;
                self.close();
                return Some(handle);
            }
        }
        self.close();
        None
    }

    pub fn update(&mut self, dt: f32) {
        let current_zoom = self.zoom.update(dt);
        self.offset_y.update(dt);
        self.offset_x.update(dt);
        self.sidebar_x.update(dt);
        self.bg_darken.update(dt);

        for card in &mut self.cards {
            card.update(dt);
        }

        if self.prev_zoom < 0.0 {
            self.prev_zoom = current_zoom;
            return;
        }

        self.prev_zoom = current_zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cockpit_view_sentinel_and_transitions() {
        let bus = EventBus::new();
        let mut cockpit = CockpitView::new(bus);

        assert_eq!(cockpit.prev_zoom, -1.0);
        cockpit.update(0.016);
        assert_eq!(cockpit.prev_zoom, 1.0);

        cockpit.open_simple(None);
        assert!(cockpit.is_open);
        assert_eq!(cockpit.zoom.target, 0.45);

        cockpit.close();
        assert!(!cockpit.is_open);
        assert_eq!(cockpit.zoom.target, 1.0);
    }

    #[test]
    fn cockpit_card_spring_from_position() {
        let mut card = CockpitCard::new(1, "App", "app", 100.0, 200.0, 800.0, 600.0);
        assert_eq!(card.pos.x.value, 100.0);
        assert_eq!(card.pos.y.value, 200.0);
        assert!((card.scale.value - 1.0).abs() < 0.01);

        card.set_card_target(500.0, 300.0);
        assert_eq!(card.pos.x.target, 500.0);
        assert_eq!(card.pos.y.target, 300.0);
        assert!((card.scale.target - 0.45).abs() < 0.01);

        // Spring settles
        for _ in 0..120 {
            card.update(1.0 / 60.0);
        }
        assert!((card.pos.x.value - 500.0).abs() < 5.0);
        assert!((card.scale.value - 0.45).abs() < 0.05);

        // Spring back
        card.spring_back();
        for _ in 0..120 {
            card.update(1.0 / 60.0);
        }
        assert!((card.pos.x.value - 100.0).abs() < 5.0);
        assert!((card.scale.value - 1.0).abs() < 0.05);
    }

    #[test]
    fn cockpit_card_hit_test() {
        let mut card = CockpitCard::new(1, "App", "app", 100.0, 100.0, 800.0, 600.0);
        card.set_card_target(200.0, 200.0);
        for _ in 0..120 {
            card.update(1.0 / 60.0);
        }
        let s = card.scale.value;
        let w = card.width * s;
        let h = card.height * s;
        assert!(card.hit_test(200.0 + w * 0.5, 200.0 + h * 0.5));
        assert!(!card.hit_test(50.0, 50.0));
    }

    #[test]
    fn cockpit_open_with_cards_layouts_them() {
        let bus = EventBus::new();
        let mut cockpit = CockpitView::new(bus);
        cockpit.set_cards(vec![
            CockpitCard::new(1, "A", "a", 100.0, 100.0, 800.0, 600.0),
            CockpitCard::new(2, "B", "b", 200.0, 200.0, 800.0, 600.0),
            CockpitCard::new(3, "C", "c", 300.0, 300.0, 800.0, 600.0),
        ]);

        cockpit.open(None, 1920.0, 1080.0);
        assert!(cockpit.is_open);
        assert_eq!(cockpit.cards.len(), 3);

        // Cards should have targets set (not at original positions)
        let card0_x = cockpit.cards[0].pos.x.target;
        let card1_x = cockpit.cards[1].pos.x.target;
        assert!(card1_x > card0_x); // second card should be to the right

        // All cards should target scale 0.45
        for card in &cockpit.cards {
            assert!((card.scale.target - 0.45).abs() < 0.01);
        }
    }

    #[test]
    fn cockpit_click_card_focuses_and_closes() {
        let bus = EventBus::new();
        let mut cockpit = CockpitView::new(bus);
        cockpit.set_cards(vec![
            CockpitCard::new(1, "A", "a", 100.0, 100.0, 800.0, 600.0),
        ]);
        cockpit.open(None, 1920.0, 1080.0);
        for _ in 0..120 {
            cockpit.update(1.0 / 60.0);
        }

        // Click on the card
        let card = &cockpit.cards[0];
        let cx = card.pos.x.value;
        let cy = card.pos.y.value;
        let s = card.scale.value;
        let w = card.width * s;
        let h = card.height * s;
        let handle = cockpit.on_pointer_button(cx + w * 0.5, cy + h * 0.5, true);
        assert_eq!(handle, Some(1));
        assert!(!cockpit.is_open);
    }

    #[test]
    fn cockpit_click_outside_closes() {
        let bus = EventBus::new();
        let mut cockpit = CockpitView::new(bus);
        cockpit.set_cards(vec![
            CockpitCard::new(1, "A", "a", 100.0, 100.0, 800.0, 600.0),
        ]);
        cockpit.open(None, 1920.0, 1080.0);
        for _ in 0..120 {
            cockpit.update(1.0 / 60.0);
        }

        let handle = cockpit.on_pointer_button(50.0, 50.0, true);
        assert_eq!(handle, None);
        assert!(!cockpit.is_open);
    }

    #[test]
    fn cockpit_close_springs_cards_back() {
        let bus = EventBus::new();
        let mut cockpit = CockpitView::new(bus);
        cockpit.set_cards(vec![
            CockpitCard::new(1, "A", "a", 100.0, 100.0, 800.0, 600.0),
        ]);
        cockpit.open(None, 1920.0, 1080.0);
        for _ in 0..120 {
            cockpit.update(1.0 / 60.0);
        }
        assert!((cockpit.cards[0].scale.target - 0.45).abs() < 0.01);

        cockpit.close();
        assert!((cockpit.cards[0].scale.target - 1.0).abs() < 0.01);
        assert!((cockpit.cards[0].pos.x.target - 100.0).abs() < 1.0);
    }
}
