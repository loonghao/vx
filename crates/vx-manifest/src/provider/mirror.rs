use serde::{Deserialize, Serialize};

use super::defaults::{
    default_probe_timeout, default_retention_days, default_true, default_versions_ttl,
};

/// Mirror configuration for alternative download sources
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MirrorConfig {
    /// Mirror name (e.g., "taobao", "ustc")
    pub name: String,

    /// Geographic region (e.g., "cn", "us", "eu")
    #[serde(default)]
    pub region: Option<String>,

    /// Mirror URL
    pub url: String,

    /// Priority (higher = preferred)
    #[serde(default)]
    pub priority: i32,

    /// Whether this mirror is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Mirror selection strategy
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MirrorStrategy {
    /// Automatically detect best mirror based on location
    #[serde(default)]
    pub auto_detect: bool,

    /// Fall back to other mirrors on failure
    #[serde(default = "default_true")]
    pub fallback: bool,

    /// Probe mirrors in parallel to find fastest
    #[serde(default)]
    pub parallel_probe: bool,

    /// Probe timeout in milliseconds
    #[serde(default = "default_probe_timeout")]
    pub probe_timeout_ms: u64,
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Version list cache TTL in seconds
    #[serde(default = "default_versions_ttl")]
    pub versions_ttl: u64,

    /// Whether to cache downloads
    #[serde(default = "default_true")]
    pub cache_downloads: bool,

    /// Download retention in days
    #[serde(default = "default_retention_days")]
    pub downloads_retention_days: u32,

    /// Maximum cache size in MB
    #[serde(default)]
    pub max_cache_size_mb: Option<u64>,

    /// Use shared cache across projects
    #[serde(default = "default_true")]
    pub shared_cache: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            versions_ttl: 3600,
            cache_downloads: true,
            downloads_retention_days: 30,
            max_cache_size_mb: None,
            shared_cache: true,
        }
    }
}
