//! Persistent version list cache for Starlark providers.
//!
//! # Design
//!
//! Two-level cache architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  L1: In-memory cache (per-process, instant lookup)          │
//! │      Key: provider_name                                      │
//! │      Value: (Vec<VersionInfo>, Instant)                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  L2: Disk cache (~/.vx/cache/versions/<name>.json)          │
//! │      Key: provider_name + script_hash                        │
//! │      Value: CachedVersions { versions, cached_at, ttl_secs }│
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! Cache invalidation strategy:
//! - **TTL-based**: entries expire after `ttl_secs` (default 24h)
//! - **Content-hash-based**: if the provider.star script changes (new hash),
//!   the cached entry is considered stale regardless of TTL
//!
//! This is inspired by Buck2's content-addressed caching and pnpm's
//! content-addressable store.

use crate::context::VersionInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

/// Default TTL for version cache entries: 24 hours
pub const DEFAULT_VERSION_CACHE_TTL_SECS: u64 = 24 * 60 * 60;

/// Short TTL for development/testing: 5 minutes
pub const DEV_VERSION_CACHE_TTL_SECS: u64 = 5 * 60;

// ---------------------------------------------------------------------------
// Serializable cache entry (for disk persistence)
// ---------------------------------------------------------------------------

/// A serializable version cache entry stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedVersionEntry {
    /// Provider name
    pub provider: String,
    /// SHA256 (or fast hash) of the provider.star content at cache time.
    /// If the script changes, this hash won't match and the entry is stale.
    pub script_hash_hex: String,
    /// Cached version list
    pub versions: Vec<CachedVersionInfo>,
    /// Unix timestamp (seconds) when this entry was cached
    pub cached_at_unix: u64,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Cache format version (for future migrations)
    pub format_version: u8,
}

impl CachedVersionEntry {
    /// Check if this entry has expired based on TTL
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.cached_at_unix) > self.ttl_secs
    }

    /// Check if this entry is still valid for the given script hash
    pub fn is_valid_for_hash(&self, script_hash_hex: &str) -> bool {
        !self.is_expired() && self.script_hash_hex == script_hash_hex
    }

    /// Age of this entry in seconds
    pub fn age_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.cached_at_unix)
    }
}

/// Serializable version info (mirrors `VersionInfo` but with serde support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedVersionInfo {
    pub version: String,
    pub lts: bool,
    pub stable: bool,
    pub date: Option<String>,
}

impl From<&VersionInfo> for CachedVersionInfo {
    fn from(v: &VersionInfo) -> Self {
        Self {
            version: v.version.clone(),
            lts: v.lts,
            stable: v.stable,
            date: v.date.clone(),
        }
    }
}

impl From<CachedVersionInfo> for VersionInfo {
    fn from(v: CachedVersionInfo) -> Self {
        Self {
            version: v.version,
            lts: v.lts,
            stable: v.stable,
            date: v.date,
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory L1 cache entry
// ---------------------------------------------------------------------------

struct MemoryCacheEntry {
    versions: Vec<VersionInfo>,
    cached_at: Instant,
    ttl: Duration,
    script_hash_hex: String,
}

impl MemoryCacheEntry {
    fn is_valid(&self, script_hash_hex: &str) -> bool {
        self.cached_at.elapsed() < self.ttl && self.script_hash_hex == script_hash_hex
    }
}

// ---------------------------------------------------------------------------
// VersionCache
// ---------------------------------------------------------------------------

/// Two-level version list cache (L1: memory, L2: disk)
///
/// Thread-safe via `Arc<RwLock<...>>`. Clone is cheap (Arc clone).
#[derive(Clone)]
pub struct VersionCache {
    /// L1: in-memory cache
    memory: Arc<RwLock<HashMap<String, MemoryCacheEntry>>>,
    /// L2: disk cache directory (`~/.vx/cache/versions/`)
    cache_dir: PathBuf,
    /// Default TTL for new entries
    ttl: Duration,
}

impl VersionCache {
    /// Create a new VersionCache with the given cache directory and TTL
    pub fn new(cache_dir: impl Into<PathBuf>, ttl_secs: u64) -> Self {
        Self {
            memory: Arc::new(RwLock::new(HashMap::new())),
            cache_dir: cache_dir.into(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Create a VersionCache using the default vx cache directory
    pub fn with_default_dir() -> Self {
        let cache_dir = vx_paths::VxPaths::new()
            .map(|p| p.cache_dir.join("versions"))
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_default()
                    .join(".vx")
                    .join("cache")
                    .join("versions")
            });
        Self::new(cache_dir, DEFAULT_VERSION_CACHE_TTL_SECS)
    }

    /// Look up versions from cache (L1 → L2 → miss)
    ///
    /// Returns `Some(versions)` on cache hit, `None` on miss or stale entry.
    pub async fn get(&self, provider: &str, script_hash_hex: &str) -> Option<Vec<VersionInfo>> {
        // L1: memory cache
        {
            let mem = self.memory.read().await;
            if let Some(entry) = mem.get(provider)
                && entry.is_valid(script_hash_hex)
            {
                trace!(
                    provider = %provider,
                    age_ms = %entry.cached_at.elapsed().as_millis(),
                    "L1 version cache hit"
                );
                return Some(entry.versions.clone());
            }
        }

        // L2: disk cache
        if let Some(versions) = self.read_disk_cache(provider, script_hash_hex).await {
            // Promote to L1
            self.insert_memory(provider, script_hash_hex, versions.clone())
                .await;
            return Some(versions);
        }

        None
    }

    /// Store versions in both L1 and L2 cache
    pub async fn put(&self, provider: &str, script_hash_hex: &str, versions: &[VersionInfo]) {
        // L1: memory
        self.insert_memory(provider, script_hash_hex, versions.to_vec())
            .await;

        // L2: disk
        self.write_disk_cache(provider, script_hash_hex, versions)
            .await;
    }

    /// Invalidate all cache entries for a provider (both L1 and L2)
    pub async fn invalidate(&self, provider: &str) {
        // L1
        {
            let mut mem = self.memory.write().await;
            mem.remove(provider);
        }

        // L2
        let disk_path = self.disk_path(provider);
        if disk_path.exists() {
            if let Err(e) = std::fs::remove_file(&disk_path) {
                warn!(provider = %provider, error = %e, "Failed to remove disk cache entry");
            } else {
                debug!(provider = %provider, "Invalidated disk version cache");
            }
        }
    }

    /// Invalidate all cache entries (both L1 and L2)
    pub async fn invalidate_all(&self) {
        // L1
        {
            let mut mem = self.memory.write().await;
            mem.clear();
        }

        // L2: remove all .json files in cache_dir
        if self.cache_dir.exists() {
            match std::fs::read_dir(&self.cache_dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "json").unwrap_or(false) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to read version cache directory for cleanup");
                }
            }
        }

        debug!("Invalidated all version caches");
    }

    /// Get cache statistics
    pub async fn stats(&self) -> VersionCacheStats {
        let mem = self.memory.read().await;
        let memory_entries = mem.len();
        drop(mem);

        let disk_entries = if self.cache_dir.exists() {
            std::fs::read_dir(&self.cache_dir)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "json")
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        };

        VersionCacheStats {
            memory_entries,
            disk_entries,
            cache_dir: self.cache_dir.clone(),
            ttl_secs: self.ttl.as_secs(),
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn disk_path(&self, provider: &str) -> PathBuf {
        // Sanitize provider name for use as filename
        let safe_name: String = provider
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.cache_dir.join(format!("{}.json", safe_name))
    }

    async fn insert_memory(
        &self,
        provider: &str,
        script_hash_hex: &str,
        versions: Vec<VersionInfo>,
    ) {
        let mut mem = self.memory.write().await;
        mem.insert(
            provider.to_string(),
            MemoryCacheEntry {
                versions,
                cached_at: Instant::now(),
                ttl: self.ttl,
                script_hash_hex: script_hash_hex.to_string(),
            },
        );
    }

    async fn read_disk_cache(
        &self,
        provider: &str,
        script_hash_hex: &str,
    ) -> Option<Vec<VersionInfo>> {
        let path = self.disk_path(provider);
        if !path.exists() {
            return None;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warn!(provider = %provider, error = %e, "Failed to read disk version cache");
                return None;
            }
        };

        let entry: CachedVersionEntry = match serde_json::from_str(&content) {
            Ok(e) => e,
            Err(e) => {
                warn!(provider = %provider, error = %e, "Failed to parse disk version cache, ignoring");
                return None;
            }
        };

        if !entry.is_valid_for_hash(script_hash_hex) {
            if entry.is_expired() {
                debug!(
                    provider = %provider,
                    age_secs = %entry.age_secs(),
                    ttl_secs = %entry.ttl_secs,
                    "Disk version cache expired"
                );
            } else {
                debug!(
                    provider = %provider,
                    "Disk version cache stale (script changed)"
                );
            }
            return None;
        }

        debug!(
            provider = %provider,
            count = %entry.versions.len(),
            age_secs = %entry.age_secs(),
            "L2 disk version cache hit"
        );

        Some(entry.versions.into_iter().map(VersionInfo::from).collect())
    }

    async fn write_disk_cache(
        &self,
        provider: &str,
        script_hash_hex: &str,
        versions: &[VersionInfo],
    ) {
        // Ensure cache directory exists
        if let Err(e) = std::fs::create_dir_all(&self.cache_dir) {
            warn!(
                dir = %self.cache_dir.display(),
                error = %e,
                "Failed to create version cache directory"
            );
            return;
        }

        let entry = CachedVersionEntry {
            provider: provider.to_string(),
            script_hash_hex: script_hash_hex.to_string(),
            versions: versions.iter().map(CachedVersionInfo::from).collect(),
            cached_at_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl_secs: self.ttl.as_secs(),
            format_version: 1,
        };

        let path = self.disk_path(provider);
        match serde_json::to_string_pretty(&entry) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    warn!(
                        provider = %provider,
                        path = %path.display(),
                        error = %e,
                        "Failed to write disk version cache"
                    );
                } else {
                    debug!(
                        provider = %provider,
                        count = %versions.len(),
                        path = %path.display(),
                        "Wrote version cache to disk"
                    );
                }
            }
            Err(e) => {
                warn!(provider = %provider, error = %e, "Failed to serialize version cache");
            }
        }
    }
}

impl std::fmt::Debug for VersionCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VersionCache")
            .field("cache_dir", &self.cache_dir)
            .field("ttl_secs", &self.ttl.as_secs())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

/// Version cache statistics
#[derive(Debug, Clone)]
pub struct VersionCacheStats {
    /// Number of entries in the L1 memory cache
    pub memory_entries: usize,
    /// Number of entries in the L2 disk cache
    pub disk_entries: usize,
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// TTL in seconds
    pub ttl_secs: u64,
}

impl std::fmt::Display for VersionCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VersionCache {{ memory: {}, disk: {}, ttl: {}h, dir: {} }}",
            self.memory_entries,
            self.disk_entries,
            self.ttl_secs / 3600,
            self.cache_dir.display()
        )
    }
}

// ---------------------------------------------------------------------------
// Global version cache instance
// ---------------------------------------------------------------------------

/// Global version cache (lazy-initialized, shared across all providers)
static GLOBAL_VERSION_CACHE: once_cell::sync::Lazy<VersionCache> =
    once_cell::sync::Lazy::new(VersionCache::with_default_dir);

/// Get the global version cache instance
pub fn global_version_cache() -> &'static VersionCache {
    &GLOBAL_VERSION_CACHE
}
