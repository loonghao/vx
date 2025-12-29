//! Error types for vx-setup

use thiserror::Error;

/// Setup-specific errors
#[derive(Error, Debug)]
pub enum SetupError {
    /// Hook execution failed
    #[error("Hook '{name}' failed: {message}")]
    HookFailed { name: String, message: String },

    /// Path export failed
    #[error("Failed to export paths: {0}")]
    PathExportFailed(String),

    /// CI environment error
    #[error("CI environment error: {0}")]
    CiEnvironmentError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Result type for setup operations
pub type SetupResult<T> = Result<T, SetupError>;
