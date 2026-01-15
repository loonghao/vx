//! rcedit runtime implementation
//!
//! rcedit is a command-line tool to edit resources of Windows executables.
//! https://github.com/electron/rcedit

use crate::config::RceditUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// rcedit runtime implementation
#[derive(Debug, Clone, Default)]
pub struct RceditRuntime;

impl RceditRuntime {
    /// Create a new rcedit runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for RceditRuntime {
    fn name(&self) -> &str {
        "rcedit"
    }

    fn description(&self) -> &str {
        "rcedit - Command-line tool to edit resources of Windows executables"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/electron/rcedit".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://github.com/electron/rcedit#readme".to_string(),
        );
        meta.insert("category".to_string(), "windows-tools".to_string());
        meta
    }

    /// rcedit only supports Windows
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::windows_only()
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // rcedit is installed to bin/rcedit.exe after post_extract renames it
        if platform.os == vx_runtime::Os::Windows {
            "bin/rcedit.exe".to_string()
        } else {
            // Unsupported platform, return original name
            RceditUrlBuilder::get_executable_name(platform).to_string()
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "rcedit",
            "electron",
            "rcedit",
            GitHubReleaseOptions::new().strip_v_prefix(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(RceditUrlBuilder::download_url(version, platform))
    }

    fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
        // Rename downloaded file to standard name
        // Original: rcedit-x64.exe / rcedit-arm64.exe / rcedit-x86.exe
        // Standard: bin/rcedit.exe
        use std::fs;

        let platform = Platform::current();

        // rcedit only supports Windows
        if platform.os != vx_runtime::Os::Windows {
            return Ok(());
        }

        // Get the original downloaded file name
        let original_name = RceditUrlBuilder::get_executable_name(&platform);

        // Create bin/ directory
        let bin_dir = install_path.join("bin");
        fs::create_dir_all(&bin_dir)?;

        let original_path = bin_dir.join(&original_name);
        let standard_path = bin_dir.join("rcedit.exe");

        // Rename if not already standard name
        if original_name != "rcedit.exe" && original_path.exists() {
            if standard_path.exists() {
                fs::remove_file(&standard_path)?;
            }
            fs::rename(&original_path, &standard_path)?;
        }

        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // rcedit only supports Windows
        if !self.is_platform_supported(platform) {
            return VerificationResult::failure(
                vec!["rcedit is only available for Windows".to_string()],
                vec!["Use a Windows system to install rcedit".to_string()],
            );
        }

        // Check for standard name (after post_extract renamed it)
        let exe_path = install_path.join("bin").join("rcedit.exe");

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "rcedit executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
