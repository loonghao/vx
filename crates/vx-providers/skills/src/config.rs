//! URL builder and platform configuration for Skills
//!
//! Skills is an npm package, so this module is primarily
//! used for reference and metadata purposes.

use vx_runtime::Platform;

/// URL builder for Skills
pub struct SkillsUrlBuilder;

impl SkillsUrlBuilder {
    /// npm registry URL for skills package
    pub const NPM_REGISTRY_URL: &'static str = "https://registry.npmjs.org/skills";

    /// GitHub repository URL
    pub const REPOSITORY_URL: &'static str = "https://github.com/vercel-labs/skills";

    /// Get the npm registry URL for version info
    pub fn registry_url() -> &'static str {
        Self::NPM_REGISTRY_URL
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            vx_runtime::Os::Windows => "skills.cmd",
            _ => "skills",
        }
    }
}
