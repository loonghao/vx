//! Error types for system package manager operations

use thiserror::Error;

/// Result type for system package manager operations
pub type Result<T> = std::result::Result<T, SystemPmError>;

/// System package manager errors
#[derive(Error, Debug)]
pub enum SystemPmError {
    /// Package manager not found
    #[error("Package manager '{0}' not found")]
    PackageManagerNotFound(String),

    /// Package manager not installed
    #[error("Package manager '{0}' is not installed")]
    PackageManagerNotInstalled(String),

    /// Package not found
    #[error("Package '{0}' not found in {1}")]
    PackageNotFound(String, String),

    /// Installation failed
    #[error("Failed to install '{package}': {reason}")]
    InstallationFailed { package: String, reason: String },

    /// Dependency resolution failed
    #[error("Failed to resolve dependency '{0}': {1}")]
    DependencyResolutionFailed(String, String),

    /// Unsupported platform
    #[error("Package manager '{0}' is not supported on this platform")]
    UnsupportedPlatform(String),

    /// Elevation required
    #[error("Administrator/root privileges required: {0}")]
    ElevationRequired(String),

    /// Command execution failed
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
