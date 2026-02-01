//! Error types for the vx-shim crate

use thiserror::Error;

/// Result type alias for shim operations
pub type ShimResult<T> = Result<T, ShimError>;

/// Errors that can occur during shim operations
#[derive(Error, Debug)]
pub enum ShimError {
    /// Failed to parse a package request
    #[error("Invalid package request: {0}")]
    InvalidRequest(String),

    /// Executable not found in the package
    #[error("Executable '{executable}' not found in package '{package}'")]
    ExecutableNotFound { package: String, executable: String },

    /// Package not installed
    #[error("Package '{ecosystem}:{package}' is not installed")]
    PackageNotInstalled { ecosystem: String, package: String },

    /// Shim file not found
    #[error("Shim not found for executable: {0}")]
    ShimNotFound(String),

    /// Runtime dependency not satisfied
    #[error("Runtime dependency not satisfied: {runtime} {version}")]
    RuntimeNotSatisfied { runtime: String, version: String },

    /// Failed to execute the shim
    #[error("Failed to execute shim: {0}")]
    ExecutionFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
