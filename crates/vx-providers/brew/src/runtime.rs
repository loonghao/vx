//! Homebrew runtime implementation

use crate::config::BrewConfig;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Arch, Ecosystem, GitHubReleaseOptions, Os, Platform, Runtime, RuntimeContext,
    VerificationResult, VersionInfo,
};

/// Homebrew runtime
#[derive(Debug, Clone)]
pub struct BrewRuntime;

impl BrewRuntime {
    /// Create a new Homebrew runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for BrewRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for BrewRuntime {
    fn name(&self) -> &str {
        "brew"
    }

    fn description(&self) -> &str {
        "Homebrew - The Missing Package Manager for macOS (or Linux)"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["homebrew"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://brew.sh/".to_string());
        meta.insert("ecosystem".to_string(), "system".to_string());
        meta.insert("category".to_string(), "package-manager".to_string());
        meta
    }

    /// Homebrew supports macOS and Linux
    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
        ]
    }

    /// Homebrew is installed via shell script, no binary archive
    fn executable_relative_path(&self, _version: &str, _platform: &Platform) -> String {
        BrewConfig::executable_name().to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Homebrew tags are like "4.4.0" (no 'v' prefix)
        ctx.fetch_github_releases(
            "brew",
            "Homebrew",
            "brew",
            GitHubReleaseOptions::new().tag_prefix(""),
        )
        .await
    }

    /// Homebrew doesn't have a download URL - it's installed via shell script
    async fn download_url(&self, _version: &str, platform: &Platform) -> Result<Option<String>> {
        // Check if platform is supported
        if !self.is_platform_supported(platform) {
            return Ok(None);
        }
        // Homebrew is installed via shell script, not a direct download
        // Return None to indicate script installation is needed
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // Check if platform is supported
        if !self.is_platform_supported(platform) {
            return VerificationResult::failure(
                vec!["Homebrew is only supported on macOS and Linux".to_string()],
                vec!["Use macOS or Linux to install Homebrew".to_string()],
            );
        }

        // Check common installation paths
        for search_path in BrewConfig::search_paths(platform) {
            let brew_path = Path::new(search_path).join(BrewConfig::executable_name());
            if brew_path.exists() {
                return VerificationResult::success(brew_path);
            }
        }

        VerificationResult::failure(
            vec!["Homebrew not found in common installation paths".to_string()],
            vec![
                format!(
                    "Install Homebrew: /bin/bash -c \"$(curl -fsSL {})\"",
                    BrewConfig::install_script_url()
                ),
                "Run 'brew --version' to verify installation".to_string(),
            ],
        )
    }

    /// Check if brew is already installed on the system
    async fn is_installed(&self, _version: &str, _ctx: &RuntimeContext) -> Result<bool> {
        let platform = Platform::current();
        for search_path in BrewConfig::search_paths(&platform) {
            let brew_path = Path::new(search_path).join(BrewConfig::executable_name());
            if brew_path.exists() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
