//! Resolution cache for speeding up repeated resolver runs.
//!
//! This module implements a minimal disk-backed cache for resolver outputs.
//! It is intentionally conservative:
//! - Cache is best-effort (any error => treat as miss)
//! - Cache entries have TTL
//! - Cache is guarded by `CacheMode` (Normal/Refresh/Offline/NoCache)
//! - Cache value is validated lightly before use

use crate::{ResolvedGraph, ResolverConfig, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::debug;
use vx_paths::{find_config_file_upward, find_project_root, VxPaths, LOCK_FILE_NAMES};
use vx_runtime::CacheMode;

/// Current schema version for resolution cache entries.
pub const RESOLUTION_CACHE_SCHEMA_VERSION: u32 = 2;

/// Default cache subdirectory under `~/.vx/cache/`.
pub const RESOLUTION_CACHE_DIR_NAME: &str = "resolutions";

/// Cache key for resolution results.
///
/// This key is hashed (SHA256) to produce a stable filename.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolutionCacheKey {
    pub schema_version: u32,
    pub vx_version: String,
    pub os: String,
    pub arch: String,
    pub cwd: PathBuf,
    pub runtime: String,
    pub version: Option<String>,
    pub args_digest: String,
    pub config_digest: Option<String>,
    pub lock_digest: Option<String>,
    pub prefer_vx_managed: bool,
    pub fallback_to_system: bool,
}

impl ResolutionCacheKey {
    /// Build a cache key from the current process context.
    ///
    /// This hashes `args` and optionally fingerprints `vx.toml` / `vx.lock` found by upward search.
    pub fn from_context(
        runtime: &str,
        version: Option<&str>,
        args: &[String],
        config: &ResolverConfig,
    ) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let args_digest = sha256_hex(&args.join("\0"));

        let config_path = find_config_file_upward(&cwd);
        let config_digest = config_path.as_deref().and_then(|p| file_sha256_hex(p).ok());

        let project_root = find_project_root(&cwd);
        let lock_digest = project_root
            .as_deref()
            .and_then(find_lock_file)
            .and_then(|p| file_sha256_hex(&p).ok());

        Self {
            schema_version: RESOLUTION_CACHE_SCHEMA_VERSION,
            vx_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cwd,
            runtime: runtime.to_string(),
            version: version.map(|s| s.to_string()),
            args_digest,
            config_digest,
            lock_digest,
            prefer_vx_managed: config.prefer_vx_managed,
            fallback_to_system: config.fallback_to_system,
        }
    }

    /// Hash this key to a stable filename.
    pub fn hash_hex(&self) -> String {
        let json = serde_json::to_vec(self).unwrap_or_default();
        sha256_hex_bytes(&json)
    }
}

/// Disk-backed resolution cache.
#[derive(Debug, Clone)]
pub struct ResolutionCache {
    cache_dir: PathBuf,
    ttl: Duration,
    mode: CacheMode,
}

impl ResolutionCache {
    /// Create a cache pointing at the default vx cache location.
    pub fn default_paths(config: &ResolverConfig) -> Result<Self> {
        let paths = VxPaths::new()?;
        let cache_dir = paths.cache_dir.join(RESOLUTION_CACHE_DIR_NAME);
        Ok(Self::new(cache_dir)
            .with_ttl(config.resolution_cache_ttl)
            .with_mode(config.resolution_cache_mode))
    }

    /// Create a cache with a custom directory.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            ttl: Duration::from_secs(15 * 60),
            mode: CacheMode::Normal,
        }
    }

    /// Set cache TTL.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set cache mode.
    pub fn with_mode(mut self, mode: CacheMode) -> Self {
        self.mode = mode;
        self
    }

    /// Get current cache mode.
    pub fn mode(&self) -> CacheMode {
        self.mode
    }

    fn cache_file(&self, key: &ResolutionCacheKey) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key.hash_hex()))
    }

    /// Read a cached resolution result.
    ///
    /// Returns:
    /// - `None` on miss / invalid / unreadable
    /// - `Some(ResolvedGraph)` on hit
    pub fn get(&self, key: &ResolutionCacheKey) -> Option<ResolvedGraph> {
        if self.mode == CacheMode::NoCache || self.mode == CacheMode::Refresh {
            return None;
        }

        let cache_file = self.cache_file(key);
        let content = std::fs::read_to_string(&cache_file).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        if entry.schema_version != RESOLUTION_CACHE_SCHEMA_VERSION {
            return None;
        }

        // Offline mode: allow expired entries
        if self.mode != CacheMode::Offline && !entry.is_valid() {
            let _ = std::fs::remove_file(&cache_file);
            return None;
        }

        // Light validation: if cached executable is absolute, it should exist.
        if entry.value.executable.is_absolute() && !entry.value.executable.exists() {
            return None;
        }

        Some(entry.value)
    }

    /// Write a cache entry.
    pub fn set(&self, key: &ResolutionCacheKey, value: &ResolvedGraph) -> Result<()> {
        if self.mode == CacheMode::NoCache {
            return Ok(());
        }

        std::fs::create_dir_all(&self.cache_dir)?;

        let entry = CacheEntry::new(value.clone(), self.ttl, key.clone());
        let data = serde_json::to_string_pretty(&entry)?;

        let dest = self.cache_file(key);
        vx_cache::atomic_write_string(&dest, &data)?;
        Ok(())
    }

    /// Remove a single cache entry.
    pub fn remove(&self, key: &ResolutionCacheKey) -> Result<()> {
        let file = self.cache_file(key);
        if file.exists() {
            std::fs::remove_file(file)?;
        }
        Ok(())
    }

    /// Best-effort prune expired entries.
    pub fn prune_expired(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut pruned = 0;
        for entry in std::fs::read_dir(&self.cache_dir)?.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(entry) = serde_json::from_str::<CacheEntry>(&content) else {
                    continue;
                };
                if !entry.is_valid() {
                    let _ = std::fs::remove_file(&path);
                    pruned += 1;
                }
            }
        }
        Ok(pruned)
    }

    /// Remove all resolution cache entries.
    pub fn clear_all(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut removed = 0;
        for entry in std::fs::read_dir(&self.cache_dir)?.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json")
                && std::fs::remove_file(&path).is_ok()
            {
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// Get cache statistics (best-effort).
    pub fn stats(&self) -> Result<vx_cache::CacheStats> {
        let mut stats = vx_cache::CacheStats::default();

        if !self.cache_dir.exists() {
            return Ok(stats);
        }

        for entry in std::fs::read_dir(&self.cache_dir)?.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                stats.total_entries += 1;
                if let Ok(metadata) = path.metadata() {
                    stats.total_size_bytes += metadata.len();
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

        Ok(stats)
    }

    /// Get underlying cache directory.
    pub fn dir(&self) -> &Path {
        &self.cache_dir
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    schema_version: u32,
    created_at: u64,
    ttl_secs: u64,
    key: ResolutionCacheKey,
    value: ResolvedGraph,
}

impl CacheEntry {
    fn new(value: ResolvedGraph, ttl: Duration, key: ResolutionCacheKey) -> Self {
        Self {
            schema_version: RESOLUTION_CACHE_SCHEMA_VERSION,
            created_at: now_epoch_secs(),
            ttl_secs: ttl.as_secs(),
            key,
            value,
        }
    }

    fn is_valid(&self) -> bool {
        if self.schema_version != RESOLUTION_CACHE_SCHEMA_VERSION {
            return false;
        }
        let now = now_epoch_secs();
        now.saturating_sub(self.created_at) < self.ttl_secs
    }
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn sha256_hex(s: &str) -> String {
    sha256_hex_bytes(s.as_bytes())
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    hex_encode(&out)
}

fn file_sha256_hex(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_hex_bytes(&bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(LUT[(b >> 4) as usize] as char);
        s.push(LUT[(b & 0x0f) as usize] as char);
    }
    s
}

fn find_lock_file(project_root: &Path) -> Option<PathBuf> {
    for name in LOCK_FILE_NAMES {
        let p = project_root.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Helper for logging cache hit/miss.
///
/// Kept small to avoid polluting hot paths.
pub(crate) fn log_cache_result(hit: bool, runtime: &str) {
    if hit {
        debug!("Resolution cache hit for {}", runtime);
    } else {
        debug!("Resolution cache miss for {}", runtime);
    }
}
