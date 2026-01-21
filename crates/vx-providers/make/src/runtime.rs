//! GNU Make runtime implementation
//!
//! GNU Make is a build automation tool.
//! On Windows, vx does not support make installation - use 'just' as a modern alternative.
//! On macOS/Linux, make is typically pre-installed or available via system package manager.

use anyhow::{bail, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Arch, Ecosystem, Os, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// Static list of known make versions
const KNOWN_VERSIONS: &[&str] = &[
    "4.4.1", "4.4", "4.3", "4.2.1", "4.2", "4.1", "4.0", "3.82", "3.81",
];

/// Make runtime implementation
#[derive(Debug, Clone, Default)]
pub struct MakeRuntime;

impl MakeRuntime {
    /// Create a new Make runtime
    pub fn new() -> Self {
        Self
    }

    /// Get the executable name for the platform
    fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "make.exe",
            _ => "make",
        }
    }
}

#[async_trait]
impl Runtime for MakeRuntime {
    fn name(&self) -> &str {
        "make"
    }

    fn description(&self) -> &str {
        "GNU Make - A build automation tool"
    }

    fn aliases(&self) -> &[&str] {
        &["gmake", "gnumake"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.gnu.org/software/make/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://www.gnu.org/software/make/manual/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://savannah.gnu.org/projects/make".to_string(),
        );
        meta.insert("category".to_string(), "build-system".to_string());
        meta.insert("license".to_string(), "GPL-3.0".to_string());
        meta.insert(
            "install_method".to_string(),
            "system_package_manager".to_string(),
        );
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        Self::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Check platform first
        let platform = Platform::current();
        if platform.os == Os::Windows {
            bail!(
                "GNU Make is not supported on Windows via vx.\n\n\
                💡 Use 'just' as a modern, cross-platform alternative:\n\
                   vx install just\n\n\
                Or install make via your Windows package manager:\n\
                   choco install make\n\
                   winget install GnuWin32.Make\n\
                   scoop install make"
            );
        }

        // Return static list of known versions for Unix
        let versions: Vec<VersionInfo> = KNOWN_VERSIONS
            .iter()
            .enumerate()
            .map(|(idx, &version)| VersionInfo {
                version: version.to_string(),
                prerelease: false,
                lts: idx == 0, // Latest is "LTS"
                released_at: None,
                download_url: None,
                checksum: None,
                metadata: HashMap::new(),
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, _version: &str, platform: &Platform) -> Result<Option<String>> {
        // Make should be installed via system package managers
        // No direct download URL available
        if platform.os == Os::Windows {
            bail!(
                "GNU Make is not supported on Windows via vx.\n\n\
                💡 Use 'just' as a modern, cross-platform alternative:\n\
                   vx install just"
            );
        }
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // Make is installed via system package managers
        // Verification is done by checking if the command exists in PATH
        VerificationResult::success_system_installed()
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        // Only Unix platforms are supported - Windows users should use 'just'
        vec![
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
        ]
    }

    fn check_platform_support(&self) -> Result<(), String> {
        let current = Platform::current();
        if current.os == Os::Windows {
            return Err("GNU Make is not supported on Windows via vx.\n\n\
                💡 Use 'just' as a modern, cross-platform alternative:\n\
                   vx install just\n\n\
                Or install make via your Windows package manager:\n\
                   choco install make\n\
                   winget install GnuWin32.Make\n\
                   scoop install make"
                .to_string());
        }
        Ok(())
    }
}
