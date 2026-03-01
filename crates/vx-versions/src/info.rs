//! Version information types
//!
//! Re-exported as [`vx_versions::VersionInfo`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Version information for a runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string (e.g., "20.0.0")
    pub version: String,
    /// Release date
    pub released_at: Option<DateTime<Utc>>,
    /// Whether this is a prerelease
    pub prerelease: bool,
    /// Whether this is an LTS version
    pub lts: bool,
    /// Download URL for current platform
    pub download_url: Option<String>,
    /// Checksum (SHA256)
    pub checksum: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl VersionInfo {
    /// Create a new version info with just the version string
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            released_at: None,
            prerelease: false,
            lts: false,
            download_url: None,
            checksum: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the download URL
    pub fn with_download_url(mut self, url: impl Into<String>) -> Self {
        self.download_url = Some(url.into());
        self
    }

    /// Set as prerelease
    pub fn with_prerelease(mut self, prerelease: bool) -> Self {
        self.prerelease = prerelease;
        self
    }

    /// Set as LTS
    pub fn with_lts(mut self, lts: bool) -> Self {
        self.lts = lts;
        self
    }

    /// Set the release date from a string
    pub fn with_release_date(mut self, date: impl Into<String>) -> Self {
        self.metadata
            .insert("release_date".to_string(), date.into());
        self
    }

    /// Set release notes
    pub fn with_release_notes(mut self, notes: impl Into<String>) -> Self {
        self.metadata
            .insert("release_notes".to_string(), notes.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
