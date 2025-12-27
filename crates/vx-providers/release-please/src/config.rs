//! URL builder and platform configuration for release-please
//!
//! release-please is installed via npm, so this module is minimal.
//! The actual download is handled by the npm package installation system.

use vx_runtime::{Os, Platform};

/// URL builder for release-please downloads
///
/// Note: release-please is installed via npm, not as a standalone binary.
/// This struct is kept for consistency with other providers.
pub struct ReleasePleaseUrlBuilder;

impl ReleasePleaseUrlBuilder {
    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "release-please.cmd",
            _ => "release-please",
        }
    }
}
