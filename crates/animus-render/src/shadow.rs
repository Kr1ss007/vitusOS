//! Dual-SDF Warm Shadow Geometry and Parameters.

use glam::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowParams {
    /// Warm black: rgb(0.102, 0.071, 0.031) (#1A1208) — Never #000000
    pub color: Vec4,
    pub blur_radius_1: f32,
    pub blur_radius_2: f32,
    pub spread_1: f32,
    pub spread_2: f32,
    pub offset_y_1: f32,
    pub offset_y_2: f32,
    pub opacity: f32,
}

impl Default for ShadowParams {
    fn default() -> Self {
        Self::standard_window()
    }
}

impl ShadowParams {
    /// Standard Window Shadow (Elevation: Mid / High).
    pub fn standard_window() -> Self {
        Self {
            color: Vec4::new(0.102, 0.071, 0.031, 0.40),
            blur_radius_1: 18.0,
            blur_radius_2: 44.0,
            spread_1: 2.0,
            spread_2: 6.0,
            offset_y_1: 8.0,
            offset_y_2: 24.0,
            opacity: 0.45,
        }
    }

    /// Floating Surface Shadow (Sheets, Notifications, Menus).
    pub fn floating() -> Self {
        Self {
            color: Vec4::new(0.102, 0.071, 0.031, 0.55),
            blur_radius_1: 24.0,
            blur_radius_2: 56.0,
            spread_1: 4.0,
            spread_2: 10.0,
            offset_y_1: 12.0,
            offset_y_2: 32.0,
            opacity: 0.60,
        }
    }
}
