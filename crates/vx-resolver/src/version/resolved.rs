//! Resolved version type

use super::constraint::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A resolved version with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedVersion {
    /// Full version number
    pub version: Version,
    /// Original version string (may include platform-specific suffixes like .windows.1)
    pub original_version: Option<String>,
    /// Original request that was resolved
    pub resolved_from: String,
    /// Source (GitHub release, npm registry, etc.)
    pub source: String,
    /// Additional metadata (e.g., release_date for python-build-standalone)
    pub metadata: HashMap<String, String>,
}

impl ResolvedVersion {
    /// Create a new resolved version
    pub fn new(version: Version, resolved_from: impl Into<String>) -> Self {
        Self {
            version,
            original_version: None,
            resolved_from: resolved_from.into(),
            source: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new resolved version with original version string
    pub fn with_original(
        version: Version,
        original: impl Into<String>,
        resolved_from: impl Into<String>,
    ) -> Self {
        Self {
            version,
            original_version: Some(original.into()),
            resolved_from: resolved_from.into(),
            source: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get the version string
    /// Returns original version string if available, otherwise the parsed version
    pub fn version_string(&self) -> String {
        self.original_version
            .clone()
            .unwrap_or_else(|| self.version.to_string())
    }

    /// Get the normalized version string (parsed version)
    pub fn normalized_version_string(&self) -> String {
        self.version.to_string()
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl fmt::Display for ResolvedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version_string())
    }
}

impl From<Version> for ResolvedVersion {
    fn from(version: Version) -> Self {
        Self::new(version, "")
    }
}
