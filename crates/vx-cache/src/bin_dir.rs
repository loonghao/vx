//! Bin directory cache for vx
//!
//! Caches resolved bin directory paths to avoid repeated walkdir traversal
//! during PATH construction. The cache is stored as a single
//! bincode-serialized file at `~/.vx/cache/bin-dirs.bin`.
//!
//! ## Cache invalidation
//!
//! The cache is invalidated (entries removed) when:
//! - A runtime is installed or uninstalled
//! - The user runs `vx cache clean`
//!
//! Individual entries are also validated on read: if the cached path no longer
//! exists on disk, the entry is removed and `None` is returned.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Cache: store_dir string → bin directory path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinDirCache {
    /// Version for cache format migration
    version: u32,
    /// Map: store_dir path string → resolved bin directory path
    entries: HashMap<String, PathBuf>,
}

const CACHE_VERSION: u32 = 1;
const CACHE_FILENAME: &str = "bin-dirs.bin";

impl BinDirCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: HashMap::new(),
        }
    }

    /// Look up a cached bin directory path.
    ///
    /// Returns `Some(path)` if found and the path still exists on disk.
    /// Returns `None` if not cached or the cached path is stale.
    pub fn get(&mut self, store_dir: &str) -> Option<PathBuf> {
        if let Some(path) = self.entries.get(store_dir) {
            if path.exists() {
                return Some(path.clone());
            }
        }
        // Stale or missing — remove if present
        self.entries.remove(store_dir);
        None
    }

    /// Store a resolved bin directory path in the cache.
    pub fn put(&mut self, store_dir: String, bin_dir: PathBuf) {
        self.entries.insert(store_dir, bin_dir);
    }

    /// Remove all entries for a specific runtime (by store dir prefix).
    pub fn invalidate_runtime(&mut self, runtime_store_prefix: &str) {
        self.entries
            .retain(|key, _| !key.starts_with(runtime_store_prefix));
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the cache file path within a cache directory.
    pub fn cache_file_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join(CACHE_FILENAME)
    }

    /// Load cache from disk. Returns a new empty cache on any error.
    pub fn load(cache_dir: &Path) -> Self {
        let path = Self::cache_file_path(cache_dir);
        Self::load_from_file(&path).unwrap_or_default()
    }

    /// Load cache from a specific file path.
    fn load_from_file(path: &Path) -> Option<Self> {
        let file = std::fs::File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        let cache: Self =
            bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;

        if cache.version != CACHE_VERSION {
            return None;
        }
        Some(cache)
    }

    /// Save cache to disk (atomic write).
    pub fn save(&self, cache_dir: &Path) -> std::io::Result<()> {
        let path = Self::cache_file_path(cache_dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temp_path = path.with_extension("bin.tmp");
        let file = std::fs::File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);
        bincode::serde::encode_into_std_write(self, &mut writer, bincode::config::standard())
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        std::fs::rename(&temp_path, &path)?;
        Ok(())
    }

    /// Remove the cache file from disk.
    pub fn remove_file(cache_dir: &Path) -> std::io::Result<()> {
        let path = Self::cache_file_path(cache_dir);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

impl Default for BinDirCache {
    fn default() -> Self {
        Self::new()
    }
}
