//! Rust runtime implementations

use crate::config::RustUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VersionInfo};

/// Cargo runtime
#[derive(Debug, Clone)]
pub struct CargoRuntime;

impl CargoRuntime {
    pub fn new() -> Self {
        Self
    }

    /// Get the directory name inside the archive for a given version
    pub fn get_archive_dir_name(version: &str) -> String {
        let platform = RustUrlBuilder::get_platform_string();
        format!("rust-{}-{}", version, platform)
    }
}

impl Default for CargoRuntime {
    fn default() -> Self {
        Self::new()
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

    /// Rust archives extract to `rust-{version}-{platform}/cargo/bin/cargo`
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = Self::get_archive_dir_name(version);
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
                .skip_prereleases(false)
                .lts_detector(|v| {
                    // Stable releases are considered LTS-like
                    // Beta and nightly are handled via prerelease flag
                    !v.contains("beta") && !v.contains("nightly")
                }),
        )
        .await
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(RustUrlBuilder::download_url(version))
    }
}

/// Rustc runtime
#[derive(Debug, Clone)]
pub struct RustcRuntime;

impl RustcRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustcRuntime {
    fn default() -> Self {
        Self::new()
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
        &[]
    }

    /// Rust archives extract to `rust-{version}-{platform}/rustc/bin/rustc`
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = CargoRuntime::get_archive_dir_name(version);
        format!("{}/rustc/bin/{}", dir_name, platform.exe_name("rustc"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        CargoRuntime::new().fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        CargoRuntime::new().download_url(version, platform).await
    }
}

/// Rustup runtime
#[derive(Debug, Clone)]
pub struct RustupRuntime;

impl RustupRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustupRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for RustupRuntime {
    fn name(&self) -> &str {
        "rustup"
    }

    fn description(&self) -> &str {
        "The Rust toolchain installer"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// Rustup is a single executable downloaded directly
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        platform.exe_name("rustup-init")
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch rustup versions from GitHub releases
        ctx.fetch_github_releases(
            "rustup",
            "rust-lang",
            "rustup",
            GitHubReleaseOptions::new().strip_v_prefix(false),
        )
        .await
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(Some(RustUrlBuilder::rustup_url()))
    }
}
