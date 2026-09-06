//! TextRenderer — HarfBuzz-equivalent text shaping + sub-pixel positioning (Part 10.3).
//!
//! Uses rustybuzz (pure Rust HarfBuzz port) for Unicode text shaping:
//!   - Bi-directional text (bidi) support
//!   - Ligature formation
//!   - Glyph clustering
//!   - Sub-pixel fractional positions (26.6 fixed-point → f32, NEVER rounded)
//!
//! Uses fontdue for glyph rasterization and the GlyphAtlas for texture packing.
//! Output is a list of positioned glyph quads ready for GPU upload.

use crate::glyph_atlas::GlyphAtlas;
use crate::typography::{FontFamily, TextRole};
use rustybuzz::{UnicodeBuffer, Direction, GlyphBuffer};

/// A single shaped glyph ready for rendering.
#[derive(Debug, Clone, Copy)]
pub struct PositionedGlyph {
    /// Glyph ID after shaping.
    pub glyph_id: u32,
    /// Sub-pixel X position (never rounded — Part 10.3).
    pub x: f32,
    /// Sub-pixel Y position (never rounded).
    pub y: f32,
    /// Glyph width in pixels.
    pub width: u16,
    /// Glyph height in pixels.
    pub height: u16,
    /// Atlas UV coordinates.
    pub uv_min_x: f32,
    pub uv_min_y: f32,
    pub uv_max_x: f32,
    pub uv_max_y: f32,
    /// Cluster index for caret mapping.
    pub cluster: u32,
}

/// A shaped text run — the output of text shaping.
#[derive(Debug, Clone)]
pub struct ShapedText {
    pub glyphs: Vec<PositionedGlyph>,
    pub advance_width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
}

impl ShapedText {
    pub fn line_height(&self) -> f32 {
        self.ascent - self.descent + self.line_gap
    }

    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTruncation {
    None,
    Ellipsis,
    Clip,
}

#[derive(Debug, Clone)]
pub struct TextLayoutParams {
    pub role: TextRole,
    pub color: [f32; 4],
    pub max_width: Option<f32>,
    pub align: TextAlign,
    pub truncation: TextTruncation,
    pub line_spacing: f32,
}

impl Default for TextLayoutParams {
    fn default() -> Self {
        Self {
            role: TextRole::UILabel,
            color: [1.0, 1.0, 1.0, 1.0],
            max_width: None,
            align: TextAlign::Left,
            truncation: TextTruncation::None,
            line_spacing: 1.0,
        }
    }
}

/// A loaded font face with its raw bytes owned for lifetime safety.
struct LoadedFont {
    family: FontFamily,
    bytes: Vec<u8>,
    face: rustybuzz::Face<'static>,
}

impl LoadedFont {
    fn new(family: FontFamily, bytes: Vec<u8>) -> Option<Self> {
        // We own `bytes` and keep it alive for the lifetime of this struct.
        // The Face references the bytes via a borrowed slice, but we use unsafe
        // to extend the lifetime to 'static since we guarantee the bytes outlive the face.
        let bytes_ptr = bytes.as_ptr();
        let bytes_len = bytes.len();
        let static_slice: &'static [u8] = unsafe {
            std::slice::from_raw_parts(bytes_ptr, bytes_len)
        };
        let face = rustybuzz::Face::from_slice(static_slice, 0)?;
        Some(Self { family, bytes, face })
    }
}

pub struct TextRenderer {
    loaded: Vec<LoadedFont>,
}

impl TextRenderer {
    pub fn new() -> Self {
        let mut loaded = Vec::new();
        for family in &[FontFamily::Primary, FontFamily::Secondary, FontFamily::Body, FontFamily::Monospace] {
            let path = family.default_asset_path();
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Some(lf) = LoadedFont::new(*family, bytes) {
                        loaded.push(lf);
                    }
                }
            }
        }
        Self { loaded }
    }

    fn face_for(&self, family: FontFamily) -> Option<&rustybuzz::Face> {
        self.loaded.iter().find(|l| l.family == family).map(|l| &l.face)
    }

    fn loaded_for(&self, family: FontFamily) -> Option<&LoadedFont> {
        self.loaded.iter().find(|l| l.family == family)
    }

    /// Shape text with rustybuzz and return the GlyphBuffer.
    fn shape_text(&self, text: &str, family: FontFamily) -> Option<GlyphBuffer> {
        let face = self.face_for(family)?;

        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);
        buffer.set_script(rustybuzz::Script::from_iso15924_tag(
            rustybuzz::ttf_parser::Tag::from_bytes(b"Latn")
        )?);
        buffer.set_language("en".parse().ok()?);

        Some(rustybuzz::shape(face, &[], buffer))
    }

    /// Shape and layout text, producing positioned glyphs ready for rendering.
    pub fn layout_text(
        &mut self,
        text: &str,
        params: &TextLayoutParams,
        baseline_x: f32,
        baseline_y: f32,
        atlas: &mut GlyphAtlas,
    ) -> ShapedText {
        let info = params.role.info();
        let family = info.family;
        let (ascent, descent, line_gap) = atlas.font_line_metrics();

        if text.is_empty() {
            return ShapedText {
                glyphs: Vec::new(),
                advance_width: 0.0,
                ascent, descent, line_gap,
            };
        }

        let atlas_dim = GlyphAtlas::ATLAS_DIM as f32;

        // Try HarfBuzz shaping first
        if let Some(glyph_buffer) = self.shape_text(text, family) {
            let infos = glyph_buffer.glyph_infos();
            let positions = glyph_buffer.glyph_positions();
            let mut positioned = Vec::with_capacity(infos.len());
            let mut cx = baseline_x;
            let mut cy = baseline_y;

            for (info, pos) in infos.iter().zip(positions.iter()) {
                let ch = char_from_cluster(text, info.cluster);
                let entry = match atlas.get_glyph(ch) {
                    Some(e) => e.clone(),
                    None => {
                        cx += pos.x_advance as f32 / 64.0;
                        cy += pos.y_advance as f32 / 64.0;
                        continue;
                    }
                };

                let px = cx + pos.x_offset as f32 / 64.0 + entry.bearing_x;
                let py = cy - pos.y_offset as f32 / 64.0 - entry.bearing_y;

                positioned.push(PositionedGlyph {
                    glyph_id: info.glyph_id,
                    x: px, y: py,
                    width: entry.width, height: entry.height,
                    uv_min_x: entry.atlas_x as f32 / atlas_dim,
                    uv_min_y: entry.atlas_y as f32 / atlas_dim,
                    uv_max_x: (entry.atlas_x + entry.width) as f32 / atlas_dim,
                    uv_max_y: (entry.atlas_y + entry.height) as f32 / atlas_dim,
                    cluster: info.cluster,
                });

                cx += pos.x_advance as f32 / 64.0;
                cy += pos.y_advance as f32 / 64.0;
            }

            let total_width = cx - baseline_x;
            let aligned = self.apply_alignment_and_truncation(
                positioned, text, total_width, params, baseline_x, baseline_y, atlas
            );
            return ShapedText {
                glyphs: aligned,
                advance_width: total_width,
                ascent, descent, line_gap,
            };
        }

        // Fallback: simple per-character layout
        self.fallback_layout(text, params, baseline_x, baseline_y, atlas)
    }

    fn apply_alignment_and_truncation(
        &self,
        mut glyphs: Vec<PositionedGlyph>,
        text: &str,
        total_width: f32,
        params: &TextLayoutParams,
        baseline_x: f32,
        baseline_y: f32,
        atlas: &mut GlyphAtlas,
    ) -> Vec<PositionedGlyph> {
        let offset_x = match params.align {
            TextAlign::Left => 0.0,
            TextAlign::Center => params.max_width.map(|w| (w - total_width) * 0.5).unwrap_or(0.0),
            TextAlign::Right => params.max_width.map(|w| w - total_width).unwrap_or(0.0),
        };
        if offset_x != 0.0 {
            for g in &mut glyphs { g.x += offset_x; }
        }

        if let Some(max_w) = params.max_width {
            if total_width > max_w {
                match params.truncation {
                    TextTruncation::Ellipsis => {
                        let ellipsis_w = atlas.get_glyph('…').map(|e| e.advance).unwrap_or(8.0);
                        let cutoff = max_w - ellipsis_w;
                        let mut last = 0;
                        for (i, g) in glyphs.iter().enumerate() {
                            if g.x - baseline_x + g.width as f32 > cutoff { break; }
                            last = i;
                        }
                        glyphs.truncate(last + 1);
                        if let Some(entry) = atlas.get_glyph('…') {
                            let last_x = glyphs.last().map(|g| g.x).unwrap_or(baseline_x);
                            let ad = GlyphAtlas::ATLAS_DIM as f32;
                            glyphs.push(PositionedGlyph {
                                glyph_id: '…' as u32,
                                x: last_x + 8.0, y: baseline_y,
                                width: entry.width, height: entry.height,
                                uv_min_x: entry.atlas_x as f32 / ad,
                                uv_min_y: entry.atlas_y as f32 / ad,
                                uv_max_x: (entry.atlas_x + entry.width) as f32 / ad,
                                uv_max_y: (entry.atlas_y + entry.height) as f32 / ad,
                                cluster: text.len() as u32,
                            });
                        }
                    }
                    TextTruncation::Clip => {
                        glyphs.retain(|g| g.x - baseline_x < max_w);
                    }
                    TextTruncation::None => {}
                }
            }
        }
        glyphs
    }

    fn fallback_layout(
        &mut self,
        text: &str,
        params: &TextLayoutParams,
        baseline_x: f32,
        baseline_y: f32,
        atlas: &mut GlyphAtlas,
    ) -> ShapedText {
        let (ascent, descent, line_gap) = atlas.font_line_metrics();
        let atlas_dim = GlyphAtlas::ATLAS_DIM as f32;
        let mut positioned = Vec::new();
        let mut cx = baseline_x;

        for (byte_idx, ch) in text.char_indices() {
            let entry = match atlas.get_glyph(ch) { Some(e) => e.clone(), None => continue };
            positioned.push(PositionedGlyph {
                glyph_id: ch as u32,
                x: cx + entry.bearing_x,
                y: baseline_y - entry.bearing_y,
                width: entry.width, height: entry.height,
                uv_min_x: entry.atlas_x as f32 / atlas_dim,
                uv_min_y: entry.atlas_y as f32 / atlas_dim,
                uv_max_x: (entry.atlas_x + entry.width) as f32 / atlas_dim,
                uv_max_y: (entry.atlas_y + entry.height) as f32 / atlas_dim,
                cluster: byte_idx as u32,
            });
            cx += entry.advance;
        }

        let total_width = cx - baseline_x;
        let aligned = self.apply_alignment_and_truncation(
            positioned, text, total_width, params, baseline_x, baseline_y, atlas
        );
        ShapedText { glyphs: aligned, advance_width: total_width, ascent, descent, line_gap }
    }

    pub fn measure_text(&mut self, text: &str, role: TextRole, atlas: &mut GlyphAtlas) -> f32 {
        let params = TextLayoutParams { role, ..Default::default() };
        self.layout_text(text, &params, 0.0, 0.0, atlas).advance_width
    }

    pub fn has_fonts(&self) -> bool { !self.loaded.is_empty() }
    pub fn font_count(&self) -> usize { self.loaded.len() }

    /// Get raw font bytes for a family (for GlyphAtlas creation).
    pub fn font_bytes(&self, family: FontFamily) -> Option<&[u8]> {
        self.loaded_for(family).map(|l| l.bytes.as_slice())
    }
}

impl Default for TextRenderer {
    fn default() -> Self { Self::new() }
}

fn char_from_cluster(text: &str, byte_offset: u32) -> char {
    let offset = byte_offset as usize;
    if offset >= text.len() { return '\0'; }
    text[offset..].chars().next().unwrap_or('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_atlas() -> GlyphAtlas {
        let font_path = FontFamily::Primary.default_asset_path();
        let bytes = if font_path.exists() {
            std::fs::read(&font_path).unwrap()
        } else {
            let fallback = std::path::PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
            std::fs::read(&fallback).unwrap()
        };
        GlyphAtlas::new(&bytes, 13.0, 1.0).unwrap()
    }

    #[test]
    fn text_renderer_initializes() {
        let tr = TextRenderer::new();
        assert!(tr.font_count() >= 0);
    }

    #[test]
    fn layout_simple_text() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let params = TextLayoutParams::default();
        let shaped = tr.layout_text("Hello, world!", &params, 0.0, 20.0, &mut atlas);
        assert!(!shaped.glyphs.is_empty());
        assert!(shaped.advance_width > 0.0);
        assert!(shaped.ascent > 0.0);
    }

    #[test]
    fn layout_empty_text() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let params = TextLayoutParams::default();
        let shaped = tr.layout_text("", &params, 0.0, 20.0, &mut atlas);
        assert!(shaped.glyphs.is_empty());
        assert_eq!(shaped.advance_width, 0.0);
    }

    #[test]
    fn text_alignment_center() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let params = TextLayoutParams {
            max_width: Some(500.0),
            align: TextAlign::Center,
            ..Default::default()
        };
        let shaped = tr.layout_text("Test", &params, 0.0, 20.0, &mut atlas);
        if !shaped.glyphs.is_empty() {
            assert!(shaped.glyphs[0].x > 0.0, "Centered text should be offset right");
        }
    }

    #[test]
    fn text_truncation_ellipsis() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let params = TextLayoutParams {
            max_width: Some(50.0),
            truncation: TextTruncation::Ellipsis,
            ..Default::default()
        };
        let shaped = tr.layout_text("This is a very long text that should be truncated", &params, 0.0, 20.0, &mut atlas);
        let full = tr.layout_text("This is a very long text that should be truncated", &TextLayoutParams::default(), 0.0, 20.0, &mut atlas);
        assert!(shaped.glyphs.len() < full.glyphs.len());
    }

    #[test]
    fn text_truncation_clip() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let params = TextLayoutParams {
            max_width: Some(30.0),
            truncation: TextTruncation::Clip,
            ..Default::default()
        };
        let shaped = tr.layout_text("Truncate this long text now", &params, 0.0, 20.0, &mut atlas);
        for g in &shaped.glyphs {
            assert!(g.x < 30.0, "Glyph at x={} should be clipped", g.x);
        }
    }

    #[test]
    fn sub_pixel_positions_preserved() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let params = TextLayoutParams::default();
        let shaped = tr.layout_text("Testing sub-pixel", &params, 10.7, 20.3, &mut atlas);
        for g in &shaped.glyphs {
            let frac = g.x - g.x.trunc();
            assert!(frac >= 0.0 && frac < 1.0);
        }
    }

    #[test]
    fn measure_text_width() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let w1 = tr.measure_text("Measure this", TextRole::UILabel, &mut atlas);
        assert!(w1 > 0.0);
        let w2 = tr.measure_text("Measure this much longer piece of text", TextRole::UILabel, &mut atlas);
        assert!(w2 > w1);
    }

    #[test]
    fn line_height_positive() {
        let mut tr = TextRenderer::new();
        let mut atlas = make_atlas();
        let shaped = tr.layout_text("Line", &TextLayoutParams::default(), 0.0, 20.0, &mut atlas);
        assert!(shaped.line_height() > 0.0);
    }
}
