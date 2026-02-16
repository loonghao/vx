//! Error types for project analyzer

use std::path::PathBuf;
use thiserror::Error;

/// Result type for analyzer operations
pub type AnalyzerResult<T> = Result<T, AnalyzerError>;

/// Errors that can occur during project analysis
#[derive(Error, Debug)]
pub enum AnalyzerError {
    /// Project directory not found
    #[error("Project directory not found: {path}")]
    ProjectNotFound { path: PathBuf },

    /// Failed to read configuration file
    #[error("Failed to read configuration file '{path}': {reason}")]
    ConfigReadError { path: PathBuf, reason: String },

    /// Failed to parse configuration file
    #[error("Failed to parse configuration file '{path}': {reason}")]
    ConfigParseError { path: PathBuf, reason: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// TOML serialization error
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// Sync conflict
    #[error("Sync conflict: {message}")]
    SyncConflict { message: String },

    /// Installation failed
    #[error("Installation failed for '{tool}': {reason}")]
    InstallFailed { tool: String, reason: String },

    /// Configuration error from vx-config crate
    #[error("Config error: {0}")]
    Config(#[from] vx_config::ConfigError),

    /// Other error
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
