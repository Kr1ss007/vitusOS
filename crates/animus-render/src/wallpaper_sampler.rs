//! WallpaperTintSampler — 16-Bin Adaptive Glass Tinting & Contrast (Part 11 of spec).
//!
//! Samples wallpaper luminance and chrominance to dynamically tailor Kawase glass tinting
//! and maintain legible text contrast.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperMetrics {
    pub average_luminance: f32, // 0.0 (pitch black) to 1.0 (pure white)
    pub dominant_tint_rgba: [f32; 4],
    pub is_dark_wallpaper: bool,
    pub recommended_text_color: [f32; 4],
}

impl Default for WallpaperMetrics {
    fn default() -> Self {
        Self {
            average_luminance: 0.18,
            dominant_tint_rgba: [0.10, 0.10, 0.12, 0.65],
            is_dark_wallpaper: true,
            recommended_text_color: [0.996, 0.996, 0.996, 1.0], // #FEFEFE (FIX4-06)
        }
    }
}

pub struct WallpaperTintSampler {
    pub metrics: WallpaperMetrics,
}

impl WallpaperTintSampler {
    pub fn new() -> Self {
        Self {
            metrics: WallpaperMetrics::default(),
        }
    }

    /// Samples a 16-bin color grid from raw RGBA pixel data.
    pub fn sample_image(&mut self, width: u32, height: u32, rgba_pixels: &[u8]) {
        if rgba_pixels.len() < (width * height * 4) as usize || width == 0 || height == 0 {
            return;
        }

        let mut total_lum = 0.0f32;
        let mut r_sum = 0.0f32;
        let mut g_sum = 0.0f32;
        let mut b_sum = 0.0f32;

        let sample_count = 16;
        for i in 0..sample_count {
            let x = ((i % 4) as f32 + 0.5) / 4.0 * width as f32;
            let y = ((i / 4) as f32 + 0.5) / 4.0 * height as f32;
            let idx = ((y as u32 * width + x as u32) * 4) as usize;

            if idx + 3 < rgba_pixels.len() {
                let r = rgba_pixels[idx] as f32 / 255.0;
                let g = rgba_pixels[idx + 1] as f32 / 255.0;
                let b = rgba_pixels[idx + 2] as f32 / 255.0;

                // ITU-R BT.709 relative luminance formula
                let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                total_lum += lum;
                r_sum += r;
                g_sum += g;
                b_sum += b;
            }
        }

        let avg_lum = total_lum / sample_count as f32;
        let avg_r = r_sum / sample_count as f32;
        let avg_g = g_sum / sample_count as f32;
        let avg_b = b_sum / sample_count as f32;

        let is_dark = avg_lum < 0.55;
        let text_color = if is_dark {
            [0.996, 0.996, 0.996, 1.0] // #FEFEFE
        } else {
            [0.110, 0.110, 0.118, 1.0] // #1C1C1E
        };

        self.metrics = WallpaperMetrics {
            average_luminance: avg_lum,
            dominant_tint_rgba: [avg_r * 0.4, avg_g * 0.4, avg_b * 0.4, 0.70],
            is_dark_wallpaper: is_dark,
            recommended_text_color: text_color,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallpaper_tint_sampler_luminance() {
        let mut sampler = WallpaperTintSampler::new();
        // Create 4x4 black image
        let black_pixels = vec![0u8; 4 * 4 * 4];
        sampler.sample_image(4, 4, &black_pixels);
        assert!(sampler.metrics.is_dark_wallpaper);
        assert_eq!(sampler.metrics.recommended_text_color, [0.996, 0.996, 0.996, 1.0]);

        // Create 4x4 bright white image
        let white_pixels = vec![255u8; 4 * 4 * 4];
        sampler.sample_image(4, 4, &white_pixels);
        assert!(!sampler.metrics.is_dark_wallpaper);
        assert_eq!(sampler.metrics.recommended_text_color, [0.110, 0.110, 0.118, 1.0]);
    }
}
