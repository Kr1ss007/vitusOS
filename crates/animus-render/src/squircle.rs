//! Continuous Squircle (Superellipse) Signed Distance Field (SDF) Geometry.
//!
//! Implements Apple-grade G2 continuous curvature ($n \approx 4.0-5.0$) rather than
//! abrupt circular arcs, eliminating tangent breaks on windows, icons, and glass sheets.

use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SquircleParams {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    /// Superellipse exponent (canonical Apple squircle: n = 4.4)
    pub exponent: f32,
    pub border_width: f32,
    pub specular_highlight_alpha: f32,
}

impl Default for SquircleParams {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            corner_radius: 22.0,
            exponent: 4.4,
            border_width: 1.0,
            specular_highlight_alpha: 0.18,
        }
    }
}

impl SquircleParams {
    pub fn for_window(w: f32, h: f32) -> Self {
        Self {
            width: w,
            height: h,
            corner_radius: 16.0,
            exponent: 4.4,
            border_width: 1.0,
            specular_highlight_alpha: 0.15,
        }
    }

    pub fn for_dock_icon(size: f32) -> Self {
        Self {
            width: size,
            height: size,
            corner_radius: size * 0.2237, // Canonical Apple 512px -> 114.5px ratio
            exponent: 4.4,
            border_width: 0.5,
            specular_highlight_alpha: 0.22,
        }
    }

    pub fn for_sheet(w: f32, h: f32) -> Self {
        Self {
            width: w,
            height: h,
            corner_radius: 24.0,
            exponent: 4.4,
            border_width: 1.0,
            specular_highlight_alpha: 0.20,
        }
    }

    /// Evaluates the Signed Distance Field (SDF) at point (x, y) relative to center.
    /// Returns negative inside the shape, 0 on the contour, positive outside.
    pub fn signed_distance(&self, p: Vec2) -> f32 {
        let half_w = self.width * 0.5;
        let half_h = self.height * 0.5;
        let r = self.corner_radius.min(half_w).min(half_h);

        let d = p.abs() - Vec2::new(half_w - r, half_h - r);

        if d.x > 0.0 && d.y > 0.0 {
            // In the superellipse corner region: |x/r|^n + |y/r|^n <= 1
            let n = self.exponent;
            let nx = (d.x / r).powf(n);
            let ny = (d.y / r).powf(n);
            let d_power = (nx + ny).powf(1.0 / n);
            (d_power - 1.0) * r
        } else {
            // Outside corner region: standard linear box distance
            d.x.max(d.y) - r
        }
    }

    /// Computes anti-aliased subpixel alpha coverage at point (x, y).
    pub fn coverage_at(&self, p: Vec2) -> f32 {
        let dist = self.signed_distance(p);
        // Smoothstep over 1-pixel band
        (1.0 - dist).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squircle_geometry_and_distances() {
        let squircle = SquircleParams::for_dock_icon(100.0);

        // Center point is deeply inside
        let d_center = squircle.signed_distance(Vec2::ZERO);
        assert!(d_center < -30.0);

        // Point at border edge
        let d_edge = squircle.signed_distance(Vec2::new(50.0, 0.0));
        assert!(d_edge.abs() < 0.5);

        // Point far outside
        let d_outside = squircle.signed_distance(Vec2::new(100.0, 100.0));
        assert!(d_outside > 20.0);

        // Subpixel coverage inside vs outside
        assert_eq!(squircle.coverage_at(Vec2::ZERO), 1.0);
        assert_eq!(squircle.coverage_at(Vec2::new(100.0, 100.0)), 0.0);
    }
}
