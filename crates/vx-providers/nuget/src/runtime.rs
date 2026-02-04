//! NuGet runtime implementation

use crate::config::NugetUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// NuGet runtime
#[derive(Debug, Clone)]
pub struct NugetRuntime;

impl NugetRuntime {
    /// Create a new NuGet runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for NugetRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for NugetRuntime {
    fn name(&self) -> &str {
        "nuget"
    }

    fn description(&self) -> &str {
        "NuGet - The package manager for .NET"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Dotnet
    }

    fn aliases(&self) -> &[&str] {
        &["nuget-cli"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.nuget.org/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "dotnet".to_string());
        meta.insert("category".to_string(), "package-manager".to_string());
        meta
    }

    /// NuGet CLI (nuget.exe) is Windows-only
    /// On other platforms, use `dotnet nuget` commands
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::windows_only()
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // BinaryHandler installs to bin/ subdirectory
        format!("bin/{}", NugetUrlBuilder::get_executable_name(platform))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // NuGet CLI versions are available at nuget.org
        // We'll use a simplified version list from known stable releases
        // In a full implementation, this would fetch from nuget.org API
        let versions = vec![
            VersionInfo::new("6.11.1"),
            VersionInfo::new("6.10.2"),
            VersionInfo::new("6.9.1"),
            VersionInfo::new("6.8.1"),
            VersionInfo::new("6.7.0"),
            VersionInfo::new("6.6.1"),
            VersionInfo::new("6.5.0"),
            VersionInfo::new("6.4.0"),
            VersionInfo::new("6.3.1"),
            VersionInfo::new("6.2.1"),
            VersionInfo::new("6.1.0"),
            VersionInfo::new("6.0.0"),
            VersionInfo::new("5.11.0"),
            VersionInfo::new("5.10.0"),
        ];
        
        // Try to fetch latest from nuget.org (if available)
        let _ = ctx; // Silence unused warning for now
        
        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        if !self.is_platform_supported(platform) {
            return Ok(None);
        }
        Ok(Some(NugetUrlBuilder::download_url(version)))
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        if !self.is_platform_supported(platform) {
            return VerificationResult::failure(
                vec!["NuGet CLI (nuget.exe) is only supported on Windows".to_string()],
                vec![
                    "On macOS/Linux, use 'dotnet nuget' commands instead".to_string(),
                    "Install .NET SDK: vx install dotnet".to_string(),
                ],
            );
        }

        let exe_path = install_path.join(self.executable_relative_path(version, platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "NuGet executable not found at: {}",
                    exe_path.display()
                )],
                vec![
                    "Try reinstalling NuGet".to_string(),
                    "Or use 'choco install nuget.commandline'".to_string(),
                ],
            )
        }
    }
}
