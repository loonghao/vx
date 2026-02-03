//! Yarn runtime implementation
//!
//! This module provides Yarn runtime support for both:
//! - Yarn 1.x (Classic): Direct download from GitHub releases
//! - Yarn 2.x+ (Berry): Managed via corepack (bundled with Node.js 16.10+)

use crate::config::YarnUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

// Note: Yarn's Node.js version constraints are now defined in vx_runtime::ConstraintsRegistry
// This provides version-aware constraints:
// - Yarn 1.x: Node.js 12-22 (native module compatibility)
// - Yarn 2.x-3.x: Node.js 16+ with corepack
// - Yarn 4.x: Node.js 18+ with corepack

/// Yarn runtime
#[derive(Debug, Clone)]
pub struct YarnRuntime;

impl YarnRuntime {
    /// Create a new Yarn runtime
    pub fn new() -> Self {
        Self
    }

    /// Get the directory name inside the archive for a given version
    fn get_archive_dir_name(version: &str) -> String {
        format!("yarn-v{}", version)
    }

    /// Check if version uses corepack (Yarn 2.x+)
    pub fn uses_corepack(version: &str) -> bool {
        !version.starts_with('1')
    }

    /// Enable corepack for Yarn 2.x+ support
    ///
    /// This runs `corepack enable` using the provided Node.js executable
    pub async fn enable_corepack(node_executable: &Path) -> Result<()> {
        info!("Enabling corepack for Yarn 2.x+ support...");

        let output = Command::new(node_executable)
            .args(["--eval", "require('child_process').execSync('corepack enable', {stdio: 'inherit'})"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to enable corepack: {}", stderr));
        }

        info!("Corepack enabled successfully");
        Ok(())
    }

    /// Check if corepack is already enabled
    pub async fn is_corepack_enabled() -> bool {
        // Check if yarn is available and works via corepack
        if let Ok(output) = Command::new("yarn").arg("--version").output().await {
            output.status.success()
        } else {
            false
        }
    }
}

impl Default for YarnRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for YarnRuntime {
    fn name(&self) -> &str {
        "yarn"
    }

    fn description(&self) -> &str {
        "Fast, reliable, and secure dependency management"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    // Dependencies are now managed by vx_runtime::ConstraintsRegistry
    // which provides version-aware constraints for yarn@1.x, yarn@2.x, etc.

    /// Yarn uses .cmd on Windows
    fn executable_extensions(&self) -> &[&str] {
        &[".cmd", ".exe"]
    }

    /// Yarn 1.x archives extract to `yarn-v{version}/bin/`
    /// Yarn 2.x+ is not directly installable (uses corepack)
    fn executable_dir_path(&self, version: &str, _platform: &Platform) -> Option<String> {
        // Only Yarn 1.x has a predictable archive structure
        if version.starts_with('1') {
            let dir_name = Self::get_archive_dir_name(version);
            Some(format!("{}/bin", dir_name))
        } else {
            // Yarn 2.x+ is not directly installable via archive
            None
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Only Yarn 1.x (Classic) is directly installable via vx
        // Yarn 2.x+ (Berry) should be managed via corepack (bundled with Node.js)
        // Fetch versions from npm registry and filter to only 1.x versions
        let versions = VersionFetcherBuilder::npm("yarn")
            .skip_prereleases()
            .lts_pattern("1.22.")
            .limit(100)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Filter to only Yarn 1.x versions
        let classic_versions: Vec<VersionInfo> = versions
            .into_iter()
            .filter(|v| v.version.starts_with('1'))
            .collect();

        Ok(classic_versions)
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        // Only Yarn 1.x (Classic) is directly installable
        if version.starts_with('1') {
            Ok(YarnUrlBuilder::download_url(version))
        } else {
            // Yarn 2.x+ (Berry) uses corepack - no direct download URL
            Ok(None)
        }
    }

    /// Pre-run hook for yarn commands
    ///
    /// For "yarn run" commands, ensures project dependencies are installed first.
    async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool> {
        // Handle "yarn run" commands
        if args.first().is_some_and(|a| a == "run") {
            ensure_node_modules_installed(executable).await?;
        }
        Ok(true)
    }
}

/// Helper function to ensure node_modules is installed before running commands
async fn ensure_node_modules_installed(executable: &Path) -> Result<()> {
    // Check if package.json exists
    let package_json = Path::new("package.json");
    if !package_json.exists() {
        debug!("No package.json found, skipping dependency install");
        return Ok(());
    }

    // Check if node_modules exists
    let node_modules = Path::new("node_modules");
    if node_modules.exists() {
        debug!("node_modules exists, assuming dependencies are installed");
        return Ok(());
    }

    info!("Installing dependencies with yarn...");

    let status = Command::new(executable)
        .arg("install")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        warn!("yarn install failed, continuing anyway...");
    }

    Ok(())
}
