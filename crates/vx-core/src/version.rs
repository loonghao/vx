//! Version fetching trait and related types

use crate::Result;
use serde::{Deserialize, Serialize};

/// Trait for fetching version information from external sources
#[async_trait::async_trait]
pub trait VersionFetcher: Send + Sync {
    /// Get the name of the tool this fetcher supports
    fn tool_name(&self) -> &str;

    /// Fetch available versions for the tool
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Get the latest stable version
    async fn get_latest_version(&self) -> Result<Option<VersionInfo>> {
        let versions = self.fetch_versions(false).await?;
        Ok(versions.into_iter().next())
    }

    /// Get the latest version (including prereleases)
    async fn get_latest_version_including_prerelease(&self) -> Result<Option<VersionInfo>> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions.into_iter().next())
    }

    /// Check if a specific version exists
    async fn version_exists(&self, version: &str) -> Result<bool> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions.iter().any(|v| v.version == version))
    }
}

/// Information about a specific version of a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub metadata: std::collections::HashMap<String, String>,
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
            metadata: std::collections::HashMap::new(),
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

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Version comparison utilities
pub mod version_utils {
    use super::VersionInfo;

    /// Sort versions in descending order (latest first)
    pub fn sort_versions_desc(mut versions: Vec<VersionInfo>) -> Vec<VersionInfo> {
        versions.sort_by(|a, b| {
            // Simple string comparison for now
            // TODO: Implement proper semantic version comparison
            b.version.cmp(&a.version)
        });
        versions
    }

    /// Filter out prerelease versions
    pub fn filter_stable_only(versions: Vec<VersionInfo>) -> Vec<VersionInfo> {
        versions.into_iter().filter(|v| !v.is_prerelease).collect()
    }

    /// Get the latest N versions
    pub fn take_latest(versions: Vec<VersionInfo>, count: usize) -> Vec<VersionInfo> {
        versions.into_iter().take(count).collect()
    }
}
