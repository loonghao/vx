//! Azure CLI runtime implementation

use crate::config::AzCliUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Azure CLI runtime
#[derive(Debug, Clone)]
pub struct AzCliRuntime;

impl AzCliRuntime {
    /// Create a new Azure CLI runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for AzCliRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for AzCliRuntime {
    fn name(&self) -> &str {
        "az"
    }

    fn description(&self) -> &str {
        "Azure CLI - Microsoft Azure command-line interface"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cloud".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["azcli", "azure-cli", "azure"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://docs.microsoft.com/cli/azure/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "cloud".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/Azure/azure-cli".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::linux_x64(),
            Platform::linux_arm64(),
            Platform::macos_x64(),
            Platform::macos_arm64(),
            Platform::windows_x64(),
            Platform::windows_arm64(),
        ]
    }

    /// Azure CLI executable path varies by platform
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;

        match &platform.os {
            // Linux/macOS: extracted to bin/az
            Os::Linux | Os::MacOS => "bin/az".to_string(),
            // Windows: after msi extraction
            Os::Windows => "Microsoft SDKs/Azure/CLI2/wbin/az.cmd".to_string(),
            _ => "az".to_string(),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from Azure/azure-cli GitHub releases
        // Azure CLI tags are in format "azure-cli-X.Y.Z"
        ctx.fetch_github_releases(
            "az",
            "Azure",
            "azure-cli",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)
                .tag_prefix("azure-cli-"),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(AzCliUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));
        if exe_path.exists() {
            VerificationResult::success()
        } else {
            VerificationResult::failure(format!(
                "Azure CLI executable not found at {}",
                exe_path.display()
            ))
        }
    }
}
