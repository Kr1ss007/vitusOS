//! Color Models, OKLab Perceptual Math, and Semantic Color Roles.

use glam::Vec4;
use serde::{Deserialize, Serialize};

/// Semantic color roles — prevents arbitrary hex colors in application code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextColor {
    /// #1A1A1A on light content / #F0F0F0 on dark glass
    Primary,
    /// #808080 (Cosmic Gray)
    Secondary,
    /// #3D3D3D (Subdued labels)
    Muted,
    /// #E85D00 / #FF6B2B (Space Orange)
    Accent,
    /// #FFFFFF (On orange buttons)
    OnAccent,
    /// #F0F0F0 (On dark glass surfaces)
    OnDark,
}

impl TextColor {
    #[inline]
    pub const fn to_argb(&self) -> u32 {
        match self {
            Self::Primary => 0xFF1A1A1A,
            Self::Secondary => 0xFF808080,
            Self::Muted => 0xFF3D3D3D,
            Self::Accent => 0xFFE85D00,
            Self::OnAccent => 0xFFFFFFFF,
            Self::OnDark => 0xFFF0F0F0,
        }
    }

    #[inline]
    pub const fn to_rgba_vec4(&self) -> Vec4 {
        match self {
            Self::Primary => Vec4::new(0.102, 0.102, 0.102, 1.0),
            Self::Secondary => Vec4::new(0.502, 0.502, 0.502, 1.0),
            Self::Muted => Vec4::new(0.239, 0.239, 0.239, 1.0),
            Self::Accent => Vec4::new(0.910, 0.365, 0.0, 1.0),
            Self::OnAccent => Vec4::new(1.0, 1.0, 1.0, 1.0),
            Self::OnDark => Vec4::new(0.941, 0.941, 0.941, 1.0),
        }
    }
}

/// Hardcoded System Color Tokens (Strictly Locked).
pub struct SystemColors;
impl SystemColors {
    pub const WARM_BLACK: u32 = 0xFF1A1208; // #1A1208 (Never pure black except shutdown)
    pub const CONTENT_BG: u32 = 0xFFFEFEFE; // #FEFEFE (Never pure white)
    pub const SPACE_ORANGE: u32 = 0xFFE85D00; // Primary brand accent
    pub const SPACE_ORANGE_BRIGHT: u32 = 0xFFFF6B2B; // Highlight / hover accent
    pub const TL_CLOSE: u32 = 0xFFFF3B30; // Space Red (#FF3B30)
    pub const TL_MINIMIZE: u32 = 0xFFFFCC00; // Space Yellow (#FFCC00)
    pub const TL_MAXIMIZE: u32 = 0xFF007AFF; // Space Blue (#007AFF - locked, never green)
}

/// OKLab color struct for perceptual color blending and luminosity boosting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Oklab {
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }

    pub fn to_srgb(&self) -> (f32, f32, f32) {
        let l_ = self.l + 0.3963377774 * self.a + 0.2158037573 * self.b;
        let m_ = self.l - 0.1055613458 * self.a - 0.0638541728 * self.b;
        let s_ = self.l - 0.0894841775 * self.a - 1.2914855480 * self.b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        let r_lin = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
        let g_lin = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
        let b_lin = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

        let to_srgb = |c: f32| {
            if c > 0.0031308 {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            } else {
                12.92 * c
            }
        };

        (
            to_srgb(r_lin.max(0.0)).clamp(0.0, 1.0),
            to_srgb(g_lin.max(0.0)).clamp(0.0, 1.0),
            to_srgb(b_lin.max(0.0)).clamp(0.0, 1.0),
        )
    }

    pub fn from_srgb(r: f32, g: f32, b: f32) -> Self {
        // Convert sRGB to linear RGB
        let to_linear = |c: f32| {
            if c > 0.04045 {
                ((c + 0.055) / 1.055).powf(2.4)
            } else {
                c / 12.92
            }
        };

        let lr = to_linear(r);
        let lg = to_linear(g);
        let lb = to_linear(b);

        // Linear RGB to LMS
        let l_ = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
        let m_ = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
        let s_ = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

        let l_c = l_.cbrt();
        let m_c = m_.cbrt();
        let s_c = s_.cbrt();

        Self {
            l: 0.2104542553 * l_c + 0.7936177850 * m_c - 0.0040720468 * s_c,
            a: 1.9779984951 * l_c - 2.4285922050 * m_c + 0.4505937099 * s_c,
            b: 0.0259040371 * l_c + 0.7827717662 * m_c - 0.8086757660 * s_c,
        }
    }
}
