//! Error types for vx-config

use std::fmt;

/// Result type alias for vx-config operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Configuration-related errors
#[derive(Debug)]
pub enum ConfigError {
    /// IO error when reading/writing configuration files
    Io {
        message: String,
        source: std::io::Error,
    },
    /// Error parsing configuration files
    Parse { message: String, file_type: String },
    /// Configuration validation error
    Validation { message: String },
    /// Project detection error
    Detection { message: String },
    /// Generic configuration error
    Other { message: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io { message, .. } => write!(f, "IO error: {}", message),
            ConfigError::Parse { message, file_type } => {
                write!(f, "Parse error in {}: {}", file_type, message)
            }
            ConfigError::Validation { message } => write!(f, "Validation error: {}", message),
            ConfigError::Detection { message } => write!(f, "Detection error: {}", message),
            ConfigError::Other { message } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::Parse {
            message: err.to_string(),
            file_type: "TOML".to_string(),
        }
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::Parse {
            message: err.to_string(),
            file_type: "TOML".to_string(),
        }
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Parse {
            message: err.to_string(),
            file_type: "JSON".to_string(),
        }
    }
}

impl From<figment::Error> for ConfigError {
    fn from(err: figment::Error) -> Self {
        ConfigError::Other {
            message: format!("Figment error: {}", err),
        }
    }
}
