//! Error types for vx-installer

use std::path::PathBuf;

/// Result type alias for vx-installer operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur during installation operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Download failed
    #[error("Download failed from {url}: {reason}")]
    DownloadFailed { url: String, reason: String },

    /// Installation failed
    #[error("Installation failed for {tool_name} v{version}: {message}")]
    InstallationFailed {
        tool_name: String,
        version: String,
        message: String,
    },

    /// Archive extraction failed
    #[error("Failed to extract archive {archive_path}: {reason}")]
    ExtractionFailed {
        archive_path: PathBuf,
        reason: String,
    },

    /// Unsupported archive format
    #[error("Unsupported archive format: {format}")]
    UnsupportedFormat { format: String },

    /// Executable not found after installation
    #[error("Executable not found for {tool_name} in {search_path}")]
    ExecutableNotFound {
        tool_name: String,
        search_path: PathBuf,
    },

    /// Checksum verification failed
    #[error("Checksum verification failed for {file_path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        file_path: PathBuf,
        expected: String,
        actual: String,
    },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// Permission denied
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    /// Tool already installed
    #[error("Tool {tool_name} v{version} is already installed")]
    AlreadyInstalled { tool_name: String, version: String },

    /// Disk space insufficient
    #[error("Insufficient disk space: required {required} bytes, available {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },

    /// Network timeout
    #[error("Network timeout while downloading from {url}")]
    NetworkTimeout { url: String },

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Walkdir error
    #[error("Directory traversal error: {0}")]
    Walkdir(#[from] walkdir::Error),

    /// Custom error for tool-specific issues
    #[error("Tool-specific error: {message}")]
    ToolSpecific { message: String },
}

impl Error {
    /// Create a download failed error
    pub fn download_failed(url: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::DownloadFailed {
            url: url.into(),
            reason: reason.into(),
        }
    }

    /// Create an installation failed error
    pub fn installation_failed(
        tool_name: impl Into<String>,
        version: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InstallationFailed {
            tool_name: tool_name.into(),
            version: version.into(),
            message: message.into(),
        }
    }

    /// Create an extraction failed error
    pub fn extraction_failed(archive_path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::ExtractionFailed {
            archive_path: archive_path.into(),
            reason: reason.into(),
        }
    }

    /// Create an unsupported format error
    pub fn unsupported_format(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
        }
    }

    /// Create an executable not found error
    pub fn executable_not_found(
        tool_name: impl Into<String>,
        search_path: impl Into<PathBuf>,
    ) -> Self {
        Self::ExecutableNotFound {
            tool_name: tool_name.into(),
            search_path: search_path.into(),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::NetworkTimeout { .. }
                | Error::Http(_)
                | Error::DownloadFailed { .. }
                | Error::InsufficientSpace { .. }
        )
    }

    /// Check if this error is related to network issues
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            Error::Http(_) | Error::DownloadFailed { .. } | Error::NetworkTimeout { .. }
        )
    }
}
