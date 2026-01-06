//! Error types for manifest operations

use thiserror::Error;

/// Errors that can occur when working with manifests
#[derive(Debug, Error)]
pub enum ManifestError {
    /// IO error reading manifest file
    #[error("Failed to read manifest file: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error
    #[error("Failed to parse manifest TOML: {0}")]
    Parse(#[from] toml::de::Error),

    /// Validation error
    #[error("Manifest validation error: {0}")]
    Validation(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
}
