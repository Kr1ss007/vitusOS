//! SnapshotCache -- CockpitView Window Thumbnails (Part 26 of spec).
//!
//! Caches VkImage thumbnails captured via wlr_renderer_read_pixels on
//! CockpitView open. GPU memory only -- no disk persistence.
//! Always evicted on LockScreen engage (privacy).
//! Evicted under Low pressure (CockpitView not open = snapshots stale).

use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::Instant;

pub struct SnapshotEntry {
    pub width: u32,
    pub height: u32,
    pub captured_at: Instant,
}

pub struct SnapshotCache {
    snapshots: Mutex<HashMap<u64, SnapshotEntry>>,
}

impl SnapshotCache {
    pub fn new() -> Self {
        Self { snapshots: Mutex::new(HashMap::new()) }
    }

    pub fn put(&self, window_handle: u64, width: u32, height: u32) {
        self.snapshots.lock().insert(window_handle, SnapshotEntry {
            width, height, captured_at: Instant::now(),
        });
    }

    pub fn get(&self, window_handle: u64) -> Option<(u32, u32)> {
        self.snapshots.lock().get(&window_handle)
            .map(|s| (s.width, s.height))
    }

    pub fn evict_all(&self) {
        self.snapshots.lock().clear();
    }

    pub fn remove(&self, window_handle: u64) {
        self.snapshots.lock().remove(&window_handle);
    }

    pub fn count(&self) -> usize {
        self.snapshots.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_cache() {
        let cache = SnapshotCache::new();
        cache.put(1, 800, 600);
        cache.put(2, 1920, 1080);
        assert_eq!(cache.count(), 2);
        assert_eq!(cache.get(1), Some((800, 600)));
        cache.remove(1);
        assert_eq!(cache.count(), 1);
        cache.evict_all();
        assert_eq!(cache.count(), 0);
    }
}
