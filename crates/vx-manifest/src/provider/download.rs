//! Download Configuration (RFC 0020)
//!
//! This module defines the download configuration for runtimes,
//! including timeout settings, retry policies, and caching behavior.
//!
//! ## Example provider.toml
//!
//! ```toml
//! [runtimes.download]
//! # Download timeout in milliseconds (default: 300000 = 5 minutes)
//! timeout_ms = 600000  # 10 minutes for large files like ffmpeg
//!
//! # Maximum number of retry attempts (default: 3)
//! max_retries = 5
//!
//! # Whether to resume interrupted downloads (default: true)
//! resume_enabled = true
//!
//! # Execution timeout after installation (default: 30000 = 30 seconds)
//! execution_timeout_ms = 30000
//! ```

use serde::{Deserialize, Serialize};

use super::defaults::{default_download_timeout, default_execution_timeout, default_max_retries};

/// Download configuration for a runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    /// Download timeout in milliseconds
    /// Default: 300000 (5 minutes)
    #[serde(default = "default_download_timeout")]
    pub timeout_ms: u64,

    /// Maximum number of retry attempts
    /// Default: 3
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to resume interrupted downloads
    /// Default: true
    #[serde(default = "default_true")]
    pub resume_enabled: bool,

    /// Execution timeout in milliseconds (after installation)
    /// This is used when running the installed tool
    /// Default: 30000 (30 seconds)
    #[serde(default = "default_execution_timeout")]
    pub execution_timeout_ms: u64,

    /// Expected file size in bytes (optional, for progress display)
    #[serde(default)]
    pub expected_size_bytes: Option<u64>,

    /// Whether to verify checksum after download
    /// Default: true
    #[serde(default = "default_true")]
    pub verify_checksum: bool,
}

fn default_true() -> bool {
    true
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_download_timeout(),
            max_retries: default_max_retries(),
            resume_enabled: true,
            execution_timeout_ms: default_execution_timeout(),
            expected_size_bytes: None,
            verify_checksum: true,
        }
    }
}

impl DownloadConfig {
    /// Create a new download config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a config for large files (10 minute timeout, 5 retries)
    pub fn for_large_file() -> Self {
        Self {
            timeout_ms: 600_000, // 10 minutes
            max_retries: 5,
            ..Default::default()
        }
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_ms)
    }

    /// Get execution timeout as Duration
    pub fn execution_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.execution_timeout_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DownloadConfig::default();
        assert_eq!(config.timeout_ms, 300_000);
        assert_eq!(config.max_retries, 3);
        assert!(config.resume_enabled);
        assert_eq!(config.execution_timeout_ms, 30_000);
    }

    #[test]
    fn test_large_file_config() {
        let config = DownloadConfig::for_large_file();
        assert_eq!(config.timeout_ms, 600_000);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_timeout_duration() {
        let config = DownloadConfig {
            timeout_ms: 60_000,
            ..Default::default()
        };
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
    }
}
