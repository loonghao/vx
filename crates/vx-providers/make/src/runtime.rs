//! GNU Make runtime implementation
//!
//! GNU Make is a build automation tool that is typically installed via system package managers.
//! This provider detects system installations and provides a unified interface.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};

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
        use vx_runtime::Os;
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
        // Return static list of known versions
        // Make is typically installed via system package managers, so we provide
        // a reference list of common versions
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

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // Make should be installed via system package managers
        // No direct download URL available
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = Self::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Make executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec![
                    "Install make via your system package manager:".to_string(),
                    "  Windows: choco install make / winget install GnuWin32.Make / scoop install make".to_string(),
                    "  macOS: brew install make".to_string(),
                    "  Ubuntu/Debian: sudo apt install make".to_string(),
                    "  Fedora/RHEL: sudo dnf install make".to_string(),
                    "  Arch: sudo pacman -S make".to_string(),
                    "Or consider using 'just' as a modern alternative: vx install just".to_string(),
                ],
            )
        }
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        use vx_runtime::{Arch, Os};
        // Make is available on all major platforms via package managers
        vec![
            Platform {
                os: Os::Windows,
                arch: Arch::X86_64,
            },
            Platform {
                os: Os::Windows,
                arch: Arch::X86,
            },
            Platform {
                os: Os::MacOS,
                arch: Arch::X86_64,
            },
            Platform {
                os: Os::MacOS,
                arch: Arch::Aarch64,
            },
            Platform {
                os: Os::Linux,
                arch: Arch::X86_64,
            },
            Platform {
                os: Os::Linux,
                arch: Arch::Aarch64,
            },
        ]
    }

    fn check_platform_support(&self) -> Result<(), String> {
        // Make is available on all platforms via package managers
        Ok(())
    }
}
