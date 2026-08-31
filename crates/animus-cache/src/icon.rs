//! Memory-Bounded Icon Cache.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CachedIcon {
    pub width: u32,
    pub height: u32,
    pub rgba_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct IconCache {
    icons: Arc<RwLock<HashMap<String, CachedIcon>>>,
    max_entries: usize,
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new(256)
    }
}

impl IconCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            icons: Arc::new(RwLock::new(HashMap::new())),
            max_entries,
        }
    }

    pub fn insert(&self, icon_path: String, icon: CachedIcon) {
        let mut map = self.icons.write();
        if map.len() >= self.max_entries {
            map.clear(); // Simple bounded clear
        }
        map.insert(icon_path, icon);
    }

    pub fn get(&self, icon_path: &str) -> Option<CachedIcon> {
        self.icons.read().get(icon_path).cloned()
    }

    pub fn evict_all(&self) {
        self.icons.write().clear();
    }
}
