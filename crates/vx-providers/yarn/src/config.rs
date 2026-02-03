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
    ///
    /// Note: Only Yarn 1.x (Classic) is directly downloadable.
    /// Yarn 2.x+ (Berry) should be managed via corepack which is bundled with Node.js.
    /// Users should run `corepack enable` after installing Node.js to use Yarn 2.x+.
    pub fn download_url(version: &str) -> Option<String> {
        // Only Yarn 1.x (Classic) supports direct download
        // Yarn 2.x+ (Berry) should be managed via corepack
        if version.starts_with('1') {
            Some(format!(
                "https://github.com/yarnpkg/yarn/releases/download/v{}/yarn-v{}.tar.gz",
                version, version
            ))
        } else {
            // Yarn 2.x+ (Berry) - return None to indicate it's not directly installable
            // Users should use corepack instead: `corepack enable`
            None
        }
    }

    /// Check if a Yarn version is directly installable
    ///
    /// Only Yarn 1.x (Classic) can be directly downloaded and installed.
    /// Yarn 2.x+ requires corepack which is bundled with Node.js.
    pub fn is_directly_installable(version: &str) -> bool {
        version.starts_with('1')
    }
}
