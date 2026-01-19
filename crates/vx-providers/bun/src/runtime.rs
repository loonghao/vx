//! Bun runtime implementation

use crate::config::BunUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// Bun runtime
#[derive(Debug, Clone)]
pub struct BunRuntime;

impl BunRuntime {
    /// Create a new Bun runtime
    pub fn new() -> Self {
        Self
    }

    /// Get the platform-specific directory name inside the zip
    pub(crate) fn get_archive_dir_name(platform: &Platform) -> &'static str {
        use vx_runtime::{Arch, Os};
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "bun-windows-x64",
            (Os::MacOS, Arch::X86_64) => "bun-darwin-x64",
            (Os::MacOS, Arch::Aarch64) => "bun-darwin-aarch64",
            (Os::Linux, Arch::X86_64) => "bun-linux-x64",
            (Os::Linux, Arch::Aarch64) => "bun-linux-aarch64",
            _ => "bun-linux-x64",
        }
    }
}

impl Default for BunRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for BunRuntime {
    fn name(&self) -> &str {
        "bun"
    }

    fn description(&self) -> &str {
        "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// Bun archives extract to `bun-{platform}/bun`
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let dir_name = Self::get_archive_dir_name(platform);
        format!("{}/{}", dir_name, platform.exe_name("bun"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::jsdelivr("oven-sh", "bun")
            .tool_name("bun")
            .tag_prefix("bun-v")
            .prerelease_markers(&["canary", "-alpha", "-beta", "-rc"])
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        let (platform, arch) = BunUrlBuilder::get_platform_string();
        Ok(BunUrlBuilder::download_url(version, platform, arch))
    }

    /// Pre-run hook for bun commands
    ///
    /// For "bun run" commands, ensures project dependencies are installed first.
    async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool> {
        // Handle "bun run" commands
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

    info!("Installing dependencies with bun...");

    let status = Command::new(executable)
        .arg("install")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        warn!("bun install failed, continuing anyway...");
    }

    Ok(())
}

/// Bunx runtime - Bun package runner (like npx)
///
/// This is a bundled runtime that uses the same executable as bun,
/// but with a command prefix of ["x"]. The command transformation
/// (bunx args -> bun x args) is handled by the executor using
/// the manifest-defined command_prefix.
#[derive(Debug, Clone, Default)]
pub struct BunxRuntime;

impl BunxRuntime {
    /// Create a new Bunx runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for BunxRuntime {
    fn name(&self) -> &str {
        "bunx"
    }

    fn description(&self) -> &str {
        "Bun package runner (like npx)"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://bun.sh/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert("bundled_with".to_string(), "bun".to_string());
        meta
    }

    /// Bunx is bundled with Bun, so store under "bun" directory
    fn store_name(&self) -> &str {
        "bun"
    }

    /// Bunx uses the bun executable
    fn executable_name(&self) -> &str {
        "bun"
    }

    /// Bunx uses the same executable path as bun
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        // Use the same path as bun since bunx IS bun with a command prefix
        let bun_runtime = BunRuntime::new();
        bun_runtime.executable_relative_path(version, platform)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Bunx is bundled with bun, use bun's versions
        let bun_runtime = BunRuntime::new();
        bun_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Bunx is bundled with bun, use bun's download URL
        let bun_runtime = BunRuntime::new();
        bun_runtime.download_url(version, platform).await
    }
}
