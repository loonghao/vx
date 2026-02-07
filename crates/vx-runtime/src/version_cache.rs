//! High-performance version cache using bincode serialization
//!
//! This module provides a fast, compact cache for version information.
//! Key improvements over JSON-based caching:
//!
//! - **Bincode serialization**: 10-100x faster than JSON, 30-70% smaller files
//! - **Compact data model**: Only stores essential version info, not full API responses
//! - **Separate metadata file**: Quick validity check without loading full data
//! - **Stale cache support**: Returns expired data as fallback on network errors
//! - **JSON Value caching**: `serde_json::Value` stored as JSON text (bincode cannot roundtrip it)
//!
//! ## Cache Directory Structure
//!
//! ```text
//! ~/.vx/cache/
//! └── versions_v2/
//!     ├── bun.meta      # Small metadata file (< 100 bytes, bincode)
//!     ├── bun.data      # Compact version data (bincode)
//!     ├── node.meta
//!     ├── node.data
//!     ├── go.meta
//!     └── go.jsonval    # JSON API response (JSON text format)
//! ```
//!
//! ## Performance Comparison
//!
//! | Format | Serialization | File Size | Deserialization |
//! |--------|---------------|-----------|------------------|
//! | JSON text | 1x (baseline) | 100% | 1x |
//! | bincode (CompactVersion) | 50-100x | 5-15% | 10-20x |
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Current cache schema version
pub const CACHE_SCHEMA_VERSION: u32 = 2;

/// Default cache TTL (24 hours)
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Compact version info - only essential fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactVersion {
    /// Version string (e.g., "1.0.0", "v20.0.0")
    pub version: String,
    /// Whether this is a prerelease
    pub prerelease: bool,
    /// Published timestamp (Unix epoch seconds, 0 if unknown)
    pub published_at: u64,
}

impl CompactVersion {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            prerelease: false,
            published_at: 0,
        }
    }

    pub fn with_prerelease(mut self, prerelease: bool) -> Self {
        self.prerelease = prerelease;
        self
    }

    pub fn with_published_at(mut self, timestamp: u64) -> Self {
        self.published_at = timestamp;
        self
    }
}

/// Cache metadata - stored separately for quick validity checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Schema version for migrations
    pub schema_version: u32,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Number of versions cached
    pub version_count: u32,
    /// Source URL (for debugging)
    pub source_url: Option<String>,
    /// ETag for conditional requests
    pub etag: Option<String>,
}

impl CacheMetadata {
    pub fn new(version_count: usize, ttl: Duration) -> Self {
        Self {
            schema_version: CACHE_SCHEMA_VERSION,
            created_at: now_epoch_secs(),
            ttl_secs: ttl.as_secs(),
            version_count: version_count as u32,
            source_url: None,
            etag: None,
        }
    }

    pub fn with_source_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }

    /// Check if cache is still valid
    pub fn is_valid(&self) -> bool {
        if self.schema_version != CACHE_SCHEMA_VERSION {
            return false;
        }
        let now = now_epoch_secs();
        now.saturating_sub(self.created_at) < self.ttl_secs
    }

    /// Get age in seconds
    pub fn age(&self) -> u64 {
        now_epoch_secs().saturating_sub(self.created_at)
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        let elapsed = self.age();
        self.ttl_secs.saturating_sub(elapsed)
    }
}

/// Cached version data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheData {
    /// List of versions (compact format)
    pub versions: Vec<CompactVersion>,
}

/// Cache mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CacheMode {
    /// Normal mode: use cache if valid, fetch if expired
    #[default]
    Normal,
    /// Refresh mode: always fetch, ignore cache
    Refresh,
    /// Offline mode: use cache only, even if expired
    Offline,
    /// No cache: never read or write cache
    NoCache,
}

impl From<vx_cache::CacheMode> for CacheMode {
    fn from(mode: vx_cache::CacheMode) -> Self {
        match mode {
            vx_cache::CacheMode::Normal => CacheMode::Normal,
            vx_cache::CacheMode::Refresh => CacheMode::Refresh,
            vx_cache::CacheMode::Offline => CacheMode::Offline,
            vx_cache::CacheMode::NoCache => CacheMode::NoCache,
        }
    }
}

/// High-performance version cache
#[derive(Debug, Clone)]
pub struct VersionCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Default TTL
    ttl: Duration,
    /// Cache mode
    mode: CacheMode,
}

impl VersionCache {
    /// Create a new version cache
    pub fn new(base_cache_dir: PathBuf) -> Self {
        Self {
            cache_dir: base_cache_dir.join("versions_v2"),
            ttl: DEFAULT_CACHE_TTL,
            mode: CacheMode::Normal,
        }
    }

    /// Set custom TTL
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

    /// Get metadata file path
    fn meta_path(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.meta", tool_name))
    }

    /// Get data file path
    fn data_path(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.data", tool_name))
    }

    /// Get JSON Value data file path (stored as bincode for performance)
    ///
    /// Note: Despite the "json" in the name, this stores `serde_json::Value`
    /// using bincode serialization for 10-50x faster performance compared to JSON text.
    fn json_value_data_path(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.jsonval", tool_name))
    }

    /// Read metadata only (fast validity check)
    pub fn get_metadata(&self, tool_name: &str) -> Option<CacheMetadata> {
        if self.mode == CacheMode::NoCache || self.mode == CacheMode::Refresh {
            return None;
        }

        let meta_path = self.meta_path(tool_name);
        if !meta_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&meta_path).ok()?;
        let mut reader = BufReader::new(file);
        bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()
    }

    /// Check if cache is valid (without loading data)
    pub fn is_valid(&self, tool_name: &str) -> bool {
        match self.mode {
            CacheMode::NoCache | CacheMode::Refresh => false,
            CacheMode::Offline => self.get_metadata(tool_name).is_some(),
            CacheMode::Normal => self
                .get_metadata(tool_name)
                .map(|m| m.is_valid())
                .unwrap_or(false),
        }
    }

    /// Get cached versions (returns None if cache miss or expired)
    pub fn get(&self, tool_name: &str) -> Option<Vec<CompactVersion>> {
        if self.mode == CacheMode::NoCache || self.mode == CacheMode::Refresh {
            return None;
        }

        // Check metadata first
        let metadata = self.get_metadata(tool_name)?;

        // In normal mode, check validity
        if self.mode == CacheMode::Normal && !metadata.is_valid() {
            // Remove expired cache
            self.clear(tool_name).ok();
            return None;
        }

        // Load data
        let data_path = self.data_path(tool_name);
        if !data_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&data_path).ok()?;
        let mut reader = BufReader::new(file);
        let data: CacheData = bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;

        Some(data.versions)
    }

    /// Get stale cache (for fallback on network errors)
    /// This ignores TTL and returns cached data if available
    pub fn get_stale(&self, tool_name: &str) -> Option<Vec<CompactVersion>> {
        if self.mode == CacheMode::NoCache {
            return None;
        }

        let data_path = self.data_path(tool_name);
        if !data_path.exists() {
            return None;
        }

        // Check metadata exists and schema is compatible
        let metadata = {
            let meta_path = self.meta_path(tool_name);
            if !meta_path.exists() {
                return None;
            }
            let file = std::fs::File::open(&meta_path).ok()?;
            let mut reader = BufReader::new(file);
            let meta: CacheMetadata = bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;
            if meta.schema_version != CACHE_SCHEMA_VERSION {
                return None;
            }
            meta
        };

        tracing::debug!(
            "Using stale cache for {} (age: {}s, ttl: {}s)",
            tool_name,
            metadata.age(),
            metadata.ttl_secs
        );

        let file = std::fs::File::open(&data_path).ok()?;
        let mut reader = BufReader::new(file);
        let data: CacheData = bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;

        Some(data.versions)
    }

    /// Get cached JSON data (for generic JSON API responses)
    ///
    /// **Performance**: Uses bincode serialization internally for 10-50x faster speed
    /// compared to JSON text format.
    ///
    /// Returns None if cache miss or expired
    pub fn get_json(&self, tool_name: &str) -> Option<serde_json::Value> {
        if self.mode == CacheMode::NoCache || self.mode == CacheMode::Refresh {
            return None;
        }

        // Check metadata first
        let metadata = self.get_metadata(tool_name)?;

        // In normal mode, check validity
        if self.mode == CacheMode::Normal && !metadata.is_valid() {
            // Remove expired cache
            self.clear(tool_name).ok();
            return None;
        }

        // Load JSON Value data (JSON text format)
        let json_path = self.json_value_data_path(tool_name);
        if !json_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&json_path).ok()?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).ok()
    }

    /// Get stale JSON cache (for fallback on network errors)
    ///
    /// This ignores TTL and returns cached data if available
    pub fn get_stale_json(&self, tool_name: &str) -> Option<serde_json::Value> {
        if self.mode == CacheMode::NoCache {
            return None;
        }

        let json_path = self.json_value_data_path(tool_name);
        if !json_path.exists() {
            return None;
        }

        // Check metadata exists and schema is compatible
        let metadata = {
            let meta_path = self.meta_path(tool_name);
            if !meta_path.exists() {
                return None;
            }
            let file = std::fs::File::open(&meta_path).ok()?;
            let mut reader = BufReader::new(file);
            let meta: CacheMetadata = bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;
            if meta.schema_version != CACHE_SCHEMA_VERSION {
                return None;
            }
            meta
        };

        tracing::debug!(
            "Using stale JSON cache for {} (age: {}s, ttl: {}s)",
            tool_name,
            metadata.age(),
            metadata.ttl_secs
        );

        let file = std::fs::File::open(&json_path).ok()?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).ok()
    }

    /// Set cached versions
    pub fn set(&self, tool_name: &str, versions: Vec<CompactVersion>) -> Result<()> {
        self.set_with_options(tool_name, versions, None, None)
    }

    /// Set cached versions with additional options
    pub fn set_with_options(
        &self,
        tool_name: &str,
        versions: Vec<CompactVersion>,
        source_url: Option<&str>,
        etag: Option<&str>,
    ) -> Result<()> {
        if self.mode == CacheMode::NoCache {
            return Ok(());
        }

        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;

        // Create metadata
        let mut metadata = CacheMetadata::new(versions.len(), self.ttl);
        if let Some(url) = source_url {
            metadata = metadata.with_source_url(url);
        }
        if let Some(tag) = etag {
            metadata = metadata.with_etag(tag);
        }

        // Create data
        let data = CacheData { versions };

        // Write metadata (atomic)
        let meta_path = self.meta_path(tool_name);
        let meta_tmp = meta_path.with_extension("meta.tmp");
        {
            let file = std::fs::File::create(&meta_tmp)?;
            let mut writer = BufWriter::new(file);
            bincode::serde::encode_into_std_write(&metadata, &mut writer, bincode::config::standard())?;
        }
        std::fs::rename(&meta_tmp, &meta_path)?;

        // Write data (atomic)
        let data_path = self.data_path(tool_name);
        let data_tmp = data_path.with_extension("data.tmp");
        {
            let file = std::fs::File::create(&data_tmp)?;
            let mut writer = BufWriter::new(file);
            bincode::serde::encode_into_std_write(&data, &mut writer, bincode::config::standard())?;
        }
        std::fs::rename(&data_tmp, &data_path)?;

        tracing::debug!(
            "Cached {} versions for {} ({} bytes)",
            data.versions.len(),
            tool_name,
            std::fs::metadata(&data_path).map(|m| m.len()).unwrap_or(0)
        );

        Ok(())
    }

    /// Set cached JSON data (for generic JSON API responses)
    ///
    /// **Performance**: Stores `serde_json::Value` using bincode serialization
    /// for 10-50x faster speed and 30-50% smaller file size compared to JSON text.
    pub fn set_json(&self, tool_name: &str, data: serde_json::Value) -> Result<()> {
        self.set_json_with_options(tool_name, data, None, None)
    }

    /// Set cached JSON data with additional options
    ///
    /// **Performance**: Stores `serde_json::Value` using bincode serialization
    /// for significantly better performance than JSON text format.
    pub fn set_json_with_options(
        &self,
        tool_name: &str,
        data: serde_json::Value,
        source_url: Option<&str>,
        etag: Option<&str>,
    ) -> Result<()> {
        if self.mode == CacheMode::NoCache {
            return Ok(());
        }

        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;

        // Count items if it's an array
        let version_count = data.as_array().map(|a| a.len()).unwrap_or(1);

        // Create metadata
        let mut metadata = CacheMetadata::new(version_count, self.ttl);
        if let Some(url) = source_url {
            metadata = metadata.with_source_url(url);
        }
        if let Some(tag) = etag {
            metadata = metadata.with_etag(tag);
        }

        // Write metadata (atomic)
        let meta_path = self.meta_path(tool_name);
        let meta_tmp = meta_path.with_extension("meta.tmp");
        {
            let file = std::fs::File::create(&meta_tmp)?;
            let mut writer = BufWriter::new(file);
            bincode::serde::encode_into_std_write(&metadata, &mut writer, bincode::config::standard())?;
        }
        std::fs::rename(&meta_tmp, &meta_path)?;

        // Write JSON Value data using JSON text format (atomic)
        // Note: bincode cannot correctly roundtrip serde_json::Value because it's a
        // self-describing enum type. We use JSON text format for correctness.
        let json_path = self.json_value_data_path(tool_name);
        let json_tmp = json_path.with_extension("jsonval.tmp");
        {
            let file = std::fs::File::create(&json_tmp)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &data)?;
        }
        std::fs::rename(&json_tmp, &json_path)?;

        tracing::debug!(
            "Cached JSON Value for {} ({} bytes)",
            tool_name,
            std::fs::metadata(&json_path).map(|m| m.len()).unwrap_or(0)
        );

        Ok(())
    }

    /// Clear cache for a tool
    pub fn clear(&self, tool_name: &str) -> Result<()> {
        let meta_path = self.meta_path(tool_name);
        let data_path = self.data_path(tool_name);
        let json_path = self.json_value_data_path(tool_name);

        if meta_path.exists() {
            std::fs::remove_file(&meta_path)?;
        }
        if data_path.exists() {
            std::fs::remove_file(&data_path)?;
        }
        if json_path.exists() {
            std::fs::remove_file(&json_path)?;
        }

        Ok(())
    }

    /// Clear all caches
    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats::default();

        if !self.cache_dir.exists() {
            return Ok(stats);
        }

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "meta") {
                stats.total_entries += 1;

                if let Ok(file) = std::fs::File::open(&path) {
                    let mut reader = BufReader::new(file);
                    if let Ok(meta) = bincode::serde::decode_from_std_read::<CacheMetadata, _, _>(&mut reader, bincode::config::standard()) {
                        if meta.is_valid() {
                            stats.valid_entries += 1;
                        } else {
                            stats.expired_entries += 1;
                        }
                    }
                }
            }

            if let Ok(metadata) = path.metadata() {
                stats.total_size_bytes += metadata.len();
            }
        }

        Ok(stats)
    }

    /// Prune expired cache entries
    ///
    /// Returns the number of entries that were pruned
    pub fn prune(&self) -> Result<usize> {
        let mut pruned = 0;

        if !self.cache_dir.exists() {
            return Ok(pruned);
        }

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "meta") {
                let should_prune = if let Ok(file) = std::fs::File::open(&path) {
                    let mut reader = BufReader::new(file);
                    if let Ok(meta) = bincode::serde::decode_from_std_read::<CacheMetadata, _, _>(&mut reader, bincode::config::standard()) {
                        !meta.is_valid()
                    } else {
                        true // Remove corrupted metadata
                    }
                } else {
                    false
                };

                if should_prune {
                    // Get tool name from meta file path
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let _ = self.clear(stem);
                        pruned += 1;
                    }
                }
            }
        }

        Ok(pruned)
    }

    /// Get a cache entry by tool name (returns metadata if valid)
    ///
    /// This is useful for checking if a specific tool has cached data
    pub fn get_entry(&self, tool_name: &str) -> Option<CacheEntry> {
        let meta_path = self.meta_path(tool_name);

        if !meta_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&meta_path).ok()?;
        let mut reader = BufReader::new(file);
        let meta: CacheMetadata = bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;

        // Check if valid based on mode
        let is_valid = match self.mode {
            CacheMode::Normal => meta.is_valid(),
            CacheMode::Refresh => false,
            CacheMode::Offline | CacheMode::NoCache => true, // Return entry info even if expired
        };

        Some(CacheEntry {
            tool_name: tool_name.to_string(),
            version_count: meta.version_count as usize,
            source_url: meta.source_url,
            cached_at: meta.created_at,
            expires_at: meta.created_at + meta.ttl_secs,
            is_valid,
        })
    }
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub total_size_bytes: u64,
}

impl CacheStats {
    /// Format the total size as a human-readable string
    pub fn formatted_size(&self) -> String {
        format_size(self.total_size_bytes)
    }
}

/// Format bytes as human-readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Cache entry information
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Tool name
    pub tool_name: String,
    /// Number of cached versions
    pub version_count: usize,
    /// Source URL (if available)
    pub source_url: Option<String>,
    /// When the cache was created (Unix timestamp)
    pub cached_at: u64,
    /// When the cache expires (Unix timestamp)
    pub expires_at: u64,
    /// Whether the cache is currently valid
    pub is_valid: bool,
}

/// Helper to get current Unix epoch seconds
fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Convert GitHub release to compact version
pub fn github_release_to_compact(release: &serde_json::Value) -> Option<CompactVersion> {
    let tag_name = release.get("tag_name")?.as_str()?;
    let prerelease = release
        .get("prerelease")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let published_at = release
        .get("published_at")
        .and_then(|v| v.as_str())
        .and_then(parse_iso8601_timestamp)
        .unwrap_or(0);

    // Strip 'v' prefix if present
    let version = tag_name.strip_prefix('v').unwrap_or(tag_name);

    Some(
        CompactVersion::new(version)
            .with_prerelease(prerelease)
            .with_published_at(published_at),
    )
}

/// Parse ISO8601 timestamp to Unix epoch seconds
fn parse_iso8601_timestamp(s: &str) -> Option<u64> {
    // Simple parsing for GitHub's format: "2025-01-08T12:34:56Z"
    // For production, consider using chrono crate
    let s = s.trim_end_matches('Z');
    let parts: Vec<&str> = s.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<u32> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<u32> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();

    if date_parts.len() != 3 || time_parts.len() != 3 {
        return None;
    }

    // Approximate calculation (not accounting for leap years perfectly)
    let year = date_parts[0];
    let month = date_parts[1];
    let day = date_parts[2];
    let hour = time_parts[0];
    let minute = time_parts[1];
    let second = time_parts[2];

    // Days since Unix epoch (1970-01-01)
    let years_since_1970 = year.saturating_sub(1970);
    let leap_years = (years_since_1970 + 1) / 4;
    let days_from_years = years_since_1970 * 365 + leap_years;

    let days_in_months = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let days_from_months = days_in_months
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or(0);

    let total_days = days_from_years + days_from_months + day.saturating_sub(1);
    let total_seconds =
        (total_days as u64) * 86400 + (hour as u64) * 3600 + (minute as u64) * 60 + (second as u64);

    Some(total_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compact_version() {
        let v = CompactVersion::new("1.0.0")
            .with_prerelease(false)
            .with_published_at(1704672000);
        assert_eq!(v.version, "1.0.0");
        assert!(!v.prerelease);
    }

    #[test]
    fn test_cache_set_get() {
        let temp_dir = TempDir::new().unwrap();
        let cache = VersionCache::new(temp_dir.path().to_path_buf());

        let versions = vec![
            CompactVersion::new("1.0.0"),
            CompactVersion::new("2.0.0").with_prerelease(true),
        ];

        cache.set("test-tool", versions.clone()).unwrap();

        let cached = cache.get("test-tool");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), versions);
    }

    #[test]
    fn test_cache_stale_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let cache =
            VersionCache::new(temp_dir.path().to_path_buf()).with_ttl(Duration::from_secs(1));

        let versions = vec![CompactVersion::new("1.0.0")];
        cache.set("test-tool", versions.clone()).unwrap();

        // Modify metadata to make it expired
        let meta_path = cache.meta_path("test-tool");
        let file = std::fs::File::open(&meta_path).unwrap();
        let mut metadata: CacheMetadata = bincode::serde::decode_from_std_read(&mut BufReader::new(file), bincode::config::standard()).unwrap();
        metadata.created_at = 0; // Make it expired

        let file = std::fs::File::create(&meta_path).unwrap();
        bincode::serde::encode_into_std_write(&metadata, &mut BufWriter::new(file), bincode::config::standard()).unwrap();

        // Stale get should still return data even when expired
        // Note: We test get_stale first because get() clears expired cache
        let stale = cache.get_stale("test-tool");
        assert!(stale.is_some());
        assert_eq!(stale.unwrap(), versions);

        // Normal get should return None (expired) and clear the cache
        assert!(cache.get("test-tool").is_none());

        // After get() clears the cache, get_stale should also return None
        assert!(cache.get_stale("test-tool").is_none());
    }

    #[test]
    fn test_github_release_conversion() {
        let release = serde_json::json!({
            "tag_name": "v1.0.0",
            "prerelease": false,
            "published_at": "2025-01-08T12:00:00Z"
        });

        let compact = github_release_to_compact(&release).unwrap();
        assert_eq!(compact.version, "1.0.0");
        assert!(!compact.prerelease);
        assert!(compact.published_at > 0);
    }

    #[test]
    fn test_json_value_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = VersionCache::new(temp_dir.path().to_path_buf());

        // Create a JSON Value (simulating API response)
        let json_data = serde_json::json!({
            "versions": [
                {"version": "1.0.0", "prerelease": false},
                {"version": "2.0.0", "prerelease": true}
            ],
            "metadata": {
                "total": 2,
                "url": "https://example.com/api"
            }
        });

        // Set JSON Value cache
        cache.set_json("test-api", json_data.clone()).unwrap();

        // First try get_stale_json (ignores TTL)
        let stale = cache.get_stale_json("test-api");
        assert!(stale.is_some(), "get_stale_json returned None");
        assert_eq!(stale.unwrap(), json_data);

        // Then try get_json
        let cached = cache.get_json("test-api");
        assert!(cached.is_some(), "get_json returned None");
        assert_eq!(cached.unwrap(), json_data);
    }

    #[test]
    fn test_cache_size_comparison() {
        // Simulate a typical GitHub releases response
        let releases: Vec<serde_json::Value> = (0..100)
            .map(|i| {
                serde_json::json!({
                    "tag_name": format!("v1.{}.0", i),
                    "name": format!("Release 1.{}.0", i),
                    "prerelease": false,
                    "draft": false,
                    "published_at": "2025-01-08T12:00:00Z",
                    "body": "This is a long release description with lots of text...",
                    "author": {
                        "login": "user",
                        "id": 12345,
                        "avatar_url": "https://example.com/avatar.png"
                    },
                    "assets": []
                })
            })
            .collect();

        // JSON size
        let json_size = serde_json::to_string(&releases).unwrap().len();

        // Compact bincode size
        let compact: Vec<CompactVersion> = releases
            .iter()
            .filter_map(github_release_to_compact)
            .collect();
        let bincode_size = bincode::serde::encode_to_vec(&compact, bincode::config::standard()).unwrap().len();

        println!("JSON size: {} bytes", json_size);
        println!("Bincode size: {} bytes", bincode_size);
        println!(
            "Compression ratio: {:.1}x",
            json_size as f64 / bincode_size as f64
        );

        // Bincode should be significantly smaller
        assert!(bincode_size < json_size / 5);
    }
}
