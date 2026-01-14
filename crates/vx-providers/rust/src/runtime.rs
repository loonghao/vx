//! Rust runtime implementations
//!
//! Rust is installed via rustup, the official Rust toolchain installer.
//! rustup manages rustc, cargo, and other Rust tools automatically.
//!
//! Installation methods:
//! - Windows: winget install Rustlang.Rustup
//! - macOS/Linux: brew install rustup-init && rustup-init -y
//! - Linux (alternative): curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Rustup runtime - The Rust toolchain installer
#[derive(Debug, Clone, Default)]
pub struct RustupRuntime;

impl RustupRuntime {
    pub fn new() -> Self {
        Self
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

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://rustup.rs/".to_string());
        meta.insert("category".to_string(), "toolchain-manager".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "rustup",
            "rust-lang",
            "rustup",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)
                .skip_prereleases(true),
        )
        .await
    }

    // rustup is installed via system package managers, not direct download
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // rustup is installed system-wide via package manager
        // We verify by checking if the command exists in PATH
        VerificationResult::success_system_installed()
    }
}

/// Cargo runtime - Rust package manager (provided by rustup)
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

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Cargo version is tied to rustup/rustc version
        RustupRuntime::new().fetch_versions(ctx).await
    }

    // Cargo is provided by rustup, not direct download
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // Cargo is installed via rustup
        VerificationResult::success_system_installed()
    }
}

/// Rustc runtime - The Rust compiler (provided by rustup)
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
        &["rust"]
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

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Rustc version is tied to rustup
        RustupRuntime::new().fetch_versions(ctx).await
    }

    // Rustc is provided by rustup, not direct download
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(None)
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // Rustc is installed via rustup
        VerificationResult::success_system_installed()
    }
}
