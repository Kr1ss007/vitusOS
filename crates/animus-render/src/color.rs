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
