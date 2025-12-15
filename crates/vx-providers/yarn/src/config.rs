//! Yarn configuration

use serde::{Deserialize, Serialize};

/// Yarn configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YarnConfig {
    /// Default Yarn version
    pub default_version: Option<String>,
    /// Enable Plug'n'Play
    pub enable_pnp: Option<bool>,
    /// Cache folder
    pub cache_folder: Option<String>,
}

/// Yarn URL builder for download URLs
pub struct YarnUrlBuilder;

impl YarnUrlBuilder {
    /// Generate download URL for Yarn version
    pub fn download_url(version: &str) -> Option<String> {
        // Yarn 1.x uses npm registry, Yarn 2+ (Berry) uses GitHub
        if version.starts_with('1') {
            Some(format!(
                "https://github.com/yarnpkg/yarn/releases/download/v{}/yarn-v{}.tar.gz",
                version, version
            ))
        } else {
            // Yarn Berry (2.x+)
            Some(format!(
                "https://repo.yarnpkg.com/{}/packages/yarnpkg-cli/bin/yarn.js",
                version
            ))
        }
    }
}
