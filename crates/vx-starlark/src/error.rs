//! Error types for vx-starlark

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for vx-starlark operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for Starlark provider operations
#[derive(Error, Debug)]
pub enum Error {
    /// Script file not found
    #[error("Starlark script not found: {0}")]
    ScriptNotFound(PathBuf),

    /// Failed to parse Starlark script
    #[error("Failed to parse Starlark script: {0}")]
    ParseError(String),

    /// Failed to evaluate Starlark expression
    #[error("Failed to evaluate Starlark expression: {0}")]
    EvalError(String),

    /// Required function not found in script
    #[error("Required function '{name}' not found in provider script")]
    FunctionNotFound { name: String },

    /// Function returned wrong type
    #[error("Function '{name}' returned wrong type: expected {expected}, got {actual}")]
    TypeError {
        name: String,
        expected: String,
        actual: String,
    },

    /// Sandbox violation
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    /// File system operation denied
    #[error("File system access denied: {path} is not in allowed paths")]
    FsAccessDenied { path: PathBuf },

    /// HTTP request denied
    #[error("HTTP request denied: {host} is not in allowed hosts")]
    HttpHostDenied { host: String },

    /// Command execution denied
    #[error("Command execution denied: {command}")]
    CommandDenied { command: String },

    /// Script execution timeout
    #[error("Script execution timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Memory limit exceeded
    #[error("Script exceeded memory limit of {limit_bytes} bytes")]
    MemoryLimitExceeded { limit_bytes: usize },

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Internal Starlark error
    #[error("Starlark error: {0}")]
    Starlark(String),
}

impl Error {
    /// Create a sandbox violation error
    pub fn sandbox_violation(msg: impl Into<String>) -> Self {
        Self::SandboxViolation(msg.into())
    }

    /// Create a function not found error
    pub fn function_not_found(name: impl Into<String>) -> Self {
        Self::FunctionNotFound { name: name.into() }
    }

    /// Create a type error
    pub fn type_error(
        name: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::TypeError {
            name: name.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}
