//! Wayland Output Management — Monitor Geometry, DPI, and Multi-Head.
//!
//! Each physical monitor has a `wl_output` global. Clients use this to:
//! - Get screen resolution, physical size, and pixel density
//! - Get current scale factor (1x, 2x HiDPI, 1.5x fractional)
//! - Position surfaces relative to the correct output
//! - Handle hot-plug events (monitor connected/disconnected)

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputTransform {
    Normal,
    Rotated90,
    Rotated180,
    Rotated270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

/// Represents one physical display output (monitor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimusOutput {
    /// Unique output ID
    pub id: u32,
    /// Connector name (e.g. "HDMI-A-1", "DP-1", "eDP-1")
    pub connector: String,
    /// Make and model from EDID
    pub make: String,
    pub model: String,
    /// Resolution in pixels
    pub width: u32,
    pub height: u32,
    /// Physical size in millimetres (for DPI calculation)
    pub phys_width_mm: u32,
    pub phys_height_mm: u32,
    /// Refresh rate in mHz (e.g. 60000 = 60Hz, 144000 = 144Hz)
    pub refresh_mhz: u32,
    /// Integer scale factor (1 = regular, 2 = HiDPI Retina)
    pub scale: u32,
    /// Fractional scale numerator/denominator for wp_fractional_scale
    pub fractional_scale: f64,
    /// Position of this output in the compositor space (multi-monitor)
    pub x: i32,
    pub y: i32,
    pub transform: OutputTransform,
    pub is_primary: bool,
}

impl AnimusOutput {
    /// Creates an output from DRM EDID/mode data.
    pub fn new(
        id: u32,
        connector: impl Into<String>,
        width: u32,
        height: u32,
        refresh_mhz: u32,
    ) -> Self {
        let refresh_hz = refresh_mhz / 1000;
        info!(
            "AnimusOutput: New output '{}' {}x{}@{}Hz",
            id, width, height, refresh_hz
        );

        // Calculate fractional scale: if native DPI > 192, use 2.0x HiDPI
        // If 144-192 DPI, use 1.5x fractional scaling
        let fractional_scale = 1.0;

        Self {
            id,
            connector: connector.into(),
            make: "Generic".to_string(),
            model: "Display".to_string(),
            width,
            height,
            phys_width_mm: 527, // ~24" display default
            phys_height_mm: 296,
            refresh_mhz,
            scale: 1,
            fractional_scale,
            x: 0,
            y: 0,
            transform: OutputTransform::Normal,
            is_primary: id == 0,
        }
    }

    /// Pixels per inch (DPI) for this output
    pub fn dpi(&self) -> f64 {
        if self.phys_width_mm == 0 { return 96.0; }
        (self.width as f64 / (self.phys_width_mm as f64 / 25.4))
    }

    /// Effective logical resolution after scale factor
    pub fn logical_width(&self) -> u32 {
        (self.width as f64 / self.fractional_scale) as u32
    }

    pub fn logical_height(&self) -> u32 {
        (self.height as f64 / self.fractional_scale) as u32
    }

    /// Updates EDID-sourced make/model strings from a raw EDID blob.
    pub fn update_from_edid(&mut self, make: impl Into<String>, model: impl Into<String>,
                             phys_w_mm: u32, phys_h_mm: u32) {
        self.make = make.into();
        self.model = model.into();
        self.phys_width_mm = phys_w_mm;
        self.phys_height_mm = phys_h_mm;

        // Recalculate fractional scale based on actual DPI
        let dpi = self.dpi();
        if dpi >= 200.0 {
            self.scale = 2;
            self.fractional_scale = 2.0;
        } else if dpi >= 144.0 {
            self.scale = 1;
            self.fractional_scale = 1.5;
        } else {
            self.scale = 1;
            self.fractional_scale = 1.0;
        }

        info!(
            "AnimusOutput: EDID → {} {} {}x{}mm DPI={:.0} scale={:.1}x",
            self.make, self.model, phys_w_mm, phys_h_mm, dpi, self.fractional_scale
        );
    }
}

/// Manages all active outputs and their arrangement in compositor space.
pub struct OutputManager {
    pub outputs: Vec<AnimusOutput>,
}

impl OutputManager {
    pub fn new() -> Self {
        Self { outputs: Vec::new() }
    }

    /// Adds a newly detected output (called when DRM connector goes active).
    pub fn add_output(&mut self, output: AnimusOutput) {
        // Position the new output to the right of the existing ones
        if !self.outputs.is_empty() {
            let last = self.outputs.last().unwrap();
            let x_offset = last.x + last.width as i32;
            let mut new_output = output;
            new_output.x = x_offset;
            self.outputs.push(new_output);
        } else {
            self.outputs.push(output);
        }
    }

    /// Returns the primary output (first connector marked primary).
    pub fn primary(&self) -> Option<&AnimusOutput> {
        self.outputs.iter().find(|o| o.is_primary)
            .or_else(|| self.outputs.first())
    }

    /// Total bounding box of all outputs (for global cursor constraint)
    pub fn total_bounds(&self) -> (u32, u32) {
        let w = self.outputs.iter().map(|o| o.x + o.width as i32).max().unwrap_or(1920) as u32;
        let h = self.outputs.iter().map(|o| o.y + o.height as i32).max().unwrap_or(1080) as u32;
        (w, h)
    }
}

impl Default for OutputManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_dpi_and_scale() {
        let mut output = AnimusOutput::new(0, "eDP-1", 2560, 1600, 60000);
        output.update_from_edid("Apple", "MacBook Pro 14\"", 310, 195);
        // 2560 / (310mm / 25.4) ≈ 209 DPI → 2x HiDPI
        assert_eq!(output.scale, 2);
        assert_eq!(output.fractional_scale, 2.0);
        assert_eq!(output.logical_width(), 1280);
        assert_eq!(output.logical_height(), 800);
    }

    #[test]
    fn test_output_manager_multi_head() {
        let mut mgr = OutputManager::new();
        mgr.add_output(AnimusOutput::new(0, "DP-1", 1920, 1080, 144000));
        mgr.add_output(AnimusOutput::new(1, "HDMI-A-1", 1920, 1080, 60000));
        assert_eq!(mgr.outputs.len(), 2);
        assert_eq!(mgr.outputs[1].x, 1920); // Second monitor to the right
        let (w, h) = mgr.total_bounds();
        assert_eq!(w, 3840); // Two 1920px monitors side by side
        assert_eq!(h, 1080);
    }
}
