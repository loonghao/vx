//! Node.js runtime implementations
//!
//! This module provides runtime implementations for:
//! - Node.js JavaScript runtime
//! - NPM package manager
//! - NPX package runner

use crate::config::NodeUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, RuntimeDependency, VersionInfo};

/// Node.js JavaScript runtime
#[derive(Debug, Clone, Default)]
pub struct NodeRuntime;

impl NodeRuntime {
    pub fn new() -> Self {
        Self
    }

    /// Get the directory name inside the archive for a given version and platform
    pub fn get_archive_dir_name(version: &str, platform: &Platform) -> String {
        let platform_str = NodeUrlBuilder::get_platform_string(platform);
        format!("node-v{}-{}", version, platform_str)
    }
}

#[async_trait]
impl Runtime for NodeRuntime {
    fn name(&self) -> &str {
        "node"
    }

    fn description(&self) -> &str {
        "Node.js JavaScript runtime"
    }

    fn aliases(&self) -> &[&str] {
        &["nodejs"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://nodejs.org/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta
    }

    /// Node.js archives extract to:
    /// - Unix: `node-v{version}-{platform}/bin/`
    /// - Windows: `node-v{version}-win-x64/` (no bin subdirectory)
    fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
        let dir_name = Self::get_archive_dir_name(version, platform);
        if platform.is_windows() {
            Some(dir_name)
        } else {
            Some(format!("{}/bin", dir_name))
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://nodejs.org/dist/index.json";

        let response = ctx
            .get_cached_or_fetch("node", || async { ctx.http.get_json_value(url).await })
            .await?;

        let versions: Vec<VersionInfo> = response
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
            .iter()
            .filter_map(|v| {
                let version_str = v.get("version")?.as_str()?;
                // Remove 'v' prefix
                let version = version_str.strip_prefix('v').unwrap_or(version_str);
                let lts = v.get("lts").and_then(|l| l.as_str()).is_some();
                let date_str = v.get("date").and_then(|d| d.as_str());
                let released_at = date_str.and_then(|d| {
                    chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                        .ok()
                        .and_then(|date| {
                            date.and_hms_opt(0, 0, 0).map(|dt| {
                                chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc)
                            })
                        })
                });

                Some(VersionInfo {
                    version: version.to_string(),
                    released_at,
                    prerelease: false,
                    lts,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                })
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(NodeUrlBuilder::download_url(version, platform))
    }

    /// Ensure bundled tools (npm, npx) have correct permissions after extraction
    ///
    /// On Unix systems, npm and npx are shell scripts that need execute permissions.
    /// The tar extraction should preserve these, but in some environments (like Docker)
    /// the permissions may not be set correctly. This hook ensures they are.
    fn post_extract(&self, version: &str, install_path: &PathBuf) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let platform = Platform::current();
            if let Some(bin_dir_relative) = self.executable_dir_path(version, &platform) {
                let bin_dir = install_path.join(&bin_dir_relative);

                // List of bundled tools that need execute permissions
                let bundled_tools = ["npm", "npx", "node", "corepack"];

                for tool in &bundled_tools {
                    let tool_path = bin_dir.join(tool);
                    if tool_path.exists() {
                        // Always ensure the file has execute permissions for owner/group/others
                        // This is particularly important for npm/npx which are shell scripts
                        // with #!/usr/bin/env node shebang
                        debug!(
                            "Ensuring executable permissions on {}",
                            tool_path.display()
                        );
                        if let Ok(metadata) = std::fs::metadata(&tool_path) {
                            let mut permissions = metadata.permissions();
                            // Set to 0o755: rwxr-xr-x
                            permissions.set_mode(0o755);
                            if let Err(e) = std::fs::set_permissions(&tool_path, permissions) {
                                warn!(
                                    "Failed to set permissions on {}: {}",
                                    tool_path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        // Suppress unused variable warnings on non-Unix platforms
        #[cfg(not(unix))]
        {
            let _ = (version, install_path);
        }

        Ok(())
    }
}

/// NPM package manager runtime
#[derive(Debug, Clone, Default)]
pub struct NpmRuntime;

impl NpmRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for NpmRuntime {
    fn name(&self) -> &str {
        "npm"
    }

    fn description(&self) -> &str {
        "Node.js package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        // npm depends on node
        static DEPS: &[RuntimeDependency] = &[];
        // Note: We return empty here because dependencies are lazily constructed
        // In a real implementation, we'd use OnceCell or similar
        DEPS
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://www.npmjs.com/".to_string());
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert("bundled_with".to_string(), "node".to_string());
        meta
    }

    /// NPM is bundled with Node.js, so store under "node" directory
    fn store_name(&self) -> &str {
        "node"
    }

    /// NPM uses .cmd on Windows
    fn executable_extensions(&self) -> &[&str] {
        &[".cmd", ".exe"]
    }

    /// NPM is bundled with Node.js:
    /// - Unix: `node-v{version}-{platform}/bin/`
    /// - Windows: `node-v{version}-win-x64/` (no bin subdirectory)
    fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
        let dir_name = NodeRuntime::get_archive_dir_name(version, platform);
        if platform.is_windows() {
            Some(dir_name)
        } else {
            Some(format!("{}/bin", dir_name))
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // NPM is bundled with Node.js, so we fetch Node.js versions
        // and map them to the bundled npm versions
        let node_runtime = NodeRuntime::new();
        node_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // NPM is bundled with Node.js
        Ok(NodeUrlBuilder::download_url(version, platform))
    }

    /// Pre-run hook for npm commands
    ///
    /// For "npm run" commands, ensures project dependencies are installed first.
    async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool> {
        // Only handle "npm run" commands
        if args
            .first()
            .is_some_and(|a| a == "run" || a == "run-script")
        {
            ensure_node_modules_installed(executable, "npm", &["install"]).await?;
        }
        Ok(true)
    }
}

/// NPX package runner runtime
#[derive(Debug, Clone, Default)]
pub struct NpxRuntime;

impl NpxRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for NpxRuntime {
    fn name(&self) -> &str {
        "npx"
    }

    fn description(&self) -> &str {
        "Node.js package runner"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.npmjs.com/package/npx".to_string(),
        );
        meta.insert("ecosystem".to_string(), "javascript".to_string());
        meta.insert("bundled_with".to_string(), "node".to_string());
        meta
    }

    /// NPX is bundled with Node.js, so store under "node" directory
    fn store_name(&self) -> &str {
        "node"
    }

    /// NPX uses .cmd on Windows
    fn executable_extensions(&self) -> &[&str] {
        &[".cmd", ".exe"]
    }

    /// NPX is bundled with Node.js:
    /// - Unix: `node-v{version}-{platform}/bin/`
    /// - Windows: `node-v{version}-win-x64/` (no bin subdirectory)
    fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
        let dir_name = NodeRuntime::get_archive_dir_name(version, platform);
        if platform.is_windows() {
            Some(dir_name)
        } else {
            Some(format!("{}/bin", dir_name))
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // NPX is bundled with Node.js/NPM
        let node_runtime = NodeRuntime::new();
        node_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // NPX is bundled with Node.js
        Ok(NodeUrlBuilder::download_url(version, platform))
    }
}

/// Helper function to ensure node_modules is installed before running commands
///
/// Checks for package.json and node_modules directory. If package.json exists
/// but node_modules doesn't, runs the specified install command.
async fn ensure_node_modules_installed(
    executable: &Path,
    _tool_name: &str,
    install_args: &[&str],
) -> Result<()> {
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

    info!("Installing dependencies...");

    let status = Command::new(executable)
        .args(install_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        warn!("Dependency installation failed, continuing anyway...");
    }

    Ok(())
}
