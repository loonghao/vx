//! Download cache management

use crate::error::{DownloadError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use vx_config::types::TurboCdnConfig;

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Original URL
    pub url: String,
    /// Local file path
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Checksum (if available)
    pub checksum: Option<String>,
}

/// Download cache manager
pub struct DownloadCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Cache metadata
    metadata: HashMap<String, CacheEntry>,
    /// Maximum cache size in bytes
    max_size: u64,
    /// Cache enabled flag
    enabled: bool,
}

impl DownloadCache {
    /// Create a new download cache
    pub fn new(cache_config: &TurboCdnConfig) -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;

        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            DownloadError::filesystem(format!("Failed to create cache directory: {}", e))
        })?;

        let mut cache = Self {
            cache_dir,
            metadata: HashMap::new(),
            max_size: cache_config.cache_max_size,
            enabled: cache_config.cache_enabled,
        };

        // Load existing metadata
        cache.load_metadata()?;

        Ok(cache)
    }

    /// Get cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| DownloadError::config("Failed to get cache directory"))?
            .join("vx")
            .join("downloads");

        Ok(cache_dir)
    }

    /// Load cache metadata from disk
    fn load_metadata(&mut self) -> Result<()> {
        let metadata_file = self.cache_dir.join("metadata.json");

        if metadata_file.exists() {
            let content = std::fs::read_to_string(&metadata_file)
                .map_err(|e| DownloadError::cache(format!("Failed to read metadata: {}", e)))?;

            self.metadata = serde_json::from_str(&content)
                .map_err(|e| DownloadError::cache(format!("Failed to parse metadata: {}", e)))?;
        }

        Ok(())
    }

    /// Save cache metadata to disk
    fn save_metadata(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let metadata_file = self.cache_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| DownloadError::cache(format!("Failed to serialize metadata: {}", e)))?;

        std::fs::write(&metadata_file, content)
            .map_err(|e| DownloadError::cache(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// Generate cache key for URL
    fn cache_key(&self, url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Check if URL is cached and valid
    pub fn get(&mut self, url: &str) -> Option<PathBuf> {
        if !self.enabled {
            return None;
        }

        let key = self.cache_key(url);

        if let Some(entry) = self.metadata.get_mut(&key) {
            // Check if file still exists
            if entry.path.exists() {
                // Update last accessed time
                entry.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                return Some(entry.path.clone());
            } else {
                // File was deleted, remove from cache
                self.metadata.remove(&key);
            }
        }

        None
    }

    /// Add file to cache
    pub fn put(&mut self, url: &str, file_path: &Path, checksum: Option<String>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let key = self.cache_key(url);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get file size
        let size = file_path
            .metadata()
            .map_err(|e| DownloadError::filesystem(format!("Failed to get file metadata: {}", e)))?
            .len();

        // Create cache entry
        let entry = CacheEntry {
            url: url.to_string(),
            path: file_path.to_path_buf(),
            size,
            created_at: now,
            last_accessed: now,
            checksum,
        };

        self.metadata.insert(key, entry);

        // Check cache size and cleanup if needed
        self.cleanup_if_needed()?;

        // Save metadata
        self.save_metadata()?;

        Ok(())
    }

    /// Cleanup cache if it exceeds maximum size
    fn cleanup_if_needed(&mut self) -> Result<()> {
        let total_size: u64 = self.metadata.values().map(|entry| entry.size).sum();

        if total_size <= self.max_size {
            return Ok(());
        }

        // Sort entries by last accessed time (oldest first)
        let mut entries: Vec<_> = self
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);

        let mut removed_size = 0u64;
        let target_size = self.max_size * 80 / 100; // Remove until 80% of max size

        for (key, entry) in entries {
            if total_size - removed_size <= target_size {
                break;
            }

            // Remove file
            if entry.path.exists() {
                std::fs::remove_file(&entry.path).map_err(|e| {
                    DownloadError::filesystem(format!("Failed to remove cached file: {}", e))
                })?;
            }

            removed_size += entry.size;
            self.metadata.remove(&key);
        }

        Ok(())
    }

    /// Clear all cache
    pub fn clear(&mut self) -> Result<()> {
        for entry in self.metadata.values() {
            if entry.path.exists() {
                std::fs::remove_file(&entry.path).map_err(|e| {
                    DownloadError::filesystem(format!("Failed to remove cached file: {}", e))
                })?;
            }
        }

        self.metadata.clear();
        self.save_metadata()?;

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_size: u64 = self.metadata.values().map(|entry| entry.size).sum();
        let entry_count = self.metadata.len();

        CacheStats {
            total_size,
            entry_count,
            max_size: self.max_size,
            enabled: self.enabled,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total cache size in bytes
    pub total_size: u64,
    /// Number of cached entries
    pub entry_count: usize,
    /// Maximum cache size in bytes
    pub max_size: u64,
    /// Whether cache is enabled
    pub enabled: bool,
}

impl CacheStats {
    /// Get cache usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.total_size as f64 / self.max_size as f64) * 100.0
        }
    }

    /// Get human-readable total size
    pub fn total_size_human(&self) -> String {
        let size = self.total_size as f64;
        if size >= 1_000_000_000.0 {
            format!("{:.1} GB", size / 1_000_000_000.0)
        } else if size >= 1_000_000.0 {
            format!("{:.1} MB", size / 1_000_000.0)
        } else if size >= 1_000.0 {
            format!("{:.1} KB", size / 1_000.0)
        } else {
            format!("{} B", size)
        }
    }
}
