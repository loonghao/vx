//! Executable path cache for vx
//!
//! Caches resolved executable paths to avoid repeated filesystem traversal
//! (walkdir) on every command invocation. The cache is stored as a single
//! bincode-serialized file at `~/.vx/cache/exec-paths.bin`.
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

/// Cache key: (runtime_store_dir, exe_name) → executable path
///
/// We use the store directory + exe name as key because the same exe name
/// may exist in different store directories (different versions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecPathCache {
    /// Version for cache format migration
    version: u32,
    /// Map: "dir_path\0exe_name" → resolved executable path
    entries: HashMap<String, PathBuf>,
}

const CACHE_VERSION: u32 = 1;
const CACHE_FILENAME: &str = "exec-paths.bin";

impl ExecPathCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: HashMap::new(),
        }
    }

    /// Build the cache key from directory and executable name
    fn make_key(dir: &Path, exe_name: &str) -> String {
        format!("{}\0{}", dir.to_string_lossy(), exe_name)
    }

    /// Look up a cached executable path.
    ///
    /// Returns `Some(path)` if found and the path still exists on disk.
    /// Returns `None` if not cached or the cached path is stale.
    pub fn get(&mut self, dir: &Path, exe_name: &str) -> Option<PathBuf> {
        let key = Self::make_key(dir, exe_name);
        // Check if entry exists and validate on disk
        if let Some(path) = self.entries.get(&key)
            && path.exists()
        {
            return Some(path.clone());
        }
        // Stale or missing — remove if present
        self.entries.remove(&key);
        None
    }

    /// Store a resolved executable path in the cache.
    pub fn put(&mut self, dir: &Path, exe_name: &str, exe_path: PathBuf) {
        let key = Self::make_key(dir, exe_name);
        self.entries.insert(key, exe_path);
    }

    /// Remove all entries for a specific runtime (by store dir prefix).
    ///
    /// Call this when installing or uninstalling a runtime version.
    pub fn invalidate_runtime(&mut self, runtime_store_dir: &Path) {
        let prefix = runtime_store_dir.to_string_lossy().to_string();
        self.entries.retain(|key, _| !key.starts_with(&prefix));
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

        // Version check: discard if format changed
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

        // Atomic rename
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

impl Default for ExecPathCache {
    fn default() -> Self {
        Self::new()
    }
}
