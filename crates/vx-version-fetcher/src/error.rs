//! Error types for version fetching

use thiserror::Error;

/// Errors that can occur during version fetching
#[derive(Debug, Error)]
pub enum FetchError {
    /// Network error (HTTP request failed)
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid response format
    #[error("Invalid response format from {0}: {1}")]
    InvalidFormat(String, String),

    /// No versions found
    #[error("No versions found for {0}")]
    NoVersionsFound(String),

    /// Rate limited
    #[error("Rate limited by {0}. Try again later or use a different data source.")]
    RateLimited(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Other error
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl FetchError {
    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create an invalid format error
    pub fn invalid_format(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidFormat(source.into(), message.into())
    }

    /// Create a no versions found error
    pub fn no_versions(tool: impl Into<String>) -> Self {
        Self::NoVersionsFound(tool.into())
    }

    /// Create a rate limited error
    pub fn rate_limited(source: impl Into<String>) -> Self {
        Self::RateLimited(source.into())
    }
}

/// Result type for version fetching operations
pub type FetchResult<T> = Result<T, FetchError>;
