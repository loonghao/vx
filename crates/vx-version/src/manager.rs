//! Version management utilities

use crate::{GitHubVersionFetcher, NodeVersionFetcher, Result, VersionError, VersionFetcher};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::process::Command;
use which::which;

/// Semantic version representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl Version {
    /// Parse a version string into a Version struct
    pub fn parse(version_str: &str) -> Result<Self> {
        let version_str = version_str.trim_start_matches('v');

        // First split by dash to separate main version from prerelease
        let (main_version, pre) = if let Some(dash_pos) = version_str.find('-') {
            let main = &version_str[..dash_pos];
            let pre_part = &version_str[dash_pos + 1..];
            (main, Some(pre_part.to_string()))
        } else {
            (version_str, None)
        };

        // Now split the main version by dots
        let parts: Vec<&str> = main_version.split('.').collect();

        if parts.len() < 3 {
            return Err(VersionError::InvalidVersion {
                version: version_str.to_string(),
                reason: "Version must have at least major.minor.patch".to_string(),
            });
        }

        let major = parts[0].parse().map_err(|_| VersionError::InvalidVersion {
            version: version_str.to_string(),
            reason: format!("Invalid major version: {}", parts[0]),
        })?;

        let minor = parts[1].parse().map_err(|_| VersionError::InvalidVersion {
            version: version_str.to_string(),
            reason: format!("Invalid minor version: {}", parts[1]),
        })?;

        let patch = parts[2].parse().map_err(|_| VersionError::InvalidVersion {
            version: version_str.to_string(),
            reason: format!("Invalid patch version: {}", parts[2]),
        })?;

        Ok(Self {
            major,
            minor,
            patch,
            pre,
        })
    }

    /// Convert version to string representation
    pub fn as_string(&self) -> String {
        match &self.pre {
            Some(pre) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }

    /// Check if this is a prerelease version
    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Version manager for handling tool versions
pub struct VersionManager;

impl VersionManager {
    /// Check if a tool is installed and get its version
    pub fn get_installed_version(tool_name: &str) -> Result<Option<Version>> {
        // Check if tool is available in PATH
        if which(tool_name).is_err() {
            return Ok(None);
        }

        // Try to get version
        let output = Command::new(tool_name)
            .arg("--version")
            .output()
            .map_err(|e| VersionError::CommandError {
                command: format!("{} --version", tool_name),
                source: e,
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        let version_line = version_output.lines().next().unwrap_or("");

        // Extract version number from output
        let version_str = Self::extract_version_from_output(version_line)?;
        let version = Version::parse(&version_str)?;

        Ok(Some(version))
    }

    /// Get latest stable version from various sources
    pub async fn get_latest_version(tool_name: &str) -> Result<Version> {
        match tool_name {
            "uv" => {
                let fetcher = GitHubVersionFetcher::new("astral-sh", "uv");
                let version_info = fetcher.get_latest_version().await?.ok_or_else(|| {
                    VersionError::VersionNotFound {
                        version: "latest".to_string(),
                        tool: tool_name.to_string(),
                    }
                })?;
                Version::parse(&version_info.version)
            }
            "node" => {
                let fetcher = NodeVersionFetcher::new();
                let version_info = fetcher.get_latest_version().await?.ok_or_else(|| {
                    VersionError::VersionNotFound {
                        version: "latest".to_string(),
                        tool: tool_name.to_string(),
                    }
                })?;
                Version::parse(&version_info.version)
            }
            _ => Err(VersionError::Other {
                message: format!("Unsupported tool for version checking: {}", tool_name),
            }),
        }
    }

    /// Extract version string from command output
    pub fn extract_version_from_output(output: &str) -> Result<String> {
        // Common patterns for version extraction
        let patterns = [
            r"(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)",  // Standard semver
            r"v(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)", // With 'v' prefix
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(output) {
                    if let Some(version) = captures.get(1) {
                        return Ok(version.as_str().to_string());
                    }
                }
            }
        }

        Err(VersionError::Other {
            message: format!("Could not extract version from output: {}", output),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre, None);
        assert!(!version.is_prerelease());

        let prerelease = Version::parse("1.2.3-alpha.1").unwrap();
        assert_eq!(prerelease.major, 1);
        assert_eq!(prerelease.minor, 2);
        assert_eq!(prerelease.patch, 3);
        assert_eq!(prerelease.pre, Some("alpha.1".to_string()));
        assert!(prerelease.is_prerelease());
    }

    #[test]
    fn test_version_display() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(format!("{}", version), "1.2.3");

        let prerelease = Version::parse("1.2.3-alpha.1").unwrap();
        assert_eq!(format!("{}", prerelease), "1.2.3-alpha.1");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::parse("1.2.4").unwrap();
        let v3 = Version::parse("1.3.0").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_extract_version_from_output() {
        assert_eq!(
            VersionManager::extract_version_from_output("node v18.17.0").unwrap(),
            "18.17.0"
        );
        assert_eq!(
            VersionManager::extract_version_from_output("uv 0.1.0").unwrap(),
            "0.1.0"
        );
        assert_eq!(
            VersionManager::extract_version_from_output("go version go1.21.0 linux/amd64").unwrap(),
            "1.21.0"
        );
    }
}
