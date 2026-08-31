//! AESurface Altitude and Glass Material Properties (AnimusEngine Surfaces).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurfaceAltitude {
    /// 0px Blur, 100% Opacity (#FEFEFE Opaque content, 0% blur overhead) (AEContent)
    Grounded = 0,
    /// 8px Kawase Blur, 94% Opacity (AEToolbar, AEPanel)
    Low = 1,
    /// 20px Kawase Blur, 82% Opacity (AESidebar, AEWindow, AESheet, AEDock)
    Mid = 2,
    /// 32px Kawase Blur, 72% Opacity (AEDropdown, AEPopover, AECockpitView)
    High = 3,
    /// 48px Kawase Blur, 64% Opacity (AEContextMenu, AENotification, AETooltip)
    Floating = 4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlassProperties {
    pub blur_radius_px: f32,
    pub opacity: f32,
    pub luminosity_boost: f32,
}

impl SurfaceAltitude {
    #[inline]
    pub const fn glass_properties(&self) -> GlassProperties {
        match self {
            Self::Grounded => GlassProperties {
                blur_radius_px: 0.0,
                opacity: 1.0,
                luminosity_boost: 1.0,
            },
            Self::Low => GlassProperties {
                blur_radius_px: 8.0,
                opacity: 0.94,
                luminosity_boost: 1.04,
            },
            Self::Mid => GlassProperties {
                blur_radius_px: 20.0,
                opacity: 0.82,
                luminosity_boost: 1.08,
            },
            Self::High => GlassProperties {
                blur_radius_px: 32.0,
                opacity: 0.72,
                luminosity_boost: 1.12,
            },
            Self::Floating => GlassProperties {
                blur_radius_px: 48.0,
                opacity: 0.64,
                luminosity_boost: 1.16,
            },
        }
    }
}
