//! Version parsing utilities
//!
//! This module provides utilities for parsing and normalizing version strings
//! from various sources like GitHub tags and tool output.

use crate::error::{ConfigError, Result};
use crate::types::{ToolConfig, VersionParsingConfig};
use regex::Regex;
use semver::Version;

/// Version parser for tools
pub struct VersionParser {
    config: VersionParsingConfig,
}

impl VersionParser {
    /// Create a new version parser with the given configuration
    pub fn new(config: VersionParsingConfig) -> Self {
        Self { config }
    }

    /// Create a version parser from tool configuration
    pub fn from_tool_config(tool_config: &ToolConfig) -> Option<Self> {
        tool_config
            .version_parsing
            .as_ref()
            .map(|config| Self::new(config.clone()))
    }

    /// Parse version from a GitHub release tag
    pub fn parse_tag(&self, tag: &str) -> Result<String> {
        if let Some(tag_regex) = &self.config.tag_regex {
            let regex = Regex::new(tag_regex).map_err(|e| ConfigError::Other {
                message: format!("Invalid tag regex '{}': {}", tag_regex, e),
            })?;

            if let Some(captures) = regex.captures(tag) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str();
                    return self.normalize_version(version);
                }
            }
        }

        // Fallback: return the tag as-is
        self.normalize_version(tag)
    }

    /// Parse version from tool output
    pub fn parse_output(&self, output: &str) -> Result<Option<String>> {
        if let Some(output_regex) = &self.config.output_regex {
            let regex = Regex::new(output_regex).map_err(|e| ConfigError::Other {
                message: format!("Invalid output regex '{}': {}", output_regex, e),
            })?;

            if let Some(captures) = regex.captures(output) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str();
                    return Ok(Some(self.normalize_version(version)?));
                }
            }
        }

        Ok(None)
    }

    /// Normalize version to semver format if configured
    fn normalize_version(&self, version: &str) -> Result<String> {
        if self.config.normalize_semver.unwrap_or(true) {
            // Try to parse as semver
            match Version::parse(version) {
                Ok(semver) => Ok(semver.to_string()),
                Err(_) => {
                    // Try to fix common version formats
                    let normalized = normalize_version_string(version);
                    match Version::parse(&normalized) {
                        Ok(semver) => Ok(semver.to_string()),
                        Err(_) => {
                            // If still can't parse, return original
                            Ok(version.to_string())
                        }
                    }
                }
            }
        } else {
            Ok(version.to_string())
        }
    }
}

/// Normalize common version string formats to semver
fn normalize_version_string(version: &str) -> String {
    let mut normalized = version.trim();

    // Remove common prefixes
    if normalized.starts_with('v') {
        normalized = &normalized[1..];
    }

    // Handle Go-style versions (e.g., "1.21" -> "1.21.0")
    if normalized.matches('.').count() == 1 {
        return format!("{}.0", normalized);
    }

    // Handle single number versions (e.g., "1" -> "1.0.0")
    if !normalized.contains('.') && normalized.parse::<u32>().is_ok() {
        return format!("{}.0.0", normalized);
    }

    normalized.to_string()
}

/// Extract version from various tool outputs
pub fn extract_version_from_output(output: &str, tool_name: &str) -> Option<String> {
    // Common patterns for different tools
    let patterns = match tool_name {
        "bun" => vec![
            r"bun\s+([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
            r"([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
        ],
        "node" => vec![
            r"v([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
            r"([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
        ],
        "go" => vec![
            r"go version go([0-9]+\.[0-9]+(?:\.[0-9]+)?(?:-[a-zA-Z0-9.-]+)?)",
            r"go([0-9]+\.[0-9]+(?:\.[0-9]+)?)",
        ],
        "python" => vec![
            r"Python\s+([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
            r"([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
        ],
        _ => vec![
            r"([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
            r"v([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?)",
        ],
    };

    for pattern in patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(output) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str();
                    return Some(normalize_version_string(version));
                }
            }
        }
    }

    None
}

/// Parse version from GitHub release tag
pub fn parse_github_tag(tag: &str, tool_name: &str) -> String {
    // Common tag patterns for different tools
    let patterns = match tool_name {
        "bun" => vec![r"^bun-v(.+)$", r"^v(.+)$", r"^(.+)$"],
        "node" => vec![r"^v(.+)$", r"^(.+)$"],
        "go" => vec![r"^go(.+)$", r"^v(.+)$", r"^(.+)$"],
        _ => vec![r"^v(.+)$", r"^(.+)$"],
    };

    for pattern in patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(tag) {
                if let Some(version_match) = captures.get(1) {
                    let version = version_match.as_str();
                    return normalize_version_string(version);
                }
            }
        }
    }

    // Fallback: return the tag as-is
    normalize_version_string(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_version_string() {
        assert_eq!(normalize_version_string("v1.2.3"), "1.2.3");
        assert_eq!(normalize_version_string("1.21"), "1.21.0");
        assert_eq!(normalize_version_string("1"), "1.0.0");
        assert_eq!(normalize_version_string("1.2.3-beta.1"), "1.2.3-beta.1");
    }

    #[test]
    fn test_extract_version_from_output() {
        assert_eq!(
            extract_version_from_output("bun 1.2.3", "bun"),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            extract_version_from_output("v1.2.3", "node"),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            extract_version_from_output("go version go1.21.6 linux/amd64", "go"),
            Some("1.21.6".to_string())
        );
    }

    #[test]
    fn test_parse_github_tag() {
        assert_eq!(parse_github_tag("bun-v1.2.3", "bun"), "1.2.3");
        assert_eq!(parse_github_tag("v1.2.3", "node"), "1.2.3");
        assert_eq!(parse_github_tag("go1.21.6", "go"), "1.21.6");
    }

    #[test]
    fn test_version_parser() {
        let config = VersionParsingConfig {
            tag_regex: Some("^bun-v(.+)$".to_string()),
            output_regex: Some(r"bun\s+([0-9]+\.[0-9]+\.[0-9]+)".to_string()),
            normalize_semver: Some(true),
        };

        let parser = VersionParser::new(config);

        assert_eq!(parser.parse_tag("bun-v1.2.3").unwrap(), "1.2.3");
        assert_eq!(
            parser.parse_output("bun 1.2.3").unwrap(),
            Some("1.2.3".to_string())
        );
    }
}
