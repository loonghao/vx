//! Yarn runtime implementation

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
// - Yarn 2.x-3.x: Node.js 16+
// - Yarn 4.x: Node.js 18+

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
    fn executable_dir_path(&self, version: &str, _platform: &Platform) -> Option<String> {
        let dir_name = Self::get_archive_dir_name(version);
        Some(format!("{}/bin", dir_name))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Use npm registry API instead of GitHub API to avoid rate limits
        // Note: This is for Yarn 1.x (classic), Yarn 2+ uses different distribution
        VersionFetcherBuilder::npm("yarn")
            .skip_prereleases()
            .lts_pattern("1.22.")
            .limit(100)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(YarnUrlBuilder::download_url(version))
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
