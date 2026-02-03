//! Windows Package Manager runtime implementation

use crate::config::WingetConfig;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Windows Package Manager runtime
#[derive(Debug, Clone)]
pub struct WingetRuntime;

impl WingetRuntime {
    /// Create a new winget runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for WingetRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for WingetRuntime {
    fn name(&self) -> &str {
        "winget"
    }

    fn description(&self) -> &str {
        "Windows Package Manager - Official package manager for Windows"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["winget-cli"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://learn.microsoft.com/windows/package-manager/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "windows".to_string());
        meta.insert("category".to_string(), "package-manager".to_string());
        meta
    }

    /// winget only supports Windows
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::windows_only()
    }

    /// winget is typically installed via App Installer, not directly downloadable
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        WingetConfig::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // winget releases are tagged like "v1.9.25200"
        ctx.fetch_github_releases(
            "winget",
            "microsoft",
            "winget-cli",
            GitHubReleaseOptions::new().tag_prefix("v"),
        )
        .await
    }

    async fn download_url(&self, _version: &str, platform: &Platform) -> Result<Option<String>> {
        // winget is distributed as msixbundle which requires special installation
        // via Add-AppxPackage, not a simple download and extract
        if !self.is_platform_supported(platform) {
            return Ok(None);
        }
        // Return None to indicate this runtime uses script_install instead
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // winget is installed via system (App Installer), not in vx directory
        if !self.is_platform_supported(platform) {
            return VerificationResult::failure(
                vec!["winget is only supported on Windows".to_string()],
                vec!["Use a Windows 10/11 system".to_string()],
            );
        }

        // Check system paths for winget
        let system_paths = [
            r"C:\Users\Default\AppData\Local\Microsoft\WindowsApps\winget.exe",
            r"C:\Program Files\WindowsApps\Microsoft.DesktopAppInstaller_*\winget.exe",
        ];

        for path_pattern in &system_paths {
            // For glob patterns, just check if the base path exists
            let base_path = path_pattern.split('*').next().unwrap_or(path_pattern);
            if Path::new(base_path).exists() || std::fs::metadata(path_pattern).is_ok() {
                return VerificationResult::success(Path::new(path_pattern).to_path_buf());
            }
        }

        VerificationResult::failure(
            vec!["winget not found in system paths".to_string()],
            vec![
                "Install 'App Installer' from Microsoft Store".to_string(),
                "Or download from https://github.com/microsoft/winget-cli/releases".to_string(),
            ],
        )
    }
}
