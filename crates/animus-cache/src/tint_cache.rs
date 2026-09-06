//! TintCache -- Wallpaper TintSampler OKLab k-means Results (Part 26 of spec).
//!
//! Caches WallpaperTintSampler results per wallpaper path. Max 8 entries.
//! Eviction: path-based -- new wallpaper set makes old entries irrelevant.
//! Under Critical pressure: evict all. TintSampler recomputes on demand.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

const MAX_ENTRIES: usize = 8;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TintResult {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub luminosity_boost: f32,
    pub chroma_reduce: f32,
    pub alpha: f32,
}

struct Entry {
    path: String,
    result: TintResult,
}

pub struct TintCache {
    entries: Mutex<Vec<Entry>>,
}

impl TintCache {
    pub fn new() -> Self {
        Self { entries: Mutex::new(Vec::new()) }
    }

    pub fn get(&self, wallpaper_path: &str) -> Option<TintResult> {
        self.entries.lock().iter()
            .find(|e| e.path == wallpaper_path)
            .map(|e| e.result)
    }

    pub fn put(&self, wallpaper_path: &str, result: TintResult) {
        let mut entries = self.entries.lock();
        entries.retain(|e| e.path != wallpaper_path);
        entries.push(Entry { path: wallpaper_path.to_string(), result });
        if entries.len() > MAX_ENTRIES {
            entries.remove(0); // FIFO eviction
        }
    }

    pub fn evict_all(&self) {
        self.entries.lock().clear();
    }

    pub fn evict_stale(&self, current_wallpaper: &str) {
        self.entries.lock().retain(|e| e.path == current_wallpaper);
    }

    pub fn entry_count(&self) -> usize {
        self.entries.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tint_cache_put_get() {
        let cache = TintCache::new();
        let result = TintResult { r: 0.5, g: 0.3, b: 0.1, luminosity_boost: 0.08, chroma_reduce: 0.15, alpha: 0.22 };
        cache.put("/usr/share/wallpapers/mars.jpg", result);
        assert!(cache.get("/usr/share/wallpapers/mars.jpg").is_some());
        assert!(cache.get("/usr/share/wallpapers/other.jpg").is_none());
    }

    #[test]
    fn test_tint_cache_max_entries() {
        let cache = TintCache::new();
        for i in 0..12 {
            cache.put(&format!("/wp/{}.jpg", i), TintResult { r: 0.0, g: 0.0, b: 0.0, luminosity_boost: 0.0, chroma_reduce: 0.0, alpha: 0.0 });
        }
        assert_eq!(cache.entry_count(), MAX_ENTRIES);
        // Oldest entries evicted (FIFO)
        assert!(cache.get("/wp/0.jpg").is_none());
        assert!(cache.get("/wp/11.jpg").is_some());
    }

    #[test]
    fn test_tint_cache_evict_stale() {
        let cache = TintCache::new();
        cache.put("/wp/a.jpg", TintResult { r: 0.0, g: 0.0, b: 0.0, luminosity_boost: 0.0, chroma_reduce: 0.0, alpha: 0.0 });
        cache.put("/wp/b.jpg", TintResult { r: 0.0, g: 0.0, b: 0.0, luminosity_boost: 0.0, chroma_reduce: 0.0, alpha: 0.0 });
        cache.evict_stale("/wp/a.jpg");
        assert_eq!(cache.entry_count(), 1);
        assert!(cache.get("/wp/a.jpg").is_some());
        assert!(cache.get("/wp/b.jpg").is_none());
    }
}
