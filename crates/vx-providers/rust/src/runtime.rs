//! Rust runtime implementations
//!
//! Rust is installed via rustup, the official Rust toolchain installer.
//! rustup manages rustc, cargo, and other Rust tools automatically.
//!
//! rustup-init binary is downloaded from static.rust-lang.org and then
//! executed to install the Rust toolchain.

use crate::config::RustupUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Arch, Ecosystem, GitHubReleaseOptions, Os, Platform, Runtime, RuntimeContext,
    VerificationResult, VersionInfo,
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

    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::new(Os::Windows, Arch::X86_64),
            Platform::new(Os::Windows, Arch::X86),
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
        ]
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // After installation, rustup is renamed from rustup-init
        RustupUrlBuilder::get_final_executable_name(platform).to_string()
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

    /// Download URL for rustup-init binary
    async fn download_url(&self, _version: &str, platform: &Platform) -> Result<Option<String>> {
        // Note: rustup downloads are not versioned - always downloads latest from the URL
        // The version parameter is used for tracking but the URL gives latest
        Ok(RustupUrlBuilder::download_url(platform))
    }

    /// Post-extract: rename rustup-init to rustup and run it to install toolchain
    fn post_extract(&self, _version: &str, install_path: &std::path::PathBuf) -> Result<()> {
        use std::process::Command;

        let platform = Platform::current();
        let init_name = RustupUrlBuilder::get_executable_name(&platform);
        let final_name = RustupUrlBuilder::get_final_executable_name(&platform);

        let init_path = install_path.join(init_name);
        let final_path = install_path.join(final_name);

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if init_path.exists() {
                let mut perms = std::fs::metadata(&init_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&init_path, perms)?;
            }
        }

        eprintln!("📦 Running rustup-init to install Rust toolchain...");
        eprintln!("   This may take a few minutes on first install...");

        // Set RUSTUP_HOME and CARGO_HOME to install in our managed directory
        let rustup_home = install_path.join(".rustup");
        let cargo_home = install_path.join(".cargo");

        // Run rustup-init with -y for non-interactive install
        // --no-modify-path to avoid modifying user's shell profile
        let output = Command::new(&init_path)
            .arg("-y")
            .arg("--no-modify-path")
            .arg("--default-toolchain")
            .arg("stable")
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "Failed to run rustup-init\nstdout: {}\nstderr: {}",
                stdout,
                stderr
            ));
        }

        // Rename rustup-init to rustup for future use
        if init_path.exists() && init_name != final_name {
            std::fs::rename(&init_path, &final_path)?;
        }

        eprintln!("✓ Rust toolchain installed successfully");
        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = RustupUrlBuilder::get_final_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            // Check in .cargo/bin as well (where rustup installs itself)
            let cargo_bin = install_path.join(".cargo").join("bin").join(exe_name);
            if cargo_bin.exists() {
                return VerificationResult::success(cargo_bin);
            }

            VerificationResult::failure(
                vec![format!(
                    "rustup not found at {} or {}",
                    exe_path.display(),
                    cargo_bin.display()
                )],
                vec!["Try reinstalling rustup".to_string()],
            )
        }
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
