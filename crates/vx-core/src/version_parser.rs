//! Version parsing utilities for different tools

use crate::{Result, VersionInfo};
use serde_json::Value;

/// Temporary VersionParser trait during migration
pub trait VersionParser: Send + Sync {
    fn parse_versions(&self, json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>>;
}

/// Version parser for Node.js
#[derive(Debug, Clone)]
pub struct NodeVersionParser;

impl Default for NodeVersionParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeVersionParser {
    /// Create a new NodeVersionParser instance
    pub fn new() -> Self {
        Self
    }
    /// Parse Node.js versions from the official API response
    pub fn parse_versions(json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let mut versions = Vec::new();

        if let Some(releases_array) = json.as_array() {
            for release in releases_array {
                let version = release["version"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches('v')
                    .to_string();

                if version.is_empty() {
                    continue;
                }

                let is_prerelease =
                    version.contains("alpha") || version.contains("beta") || version.contains("rc");

                if !include_prerelease && is_prerelease {
                    continue;
                }

                let release_date = release["date"].as_str().map(|s| s.to_string());
                let lts_info = release["lts"].as_str();
                let is_lts = lts_info.is_some() && lts_info != Some("false");

                let mut version_info = VersionInfo::new(version).with_prerelease(is_prerelease);

                if let Some(date) = release_date {
                    version_info = version_info.with_release_date(date);
                }

                if is_lts {
                    let release_notes = format!("LTS release ({})", lts_info.unwrap_or("LTS"));
                    version_info = version_info
                        .with_release_notes(release_notes)
                        .with_metadata("lts".to_string(), "true".to_string());

                    if let Some(lts_name) = lts_info {
                        version_info = version_info
                            .with_metadata("lts_name".to_string(), lts_name.to_string());
                    }
                } else {
                    version_info = version_info.with_release_notes("Current release".to_string());
                }

                // Add download URL
                if let Some(download_url) =
                    crate::url_builder::NodeUrlBuilder::download_url(&version_info.version)
                {
                    version_info = version_info.with_download_url(download_url);
                }

                versions.push(version_info);
            }
        }

        Ok(versions)
    }
}

impl VersionParser for NodeVersionParser {
    fn parse_versions(&self, json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        Self::parse_versions(json, include_prerelease)
    }
}

/// Version parser for Go
pub struct GoVersionParser;

impl GoVersionParser {
    /// Parse Go versions from the official API response
    pub fn parse_versions(json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let mut versions = Vec::new();

        if let Some(releases_array) = json.as_array() {
            for release in releases_array {
                let version = release["version"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches("go")
                    .to_string();

                if version.is_empty() {
                    continue;
                }

                let is_prerelease =
                    version.contains("beta") || version.contains("rc") || version.contains("alpha");

                if !include_prerelease && is_prerelease {
                    continue;
                }

                // Check if it's stable
                let stable = release["stable"].as_bool().unwrap_or(false);
                if !include_prerelease && !stable {
                    continue;
                }

                let mut version_info = VersionInfo::new(version)
                    .with_prerelease(is_prerelease)
                    .with_release_notes("Go release".to_string());

                if stable {
                    version_info =
                        version_info.with_metadata("stable".to_string(), "true".to_string());
                }

                // Add download URL
                if let Some(download_url) =
                    crate::url_builder::GoUrlBuilder::download_url(&version_info.version)
                {
                    version_info = version_info.with_download_url(download_url);
                }

                versions.push(version_info);
            }
        }

        Ok(versions)
    }
}

impl VersionParser for GoVersionParser {
    fn parse_versions(&self, json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        Self::parse_versions(json, include_prerelease)
    }
}

/// Version parser for GitHub releases (used by Rust, Python, etc.)
#[derive(Debug, Clone)]
pub struct GitHubVersionParser {
    owner: String,
    repo: String,
}

impl GitHubVersionParser {
    /// Create a new GitHubVersionParser instance
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        }
    }

    /// Get the versions URL for this repository
    pub fn versions_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.owner, self.repo
        )
    }
    /// Parse versions from GitHub releases API response
    pub fn parse_versions(json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        let mut versions = Vec::new();

        if let Some(releases_array) = json.as_array() {
            for release in releases_array {
                let version = release["tag_name"].as_str().unwrap_or("").to_string();

                if version.is_empty() {
                    continue;
                }

                let is_prerelease = release["prerelease"].as_bool().unwrap_or(false);

                if !include_prerelease && is_prerelease {
                    continue;
                }

                let release_date = release["published_at"]
                    .as_str()
                    .map(|s| s.split('T').next().unwrap_or(s).to_string());

                let release_notes = release["body"].as_str().map(|s| {
                    // Truncate long release notes
                    if s.len() > 200 {
                        format!("{}...", &s[..197])
                    } else {
                        s.to_string()
                    }
                });

                let mut version_info = VersionInfo::new(version).with_prerelease(is_prerelease);

                if let Some(date) = release_date {
                    version_info = version_info.with_release_date(date);
                }

                if let Some(notes) = release_notes {
                    version_info = version_info.with_release_notes(notes);
                }

                versions.push(version_info);
            }
        }

        Ok(versions)
    }
}

/// Generic version parser utilities
pub struct VersionParserUtils;

impl VersionParserUtils {
    /// Check if a version string indicates a prerelease
    pub fn is_prerelease(version: &str) -> bool {
        version.contains("alpha")
            || version.contains("beta")
            || version.contains("rc")
            || version.contains("pre")
            || version.contains("dev")
            || version.contains("snapshot")
    }

    /// Clean version string (remove prefixes like 'v', 'go', etc.)
    pub fn clean_version(version: &str, prefixes: &[&str]) -> String {
        let mut cleaned = version.to_string();
        for prefix in prefixes {
            if cleaned.starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].to_string();
                break;
            }
        }
        cleaned
    }

    /// Sort versions in descending order (latest first)
    pub fn sort_versions_desc(mut versions: Vec<VersionInfo>) -> Vec<VersionInfo> {
        versions.sort_by(|a, b| {
            // Simple string comparison for now
            // TODO: Implement proper semantic version comparison
            b.version.cmp(&a.version)
        });
        versions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_version_parser_utils() {
        assert!(VersionParserUtils::is_prerelease("1.0.0-alpha"));
        assert!(VersionParserUtils::is_prerelease("2.0.0-beta.1"));
        assert!(VersionParserUtils::is_prerelease("3.0.0-rc.1"));
        assert!(!VersionParserUtils::is_prerelease("1.0.0"));

        assert_eq!(VersionParserUtils::clean_version("v1.0.0", &["v"]), "1.0.0");
        assert_eq!(
            VersionParserUtils::clean_version("go1.21.0", &["go"]),
            "1.21.0"
        );
        assert_eq!(
            VersionParserUtils::clean_version("1.0.0", &["v", "go"]),
            "1.0.0"
        );
    }

    #[test]
    fn test_node_version_parser() {
        let json = json!([
            {
                "version": "v18.0.0",
                "date": "2022-04-19",
                "lts": false
            },
            {
                "version": "v16.20.0",
                "date": "2023-03-28",
                "lts": "Gallium"
            }
        ]);

        let versions = NodeVersionParser::parse_versions(&json, false).unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, "18.0.0");
        assert_eq!(versions[1].version, "16.20.0");
        assert_eq!(versions[1].metadata.get("lts"), Some(&"true".to_string()));
    }
}
