//! Runtime request parsing
//!
//! This module provides parsing for runtime specifications with optional version constraints.
//! Supports formats like:
//! - `yarn` - runtime name only, use latest/default version
//! - `yarn@1.21.1` - runtime with exact version
//! - `node@20` - runtime with major version constraint
//! - `node@^18.0.0` - runtime with semver constraint

use std::fmt;

/// A parsed runtime request with optional version constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequest {
    /// The runtime name (e.g., "yarn", "node", "npm")
    pub name: String,

    /// Optional version constraint (e.g., "1.21.1", "20", "^18.0.0")
    pub version: Option<String>,
}

impl RuntimeRequest {
    /// Create a new runtime request with just a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
        }
    }

    /// Create a new runtime request with a specific version
    pub fn with_version(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: Some(version.into()),
        }
    }

    /// Parse a runtime request from a string
    ///
    /// Supports formats:
    /// - `runtime` - just the runtime name
    /// - `runtime@version` - runtime with version constraint
    ///
    /// # Examples
    ///
    /// ```
    /// use vx_resolver::RuntimeRequest;
    ///
    /// let req = RuntimeRequest::parse("yarn");
    /// assert_eq!(req.name, "yarn");
    /// assert_eq!(req.version, None);
    ///
    /// let req = RuntimeRequest::parse("yarn@1.21.1");
    /// assert_eq!(req.name, "yarn");
    /// assert_eq!(req.version, Some("1.21.1".to_string()));
    ///
    /// let req = RuntimeRequest::parse("node@20");
    /// assert_eq!(req.name, "node");
    /// assert_eq!(req.version, Some("20".to_string()));
    /// ```
    pub fn parse(spec: &str) -> Self {
        if let Some((name, version)) = spec.split_once('@') {
            Self {
                name: name.to_string(),
                version: if version.is_empty() {
                    None
                } else {
                    Some(version.to_string())
                },
            }
        } else {
            Self {
                name: spec.to_string(),
                version: None,
            }
        }
    }

    /// Check if a specific version is requested
    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    /// Get the version constraint or "latest" as default
    pub fn version_or_latest(&self) -> &str {
        self.version.as_deref().unwrap_or("latest")
    }
}

impl fmt::Display for RuntimeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(version) = &self.version {
            write!(f, "{}@{}", self.name, version)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl From<&str> for RuntimeRequest {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl From<String> for RuntimeRequest {
    fn from(s: String) -> Self {
        Self::parse(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_only() {
        let req = RuntimeRequest::parse("yarn");
        assert_eq!(req.name, "yarn");
        assert_eq!(req.version, None);
    }

    #[test]
    fn test_parse_with_exact_version() {
        let req = RuntimeRequest::parse("yarn@1.21.1");
        assert_eq!(req.name, "yarn");
        assert_eq!(req.version, Some("1.21.1".to_string()));
    }

    #[test]
    fn test_parse_with_major_version() {
        let req = RuntimeRequest::parse("node@20");
        assert_eq!(req.name, "node");
        assert_eq!(req.version, Some("20".to_string()));
    }

    #[test]
    fn test_parse_with_semver_constraint() {
        let req = RuntimeRequest::parse("node@^18.0.0");
        assert_eq!(req.name, "node");
        assert_eq!(req.version, Some("^18.0.0".to_string()));
    }

    #[test]
    fn test_parse_empty_version() {
        let req = RuntimeRequest::parse("yarn@");
        assert_eq!(req.name, "yarn");
        assert_eq!(req.version, None);
    }

    #[test]
    fn test_display() {
        let req = RuntimeRequest::with_version("yarn", "1.21.1");
        assert_eq!(format!("{}", req), "yarn@1.21.1");

        let req = RuntimeRequest::new("yarn");
        assert_eq!(format!("{}", req), "yarn");
    }

    #[test]
    fn test_version_or_latest() {
        let req = RuntimeRequest::new("yarn");
        assert_eq!(req.version_or_latest(), "latest");

        let req = RuntimeRequest::with_version("yarn", "1.21.1");
        assert_eq!(req.version_or_latest(), "1.21.1");
    }
}
