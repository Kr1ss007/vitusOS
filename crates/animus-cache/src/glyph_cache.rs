//! GlyphCache -- LRU Rasterized FreeType Glyph Bitmaps (Part 26 of spec).
//!
//! Stores rasterized glyph bitmaps in CPU memory so HarfBuzz + FreeType
//! rasterization is not repeated. Key: codepoint + ptSize + dpiScale
//! packed into u64. Latin Basic (U+0020-U+007F) is never evicted.
//! Eviction: LRU by last access time. Under Critical pressure: evict all
//! except Latin Basic (GlyphAtlas re-rasterizes on demand).

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

const LATIN_BASIC_START: u32 = 0x0020;
const LATIN_BASIC_END: u32 = 0x007F;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedGlyph {
    pub bitmap: Vec<u8>,
    pub width: u32,
    pub rows: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance_x: i32,
    pub atlas_x: u16,
    pub atlas_y: u16,
    pub in_atlas: bool,
    #[serde(skip)]
    pub last_access: Option<Instant>,
}

impl Default for CachedGlyph {
    fn default() -> Self {
        Self {
            bitmap: Vec::new(),
            width: 0,
            rows: 0,
            bearing_x: 0,
            bearing_y: 0,
            advance_x: 0,
            atlas_x: 0,
            atlas_y: 0,
            in_atlas: false,
            last_access: None,
        }
    }
}

pub struct GlyphCache {
    glyphs: Mutex<HashMap<u64, CachedGlyph>>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self { glyphs: Mutex::new(HashMap::new()) }
    }

    /// Encodes cache key: codepoint (upper 32) | ptSize Q16 (bits 8-31) | dpiScale Q8 (bits 0-7).
    pub fn make_key(codepoint: u32, pt_size: f32, dpi_scale: f32) -> u64 {
        let pt_q16 = (pt_size * 256.0) as u32;
        let dpi_q8 = (dpi_scale * 128.0) as u32;
        ((codepoint as u64) << 32) | ((pt_q16 as u64) << 8) | (dpi_q8 as u64 & 0xFF)
    }

    pub fn get(&self, key: u64) -> Option<CachedGlyph> {
        let mut glyphs = self.glyphs.lock();
        if let Some(g) = glyphs.get_mut(&key) {
            g.last_access = Some(Instant::now());
            return Some(g.clone());
        }
        None
    }

    pub fn put(&self, key: u64, mut glyph: CachedGlyph) {
        glyph.last_access = Some(Instant::now());
        self.glyphs.lock().insert(key, glyph);
    }

    pub fn mark_in_atlas(&self, key: u64, atlas_x: u16, atlas_y: u16) {
        let mut glyphs = self.glyphs.lock();
        if let Some(g) = glyphs.get_mut(&key) {
            g.atlas_x = atlas_x;
            g.atlas_y = atlas_y;
            g.in_atlas = true;
        }
    }

    /// Evicts glyphs older than threshold. Never evicts Latin Basic.
    pub fn evict_older_than(&self, age_threshold_s: f64) -> usize {
        let mut glyphs = self.glyphs.lock();
        let now = Instant::now();
        let before = glyphs.len();
        glyphs.retain(|key, g| {
            let codepoint = (key >> 32) as u32;
            if codepoint >= LATIN_BASIC_START && codepoint <= LATIN_BASIC_END {
                return true; // Never evict Latin Basic
            }
            if let Some(access) = g.last_access {
                now.duration_since(access).as_secs_f64() < age_threshold_s
            } else {
                true
            }
        });
        before - glyphs.len()
    }

    /// Evicts everything except Latin Basic.
    pub fn evict_all(&self) -> usize {
        let mut glyphs = self.glyphs.lock();
        let before = glyphs.len();
        glyphs.retain(|key, _| {
            let codepoint = (key >> 32) as u32;
            codepoint >= LATIN_BASIC_START && codepoint <= LATIN_BASIC_END
        });
        before - glyphs.len()
    }

    pub fn byte_size(&self) -> usize {
        self.glyphs.lock().values().map(|g| g.bitmap.len()).sum()
    }

    pub fn entry_count(&self) -> usize {
        self.glyphs.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_cache_put_get() {
        let cache = GlyphCache::new();
        let key = GlyphCache::make_key(0x0041, 14.0, 1.0); // 'A'
        cache.put(key, CachedGlyph { bitmap: vec![0xFF; 100], width: 10, rows: 10, ..Default::default() });
        assert!(cache.get(key).is_some());
        assert_eq!(cache.entry_count(), 1);
        assert_eq!(cache.byte_size(), 100);
    }

    #[test]
    fn test_latin_basic_never_evicted() {
        let cache = GlyphCache::new();
        let latin_key = GlyphCache::make_key(0x0041, 14.0, 1.0); // 'A'
        cache.put(latin_key, CachedGlyph::default());
        let extended_key = GlyphCache::make_key(0x00C0, 14.0, 1.0); // 'A' with grave
        cache.put(extended_key, CachedGlyph::default());

        let evicted = cache.evict_all();
        assert_eq!(evicted, 1); // Only extended evicted
        assert!(cache.get(latin_key).is_some()); // Latin Basic preserved
        assert!(cache.get(extended_key).is_none());
    }
}
