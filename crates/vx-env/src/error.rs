//! Error types for vx-env

use thiserror::Error;

/// Errors that can occur during environment operations
#[derive(Error, Debug)]
pub enum EnvError {
    /// Failed to create temporary script file
    #[error("Failed to create script file: {0}")]
    ScriptCreation(#[from] std::io::Error),

    /// Failed to execute script
    #[error("Failed to execute script: {0}")]
    Execution(String),

    /// Failed to parse command string
    #[error("Failed to parse command: {0}")]
    CommandParse(String),

    /// Shell not found
    #[error("Shell not found: {shell}. Tried: {tried:?}")]
    ShellNotFound { shell: String, tried: Vec<String> },

    /// Invalid environment variable
    #[error("Invalid environment variable: {name}={value}")]
    InvalidEnvVar { name: String, value: String },
}
