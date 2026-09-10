//! AESheet — modal sheet attached to parent window.
//! SurfaceAltitude::Mid — 20px blur, 82% opacity.
//! Drops from parent window title bar (not top of screen).
//! Spatially attached to parent — moves with parent window drag.
//! Cannot be dismissed by clicking outside (modal by design).
//! Dismissed only by explicit action (button within sheet).

use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;

pub struct AESheet {
    pub parent_x: f32,
    pub parent_y: f32,
    pub parent_w: f32,
    pub parent_h: f32,
    pub width: f32,
    pub height: f32,
    pub title: String,
    pub is_visible: bool,
    on_dismiss: Option<Box<dyn FnMut() + Send>>,

    pub slide_y: SpringSolver,
    pub opacity: SpringSolver,
    pub dim_overlay: SpringSolver,
}

impl AESheet {
    pub const CORNER_RADIUS: f32 = 10.0;
    pub const ATTACH_OFFSET_Y: f32 = 28.0;
    pub const ALTITUDE: SurfaceAltitude = SurfaceAltitude::Mid;

    pub fn new(
        parent_x: f32,
        parent_y: f32,
        parent_w: f32,
        parent_h: f32,
        width: f32,
        height: f32,
        title: impl Into<String>,
        on_dismiss: impl FnMut() + Send + 'static,
    ) -> Self {
        let _target_y = parent_y + (parent_h - height) * 0.5;
        let start_y = parent_y + Self::ATTACH_OFFSET_Y;
        Self {
            parent_x,
            parent_y,
            parent_w,
            parent_h,
            width,
            height,
            title: title.into(),
            is_visible: false,
            on_dismiss: Some(Box::new(on_dismiss)),
            slide_y: SpringSolver::new(start_y, SpringProfile::Sheet),
            opacity: SpringSolver::new(0.0, SpringProfile::Hover),
            dim_overlay: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    pub fn open(&mut self) {
        let start_y = self.parent_y + Self::ATTACH_OFFSET_Y;
        let target_y = self.parent_y + (self.parent_h - self.height) * 0.5;
        self.is_visible = true;
        self.slide_y.snap(start_y);
        self.slide_y.set_target(target_y);
        self.opacity.snap(0.0);
        self.opacity.set_target(1.0);
        self.dim_overlay.snap(0.0);
        self.dim_overlay.set_target(1.0);
    }

    pub fn dismiss(&mut self) {
        self.is_visible = false;
        let start_y = self.parent_y + Self::ATTACH_OFFSET_Y;
        self.slide_y.set_target(start_y);
        self.opacity.set_target(0.0);
        self.dim_overlay.set_target(0.0);
        if let Some(cb) = &mut self.on_dismiss {
            cb();
        }
    }

    pub fn update_parent(&mut self, px: f32, py: f32, pw: f32, ph: f32) {
        let target_y = py + (ph - self.height) * 0.5;
        self.parent_x = px;
        self.parent_y = py;
        self.parent_w = pw;
        self.parent_h = ph;
        if self.is_visible {
            self.slide_y.set_target(target_y);
        }
    }

    pub fn sheet_x(&self) -> f32 {
        self.parent_x + (self.parent_w - self.width) * 0.5
    }

    pub fn sheet_y(&self) -> f32 {
        self.slide_y.value
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let sx = self.sheet_x();
        let sy = self.sheet_y();
        mx >= sx && mx <= sx + self.width && my >= sy && my <= sy + self.height
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_visible
            && self.opacity.value < 0.01
            && self.opacity.is_settled()
            && self.dim_overlay.is_settled()
        {
            return;
        }
        self.slide_y.update(dt);
        self.opacity.update(dt);
        self.dim_overlay.update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_open_dismiss_cycle() {
        let mut s = AESheet::new(
            100.0, 100.0, 800.0, 600.0,
            400.0, 300.0,
            "Settings", || {},
        );
        assert!(!s.is_visible);

        s.open();
        assert!(s.is_visible);

        for _ in 0..120 {
            s.update(1.0 / 60.0);
        }
        let expected_y = 100.0 + (600.0 - 300.0) * 0.5;
        assert!((s.sheet_y() - expected_y).abs() < 3.0);
        assert!((s.opacity.value - 1.0).abs() < 0.05);
        assert!((s.dim_overlay.value - 1.0).abs() < 0.05);

        s.dismiss();
        assert!(!s.is_visible);
        let start_y = 100.0 + AESheet::ATTACH_OFFSET_Y;
        assert!((s.slide_y.target - start_y).abs() < 0.01);
    }

    #[test]
    fn sheet_follows_parent() {
        let mut s = AESheet::new(
            100.0, 100.0, 800.0, 600.0,
            400.0, 300.0,
            "Settings", || {},
        );
        s.open();
        s.update_parent(200.0, 50.0, 900.0, 700.0);

        for _ in 0..120 {
            s.update(1.0 / 60.0);
        }
        let expected_y = 50.0 + (700.0 - 300.0) * 0.5;
        assert!((s.sheet_y() - expected_y).abs() < 3.0);
        assert!((s.sheet_x() - (200.0 + (900.0 - 400.0) * 0.5)).abs() < 3.0);
    }

    #[test]
    fn sheet_not_dismissed_by_outside_click() {
        use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
        let dismissed = Arc::new(AtomicBool::new(false));
        let d2 = dismissed.clone();
        let mut s = AESheet::new(
            100.0, 100.0, 800.0, 600.0,
            400.0, 300.0,
            "Settings",
            move || { d2.store(true, Ordering::Relaxed); },
        );
        s.open();
        s.update(1.0 / 60.0);

        // AESheet is modal — outside clicks do not dismiss
        assert!(!s.hit_test(50.0, 50.0));
        // Only explicit dismiss() works
        s.dismiss();
        assert!(dismissed.load(Ordering::Relaxed));
    }
}
