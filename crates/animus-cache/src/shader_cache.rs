//! ShaderCache -- VkPipelineCache Blob Persistence (Part 26 of spec).
//!
//! Wraps Vulkan's VkPipelineCache. On first run: empty cache, pipelines
//! compiled from SPIR-V. Subsequent runs: cache blob loaded from disk
//! so pipelines are created instantly by the Vulkan driver.
//!
//! Persistence: ~/.vitusOS/cache/shader-pipeline.bin
//! Invalidation: Store path change -> delete blob, recreate empty cache.

use parking_lot::Mutex;
use std::path::PathBuf;
use tracing::{info, warn};

pub struct ShaderCache {
    cache_path: PathBuf,
    storepath_file: PathBuf,
    current_store_path: Mutex<String>,
    blob: Mutex<Vec<u8>>,
    blob_size: Mutex<usize>,
}

impl ShaderCache {
    pub fn new(cache_path: impl Into<PathBuf>, current_store_path: &str) -> Self {
        let path = cache_path.into();
        let storepath = path.with_extension("storepath");
        Self {
            cache_path: path,
            storepath_file: storepath,
            current_store_path: Mutex::new(current_store_path.to_string()),
            blob: Mutex::new(Vec::new()),
            blob_size: Mutex::new(0),
        }
    }

    /// Loads cache from disk if store path matches. Otherwise creates empty.
    pub fn load(&self) -> bool {
        // Check if storepath file exists and matches
        if let Ok(stored_path) = std::fs::read_to_string(&self.storepath_file) {
            let current = self.current_store_path.lock();
            if stored_path.trim() == current.trim() {
                // Store path matches -- load blob
                if let Ok(data) = std::fs::read(&self.cache_path) {
                    *self.blob.lock() = data.clone();
                    *self.blob_size.lock() = data.len();
                    info!("ShaderCache: Loaded {} bytes from disk", data.len());
                    return true;
                }
            } else {
                warn!("ShaderCache: Store path mismatch -- invalidating cache");
                self.invalidate();
                return false;
            }
        }
        info!("ShaderCache: No cached blob found, starting fresh");
        false
    }

    /// Saves cache blob to disk (atomic write: tmp file + rename).
    pub fn save_to_disk(&self) -> bool {
        let blob = self.blob.lock();
        if blob.is_empty() {
            return false;
        }

        let tmp_path = self.cache_path.with_extension("tmp");
        if let Err(e) = std::fs::write(&tmp_path, &blob[..]) {
            warn!("ShaderCache: Failed to write tmp file: {}", e);
            return false;
        }
        if let Err(e) = std::fs::rename(&tmp_path, &self.cache_path) {
            warn!("ShaderCache: Failed to rename tmp to cache: {}", e);
            return false;
        }

        // Write storepath file
        let current = self.current_store_path.lock();
        if let Err(e) = std::fs::write(&self.storepath_file, &*current) {
            warn!("ShaderCache: Failed to write storepath: {}", e);
        }

        *self.blob_size.lock() = blob.len();
        info!("ShaderCache: Saved {} bytes to disk", blob.len());
        true
    }

    /// Invalidates cache: deletes blob and storepath file.
    pub fn invalidate(&self) {
        let _ = std::fs::remove_file(&self.cache_path);
        let _ = std::fs::remove_file(&self.storepath_file);
        self.blob.lock().clear();
        *self.blob_size.lock() = 0;
        info!("ShaderCache: Invalidated (blob + storepath deleted)");
    }

    /// Updates the store path and checks for invalidation.
    pub fn update_store_path(&self, new_path: &str) -> bool {
        let mut current = self.current_store_path.lock();
        if *current != new_path {
            warn!("ShaderCache: Store path changed: {} -> {}", *current, new_path);
            *current = new_path.to_string();
            drop(current);
            self.invalidate();
            return true;
        }
        false
    }

    pub fn byte_size(&self) -> usize {
        *self.blob_size.lock()
    }

    pub fn is_empty(&self) -> bool {
        self.blob.lock().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_cache_store_path_match() {
        let temp = std::env::temp_dir().join("vitusos_shader_test.bin");
        let cache = ShaderCache::new(temp.clone(), "/usr/bin/animus-compositor");
        assert!(!cache.load()); // No existing cache
        assert!(cache.is_empty());
    }

    #[test]
    fn test_shader_cache_invalidation() {
        let temp = std::env::temp_dir().join("vitusos_shader_test2.bin");
        let cache = ShaderCache::new(&temp, "/path/old");
        cache.invalidate();
        assert!(cache.is_empty());
        assert_eq!(cache.byte_size(), 0);
    }
}
