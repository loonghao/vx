//! PNPM runtime implementation

use crate::config::PnpmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// PNPM runtime
#[derive(Debug, Clone)]
pub struct PnpmRuntime;

impl PnpmRuntime {
    /// Create a new PNPM runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for PnpmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for PnpmRuntime {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn description(&self) -> &str {
        "Fast, disk space efficient package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// PNPM executable path - in bin/ directory after BinaryHandler extraction
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("bin/{}", platform.exe_name("pnpm"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::npm("pnpm")
            .skip_prereleases()
            .limit(100)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(PnpmUrlBuilder::download_url(version, platform))
    }

    fn mirror_urls(&self) -> Vec<vx_manifest::MirrorConfig> {
        vec![vx_manifest::MirrorConfig {
            name: "taobao".to_string(),
            region: Some("cn".to_string()),
            url: "https://npmmirror.com/mirrors/pnpm".to_string(),
            priority: 100,
            enabled: true,
        }]
    }

    async fn download_url_for_mirror(
        &self,
        mirror_base_url: &str,
        version: &str,
        platform: &Platform,
    ) -> Result<Option<String>> {
        let filename = PnpmUrlBuilder::get_filename(platform);
        Ok(Some(format!(
            "{}/v{}/{}",
            mirror_base_url.trim_end_matches('/'),
            version,
            filename
        )))
    }

    /// Rename the downloaded file to standard name (runs before verification)
    fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
        let platform = Platform::current();

        // BinaryHandler puts files in bin/ directory
        let bin_dir = install_path.join("bin");

        // Downloaded filename (e.g., pnpm-linux-x64, pnpm-win-x64.exe)
        let downloaded_name = PnpmUrlBuilder::get_filename(&platform);
        let downloaded_path = bin_dir.join(&downloaded_name);

        // Standard filename (e.g., pnpm, pnpm.exe)
        let standard_name = platform.exe_name("pnpm");
        let standard_path = bin_dir.join(&standard_name);

        // Rename if the downloaded file exists and standard doesn't
        if downloaded_path.exists() && !standard_path.exists() {
            std::fs::rename(&downloaded_path, &standard_path)?;
        }

        Ok(())
    }

    /// Pre-run hook for pnpm commands
    ///
    /// For "pnpm run" commands, ensures project dependencies are installed first.
    async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool> {
        // Handle "pnpm run" commands
        if args
            .first()
            .is_some_and(|a| a == "run" || a == "run-script")
        {
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

    info!("Installing dependencies with pnpm...");

    let status = Command::new(executable)
        .arg("install")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        warn!("pnpm install failed, continuing anyway...");
    }

    Ok(())
}
