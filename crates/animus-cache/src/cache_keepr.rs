//! CacheKeepr -- Cross-Subsystem Memory Cache Manager (Part 26 of spec).
//!
//! Peer-level component. Not owned by CrashManager but CrashManager has
//! eviction authority. Initialized before Shell, after AnimusEngine
//! subsystems exist but before any need cached data.
//!
//! Owns: GlyphCache, ShaderCache, TintCache, AppIndexCache, IconCache, SnapshotCache.
//! Pressure response: Low/Medium/Critical tiers with progressive eviction.
//! Invalidation: Store path change IS the invalidation signal.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, warn};

use crate::glyph_cache::GlyphCache;
use crate::icon::IconCache;
use crate::shader_cache::ShaderCache;
use crate::snapshot_cache::SnapshotCache;
use crate::tint_cache::TintCache;
use crate::app_index::AppIndexCache;

/// Pressure levels from CrashManager (Part 21.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PressureLevel {
    Normal = 0,
    Low = 1,
    Medium = 2,
    Critical = 3,
}

/// Cache status for Supervisor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStatus {
    pub glyphs_bytes: usize,
    pub shaders_bytes: usize,
    pub tints_count: usize,
    pub apps_count: usize,
    pub icons_count: usize,
    pub snapshots_count: usize,
    pub total_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f32,
    pub last_pressure: u8,
}

/// Eviction statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvictionStats {
    pub glyphs_evicted: usize,
    pub icons_evicted: usize,
    pub tints_evicted: usize,
    pub snapshots_evicted: usize,
}

pub struct CacheKeepr {
    pub glyphs: GlyphCache,
    pub shaders: ShaderCache,
    pub tints: TintCache,
    pub apps: AppIndexCache,
    pub icons: IconCache,
    pub snapshots: SnapshotCache,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    last_pressure: Mutex<PressureLevel>,
}

impl CacheKeepr {
    pub fn new(cache_dir: &str, store_path: &str) -> Self {
        let shader_path = format!("{}/shader-pipeline.bin", cache_dir);
        Self {
            glyphs: GlyphCache::new(),
            shaders: ShaderCache::new(shader_path, store_path),
            tints: TintCache::new(),
            apps: AppIndexCache::new(),
            icons: IconCache::default(),
            snapshots: SnapshotCache::new(),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            last_pressure: Mutex::new(PressureLevel::Normal),
        }
    }

    pub fn record_hit(&self) { self.hit_count.fetch_add(1, Ordering::Relaxed); }
    pub fn record_miss(&self) { self.miss_count.fetch_add(1, Ordering::Relaxed); }

    /// Pressure-based eviction (Part 26.3).
    pub fn on_pressure_changed(&self, level: PressureLevel) -> EvictionStats {
        *self.last_pressure.lock() = level;
        let mut stats = EvictionStats::default();

        match level {
            PressureLevel::Low => {
                // Evict stale tints and old snapshots
                self.tints.evict_all();
                self.snapshots.evict_all();
                stats.tints_evicted = 1;
                stats.snapshots_evicted = 1;
                info!("CacheKeepr: Low pressure -- evicted tints + snapshots");
            }
            PressureLevel::Medium => {
                // Evict all icons and tints
                self.icons.evict_all();
                self.tints.evict_all();
                stats.icons_evicted = 1;
                stats.tints_evicted = 1;
                info!("CacheKeepr: Medium pressure -- evicted icons + tints");
            }
            PressureLevel::Critical => {
                // Maximum eviction: keep only AppIndexCache + ShaderCache
                let g = self.glyphs.evict_all();
                self.icons.evict_all();
                self.tints.evict_all();
                self.snapshots.evict_all();
                stats.glyphs_evicted = g;
                stats.icons_evicted = 1;
                stats.tints_evicted = 1;
                stats.snapshots_evicted = 1;
                info!("CacheKeepr: Critical pressure -- evicted glyphs + icons + tints + snapshots (kept apps + shaders)");
            }
            PressureLevel::Normal => {}
        }

        stats
    }

    /// Store path changed -- invalidate affected caches (Part 26.3).
    pub fn on_store_path_changed(&self, component: &str, new_path: &str) {
        match component {
            "animus-engine" => {
                self.shaders.update_store_path(new_path);
                self.glyphs.evict_all();
                warn!("CacheKeepr: Store path changed for {} -- invalidated shaders + glyphs", component);
            }
            "apps" => {
                self.apps.invalidate();
                self.icons.evict_all();
                warn!("CacheKeepr: Store path changed for apps -- invalidated app index + icons");
            }
            "fonts" => {
                self.glyphs.evict_all();
                warn!("CacheKeepr: Store path changed for fonts -- invalidated glyphs");
            }
            "icons" => {
                self.icons.evict_all();
                warn!("CacheKeepr: Store path changed for icons -- invalidated icon cache");
            }
            _ => {}
        }
    }

    /// Called on LockScreen engage -- privacy (Part 26).
    pub fn on_lock_screen(&self) {
        self.snapshots.evict_all();
        info!("CacheKeepr: LockScreen engage -- evicted all snapshots (privacy)");
    }

    pub fn status(&self) -> CacheStatus {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let total = hits + misses;
        let glyphs_bytes = self.glyphs.byte_size();
        let shaders_bytes = self.shaders.byte_size();
        let total_bytes = glyphs_bytes + shaders_bytes;

        CacheStatus {
            glyphs_bytes,
            shaders_bytes,
            tints_count: self.tints.entry_count(),
            apps_count: 0, // AppIndexCache doesn't expose count yet
            icons_count: 0,
            snapshots_count: self.snapshots.count(),
            total_bytes,
            hit_count: hits,
            miss_count: misses,
            hit_rate: if total > 0 { hits as f32 / total as f32 } else { 1.0 },
            last_pressure: *self.last_pressure.lock() as u8,
        }
    }

    pub fn save_all(&self) {
        self.shaders.save_to_disk();
        info!("CacheKeepr: Saved persistent caches to disk");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_keepr_pressure_eviction() {
        let ck = CacheKeepr::new("/tmp/vitusos_test", "/usr/bin/test");

        // Add some data
        ck.glyphs.put(GlyphCache::make_key(0x0041, 14.0, 1.0),
            crate::glyph_cache::CachedGlyph::default());
        ck.tints.put("/wp/test.jpg", TintResult { r: 0.5, g: 0.3, b: 0.1, luminosity_boost: 0.08, chroma_reduce: 0.15, alpha: 0.22 });
        ck.snapshots.put(1, 800, 600);

        // Low pressure: evicts tints + snapshots
        use crate::tint_cache::TintResult;
        let _stats = ck.on_pressure_changed(PressureLevel::Low);
        assert_eq!(ck.tints.entry_count(), 0);
        assert_eq!(ck.snapshots.count(), 0);
    }

    #[test]
    fn test_cache_keepr_lock_screen() {
        let ck = CacheKeepr::new("/tmp/vitusos_test2", "/usr/bin/test");
        ck.snapshots.put(1, 800, 600);
        ck.snapshots.put(2, 1920, 1080);
        assert_eq!(ck.snapshots.count(), 2);

        ck.on_lock_screen();
        assert_eq!(ck.snapshots.count(), 0);
    }

    #[test]
    fn test_cache_keepr_hit_miss() {
        let ck = CacheKeepr::new("/tmp/vitusos_test3", "/usr/bin/test");
        ck.record_hit();
        ck.record_hit();
        ck.record_miss();

        let status = ck.status();
        assert_eq!(status.hit_count, 2);
        assert_eq!(status.miss_count, 1);
        assert!((status.hit_rate - (2.0 / 3.0)).abs() < 0.01);
    }
}
