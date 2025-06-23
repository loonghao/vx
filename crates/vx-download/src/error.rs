//! Error types for vx download manager

use thiserror::Error;

/// Result type for download operations
pub type Result<T> = std::result::Result<T, DownloadError>;

/// Download error types
#[derive(Error, Debug)]
pub enum DownloadError {
    /// Network-related errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// File system errors
    #[error("File system error: {message}")]
    FileSystem { message: String },

    /// URL parsing errors
    #[error("Invalid URL: {url} - {reason}")]
    InvalidUrl { url: String, reason: String },

    /// Tool not found errors
    #[error("Tool '{tool}' not found or not supported")]
    ToolNotFound { tool: String },

    /// Version not found errors
    #[error("Version '{version}' not found for tool '{tool}'")]
    VersionNotFound { tool: String, version: String },

    /// Download timeout
    #[error("Download timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// Checksum verification failed
    #[error("Checksum verification failed for {filename}")]
    ChecksumMismatch { filename: String },

    /// Cache errors
    #[error("Cache error: {message}")]
    Cache { message: String },

    /// Turbo CDN errors
    #[error("Turbo CDN error: {0}")]
    TurboCdn(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors
    #[error("Download failed: {message}")]
    Other { message: String },
}

impl DownloadError {
    /// Create a network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a file system error
    pub fn filesystem<S: Into<String>>(message: S) -> Self {
        Self::FileSystem {
            message: message.into(),
        }
    }

    /// Create an invalid URL error
    pub fn invalid_url<S: Into<String>>(url: S, reason: S) -> Self {
        Self::InvalidUrl {
            url: url.into(),
            reason: reason.into(),
        }
    }

    /// Create a tool not found error
    pub fn tool_not_found<S: Into<String>>(tool: S) -> Self {
        Self::ToolNotFound { tool: tool.into() }
    }

    /// Create a version not found error
    pub fn version_not_found<S: Into<String>>(tool: S, version: S) -> Self {
        Self::VersionNotFound {
            tool: tool.into(),
            version: version.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Create a checksum mismatch error
    pub fn checksum_mismatch<S: Into<String>>(filename: S) -> Self {
        Self::ChecksumMismatch {
            filename: filename.into(),
        }
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(message: S) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}
