//! Configuration error types

use thiserror::Error;

/// Configuration error type
#[derive(Error, Debug)]
pub enum ConfigError {
    /// File not found
    #[error("Configuration file not found: {path}")]
    NotFound { path: String },

    /// IO error
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),

    /// IO error with message
    #[error("{0}")]
    IoError(String),

    /// TOML parsing error
    #[error("Failed to parse TOML: {0}")]
    Parse(#[from] toml::de::Error),

    /// Parse error with message
    #[error("{0}")]
    ParseError(String),

    /// Validation error
    #[error("Configuration validation failed: {message}")]
    Validation { message: String },

    /// Version mismatch
    #[error("Configuration requires vx {required}, but current version is {current}")]
    VersionMismatch { required: String, current: String },

    /// Unknown field (warning, not error by default)
    #[error("Unknown configuration field: {field}")]
    UnknownField { field: String },

    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    /// Invalid value
    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;
