//! Smart cache management for vx downloads with deduplication and sharing

use crate::error::{DownloadError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

// Re-export SmartCacheConfig from vx-config
pub use vx_config::types::SmartCacheConfig;

/// Smart cache entry with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCacheEntry {
    /// Original URL
    pub url: String,
    /// Local file path
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// SHA256 checksum for deduplication
    pub checksum: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Access count for popularity tracking
    pub access_count: u64,
    /// Tool that created this cache entry
    pub tool_name: String,
    /// Version associated with this cache
    pub version: String,
    /// Content type (e.g., "binary", "archive", "installer")
    pub content_type: String,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Cache statistics with detailed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCacheStats {
    /// Total cache size in bytes
    pub total_size: u64,
    /// Number of cached entries
    pub entry_count: usize,
    /// Maximum cache size in bytes
    pub max_size: u64,
    /// Cache hit count
    pub hit_count: u64,
    /// Cache miss count
    pub miss_count: u64,
    /// Number of deduplicated files
    pub dedup_count: u64,
    /// Space saved through deduplication
    pub dedup_saved_bytes: u64,
    /// Cache enabled flag
    pub enabled: bool,
    /// Cache by tool breakdown
    pub tool_breakdown: HashMap<String, u64>,
    /// Cache by content type breakdown
    pub content_type_breakdown: HashMap<String, u64>,
}

/// Smart cache manager with deduplication and cross-tool sharing
pub struct SmartCacheManager {
    /// Cache directory
    cache_dir: PathBuf,
    /// Cache metadata
    metadata: HashMap<String, SmartCacheEntry>,
    /// Checksum to path mapping for deduplication
    checksum_map: HashMap<String, PathBuf>,
    /// Configuration
    config: SmartCacheConfig,
    /// Statistics
    stats: SmartCacheStats,
}

impl SmartCacheManager {
    /// Create a new smart cache manager
    pub fn new(config: SmartCacheConfig) -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;

        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            DownloadError::filesystem(format!("Failed to create cache directory: {}", e))
        })?;

        let mut manager = Self {
            cache_dir,
            metadata: HashMap::new(),
            checksum_map: HashMap::new(),
            config,
            stats: SmartCacheStats {
                total_size: 0,
                entry_count: 0,
                max_size: 10 * 1024 * 1024 * 1024,
                hit_count: 0,
                miss_count: 0,
                dedup_count: 0,
                dedup_saved_bytes: 0,
                enabled: true,
                tool_breakdown: HashMap::new(),
                content_type_breakdown: HashMap::new(),
            },
        };

        // Load existing metadata
        manager.load_metadata()?;
        manager.rebuild_checksum_map()?;

        info!(
            "SmartCacheManager initialized with {} entries",
            manager.metadata.len()
        );
        debug!("Cache directory: {}", manager.cache_dir.display());

        Ok(manager)
    }

    /// Get cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| DownloadError::config("Failed to get cache directory"))?
            .join("vx")
            .join("smart-cache");

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

        // Load statistics
        let stats_file = self.cache_dir.join("stats.json");
        if stats_file.exists() {
            let content = std::fs::read_to_string(&stats_file)
                .map_err(|e| DownloadError::cache(format!("Failed to read stats: {}", e)))?;

            self.stats = serde_json::from_str(&content).unwrap_or_default();
        }

        Ok(())
    }

    /// Save cache metadata to disk
    fn save_metadata(&self) -> Result<()> {
        let metadata_file = self.cache_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| DownloadError::cache(format!("Failed to serialize metadata: {}", e)))?;

        std::fs::write(&metadata_file, content)
            .map_err(|e| DownloadError::cache(format!("Failed to write metadata: {}", e)))?;

        // Save statistics
        let stats_file = self.cache_dir.join("stats.json");
        let stats_content = serde_json::to_string_pretty(&self.stats)
            .map_err(|e| DownloadError::cache(format!("Failed to serialize stats: {}", e)))?;

        std::fs::write(&stats_file, stats_content)
            .map_err(|e| DownloadError::cache(format!("Failed to write stats: {}", e)))?;

        Ok(())
    }

    /// Rebuild checksum map from metadata
    fn rebuild_checksum_map(&mut self) -> Result<()> {
        self.checksum_map.clear();

        for entry in self.metadata.values() {
            if entry.path.exists() {
                self.checksum_map
                    .insert(entry.checksum.clone(), entry.path.clone());
            }
        }

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

    /// Calculate SHA256 checksum of a file
    fn calculate_checksum(&self, file_path: &Path) -> Result<String> {
        use std::io::Read;

        let mut file = std::fs::File::open(file_path).map_err(|e| {
            DownloadError::filesystem(format!("Failed to open file for checksum: {}", e))
        })?;

        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).map_err(|e| {
                DownloadError::filesystem(format!("Failed to read file for checksum: {}", e))
            })?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Check if URL is cached and valid
    pub fn get(&mut self, url: &str) -> Option<PathBuf> {
        let key = self.cache_key(url);

        if let Some(entry) = self.metadata.get_mut(&key) {
            // Check if file still exists
            if entry.path.exists() {
                // Update access statistics
                entry.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                entry.access_count += 1;

                self.stats.hit_count += 1;

                info!("Cache hit for URL: {}", url);
                return Some(entry.path.clone());
            } else {
                // File was deleted, remove from cache
                self.metadata.remove(&key);
                warn!("Cache entry removed (file missing): {}", url);
            }
        }

        self.stats.miss_count += 1;
        debug!("Cache miss for URL: {}", url);
        None
    }

    /// Add file to cache with smart deduplication
    pub fn put(
        &mut self,
        url: &str,
        file_path: &Path,
        tool_name: &str,
        version: &str,
        content_type: &str,
    ) -> Result<()> {
        if !self.config.enable_sharing && !self.config.enable_dedup {
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

        // Calculate checksum for deduplication
        let checksum = if self.config.enable_dedup && size >= self.config.min_dedup_size {
            self.calculate_checksum(file_path)?
        } else {
            String::new()
        };

        // Check for deduplication opportunity
        let final_path = if self.config.enable_dedup && !checksum.is_empty() {
            if let Some(existing_path) = self.checksum_map.get(&checksum) {
                if existing_path.exists() {
                    info!("Deduplication: reusing existing file for {}", url);
                    self.stats.dedup_count += 1;
                    self.stats.dedup_saved_bytes += size;
                    existing_path.clone()
                } else {
                    // Existing file was deleted, update map
                    self.checksum_map.remove(&checksum);
                    file_path.to_path_buf()
                }
            } else {
                // New unique file
                self.checksum_map
                    .insert(checksum.clone(), file_path.to_path_buf());
                file_path.to_path_buf()
            }
        } else {
            file_path.to_path_buf()
        };

        // Create cache entry
        let entry = SmartCacheEntry {
            url: url.to_string(),
            path: final_path,
            size,
            checksum,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            tool_name: tool_name.to_string(),
            version: version.to_string(),
            content_type: content_type.to_string(),
            tags: vec![tool_name.to_string(), content_type.to_string()],
        };

        self.metadata.insert(key, entry);

        // Update statistics
        self.update_stats();

        // Check cache size and cleanup if needed
        self.cleanup_if_needed()?;

        // Save metadata
        self.save_metadata()?;

        Ok(())
    }

    /// Update cache statistics
    fn update_stats(&mut self) {
        self.stats.total_size = self.metadata.values().map(|entry| entry.size).sum();
        self.stats.entry_count = self.metadata.len();

        // Update tool breakdown
        self.stats.tool_breakdown.clear();
        self.stats.content_type_breakdown.clear();

        for entry in self.metadata.values() {
            *self
                .stats
                .tool_breakdown
                .entry(entry.tool_name.clone())
                .or_insert(0) += entry.size;
            *self
                .stats
                .content_type_breakdown
                .entry(entry.content_type.clone())
                .or_insert(0) += entry.size;
        }
    }

    /// Cleanup cache if it exceeds threshold
    fn cleanup_if_needed(&mut self) -> Result<()> {
        if self.stats.total_size
            <= (self.config.max_size as f64 * self.config.cleanup_threshold) as u64
        {
            return Ok(());
        }

        info!(
            "Cache cleanup triggered: {} bytes / {} bytes",
            self.stats.total_size, self.config.max_size
        );

        // Sort entries by priority (LRU + access count + size)
        let mut entries: Vec<_> = self
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by(|a, b| {
            let score_a = self.calculate_cleanup_score(&a.1);
            let score_b = self.calculate_cleanup_score(&b.1);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let target_size = (self.config.max_size as f64 * 0.7) as u64; // Clean to 70%
        let mut removed_size = 0u64;

        for (key, entry) in entries {
            if self.stats.total_size - removed_size <= target_size {
                break;
            }

            // Remove file if it's not shared (deduplicated)
            if !self.is_file_shared(&entry.path) {
                if entry.path.exists() {
                    std::fs::remove_file(&entry.path).map_err(|e| {
                        DownloadError::filesystem(format!("Failed to remove cached file: {}", e))
                    })?;
                }
            }

            removed_size += entry.size;
            self.metadata.remove(&key);

            if !entry.checksum.is_empty() {
                self.checksum_map.remove(&entry.checksum);
            }
        }

        info!("Cache cleanup completed: removed {} bytes", removed_size);
        self.update_stats();

        Ok(())
    }

    /// Calculate cleanup score (lower = higher priority for removal)
    fn calculate_cleanup_score(&self, entry: &SmartCacheEntry) -> f64 {
        let age_weight = 0.3;
        let access_weight = 0.4;
        let size_weight = 0.3;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age_score = (now - entry.last_accessed) as f64 / self.config.ttl_seconds as f64;
        let access_score = 1.0 / (entry.access_count as f64 + 1.0);
        let size_score = entry.size as f64 / (1024.0 * 1024.0 * 1024.0); // GB

        age_weight * age_score + access_weight * access_score + size_weight * size_score
    }

    /// Check if file is shared (used by multiple cache entries)
    fn is_file_shared(&self, file_path: &Path) -> bool {
        self.metadata
            .values()
            .filter(|entry| entry.path == file_path)
            .count()
            > 1
    }

    /// Get cache statistics
    pub fn stats(&self) -> SmartCacheStats {
        self.stats.clone()
    }

    /// Clear all cache
    pub fn clear(&mut self) -> Result<()> {
        for entry in self.metadata.values() {
            if entry.path.exists() && !self.is_file_shared(&entry.path) {
                std::fs::remove_file(&entry.path).map_err(|e| {
                    DownloadError::filesystem(format!("Failed to remove cached file: {}", e))
                })?;
            }
        }

        self.metadata.clear();
        self.checksum_map.clear();
        self.stats = SmartCacheStats::default();
        self.save_metadata()?;

        info!("Cache cleared");
        Ok(())
    }
}

impl SmartCacheStats {
    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total_requests = self.hit_count + self.miss_count;
        if total_requests == 0 {
            0.0
        } else {
            self.hit_count as f64 / total_requests as f64
        }
    }

    /// Get deduplication efficiency
    pub fn dedup_efficiency(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            self.dedup_saved_bytes as f64 / (self.total_size + self.dedup_saved_bytes) as f64
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

    /// Get human-readable saved space
    pub fn saved_space_human(&self) -> String {
        let size = self.dedup_saved_bytes as f64;
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

impl Default for SmartCacheStats {
    fn default() -> Self {
        Self {
            total_size: 0,
            entry_count: 0,
            max_size: 10 * 1024 * 1024 * 1024,
            hit_count: 0,
            miss_count: 0,
            dedup_count: 0,
            dedup_saved_bytes: 0,
            enabled: true,
            tool_breakdown: HashMap::new(),
            content_type_breakdown: HashMap::new(),
        }
    }
}
