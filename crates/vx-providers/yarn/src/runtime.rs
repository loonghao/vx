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
use vx_core::command::{build_command, spawn_command};
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
    /// This runs `corepack enable` using the corepack executable in the same directory as node.
    /// Uses vx_core::command for proper cross-platform handling of .cmd files.
    pub async fn enable_corepack(node_executable: &Path) -> Result<()> {
        info!("Enabling corepack for Yarn 2.x+ support...");

        // Find corepack executable in the same directory as node
        let corepack_path = node_executable.parent().map(|p| {
            if cfg!(windows) {
                p.join("corepack.cmd")
            } else {
                p.join("corepack")
            }
        });

        let corepack = corepack_path.ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot find corepack - Node.js at {} has no parent directory",
                node_executable.display()
            )
        })?;

        if !corepack.exists() {
            return Err(anyhow::anyhow!(
                "Corepack not found at {}. This Node.js version may not include corepack. \
                Please use Node.js 16.10+ or 20.x+ which includes corepack.",
                corepack.display()
            ));
        }

        debug!("Using corepack at: {}", corepack.display());
        let output = spawn_command(&corepack, &["enable"]).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let combined = if stderr.is_empty() && stdout.is_empty() {
                format!("Exit code: {:?}", output.status.code())
            } else if stderr.is_empty() {
                stdout.to_string()
            } else {
                stderr.to_string()
            };
            return Err(anyhow::anyhow!("Failed to enable corepack: {}", combined));
        }

        info!("Corepack enabled successfully");
        Ok(())
    }

    /// Check if corepack is already enabled
    ///
    /// Note: This checks if yarn is available via system PATH (corepack-managed).
    /// We use tokio::process::Command directly here since we're looking for
    /// yarn in the system PATH, not a specific executable path.
    pub async fn is_corepack_enabled() -> bool {
        // Check if yarn is available and works via corepack
        // For system PATH lookup, we use Command::new directly since the shell
        // will handle finding the executable
        let mut cmd = Command::new("yarn");
        cmd.arg("--version");
        if let Ok(output) = cmd.output().await {
            output.status.success()
        } else {
            false
        }
    }

    /// Prepare corepack for a specific yarn version
    ///
    /// This sets up corepack to use the specified yarn version by running
    /// `corepack prepare yarn@<version> --activate`
    ///
    /// Uses vx_core::command for proper cross-platform handling of .cmd files.
    pub async fn prepare_corepack_version(node_executable: &Path, version: &str) -> Result<()> {
        info!("Preparing corepack for yarn@{}...", version);

        // Find corepack executable in the same directory as node
        let corepack_path = node_executable.parent().map(|p| {
            if cfg!(windows) {
                p.join("corepack.cmd")
            } else {
                p.join("corepack")
            }
        });

        let corepack = corepack_path.ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot find corepack - Node.js at {} has no parent directory",
                node_executable.display()
            )
        })?;

        if !corepack.exists() {
            return Err(anyhow::anyhow!(
                "Corepack not found at {}. This Node.js version may not include corepack. \
                Please use Node.js 16.10+ or 20.x+ which includes corepack.",
                corepack.display()
            ));
        }

        let yarn_spec = format!("yarn@{}", version);

        debug!("Using corepack at: {}", corepack.display());
        let output = spawn_command(&corepack, &["prepare", &yarn_spec, "--activate"]).await?;

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

        // Prepare the specific yarn version via corepack
        // Note: We skip `corepack enable` because:
        // 1. It requires writing to global bin directories (which may need admin permissions)
        // 2. `corepack prepare --activate` is sufficient for our use case
        // 3. We'll execute yarn via `corepack yarn`, not via a direct yarn shim
        Self::prepare_corepack_version(&node_exe, version).await?;

        // Get the corepack executable path
        // We'll use `corepack yarn` to run yarn commands
        let corepack_exe = node_exe.parent().map(|p| {
            if cfg!(windows) {
                p.join("corepack.cmd")
            } else {
                p.join("corepack")
            }
        });

        // Return execution prep that uses corepack as the executable with "yarn" as command prefix
        Ok(ExecutionPrep {
            use_system_path: false,
            proxy_ready: true,
            executable_override: corepack_exe,
            command_prefix: vec!["yarn".to_string()],
            message: Some(format!(
                "Using yarn@{} via corepack (Node.js package manager proxy)",
                version
            )),
            path_prepend: vec![node_exe.parent().unwrap().to_path_buf()],
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

    fn mirror_urls(&self) -> Vec<vx_manifest::MirrorConfig> {
        vec![vx_manifest::MirrorConfig {
            name: "taobao".to_string(),
            region: Some("cn".to_string()),
            url: "https://npmmirror.com/mirrors/yarn".to_string(),
            priority: 100,
            enabled: true,
        }]
    }

    async fn download_url_for_mirror(
        &self,
        mirror_base_url: &str,
        version: &str,
        _platform: &Platform,
    ) -> Result<Option<String>> {
        // Only Yarn 1.x is directly downloadable
        if !version.starts_with('1') {
            return Ok(None);
        }
        Ok(Some(format!(
            "{}/v{}/yarn-v{}.tar.gz",
            mirror_base_url.trim_end_matches('/'),
            version,
            version
        )))
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
/// This function prioritizes finding Node.js in the following order:
/// 1. VX_NODE_BIN environment variable (set by vx executor for proper isolation)
/// 2. Execution context's PATH (includes vx-managed tool directories)
/// 3. VxPaths API with context's VX_HOME (for isolated test environments)
/// 4. Default VxPaths API (reads system VX_HOME or ~/.vx)
///
/// For strategies 3 and 4, we specifically look for a Node.js version that has
/// corepack bundled (required for Yarn 2.x+). We prefer older LTS versions over
/// newer ones if the newer versions don't have corepack.
///
/// This ensures proper isolation in test environments and when vx manages
/// the Node.js installation in a custom VX_HOME directory.
async fn find_node_executable(ctx: &ExecutionContext) -> Result<std::path::PathBuf> {
    let node_exe_name = vx_paths::with_executable_extension("node");
    let corepack_exe_name = if cfg!(windows) {
        "corepack.cmd"
    } else {
        "corepack"
    };

    // Strategy 1: Check VX_NODE_BIN environment variable
    // This is set by vx executor's prepare_runtime_environment for proper isolation
    // However, for Yarn 2.x+, we need Node.js with corepack, so we verify it exists
    if let Some(node_bin_dir) = ctx.env.get("VX_NODE_BIN") {
        let node_path = std::path::Path::new(node_bin_dir).join(&node_exe_name);
        if node_path.exists() {
            // Check if this node has corepack
            let corepack_path = std::path::Path::new(node_bin_dir).join(corepack_exe_name);
            if corepack_path.exists() {
                debug!(
                    "Found node with corepack via VX_NODE_BIN environment variable: {}",
                    node_path.display()
                );
                return Ok(node_path);
            }
            debug!(
                "VX_NODE_BIN points to {} but corepack not found there, will search for alternative",
                node_bin_dir
            );
        } else {
            debug!(
                "VX_NODE_BIN is set to {} but node executable not found there",
                node_bin_dir
            );
        }
    }

    // Strategy 2: Check execution context's PATH
    // The executor prepends vx-managed tool directories to PATH
    // However, for Yarn 2.x+, we need Node.js with corepack, so we check for it
    if let Some(path_env) = ctx.env.get("PATH") {
        for path_dir in std::env::split_paths(path_env) {
            let node_path = path_dir.join(&node_exe_name);
            if node_path.exists() {
                // Check if this node has corepack
                let corepack_path = path_dir.join(corepack_exe_name);
                if corepack_path.exists() {
                    debug!(
                        "Found node with corepack in context PATH at: {}",
                        node_path.display()
                    );
                    return Ok(node_path);
                }
                debug!(
                    "Found node in context PATH at {} but no corepack, continuing search",
                    node_path.display()
                );
            }
        }
    }

    // Helper function to find node with corepack support
    let find_node_with_corepack = |paths: &vx_paths::VxPaths| -> Option<std::path::PathBuf> {
        let manager = vx_paths::PathManager::from_paths(paths.clone());
        let mut versions = manager.list_store_versions("node").ok()?;

        // Sort versions in descending order (newest first)
        versions.sort_by(|a, b| compare_semver(b, a));

        // Find the first version that has corepack
        for version in &versions {
            if let Ok(Some(node_root)) = vx_paths::RuntimeRoot::find("node", version, paths)
                && node_root.executable_exists()
            {
                // Check if corepack exists in the same directory
                let corepack_path = node_root.bin_dir().join(corepack_exe_name);
                if corepack_path.exists() {
                    debug!(
                        "Found node {} with corepack at: {}",
                        version,
                        node_root.executable_path().display()
                    );
                    return Some(node_root.executable_path().to_path_buf());
                } else {
                    debug!(
                        "Node {} exists but has no corepack (skipping for yarn 2.x+)",
                        version
                    );
                }
            }
        }

        // Fallback: return any node if none has corepack
        for version in &versions {
            if let Ok(Some(node_root)) = vx_paths::RuntimeRoot::find("node", version, paths)
                && node_root.executable_exists()
            {
                warn!(
                    "No Node.js version with corepack found. Using {} which may not work with Yarn 2.x+",
                    version
                );
                return Some(node_root.executable_path().to_path_buf());
            }
        }

        None
    };

    // Strategy 3: Use context's VX_HOME if set (for isolated test environments)
    if let Some(vx_home) = ctx.env.get("VX_HOME") {
        let paths = vx_paths::VxPaths::with_base_dir(vx_home);
        if let Some(node_path) = find_node_with_corepack(&paths) {
            return Ok(node_path);
        }
    }

    // Strategy 4: Use default VxPaths (reads system VX_HOME or ~/.vx)
    if let Ok(paths) = vx_paths::VxPaths::new()
        && let Some(node_path) = find_node_with_corepack(&paths)
    {
        return Ok(node_path);
    }

    Err(anyhow::anyhow!(
        "Node.js with corepack is required for Yarn 2.x+ but was not found. \
        Searched in:\n\
        - VX_NODE_BIN environment variable\n\
        - Execution context PATH\n\
        - VX_HOME from context: {:?}\n\
        - Default vx store (~/.vx/store/node)\n\n\
        Please run 'vx install node@20' to install a Node.js version with corepack.",
        ctx.env.get("VX_HOME")
    ))
}

/// Compare two semver strings
fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.split('-').next())
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
        match ap.cmp(bp) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    a_parts.len().cmp(&b_parts.len())
}

/// Helper function to ensure node_modules is installed before running commands
///
/// Uses vx_core::command for proper cross-platform handling.
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

    // Use vx_core::command for proper cross-platform handling
    let mut cmd = build_command(executable, &["install"]);
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().await?;

    if !status.success() {
        warn!("yarn install failed, continuing anyway...");
    }

    Ok(())
}
