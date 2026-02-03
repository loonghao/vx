//! Yarn runtime implementation
//!
//! This module provides Yarn runtime support for both:
//! - Yarn 1.x (Classic): Direct download from GitHub releases
//! - Yarn 2.x+ (Berry): Managed via corepack (bundled with Node.js 16.10+)
//!
//! ## RFC 0028: Proxy-Managed Runtimes
//!
//! Yarn 2.x+ cannot be directly downloaded and installed. Instead, it's managed
//! via corepack, which is bundled with Node.js 16.10+. When a user requests
//! `vx yarn@4.0.0`, vx will:
//!
//! 1. Check `is_version_installable("4.0.0")` → returns `false`
//! 2. Ensure Node.js >= 16.10.0 is installed
//! 3. Call `prepare_execution("4.0.0", ctx)` → enables corepack if needed
//! 4. Execute yarn via system PATH (corepack manages the version)

use crate::config::YarnUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{
    Ecosystem, ExecutionContext, ExecutionPrep, Platform, Runtime, RuntimeContext, VersionInfo,
};
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

        // First, try using corepack directly from the same directory as node
        let corepack_path = node_executable.parent().map(|p| p.join("corepack"));

        let output = if let Some(ref corepack) = corepack_path {
            if corepack.exists() || corepack.with_extension("cmd").exists() {
                // Use corepack directly
                let corepack_exe = if cfg!(windows) {
                    corepack.with_extension("cmd")
                } else {
                    corepack.clone()
                };
                debug!("Using corepack at: {}", corepack_exe.display());
                Command::new(&corepack_exe)
                    .arg("enable")
                    .output()
                    .await?
            } else {
                // Fallback: use node to run corepack enable
                debug!("Corepack not found, using node to enable");
                Command::new(node_executable)
                    .args([
                        "--eval",
                        "require('child_process').execSync('corepack enable', {stdio: 'inherit'})",
                    ])
                    .output()
                    .await?
            }
        } else {
            Command::new(node_executable)
                .args([
                    "--eval",
                    "require('child_process').execSync('corepack enable', {stdio: 'inherit'})",
                ])
                .output()
                .await?
        };

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

    /// Prepare corepack for a specific yarn version
    ///
    /// This sets up corepack to use the specified yarn version by running
    /// `corepack prepare yarn@<version> --activate`
    pub async fn prepare_corepack_version(node_executable: &Path, version: &str) -> Result<()> {
        info!("Preparing corepack for yarn@{}...", version);

        let corepack_path = node_executable.parent().map(|p| p.join("corepack"));

        let output = if let Some(ref corepack) = corepack_path {
            let corepack_exe = if cfg!(windows) {
                if corepack.with_extension("cmd").exists() {
                    corepack.with_extension("cmd")
                } else {
                    corepack.clone()
                }
            } else {
                corepack.clone()
            };

            if corepack_exe.exists() {
                Command::new(&corepack_exe)
                    .args(["prepare", &format!("yarn@{}", version), "--activate"])
                    .output()
                    .await?
            } else {
                // Fallback: use npx corepack
                Command::new(node_executable)
                    .args([
                        "-e",
                        &format!(
                            "require('child_process').execSync('corepack prepare yarn@{} --activate', {{stdio: 'inherit'}})",
                            version
                        ),
                    ])
                    .output()
                    .await?
            }
        } else {
            return Err(anyhow::anyhow!(
                "Cannot find corepack - Node.js installation may be incomplete"
            ));
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Some errors are expected if the version is already prepared
            if !stderr.contains("already") {
                warn!("corepack prepare warning: {}", stderr);
            }
        }

        Ok(())
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

    /// Fetch ALL yarn versions (1.x, 2.x, 3.x, 4.x)
    ///
    /// RFC 0028: We return all versions so users can request any version.
    /// The `is_version_installable()` method determines how each version is handled.
    ///
    /// Yarn versions come from two npm packages:
    /// - `yarn`: versions 0.x, 1.x, 2.x (up to 2.4.3)
    /// - `@yarnpkg/cli-dist`: versions 2.4.1+, 3.x, 4.x (Berry)
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        use std::collections::HashSet;

        // Fetch versions from the "yarn" npm package (Classic: 1.x, early Berry: 2.x)
        let classic_versions = VersionFetcherBuilder::npm("yarn")
            .skip_prereleases()
            .lts_pattern("1.22.") // Mark 1.22.x as LTS for Classic users
            .limit(200)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch yarn versions: {}", e))?;

        // Fetch versions from "@yarnpkg/cli-dist" npm package (Berry: 3.x, 4.x)
        let berry_versions = VersionFetcherBuilder::npm("@yarnpkg/cli-dist")
            .skip_prereleases()
            .limit(200)
            .build()
            .fetch(ctx)
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to fetch @yarnpkg/cli-dist versions: {}", e);
                vec![]
            });

        // Merge versions, preferring berry_versions for duplicates (2.4.x exists in both)
        let berry_version_set: HashSet<_> = berry_versions.iter().map(|v| &v.version).collect();

        let mut all_versions: Vec<VersionInfo> = classic_versions
            .into_iter()
            .filter(|v| !berry_version_set.contains(&v.version))
            .collect();

        all_versions.extend(berry_versions);

        // Add metadata to distinguish version types
        let versions_with_metadata: Vec<VersionInfo> = all_versions
            .into_iter()
            .map(|mut v| {
                if v.version.starts_with('1') {
                    v.metadata
                        .insert("install_method".to_string(), "direct".to_string());
                    v.metadata
                        .insert("variant".to_string(), "classic".to_string());
                } else {
                    v.metadata
                        .insert("install_method".to_string(), "corepack".to_string());
                    v.metadata
                        .insert("variant".to_string(), "berry".to_string());
                }
                v
            })
            .collect();

        // Sort by version descending
        let mut sorted = versions_with_metadata;
        sorted.sort_by(|a, b| {
            vx_core::version_utils::compare_versions_str(&b.version, &a.version)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to reasonable number
        sorted.truncate(300);

        Ok(sorted)
    }

    /// RFC 0028: Check if version can be directly installed
    ///
    /// - Yarn 1.x: Directly installable via GitHub releases
    /// - Yarn 2.x+: Must use corepack (proxy-managed)
    fn is_version_installable(&self, version: &str) -> bool {
        version.starts_with('1')
    }

    /// RFC 0028: Prepare execution for proxy-managed versions (Yarn 2.x+)
    ///
    /// For Yarn 2.x+, this method:
    /// 1. Checks if corepack is enabled
    /// 2. Enables corepack if needed
    /// 3. Prepares the specific yarn version
    /// 4. Returns configuration to use system PATH for execution
    async fn prepare_execution(
        &self,
        version: &str,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        // For Yarn 1.x, no special preparation needed
        if self.is_version_installable(version) {
            return Ok(ExecutionPrep::default());
        }

        debug!("Preparing corepack for yarn@{}", version);

        // Find node executable from the PATH or vx store
        let node_exe = find_node_executable(ctx).await?;
        debug!("Found node at: {}", node_exe.display());

        // Check if corepack is enabled
        if !Self::is_corepack_enabled().await {
            info!("Enabling corepack...");
            Self::enable_corepack(&node_exe).await?;
        }

        // Prepare the specific yarn version via corepack
        Self::prepare_corepack_version(&node_exe, version).await?;

        // Return execution prep to use system PATH (corepack's yarn)
        Ok(ExecutionPrep {
            use_system_path: true,
            proxy_ready: true,
            message: Some(format!(
                "Using yarn@{} via corepack (Node.js package manager proxy)",
                version
            )),
            ..Default::default()
        })
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

/// Find the node executable from vx-managed installations
///
/// This function uses vx's `RuntimeRoot` API to find the Node.js executable.
/// Since vx ensures Node.js is installed before this point (via dependency mechanism),
/// we should always find it in the vx store.
async fn find_node_executable(_ctx: &ExecutionContext) -> Result<std::path::PathBuf> {
    // Use the new RuntimeRoot API for clean access to runtime paths
    if let Some(node_root) = vx_paths::get_latest_runtime_root("node")
        .map_err(|e| anyhow::anyhow!("Failed to get node runtime root: {}", e))?
    {
        if node_root.executable_exists() {
            debug!(
                "Found vx-managed node {} at: {}",
                node_root.version,
                node_root.executable_path().display()
            );
            return Ok(node_root.executable_path().to_path_buf());
        }

        // Log available environment variables for debugging
        debug!("Node runtime environment variables: {:?}", node_root.env_vars());
    }

    // Fallback: check execution context's PATH (for testing scenarios)
    if let Some(path_env) = _ctx.env.get("PATH") {
        for path_dir in std::env::split_paths(path_env) {
            let node_path = path_dir.join(vx_paths::with_executable_extension("node"));
            if node_path.exists() {
                debug!("Found node in context PATH at: {}", node_path.display());
                return Ok(node_path);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Node.js is required for Yarn 2.x+ (corepack) but was not found in vx store. \
        This is unexpected as vx should auto-install Node.js. Please run 'vx install node' first."
    ))
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
