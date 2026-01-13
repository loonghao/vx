//! Rust runtime implementations
//!
//! vx manages Rust toolchains directly, replacing the need for rustup.
//! Users can install specific Rust versions with: vx install rust@1.75.0

use crate::config::RustUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Get the directory name inside the archive for a given version and platform
fn get_archive_dir_name(version: &str, platform: &Platform) -> String {
    let platform_str = RustUrlBuilder::get_platform_string(platform);
    format!("rust-{}-{}", version, platform_str)
}

/// Cargo runtime
#[derive(Debug, Clone, Default)]
pub struct CargoRuntime;

impl CargoRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for CargoRuntime {
    fn name(&self) -> &str {
        "cargo"
    }

    fn description(&self) -> &str {
        "Rust package manager and build tool"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://doc.rust-lang.org/cargo/".to_string(),
        );
        meta.insert("category".to_string(), "package-manager".to_string());
        meta
    }

    /// Rust archives extract to `rust-{version}-{platform}/cargo/bin/cargo`
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = get_archive_dir_name(version, platform);
        format!("{}/cargo/bin/{}", dir_name, platform.exe_name("cargo"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from Rust GitHub releases
        // Rust releases use tags like "1.75.0" without 'v' prefix
        ctx.fetch_github_releases(
            "rust",
            "rust-lang",
            "rust",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false) // Rust tags don't have 'v' prefix
                .skip_prereleases(true)
                .lts_detector(|v| {
                    // Stable releases are considered LTS-like
                    !v.contains("beta") && !v.contains("nightly")
                }),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(RustUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "cargo executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}

/// Rustc runtime
#[derive(Debug, Clone, Default)]
pub struct RustcRuntime;

impl RustcRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for RustcRuntime {
    fn name(&self) -> &str {
        "rustc"
    }

    fn description(&self) -> &str {
        "The Rust compiler"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn aliases(&self) -> &[&str] {
        &["rust"] // "rust" is an alias for "rustc"
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.rust-lang.org/".to_string(),
        );
        meta.insert("category".to_string(), "compiler".to_string());
        meta
    }

    /// Rust archives extract to `rust-{version}-{platform}/rustc/bin/rustc`
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = get_archive_dir_name(version, platform);
        format!("{}/rustc/bin/{}", dir_name, platform.exe_name("rustc"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        CargoRuntime::new().fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(RustUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "rustc executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
