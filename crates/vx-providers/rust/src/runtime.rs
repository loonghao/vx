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
    layout::{ArchiveLayout, DownloadType, ExecutableLayout},
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

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

    /// After strip_prefix, the archive extracts to `cargo/bin/cargo`
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("cargo/bin/{}", platform.exe_name("cargo"))
    }

    /// Layout configuration for Rust archive extraction
    /// Rust tarballs extract to rust-{version}-{target_triple}/ which needs to be stripped
    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Some(ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec![
                    "cargo/bin/cargo.exe".to_string(),
                    "cargo/bin/cargo".to_string(),
                ],
                // Use {target_triple} variable which will be replaced with the Rust target triple
                strip_prefix: Some("rust-{version}-{target_triple}".to_string()),
                permissions: Some("755".to_string()),
            }),
            msi: None,
            windows: None,
            macos: None,
            linux: None,
        })
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

    /// After strip_prefix, the archive extracts to `rustc/bin/rustc`
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("rustc/bin/{}", platform.exe_name("rustc"))
    }

    /// Layout configuration for Rust archive extraction
    /// Rust tarballs extract to rust-{version}-{target_triple}/ which needs to be stripped
    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Some(ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec![
                    "rustc/bin/rustc.exe".to_string(),
                    "rustc/bin/rustc".to_string(),
                ],
                // Use {target_triple} variable which will be replaced with the Rust target triple
                strip_prefix: Some("rust-{version}-{target_triple}".to_string()),
                permissions: Some("755".to_string()),
            }),
            msi: None,
            windows: None,
            macos: None,
            linux: None,
        })
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
