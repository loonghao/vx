//! Error types for version management operations

use std::fmt;

/// Result type alias for version operations
pub type Result<T> = std::result::Result<T, VersionError>;

/// Error types that can occur during version operations
#[derive(Debug)]
pub enum VersionError {
    /// Invalid version format
    InvalidVersion { version: String, reason: String },

    /// Network error during version fetching
    NetworkError { url: String, source: reqwest::Error },

    /// HTTP error with status code
    HttpError {
        url: String,
        status: u16,
        message: String,
    },

    /// API rate limit exceeded
    RateLimited { message: String },

    /// JSON parsing error
    ParseError {
        content: String,
        source: serde_json::Error,
    },

    /// Version not found
    VersionNotFound { version: String, tool: String },

    /// Tool not found in system
    ToolNotFound { tool: String },

    /// Command execution error
    CommandError {
        command: String,
        source: std::io::Error,
    },

    /// Generic error
    Other { message: String },
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionError::InvalidVersion { version, reason } => {
                write!(f, "Invalid version '{}': {}", version, reason)
            }
            VersionError::NetworkError { url, source } => {
                write!(f, "Network error fetching from '{}': {}", url, source)
            }
            VersionError::HttpError {
                url,
                status,
                message,
            } => {
                write!(f, "HTTP error {} from '{}': {}", status, url, message)
            }
            VersionError::RateLimited { message } => {
                write!(f, "Rate limit exceeded: {}", message)
            }
            VersionError::ParseError { content, source } => {
                write!(f, "Failed to parse content '{}': {}", content, source)
            }
            VersionError::VersionNotFound { version, tool } => {
                write!(f, "Version '{}' not found for tool '{}'", version, tool)
            }
            VersionError::ToolNotFound { tool } => {
                write!(f, "Tool '{}' not found in system", tool)
            }
            VersionError::CommandError { command, source } => {
                write!(f, "Command '{}' failed: {}", command, source)
            }
            VersionError::Other { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for VersionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VersionError::NetworkError { source, .. } => Some(source),
            VersionError::ParseError { source, .. } => Some(source),
            VersionError::CommandError { source, .. } => Some(source),
            _ => None,
        }
    }
}

// Conversion from other error types
impl From<reqwest::Error> for VersionError {
    fn from(err: reqwest::Error) -> Self {
        VersionError::NetworkError {
            url: err
                .url()
                .map(|u| u.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            source: err,
        }
    }
}

impl From<serde_json::Error> for VersionError {
    fn from(err: serde_json::Error) -> Self {
        VersionError::ParseError {
            content: "unknown".to_string(),
            source: err,
        }
    }
}

impl From<std::io::Error> for VersionError {
    fn from(err: std::io::Error) -> Self {
        VersionError::CommandError {
            command: "unknown".to_string(),
            source: err,
        }
    }
}

impl From<anyhow::Error> for VersionError {
    fn from(err: anyhow::Error) -> Self {
        VersionError::Other {
            message: err.to_string(),
        }
    }
}
