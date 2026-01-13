//! GNU Make runtime implementation
//!
//! GNU Make is a build automation tool.
//! On Windows, we download pre-built binaries.
//! On Unix, make is typically pre-installed or available via system package manager.

use crate::config::MakeUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Os, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Make runtime implementation
#[derive(Debug, Clone, Default)]
pub struct MakeRuntime;

impl MakeRuntime {
    /// Create a new Make runtime
    pub fn new() -> Self {
        Self
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
            "https://github.com/mbuilov/gnumake-windows".to_string(),
        );
        meta.insert("category".to_string(), "build-system".to_string());
        meta.insert("license".to_string(), "GPL-3.0".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // On Windows, the zip extracts to a flat structure
        MakeUrlBuilder::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch versions from gnumake-windows GitHub releases
        ctx.fetch_github_releases(
            "make",
            "mbuilov",
            "gnumake-windows",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(MakeUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = MakeUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Make executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling with: vx install make".to_string()],
            )
        }
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        use vx_runtime::Arch;
        // Only Windows is supported for binary download
        // Unix users should use system package manager
        vec![
            Platform {
                os: Os::Windows,
                arch: Arch::X86_64,
            },
            Platform {
                os: Os::Windows,
                arch: Arch::X86,
            },
        ]
    }
}
