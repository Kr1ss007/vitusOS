//! AEPopover — lightweight overlay for supplemental content.
//! SurfaceAltitude::High — 32px blur, 72% opacity.
//! Springs from origin geometry (the trigger element), not screen center.
//! Dismissed by clicking outside.

use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};
use animus_render::altitude::SurfaceAltitude;

pub struct AEPopover {
    pub origin_x: f32,
    pub origin_y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub width: f32,
    pub height: f32,
    pub is_visible: bool,
    on_dismiss: Option<Box<dyn FnMut() + Send>>,

    pub pos: SpringSolver2D,
    pub scale: SpringSolver,
    pub opacity: SpringSolver,
}

impl AEPopover {
    pub const CORNER_RADIUS: f32 = 10.0;
    pub const ALTITUDE: SurfaceAltitude = SurfaceAltitude::High;

    pub fn new(
        origin_x: f32,
        origin_y: f32,
        width: f32,
        height: f32,
        on_dismiss: impl FnMut() + Send + 'static,
    ) -> Self {
        let target_x = origin_x;
        let target_y = origin_y;
        Self {
            origin_x,
            origin_y,
            target_x,
            target_y,
            width,
            height,
            is_visible: false,
            on_dismiss: Some(Box::new(on_dismiss)),
            pos: SpringSolver2D::new(origin_x, origin_y, SpringProfile::Selection),
            scale: SpringSolver::new(0.92, SpringProfile::Selection),
            opacity: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    pub fn open(&mut self, target_x: f32, target_y: f32) {
        self.target_x = target_x;
        self.target_y = target_y;
        self.is_visible = true;
        self.pos.snap(self.origin_x, self.origin_y);
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
        if let Some(cb) = &mut self.on_dismiss {
            cb();
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.target_x = x;
        self.target_y = y;
        self.pos.set_target(x, y);
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let (px, py) = (self.pos.x.value, self.pos.y.value);
        let s = self.scale.value;
        let w = self.width * s;
        let h = self.height * s;
        let cx = px + w * 0.5;
        let cy = py + h * 0.5;
        mx >= cx - w * 0.5 && mx <= cx + w * 0.5 && my >= cy - h * 0.5 && my <= cy + h * 0.5
    }

    pub fn on_pointer_motion(&mut self, _mx: f32, _my: f32) {}

    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) -> bool {
        if !self.is_visible || !pressed {
            return false;
        }
        if !self.hit_test(mx, my) {
            self.dismiss();
            return true;
        }
        false
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_visible && self.opacity.value < 0.01 && self.opacity.is_settled() {
            return;
        }
        self.pos.update(dt);
        self.scale.update(dt);
        self.opacity.update(dt);
    }

    pub fn render_opacity(&self) -> f32 {
        self.opacity.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popover_open_dismiss_cycle() {
        let mut p = AEPopover::new(100.0, 100.0, 200.0, 150.0, || {});
        assert!(!p.is_visible);

        p.open(300.0, 200.0);
        assert!(p.is_visible);
        assert!((p.scale.value - 0.92).abs() < 0.01);

        for _ in 0..120 {
            p.update(1.0 / 60.0);
        }
        assert!((p.scale.value - 1.0).abs() < 0.05);
        assert!((p.opacity.value - 1.0).abs() < 0.05);
        assert!((p.pos.x.value - 300.0).abs() < 5.0);
        assert!((p.pos.y.value - 200.0).abs() < 5.0);

        p.dismiss();
        assert!(!p.is_visible);
        assert!((p.opacity.target - 0.0).abs() < 0.01);
    }

    #[test]
    fn popover_hit_test() {
        let mut p = AEPopover::new(100.0, 100.0, 200.0, 150.0, || {});
        p.open(200.0, 200.0);
        for _ in 0..120 {
            p.update(1.0 / 60.0);
        }
        assert!(p.hit_test(300.0, 275.0));
        assert!(!p.hit_test(100.0, 100.0));
    }

    #[test]
    fn popover_dismiss_on_outside_click() {
        use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
        let dismissed = Arc::new(AtomicBool::new(false));
        let d2 = dismissed.clone();
        let mut p = AEPopover::new(100.0, 100.0, 200.0, 150.0, move || {
            d2.store(true, Ordering::Relaxed);
        });
        p.open(200.0, 200.0);
        for _ in 0..120 {
            p.update(1.0 / 60.0);
        }
        let triggered = p.on_pointer_button(50.0, 50.0, true);
        assert!(triggered);
        assert!(dismissed.load(Ordering::Relaxed));
    }
}
