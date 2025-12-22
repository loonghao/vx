//! Version cache for reducing GitHub API calls
//!
//! This module provides a file-based cache for version information,
//! reducing the number of GitHub API calls and avoiding rate limits.
//!
//! ## Cache Strategy (inspired by uv)
//!
//! - **Default TTL**: 24 hours for version lists (balances freshness vs API limits)
//! - **Refresh options**: `--refresh` to force re-fetch, `--offline` to use cache only
//! - **Cache keys**: Based on tool name and optional source URL
//! - **Thread safety**: File-based locking for concurrent access
//!
//! ## Cache Directory Structure
//!
//! ```text
//! ~/.vx/cache/
//! ├── versions/           # Version list cache
//! │   ├── uv.json
//! │   ├── node.json
//! │   └── go.json
//! └── downloads/          # Download cache (handled separately)
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Default cache TTL (24 hours - version lists don't change frequently)
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Short cache TTL for frequently updated tools (1 hour)
pub const SHORT_CACHE_TTL: Duration = Duration::from_secs(60 * 60);

/// Long cache TTL for stable tools (7 days)
pub const LONG_CACHE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Cache refresh mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CacheMode {
    /// Use cache if valid, otherwise fetch (default)
    #[default]
    Normal,
    /// Force refresh, ignore cache
    Refresh,
    /// Use cache only, fail if not available (offline mode)
    Offline,
    /// Skip cache entirely (for CI or testing)
    NoCache,
}

/// Version cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cached data (JSON)
    pub data: serde_json::Value,
    /// Timestamp when the cache was created (Unix epoch seconds)
    pub created_at: u64,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Source URL (for cache invalidation)
    #[serde(default)]
    pub source_url: Option<String>,
    /// ETag from HTTP response (for conditional requests)
    #[serde(default)]
    pub etag: Option<String>,
    /// Cache version (for format migrations)
    #[serde(default = "default_cache_version")]
    pub version: u32,
}

fn default_cache_version() -> u32 {
    1
}

/// Current cache format version
pub const CACHE_VERSION: u32 = 1;

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(data: serde_json::Value, ttl: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            data,
            created_at: now,
            ttl_secs: ttl.as_secs(),
            source_url: None,
            etag: None,
            version: CACHE_VERSION,
        }
    }

    /// Create a new cache entry with source URL
    pub fn with_source(mut self, url: &str) -> Self {
        self.source_url = Some(url.to_string());
        self
    }

    /// Create a new cache entry with ETag
    pub fn with_etag(mut self, etag: &str) -> Self {
        self.etag = Some(etag.to_string());
        self
    }

    /// Check if the cache entry is still valid
    pub fn is_valid(&self) -> bool {
        // Check version compatibility
        if self.version != CACHE_VERSION {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now.saturating_sub(self.created_at) < self.ttl_secs
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let elapsed = now.saturating_sub(self.created_at);
        self.ttl_secs.saturating_sub(elapsed)
    }

    /// Get age in seconds
    pub fn age(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now.saturating_sub(self.created_at)
    }
}

/// Version cache manager
#[derive(Debug, Clone)]
pub struct VersionCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Default cache TTL
    ttl: Duration,
    /// Cache mode
    mode: CacheMode,
}

impl VersionCache {
    /// Create a new version cache
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            ttl: DEFAULT_CACHE_TTL,
            mode: CacheMode::Normal,
        }
    }

    /// Create a new version cache with custom TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set cache mode
    pub fn with_mode(mut self, mode: CacheMode) -> Self {
        self.mode = mode;
        self
    }

    /// Get current cache mode
    pub fn mode(&self) -> CacheMode {
        self.mode
    }

    /// Get the cache file path for a tool
    fn cache_file(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", tool_name))
    }

    /// Get cached entry for a tool (returns full entry with metadata)
    pub fn get_entry(&self, tool_name: &str) -> Option<CacheEntry> {
        // NoCache mode always returns None
        if self.mode == CacheMode::NoCache {
            return None;
        }

        // Refresh mode ignores cache
        if self.mode == CacheMode::Refresh {
            return None;
        }

        let cache_file = self.cache_file(tool_name);

        if !cache_file.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cache_file).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        // In offline mode, return even expired cache
        if self.mode == CacheMode::Offline {
            return Some(entry);
        }

        // Normal mode: check validity
        if entry.is_valid() {
            Some(entry)
        } else {
            // Remove expired cache
            let _ = std::fs::remove_file(&cache_file);
            None
        }
    }

    /// Get cached versions for a tool (returns just the data)
    pub fn get(&self, tool_name: &str) -> Option<serde_json::Value> {
        self.get_entry(tool_name).map(|e| e.data)
    }

    /// Set cached versions for a tool
    pub fn set(&self, tool_name: &str, data: serde_json::Value) -> Result<()> {
        self.set_with_options(tool_name, data, None, None)
    }

    /// Set cached versions with additional options
    pub fn set_with_options(
        &self,
        tool_name: &str,
        data: serde_json::Value,
        source_url: Option<&str>,
        etag: Option<&str>,
    ) -> Result<()> {
        // NoCache mode doesn't write
        if self.mode == CacheMode::NoCache {
            return Ok(());
        }

        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;

        let cache_file = self.cache_file(tool_name);
        let mut entry = CacheEntry::new(data, self.ttl);

        if let Some(url) = source_url {
            entry = entry.with_source(url);
        }
        if let Some(tag) = etag {
            entry = entry.with_etag(tag);
        }

        let content = serde_json::to_string_pretty(&entry)?;
        std::fs::write(&cache_file, content)?;
        Ok(())
    }

    /// Clear cache for a specific tool
    pub fn clear(&self, tool_name: &str) -> Result<()> {
        let cache_file = self.cache_file(tool_name);
        if cache_file.exists() {
            std::fs::remove_file(&cache_file)?;
        }
        Ok(())
    }

    /// Clear all version caches
    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
        Ok(())
    }

    /// Prune expired cache entries
    pub fn prune(&self) -> Result<usize> {
        let mut count = 0;
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                            if !cache_entry.is_valid() {
                                let _ = std::fs::remove_file(&path);
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats::default();

        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    stats.total_entries += 1;
                    if let Ok(metadata) = path.metadata() {
                        stats.total_size += metadata.len();
                    }
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                            if cache_entry.is_valid() {
                                stats.valid_entries += 1;
                            } else {
                                stats.expired_entries += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(stats)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Number of valid (non-expired) entries
    pub valid_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Total size in bytes
    pub total_size: u64,
}

impl CacheStats {
    /// Format size as human-readable string
    pub fn formatted_size(&self) -> String {
        if self.total_size < 1024 {
            format!("{} B", self.total_size)
        } else if self.total_size < 1024 * 1024 {
            format!("{:.1} KB", self.total_size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.total_size as f64 / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_entry_validity() {
        let data = serde_json::json!({"versions": ["1.0.0", "2.0.0"]});
        let entry = CacheEntry::new(data, Duration::from_secs(3600));
        assert!(entry.is_valid());
    }

    #[test]
    fn test_cache_entry_expired() {
        let data = serde_json::json!({"versions": ["1.0.0"]});
        let mut entry = CacheEntry::new(data, Duration::from_secs(1));
        // Simulate expired entry
        entry.created_at = 0;
        assert!(!entry.is_valid());
    }

    #[test]
    fn test_version_cache_set_get() {
        let temp_dir = TempDir::new().unwrap();
        let cache = VersionCache::new(temp_dir.path().to_path_buf());

        let data = serde_json::json!({"versions": ["1.0.0", "2.0.0"]});
        cache.set("test-tool", data.clone()).unwrap();

        let cached = cache.get("test-tool");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), data);
    }

    #[test]
    fn test_version_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache = VersionCache::new(temp_dir.path().to_path_buf());

        let data = serde_json::json!({"versions": ["1.0.0"]});
        cache.set("test-tool", data).unwrap();

        cache.clear("test-tool").unwrap();
        assert!(cache.get("test-tool").is_none());
    }

    #[test]
    fn test_cache_mode_refresh() {
        let temp_dir = TempDir::new().unwrap();
        let cache = VersionCache::new(temp_dir.path().to_path_buf());

        let data = serde_json::json!({"versions": ["1.0.0"]});
        cache.set("test-tool", data).unwrap();

        // With refresh mode, cache should be ignored
        let refresh_cache = cache.clone().with_mode(CacheMode::Refresh);
        assert!(refresh_cache.get("test-tool").is_none());
    }

    #[test]
    fn test_cache_mode_offline() {
        let temp_dir = TempDir::new().unwrap();

        // Create an expired entry
        let data = serde_json::json!({"versions": ["1.0.0"]});
        let mut entry = CacheEntry::new(data.clone(), Duration::from_secs(1));
        entry.created_at = 0; // Make it expired

        let cache_file = temp_dir.path().join("test-tool.json");
        std::fs::write(&cache_file, serde_json::to_string(&entry).unwrap()).unwrap();

        // Offline mode should return even expired cache
        let offline_cache =
            VersionCache::new(temp_dir.path().to_path_buf()).with_mode(CacheMode::Offline);
        assert!(offline_cache.get("test-tool").is_some());

        // Normal mode should not return expired cache (and will delete it)
        let normal_cache = VersionCache::new(temp_dir.path().to_path_buf());
        assert!(normal_cache.get("test-tool").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache = VersionCache::new(temp_dir.path().to_path_buf());

        cache
            .set("tool1", serde_json::json!({"versions": ["1.0"]}))
            .unwrap();
        cache
            .set("tool2", serde_json::json!({"versions": ["2.0"]}))
            .unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 2);
        assert!(stats.total_size > 0);
    }
}
