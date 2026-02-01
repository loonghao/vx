//! Error types for ecosystem package managers

use thiserror::Error;

/// Result type for ecosystem package manager operations
pub type Result<T> = std::result::Result<T, EcosystemPmError>;

/// Errors that can occur during ecosystem package manager operations
#[derive(Error, Debug)]
pub enum EcosystemPmError {
    /// Package manager not found in PATH
    #[error("{manager} not found in PATH. Please install {runtime} first.")]
    PackageManagerNotFound {
        /// The package manager that was not found
        manager: String,
        /// The runtime that needs to be installed
        runtime: String,
    },

    /// Package installation failed
    #[error("Failed to install {package}: {message}")]
    InstallFailed {
        /// The package that failed to install
        package: String,
        /// Error message
        message: String,
    },

    /// Virtual environment creation failed
    #[error("Failed to create virtual environment: {0}")]
    VenvCreationFailed(String),

    /// Unsupported ecosystem
    #[error("Unsupported ecosystem: {0}")]
    UnsupportedEcosystem(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Command execution error
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
}
