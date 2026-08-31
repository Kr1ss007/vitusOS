use std::collections::HashMap;
use fontdue::{Font, FontSettings};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AtlasError {
    #[error("Failed to parse font: {0}")]
    ParseError(&'static str),
    #[error("Glyph atlas is full (2048x2048 limit reached)")]
    AtlasFull,
}

#[derive(Debug, Clone)]
pub struct GlyphEntry {
    pub atlas_x: u16,
    pub atlas_y: u16,
    pub width: u16,
    pub height: u16,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
}

pub struct GlyphAtlas {
    font: Font,
    pt_size: f32,
    dpi_scale: f32,
    atlas_width: usize,
    atlas_height: usize,
    atlas_data: Vec<u8>,
    cursor_x: usize,
    cursor_y: usize,
    current_row_height: usize,
    glyphs: HashMap<char, GlyphEntry>,
}

impl GlyphAtlas {
    pub const ATLAS_DIM: usize = 2048;

    pub fn new(font_bytes: &[u8], pt_size: f32, dpi_scale: f32) -> Result<Self, AtlasError> {
        let settings = FontSettings {
            scale: pt_size * dpi_scale,
            ..FontSettings::default()
        };
        let font = Font::from_bytes(font_bytes, settings)
            .map_err(AtlasError::ParseError)?;

        let mut atlas = Self {
            font,
            pt_size,
            dpi_scale,
            atlas_width: Self::ATLAS_DIM,
            atlas_height: Self::ATLAS_DIM,
            atlas_data: vec![0u8; Self::ATLAS_DIM * Self::ATLAS_DIM],
            cursor_x: 1,
            cursor_y: 1,
            current_row_height: 0,
            glyphs: HashMap::new(),
        };

        // Pre-rasterize basic printable ASCII + Latin Extended-1 (U+0020 to U+00FF)
        for cp in 0x0020u32..=0x00FFu32 {
            if let Some(ch) = std::char::from_u32(cp) {
                let _ = atlas.rasterize_glyph(ch);
            }
        }

        Ok(atlas)
    }

    pub fn get_glyph(&mut self, ch: char) -> Option<&GlyphEntry> {
        if !self.glyphs.contains_key(&ch) {
            let _ = self.rasterize_glyph(ch);
        }
        self.glyphs.get(&ch)
    }

    pub fn rasterize_glyph(&mut self, ch: char) -> Result<&GlyphEntry, AtlasError> {
        if self.glyphs.contains_key(&ch) {
            return Ok(self.glyphs.get(&ch).unwrap());
        }

        let px_size = self.pt_size * self.dpi_scale;
        let (metrics, bitmap) = self.font.rasterize(ch, px_size);

        if self.cursor_x + metrics.width + 1 > self.atlas_width {
            self.cursor_x = 1;
            self.cursor_y += self.current_row_height + 2;
            self.current_row_height = 0;
        }

        if self.cursor_y + metrics.height + 1 > self.atlas_height {
            return Err(AtlasError::AtlasFull);
        }

        let atlas_x = self.cursor_x as u16;
        let atlas_y = self.cursor_y as u16;

        // Copy rasterized subpixel coverage bitmap into 2048x2048 texture buffer
        for row in 0..metrics.height {
            let src_offset = row * metrics.width;
            let dst_offset = (self.cursor_y + row) * self.atlas_width + self.cursor_x;
            self.atlas_data[dst_offset..dst_offset + metrics.width]
                .copy_from_slice(&bitmap[src_offset..src_offset + metrics.width]);
        }

        self.cursor_x += metrics.width + 2;
        if metrics.height > self.current_row_height {
            self.current_row_height = metrics.height;
        }

        let entry = GlyphEntry {
            atlas_x,
            atlas_y,
            width: metrics.width as u16,
            height: metrics.height as u16,
            bearing_x: metrics.bounds.xmin,
            bearing_y: metrics.bounds.ymin,
            advance: metrics.advance_width,
        };

        self.glyphs.insert(ch, entry);
        Ok(self.glyphs.get(&ch).unwrap())
    }

    pub fn atlas_bytes(&self) -> &[u8] {
        &self.atlas_data
    }

    pub fn font_line_metrics(&self) -> (f32, f32, f32) {
        let px_size = self.pt_size * self.dpi_scale;
        if let Some(metrics) = self.font.horizontal_line_metrics(px_size) {
            (metrics.ascent, metrics.descent, metrics.line_gap)
        } else {
            (px_size * 0.8, -px_size * 0.2, px_size * 0.2)
        }
    }

    /// Gamma-correct linear blending (macOS CoreText model).
    /// Blends text coverage in linear light space to prevent dark halos and spindly text on glass.
    #[inline]
    pub fn blend_gamma_correct(bg_srgb: [f32; 4], fg_srgb: [f32; 4], coverage: f32) -> [f32; 4] {
        let to_linear = |c: f32| -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        let to_srgb = |c: f32| -> f32 {
            if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        };

        let bg_lin = [to_linear(bg_srgb[0]), to_linear(bg_srgb[1]), to_linear(bg_srgb[2])];
        let fg_lin = [to_linear(fg_srgb[0]), to_linear(fg_srgb[1]), to_linear(fg_srgb[2])];

        let out_r = bg_lin[0] * (1.0 - coverage) + fg_lin[0] * coverage;
        let out_g = bg_lin[1] * (1.0 - coverage) + fg_lin[1] * coverage;
        let out_b = bg_lin[2] * (1.0 - coverage) + fg_lin[2] * coverage;

        [
            to_srgb(out_r).clamp(0.0, 1.0),
            to_srgb(out_g).clamp(0.0, 1.0),
            to_srgb(out_b).clamp(0.0, 1.0),
            fg_srgb[3],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamma_correct_blending() {
        let white_bg = [1.0, 1.0, 1.0, 1.0];
        let black_fg = [0.0, 0.0, 0.0, 1.0];

        // Mid-gray 50% coverage
        let blended = GlyphAtlas::blend_gamma_correct(white_bg, black_fg, 0.5);
        assert!((blended[0] - 0.735).abs() < 0.05, "Linear blending must match perceptual gamma curve");
        assert_eq!(blended[3], 1.0);
    }
}
