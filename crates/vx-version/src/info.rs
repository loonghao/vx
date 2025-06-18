//! Version information types and utilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a specific version of a tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInfo {
    /// Version string (e.g., "1.2.3")
    pub version: String,

    /// Whether this is a prerelease version
    pub is_prerelease: bool,

    /// Release date in ISO format
    pub release_date: Option<String>,

    /// Release notes or description
    pub release_notes: Option<String>,

    /// Download URL for this version
    pub download_url: Option<String>,

    /// Checksum for verification
    pub checksum: Option<String>,

    /// File size in bytes
    pub file_size: Option<u64>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl VersionInfo {
    /// Create a new VersionInfo with minimal information
    pub fn new(version: String) -> Self {
        Self {
            version,
            is_prerelease: false,
            release_date: None,
            release_notes: None,
            download_url: None,
            checksum: None,
            file_size: None,
            metadata: HashMap::new(),
        }
    }

    /// Set prerelease status
    pub fn with_prerelease(mut self, is_prerelease: bool) -> Self {
        self.is_prerelease = is_prerelease;
        self
    }

    /// Set release date
    pub fn with_release_date(mut self, date: String) -> Self {
        self.release_date = Some(date);
        self
    }

    /// Set download URL
    pub fn with_download_url(mut self, url: String) -> Self {
        self.download_url = Some(url);
        self
    }

    /// Set release notes
    pub fn with_release_notes(mut self, notes: String) -> Self {
        self.release_notes = Some(notes);
        self
    }

    /// Set checksum
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }

    /// Set file size
    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this version is an LTS release
    pub fn is_lts(&self) -> bool {
        self.metadata.get("lts").map(|v| v == "true").unwrap_or(false)
    }

    /// Get LTS name if available
    pub fn lts_name(&self) -> Option<&String> {
        self.metadata.get("lts_name")
    }

    /// Check if this version is stable
    pub fn is_stable(&self) -> bool {
        self.metadata.get("stable").map(|v| v == "true").unwrap_or(!self.is_prerelease)
    }
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)?;
        
        if self.is_prerelease {
            write!(f, " (prerelease)")?;
        }
        
        if self.is_lts() {
            if let Some(lts_name) = self.lts_name() {
                write!(f, " (LTS: {})", lts_name)?;
            } else {
                write!(f, " (LTS)")?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_creation() {
        let version = VersionInfo::new("1.2.3".to_string());
        assert_eq!(version.version, "1.2.3");
        assert!(!version.is_prerelease);
        assert!(version.release_date.is_none());
    }

    #[test]
    fn test_version_info_builder() {
        let version = VersionInfo::new("1.2.3".to_string())
            .with_prerelease(true)
            .with_release_date("2023-01-01".to_string())
            .with_download_url("https://example.com/download".to_string())
            .with_metadata("lts".to_string(), "true".to_string())
            .with_metadata("lts_name".to_string(), "Hydrogen".to_string());

        assert_eq!(version.version, "1.2.3");
        assert!(version.is_prerelease);
        assert_eq!(version.release_date, Some("2023-01-01".to_string()));
        assert_eq!(version.download_url, Some("https://example.com/download".to_string()));
        assert!(version.is_lts());
        assert_eq!(version.lts_name(), Some(&"Hydrogen".to_string()));
    }

    #[test]
    fn test_version_info_display() {
        let version = VersionInfo::new("1.2.3".to_string());
        assert_eq!(format!("{}", version), "1.2.3");

        let prerelease = VersionInfo::new("1.2.3-alpha".to_string())
            .with_prerelease(true);
        assert_eq!(format!("{}", prerelease), "1.2.3-alpha (prerelease)");

        let lts = VersionInfo::new("16.20.0".to_string())
            .with_metadata("lts".to_string(), "true".to_string())
            .with_metadata("lts_name".to_string(), "Gallium".to_string());
        assert_eq!(format!("{}", lts), "16.20.0 (LTS: Gallium)");
    }
}