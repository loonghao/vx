//! High-performance download cache for vx
//!
//! This module provides content-addressable storage for downloaded files.
//! Key features:
//!
//! - **SHA256-based cache keys**: URL hashes for fast lookup
//! - **Bincode metadata**: Fast serialization/deserialization
//! - **Atomic writes**: Temp file + rename for consistency
//! - **ETag validation**: HTTP caching headers support
//! - **Sharded storage**: First 2 chars of hash for directory sharding
//!
//! ## Cache Directory Structure
//!
//! ```text
//! ~/.vx/cache/downloads/
//! ├── ab/
//! │   ├── cd1234567890abcdef...        # Cached file
//! │   └── cd1234567890abcdef...meta    # Metadata (bincode)
//! └── ef/
//!     └── gh9876543210fedcba...
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Download cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadCacheMetadata {
    /// Original URL
    pub url: String,
    /// File size in bytes
    pub size: u64,
    /// ETag from HTTP response (if available)
    pub etag: Option<String>,
    /// Last-Modified header (if available)
    pub last_modified: Option<String>,
    /// Content-Type header (if available)
    pub content_type: Option<String>,
    /// Timestamp when cached (epoch seconds)
    pub cached_at: u64,
    /// Original filename from URL
    pub filename: String,
}

/// Result of a cache lookup
#[derive(Debug)]
pub enum CacheLookupResult {
    /// Cache hit - file exists and is valid
    Hit {
        path: PathBuf,
        metadata: DownloadCacheMetadata,
    },
    /// Cache miss - file not found
    Miss,
    /// Cache needs revalidation (ETag check)
    NeedsRevalidation {
        path: PathBuf,
        metadata: DownloadCacheMetadata,
    },
}

/// High-performance download cache
#[derive(Debug, Clone)]
pub struct DownloadCache {
    /// Base cache directory
    cache_dir: PathBuf,
    /// Whether to use ETag revalidation
    use_etag: bool,
}

impl DownloadCache {
    /// Create a new download cache
    pub fn new(cache_dir: PathBuf) -> Self {
        let downloads_dir = cache_dir.join("downloads");
        Self {
            cache_dir: downloads_dir,
            use_etag: true,
        }
    }

    /// Create cache with custom settings
    pub fn with_etag(mut self, use_etag: bool) -> Self {
        self.use_etag = use_etag;
        self
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Compute cache key from URL (SHA256 hash)
    pub fn cache_key(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get the sharded directory path for a cache key
    fn shard_dir(&self, cache_key: &str) -> PathBuf {
        // Use first 2 characters for sharding
        let shard = &cache_key[..2.min(cache_key.len())];
        self.cache_dir.join(shard)
    }

    /// Get the file path for a cache key
    pub fn file_path(&self, cache_key: &str) -> PathBuf {
        self.shard_dir(cache_key).join(cache_key)
    }

    /// Get the metadata path for a cache key
    fn meta_path(&self, cache_key: &str) -> PathBuf {
        self.shard_dir(cache_key)
            .join(format!("{}.meta", cache_key))
    }

    /// Look up a URL in the cache
    pub fn lookup(&self, url: &str) -> CacheLookupResult {
        let cache_key = Self::cache_key(url);
        let file_path = self.file_path(&cache_key);
        let meta_path = self.meta_path(&cache_key);

        // Check if both file and metadata exist
        if !file_path.exists() || !meta_path.exists() {
            return CacheLookupResult::Miss;
        }

        // Read metadata
        let metadata = match self.read_metadata(&meta_path) {
            Some(m) => m,
            None => return CacheLookupResult::Miss,
        };

        // Verify file size matches
        if let Ok(file_meta) = std::fs::metadata(&file_path) {
            if file_meta.len() != metadata.size {
                // Size mismatch, treat as miss
                return CacheLookupResult::Miss;
            }
        } else {
            return CacheLookupResult::Miss;
        }

        // If ETag is available and we use ETag validation
        if self.use_etag && metadata.etag.is_some() {
            return CacheLookupResult::NeedsRevalidation {
                path: file_path,
                metadata,
            };
        }

        CacheLookupResult::Hit {
            path: file_path,
            metadata,
        }
    }

    /// Read metadata from file
    fn read_metadata(&self, path: &Path) -> Option<DownloadCacheMetadata> {
        let file = std::fs::File::open(path).ok()?;
        let reader = BufReader::new(file);
        bincode::deserialize_from(reader).ok()
    }

    /// Store a downloaded file in the cache
    pub fn store(
        &self,
        url: &str,
        source_path: &Path,
        etag: Option<String>,
        last_modified: Option<String>,
        content_type: Option<String>,
    ) -> std::io::Result<PathBuf> {
        let cache_key = Self::cache_key(url);
        let shard_dir = self.shard_dir(&cache_key);
        let file_path = self.file_path(&cache_key);
        let meta_path = self.meta_path(&cache_key);

        // Ensure shard directory exists
        std::fs::create_dir_all(&shard_dir)?;

        // Get file size
        let file_meta = std::fs::metadata(source_path)?;
        let size = file_meta.len();

        // Extract filename from URL
        let filename = url.split('/').last().unwrap_or("download").to_string();

        // Create metadata
        let metadata = DownloadCacheMetadata {
            url: url.to_string(),
            size,
            etag,
            last_modified,
            content_type,
            cached_at: crate::now_epoch_secs(),
            filename,
        };

        // Atomic write: copy to temp, then rename
        let temp_path = file_path.with_extension("tmp");
        std::fs::copy(source_path, &temp_path)?;

        // On Windows, remove destination first
        if file_path.exists() {
            let _ = std::fs::remove_file(&file_path);
        }
        std::fs::rename(&temp_path, &file_path)?;

        // Write metadata
        self.write_metadata(&meta_path, &metadata)?;

        Ok(file_path)
    }

    /// Write metadata to file (atomic)
    fn write_metadata(&self, path: &Path, metadata: &DownloadCacheMetadata) -> std::io::Result<()> {
        let temp_path = path.with_extension("meta.tmp");
        let file = std::fs::File::create(&temp_path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, metadata)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Atomic rename
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        std::fs::rename(&temp_path, path)?;
        Ok(())
    }

    /// Copy cached file to destination
    pub fn copy_to(&self, url: &str, dest: &Path) -> std::io::Result<bool> {
        let cache_key = Self::cache_key(url);
        let file_path = self.file_path(&cache_key);

        if !file_path.exists() {
            return Ok(false);
        }

        // Ensure parent directory exists
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(&file_path, dest)?;
        Ok(true)
    }

    /// Check if URL is cached (quick check without full validation)
    pub fn is_cached(&self, url: &str) -> bool {
        let cache_key = Self::cache_key(url);
        let file_path = self.file_path(&cache_key);
        let meta_path = self.meta_path(&cache_key);
        file_path.exists() && meta_path.exists()
    }

    /// Get cached file path if exists (without validation)
    pub fn get_cached_path(&self, url: &str) -> Option<PathBuf> {
        let cache_key = Self::cache_key(url);
        let file_path = self.file_path(&cache_key);
        if file_path.exists() {
            Some(file_path)
        } else {
            None
        }
    }

    /// Remove a cached file
    pub fn remove(&self, url: &str) -> std::io::Result<bool> {
        let cache_key = Self::cache_key(url);
        let file_path = self.file_path(&cache_key);
        let meta_path = self.meta_path(&cache_key);

        let mut removed = false;
        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
            removed = true;
        }
        if meta_path.exists() {
            std::fs::remove_file(&meta_path)?;
        }
        Ok(removed)
    }

    /// Clear all cached downloads
    pub fn clear(&self) -> std::io::Result<u64> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                for file in std::fs::read_dir(entry.path())? {
                    let file = file?;
                    if file.file_type()?.is_file() {
                        total_size += file.metadata()?.len();
                    }
                }
                std::fs::remove_dir_all(entry.path())?;
            }
        }
        Ok(total_size)
    }

    /// Get cache statistics
    pub fn stats(&self) -> DownloadCacheStats {
        let mut stats = DownloadCacheStats::default();

        if !self.cache_dir.exists() {
            return stats;
        }

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(files) = std::fs::read_dir(entry.path()) {
                        for file in files.flatten() {
                            let path = file.path();
                            if path.extension().is_some_and(|e| e == "meta") {
                                continue; // Skip metadata files in count
                            }
                            if let Ok(meta) = file.metadata() {
                                if meta.is_file() {
                                    stats.file_count += 1;
                                    stats.total_size += meta.len();
                                }
                            }
                        }
                    }
                }
            }
        }

        stats
    }
}

/// Download cache statistics
#[derive(Debug, Clone, Default)]
pub struct DownloadCacheStats {
    /// Number of cached files
    pub file_count: usize,
    /// Total size in bytes
    pub total_size: u64,
}

impl DownloadCacheStats {
    /// Format total size as human-readable string
    pub fn formatted_size(&self) -> String {
        crate::format_size(self.total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_key_generation() {
        let url = "https://example.com/file.tar.gz";
        let key = DownloadCache::cache_key(url);
        assert_eq!(key.len(), 64); // SHA256 = 32 bytes = 64 hex chars

        // Same URL should produce same key
        assert_eq!(key, DownloadCache::cache_key(url));

        // Different URL should produce different key
        let other_key = DownloadCache::cache_key("https://example.com/other.tar.gz");
        assert_ne!(key, other_key);
    }

    #[test]
    fn test_cache_store_and_lookup() {
        let temp_dir = TempDir::new().unwrap();
        let cache = DownloadCache::new(temp_dir.path().to_path_buf());

        // Create a test file
        let source_file = temp_dir.path().join("source.txt");
        std::fs::write(&source_file, b"test content").unwrap();

        let url = "https://example.com/test.txt";

        // Store the file
        let cached_path = cache
            .store(url, &source_file, Some("etag123".to_string()), None, None)
            .unwrap();

        assert!(cached_path.exists());

        // Lookup should find it (with ETag, needs revalidation)
        match cache.lookup(url) {
            CacheLookupResult::NeedsRevalidation { path, metadata } => {
                assert_eq!(path, cached_path);
                assert_eq!(metadata.url, url);
                assert_eq!(metadata.etag, Some("etag123".to_string()));
            }
            _ => panic!("Expected NeedsRevalidation"),
        }

        // Without ETag validation
        let cache_no_etag = DownloadCache::new(temp_dir.path().to_path_buf()).with_etag(false);
        match cache_no_etag.lookup(url) {
            CacheLookupResult::Hit { path, metadata } => {
                assert_eq!(path, cached_path);
                assert_eq!(metadata.size, 12); // "test content" = 12 bytes
            }
            _ => panic!("Expected Hit"),
        }
    }
}
