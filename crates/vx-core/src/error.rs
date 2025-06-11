//! Error types for vx

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type alias for vx operations
pub type Result<T> = std::result::Result<T, VxError>;

/// Main error type for vx operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VxError {
    /// Tool not found
    ToolNotFound { tool_name: String },
    
    /// Tool not installed
    ToolNotInstalled { tool_name: String },
    
    /// Version not found
    VersionNotFound { tool_name: String, version: String },
    
    /// Installation failed
    InstallationFailed { tool_name: String, version: String, reason: String },
    
    /// Download failed
    DownloadFailed { url: String, reason: String },
    
    /// Configuration error
    ConfigError { message: String },
    
    /// IO error
    IoError { message: String },
    
    /// Network error
    NetworkError { message: String },
    
    /// Parse error
    ParseError { message: String },
    
    /// Plugin error
    PluginError { plugin_name: String, message: String },
    
    /// Package manager error
    PackageManagerError { manager: String, message: String },
    
    /// Permission error
    PermissionError { message: String },
    
    /// Checksum verification failed
    ChecksumError { expected: String, actual: String },
    
    /// Unsupported operation
    UnsupportedOperation { operation: String, reason: String },
    
    /// Generic error
    Other { message: String },
}

impl fmt::Display for VxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VxError::ToolNotFound { tool_name } => {
                write!(f, "Tool '{}' not found", tool_name)
            }
            VxError::ToolNotInstalled { tool_name } => {
                write!(f, "Tool '{}' is not installed", tool_name)
            }
            VxError::VersionNotFound { tool_name, version } => {
                write!(f, "Version '{}' not found for tool '{}'", version, tool_name)
            }
            VxError::InstallationFailed { tool_name, version, reason } => {
                write!(f, "Failed to install {} {}: {}", tool_name, version, reason)
            }
            VxError::DownloadFailed { url, reason } => {
                write!(f, "Failed to download from {}: {}", url, reason)
            }
            VxError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            VxError::IoError { message } => {
                write!(f, "IO error: {}", message)
            }
            VxError::NetworkError { message } => {
                write!(f, "Network error: {}", message)
            }
            VxError::ParseError { message } => {
                write!(f, "Parse error: {}", message)
            }
            VxError::PluginError { plugin_name, message } => {
                write!(f, "Plugin '{}' error: {}", plugin_name, message)
            }
            VxError::PackageManagerError { manager, message } => {
                write!(f, "Package manager '{}' error: {}", manager, message)
            }
            VxError::PermissionError { message } => {
                write!(f, "Permission error: {}", message)
            }
            VxError::ChecksumError { expected, actual } => {
                write!(f, "Checksum verification failed: expected {}, got {}", expected, actual)
            }
            VxError::UnsupportedOperation { operation, reason } => {
                write!(f, "Unsupported operation '{}': {}", operation, reason)
            }
            VxError::Other { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for VxError {}

// Conversion from common error types
impl From<std::io::Error> for VxError {
    fn from(err: std::io::Error) -> Self {
        VxError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for VxError {
    fn from(err: reqwest::Error) -> Self {
        VxError::NetworkError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for VxError {
    fn from(err: serde_json::Error) -> Self {
        VxError::ParseError {
            message: err.to_string(),
        }
    }
}

impl From<toml::de::Error> for VxError {
    fn from(err: toml::de::Error) -> Self {
        VxError::ParseError {
            message: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for VxError {
    fn from(err: anyhow::Error) -> Self {
        VxError::Other {
            message: err.to_string(),
        }
    }
}

// Helper macros for creating errors
#[macro_export]
macro_rules! tool_not_found {
    ($tool:expr) => {
        VxError::ToolNotFound {
            tool_name: $tool.to_string(),
        }
    };
}

#[macro_export]
macro_rules! tool_not_installed {
    ($tool:expr) => {
        VxError::ToolNotInstalled {
            tool_name: $tool.to_string(),
        }
    };
}

#[macro_export]
macro_rules! version_not_found {
    ($tool:expr, $version:expr) => {
        VxError::VersionNotFound {
            tool_name: $tool.to_string(),
            version: $version.to_string(),
        }
    };
}
