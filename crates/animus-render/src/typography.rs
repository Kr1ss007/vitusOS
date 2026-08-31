//! Semantic Typography Roles and Canonical Font Definitions.
//!
//! Canonical Font Hierarchy:
//! - Primary: `Inter` (UI, Headings, Controls, Panels, Docks)
//! - Secondary: `Young Serif` (Editorial, Hero titles, Welcome banners, Lock screen)
//! - Body / Third: `Panamera` (Reading text, Documents, Notes, Articles)
//! - CLI / Monospace: `JetBrains Mono` (Terminow, CLI, Code blocks, Pathfinder query)

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontFamily {
    /// Primary system UI font: Inter
    Primary,
    /// Secondary editorial / hero font: Young Serif
    Secondary,
    /// Third / reading body font: Panamera
    Body,
    /// Monospace / CLI / Developer font: JetBrains Mono
    Monospace,
}

impl FontFamily {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Primary => "Inter",
            Self::Secondary => "Young Serif",
            Self::Body => "Panamera",
            Self::Monospace => "JetBrains Mono",
        }
    }

    pub fn default_asset_path(&self) -> PathBuf {
        let base = PathBuf::from("assets/fonts");
        match self {
            Self::Primary => base.join("Inter/Inter-Variable.ttf"),
            Self::Secondary => base.join("YoungSerif/YoungSerif-Regular.ttf"),
            Self::Body => base.join("Panamera/ttf/Panamera-Regular.ttf"),
            Self::Monospace => base.join("JetBrainsMono/JetBrainsMono-Regular.ttf"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextRole {
    /// 32px Young Serif — Hero titles, lock screen clock/date
    Hero,
    /// 24px Bold Inter — Window titles, large modal headers
    Heading1,
    /// 18px Semibold Inter — Section headings
    Heading2,
    /// 14px Regular Panamera — Reading text, body copy
    Body,
    /// 13px Regular Inter — UI labels, button text, table cells
    UILabel,
    /// 12px Regular Inter — Secondary timestamps, sub-labels
    Caption,
    /// 10px Regular Inter — Badges, tiny status counters
    Small,
    /// 11px Semibold Inter, +0.080em tracking, ALL CAPS — Sidebar headers
    SidebarHeader,
    /// 13px JetBrains Mono — CLI terminal, code blocks, Pathfinder query
    Code,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FontRoleInfo {
    pub family: FontFamily,
    pub size_px: f32,
    pub weight: u32,
    pub tracking_em: f32,
    pub uppercase: bool,
}

impl TextRole {
    #[inline]
    pub const fn info(&self) -> FontRoleInfo {
        match self {
            Self::Hero => FontRoleInfo {
                family: FontFamily::Secondary,
                size_px: 32.0,
                weight: 400,
                tracking_em: -0.01,
                uppercase: false,
            },
            Self::Heading1 => FontRoleInfo {
                family: FontFamily::Primary,
                size_px: 24.0,
                weight: 700,
                tracking_em: -0.015,
                uppercase: false,
            },
            Self::Heading2 => FontRoleInfo {
                family: FontFamily::Primary,
                size_px: 18.0,
                weight: 600,
                tracking_em: -0.01,
                uppercase: false,
            },
            Self::Body => FontRoleInfo {
                family: FontFamily::Body,
                size_px: 14.0,
                weight: 400,
                tracking_em: 0.0,
                uppercase: false,
            },
            Self::UILabel => FontRoleInfo {
                family: FontFamily::Primary,
                size_px: 13.0,
                weight: 500,
                tracking_em: 0.0,
                uppercase: false,
            },
            Self::Caption => FontRoleInfo {
                family: FontFamily::Primary,
                size_px: 12.0,
                weight: 400,
                tracking_em: 0.0,
                uppercase: false,
            },
            Self::Small => FontRoleInfo {
                family: FontFamily::Primary,
                size_px: 10.0,
                weight: 400,
                tracking_em: 0.0,
                uppercase: false,
            },
            Self::SidebarHeader => FontRoleInfo {
                family: FontFamily::Primary,
                size_px: 11.0,
                weight: 600,
                tracking_em: 0.080,
                uppercase: true,
            },
            Self::Code => FontRoleInfo {
                family: FontFamily::Monospace,
                size_px: 13.0,
                weight: 400,
                tracking_em: 0.0,
                uppercase: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_hierarchy_and_roles() {
        assert_eq!(FontFamily::Primary.name(), "Inter");
        assert_eq!(FontFamily::Secondary.name(), "Young Serif");
        assert_eq!(FontFamily::Body.name(), "Panamera");
        assert_eq!(FontFamily::Monospace.name(), "JetBrains Mono");

        assert_eq!(TextRole::Hero.info().family, FontFamily::Secondary);
        assert_eq!(TextRole::Heading1.info().family, FontFamily::Primary);
        assert_eq!(TextRole::Body.info().family, FontFamily::Body);
        assert_eq!(TextRole::Code.info().family, FontFamily::Monospace);
    }
}
