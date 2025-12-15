//! Rust runtime implementations

use crate::config::RustUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

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
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "cargo.exe"
        } else {
            "cargo"
        };
        format!("{}/cargo/bin/{}", dir_name, exe_name)
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Rust uses channels: stable, beta, nightly
        Ok(vec![
            VersionInfo::new("stable").with_lts(true),
            VersionInfo::new("beta").with_prerelease(true),
            VersionInfo::new("nightly").with_prerelease(true),
            VersionInfo::new("1.75.0"),
        ])
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
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "rustc.exe"
        } else {
            "rustc"
        };
        format!("{}/rustc/bin/{}", dir_name, exe_name)
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
        if platform.os == vx_runtime::Os::Windows {
            "rustup-init.exe".to_string()
        } else {
            "rustup-init".to_string()
        }
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Rustup has its own versioning
        Ok(vec![VersionInfo::new("1.26.0").with_lts(true)])
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(Some(RustUrlBuilder::rustup_url()))
    }
}
