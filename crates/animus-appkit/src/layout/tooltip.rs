//! AETooltip — transient help label.
//! SurfaceAltitude::Floating — 48px blur, 64% opacity.
//! 500ms dwell delay before showing, instant dismiss on leave.
//! Positioned above or below the trigger element depending on screen space.

use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;

pub struct AETooltip {
    pub text: String,
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub width: f32,
    pub height: f32,
    pub is_visible: bool,
    pub is_above: bool,
    pub show_above: bool,

    dwell_accum_ms: f32,
    is_dwelling: bool,
    pub opacity: SpringSolver,
    pub scale: SpringSolver,
}

impl AETooltip {
    pub const DWELL_DELAY_MS: f32 = 500.0;
    pub const CORNER_RADIUS: f32 = 6.0;
    pub const PADDING_H: f32 = 8.0;
    pub const PADDING_V: f32 = 6.0;
    pub const MIN_WIDTH: f32 = 40.0;
    pub const MAX_WIDTH: f32 = 240.0;
    pub const FONT_SIZE: f32 = 12.0;
    pub const ALTITUDE: SurfaceAltitude = SurfaceAltitude::Floating;

    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            anchor_x: 0.0,
            anchor_y: 0.0,
            width: 80.0,
            height: 28.0,
            is_visible: false,
            is_above: true,
            show_above: true,
            dwell_accum_ms: 0.0,
            is_dwelling: false,
            opacity: SpringSolver::new(0.0, SpringProfile::Hover),
            scale: SpringSolver::new(0.92, SpringProfile::Hover),
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn begin_dwell(&mut self, anchor_x: f32, anchor_y: f32, show_above: bool) {
        self.anchor_x = anchor_x;
        self.anchor_y = anchor_y;
        self.show_above = show_above;
        self.is_dwelling = true;
        self.dwell_accum_ms = 0.0;
    }

    pub fn cancel_dwell(&mut self) {
        self.is_dwelling = false;
        self.dwell_accum_ms = 0.0;
        if self.is_visible {
            self.is_visible = false;
            self.opacity.set_target(0.0);
            self.scale.set_target(0.92);
        }
    }

    pub fn tick(&mut self, dt: f32, _screen_h: f32) {
        if self.is_dwelling {
            self.dwell_accum_ms += dt * 1000.0;
            if self.dwell_accum_ms >= Self::DWELL_DELAY_MS {
                if !self.is_visible {
                    self.is_visible = true;
                    self.is_above = self.show_above && self.anchor_y > 60.0;
                    self.opacity.snap(0.0);
                    self.opacity.set_target(1.0);
                    self.scale.snap(0.92);
                    self.scale.set_target(1.0);
                }
                self.is_dwelling = false;
            }
        }

        if self.is_visible || self.opacity.value > 0.01 {
            self.opacity.update(dt);
            self.scale.update(dt);
        }

        if !self.is_visible && !self.is_dwelling && self.opacity.value < 0.01 && self.opacity.is_settled() {
            self.dwell_accum_ms = 0.0;
        }
    }

    pub fn tooltip_y(&self) -> f32 {
        if self.is_above {
            self.anchor_y - self.height - 4.0
        } else {
            self.anchor_y + 4.0
        }
    }

    pub fn tooltip_x(&self) -> f32 {
        self.anchor_x
    }

    pub fn set_anchor(&mut self, x: f32, y: f32) {
        self.anchor_x = x;
        self.anchor_y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_dwell_then_show() {
        let mut t = AETooltip::new("Help text");
        t.begin_dwell(100.0, 200.0, true);
        assert!(!t.is_visible);

        for _ in 0..31 {
            t.tick(1.0 / 60.0, 1080.0);
        }
        assert!(t.is_visible);

        for _ in 0..60 {
            t.tick(1.0 / 60.0, 1080.0);
        }
        assert!((t.opacity.value - 1.0).abs() < 0.05);
    }

    #[test]
    fn tooltip_cancel_before_dwell() {
        let mut t = AETooltip::new("Help");
        t.begin_dwell(100.0, 200.0, true);

        for _ in 0..10 {
            t.tick(1.0 / 60.0, 1080.0);
        }
        assert!(!t.is_visible);

        t.cancel_dwell();

        for _ in 0..60 {
            t.tick(1.0 / 60.0, 1080.0);
        }
        assert!(!t.is_visible);
    }

    #[test]
    fn tooltip_positioning_above_and_below() {
        let mut t = AETooltip::new("Help");
        t.begin_dwell(100.0, 200.0, true);
        t.is_visible = true;
        t.is_above = true;
        assert!(t.tooltip_y() < t.anchor_y);

        t.is_above = false;
        assert!(t.tooltip_y() > t.anchor_y);
    }
}
