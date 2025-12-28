//! Error types for the migration framework.

use std::path::PathBuf;
use thiserror::Error;

/// Migration error type
#[derive(Error, Debug)]
pub enum MigrationError {
    /// IO error
    #[error("IO error at {path:?}: {message}")]
    Io {
        message: String,
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },

    /// Configuration parsing error
    #[error("Config error: {message}")]
    Config {
        message: String,
        #[source]
        source: Option<toml::de::Error>,
    },

    /// Version parsing error
    #[error("Version error: {0}")]
    Version(String),

    /// Migration not found
    #[error("Migration not found: {0}")]
    NotFound(String),

    /// Migration already executed
    #[error("Migration already executed: {0}")]
    AlreadyExecuted(String),

    /// Dependency error
    #[error("Dependency error: {message}")]
    Dependency { message: String },

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Rollback error
    #[error("Rollback error: {message}")]
    Rollback {
        message: String,
        migration_id: String,
    },

    /// Context error
    #[error("Context error: {0}")]
    Context(String),

    /// Hook error
    #[error("Hook error in {hook_name}: {message}")]
    Hook { hook_name: String, message: String },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl MigrationError {
    /// Create an IO error
    pub fn io(message: impl Into<String>, path: Option<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            path,
            source,
        }
    }

    /// Create a config error
    pub fn config(message: impl Into<String>, source: Option<toml::de::Error>) -> Self {
        Self::Config {
            message: message.into(),
            source,
        }
    }

    /// Create a dependency error
    pub fn dependency(message: impl Into<String>) -> Self {
        Self::Dependency {
            message: message.into(),
        }
    }

    /// Create a rollback error
    pub fn rollback(migration_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Rollback {
            migration_id: migration_id.into(),
            message: message.into(),
        }
    }

    /// Create a hook error
    pub fn hook(hook_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Hook {
            hook_name: hook_name.into(),
            message: message.into(),
        }
    }
}

/// Result type for migration operations
pub type MigrationResult<T> = Result<T, MigrationError>;
