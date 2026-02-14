//! Offline Bundle Support
//!
//! This module provides functionality for executing tools from local bundles
//! when network connectivity is unavailable or when forced offline mode is enabled.

use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};
use vx_paths::project::{PROJECT_VX_DIR, find_vx_config};

use super::pipeline::error::ExecuteError;

// Re-export from vx_core for convenience
pub use vx_core::exit_code_from_status;

/// Bundle directory name within .vx
pub const BUNDLE_DIR: &str = "bundle";

/// Bundle manifest file name
pub const BUNDLE_MANIFEST: &str = "manifest.json";

/// Context for using a bundled tool
#[derive(Debug, Clone)]
pub struct BundleContext {
    /// Project root directory
    pub project_root: PathBuf,
    /// Tool name
    pub tool_name: String,
    /// Tool version
    pub version: String,
    /// Full path to executable
    pub executable: PathBuf,
}

/// Bundle manifest containing metadata about the bundled environment
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BundleManifest {
    /// Manifest version (1 = legacy single-platform, 2 = multi-platform)
    #[serde(default = "default_manifest_version")]
    pub version: u32,
    /// All platforms included in this bundle
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Bundled tools with their versions
    pub tools: HashMap<String, BundledToolInfo>,
}

fn default_manifest_version() -> u32 {
    1
}

/// Information about a bundled tool
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BundledToolInfo {
    /// Resolved version
    pub version: String,
    /// Legacy path (for v1 manifests)
    #[serde(default)]
    pub path: String,
    /// Platform-specific paths (for v2 manifests)
    #[serde(default)]
    pub platform_paths: HashMap<String, String>,
}

impl BundledToolInfo {
    /// Get the path for a specific platform
    pub fn path_for_platform(&self, platform: &str) -> Option<&str> {
        // First try platform-specific path
        if let Some(path) = self.platform_paths.get(platform) {
            return Some(path.as_str());
        }
        // Fall back to legacy single path (for v1 manifests)
        if !self.path.is_empty() {
            return Some(&self.path);
        }
        None
    }
}

impl BundleManifest {
    /// Check if this bundle supports a specific platform
    pub fn supports_platform(&self, platform: &str) -> bool {
        if self.platforms.is_empty() {
            // v1 manifest - assume it supports current platform
            true
        } else {
            self.platforms.contains(&platform.to_string())
        }
    }
}

/// Quick network connectivity check
///
/// Uses a fast DNS lookup to determine if the system has internet access.
/// Returns true if online, false if offline.
pub fn is_online() -> bool {
    use std::net::ToSocketAddrs;

    // Try multiple targets for reliability
    let targets = ["github.com:443", "nodejs.org:443", "pypi.org:443"];

    for target in targets {
        if let Ok(mut addrs) = target.to_socket_addrs()
            && addrs.next().is_some()
        {
            return true;
        }
    }

    false
}

/// Check if a bundle exists for the given project root
pub fn has_bundle(project_root: &std::path::Path) -> bool {
    let manifest_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join(BUNDLE_MANIFEST);
    manifest_path.exists()
}

/// Load bundle manifest from the project
fn load_bundle_manifest(project_root: &std::path::Path) -> Option<BundleManifest> {
    let manifest_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join(BUNDLE_MANIFEST);

    let content = std::fs::read_to_string(&manifest_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Get current platform string (same format as bundle.rs)
fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Get bundle store path for a tool
/// Supports both v1 (single platform) and v2 (multi-platform) bundle structures
fn get_bundle_tool_path(
    project_root: &std::path::Path,
    tool_name: &str,
    version: &str,
) -> Option<PathBuf> {
    let base_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join("store")
        .join(tool_name)
        .join(version);

    // v2 structure: store/{tool}/{version}/{platform}/
    let platform = current_platform();
    let platform_path = base_path.join(&platform);
    if platform_path.exists() {
        return Some(platform_path);
    }

    // v1 structure (legacy): store/{tool}/{version}/
    if base_path.exists() {
        // Check if this is actually a v1 layout (no platform subdirectories)
        // by verifying it's not just an empty directory or a different platform
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            let subdirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();

            // If subdirs look like platform names (e.g., "windows-x86_64"),
            // this is v2 format and current platform is not available
            let has_platform_subdirs = subdirs.iter().any(|d| {
                let name = d.file_name().to_string_lossy().to_string();
                name.contains('-')
                    && (name.contains("windows")
                        || name.contains("linux")
                        || name.contains("macos"))
            });

            if has_platform_subdirs {
                // v2 structure, but current platform not available
                debug!(
                    "Bundle has platform-specific subdirectories but not for current platform: {}",
                    platform
                );
                return None;
            }

            // v1 structure
            return Some(base_path);
        }
    }

    None
}

/// Find executable in bundle for a tool
///
/// Returns the full path to the executable if found in the bundle.
fn find_bundle_executable(
    project_root: &std::path::Path,
    tool_name: &str,
    version: &str,
) -> Option<PathBuf> {
    let bundle_tool_path = get_bundle_tool_path(project_root, tool_name, version)?;

    // Common executable search paths within a tool directory
    let search_paths = [
        "bin",     // Most tools
        "Scripts", // Windows Python
        "",        // Root directory
    ];

    #[cfg(windows)]
    let exe_names = [
        format!("{}.exe", tool_name),
        format!("{}.cmd", tool_name),
        format!("{}.bat", tool_name),
        tool_name.to_string(),
    ];

    #[cfg(not(windows))]
    let exe_names = [tool_name.to_string()];

    for search_path in &search_paths {
        let base = if search_path.is_empty() {
            bundle_tool_path.clone()
        } else {
            bundle_tool_path.join(search_path)
        };

        for exe_name in &exe_names {
            let exe_path = base.join(exe_name);
            if exe_path.exists() && exe_path.is_file() {
                return Some(exe_path);
            }
        }
    }

    // Also search in subdirectories (e.g., node-v20.0.0-win-x64/)
    if let Ok(entries) = std::fs::read_dir(&bundle_tool_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check bin subdirectory
                let bin_path = path.join("bin");
                for exe_name in &exe_names {
                    let exe_path = bin_path.join(exe_name);
                    if exe_path.exists() && exe_path.is_file() {
                        return Some(exe_path);
                    }
                }
                // Check root of subdirectory
                for exe_name in &exe_names {
                    let exe_path = path.join(exe_name);
                    if exe_path.exists() && exe_path.is_file() {
                        return Some(exe_path);
                    }
                }
            }
        }
    }

    None
}

/// Try to get bundle context for offline execution
///
/// Returns bundle information if:
/// 1. Network is offline (or force_offline is true)
/// 2. Bundle exists in the project
/// 3. The requested tool is available in the bundle
pub fn try_get_bundle_context(tool_name: &str, force_offline: bool) -> Option<BundleContext> {
    // Check if we should use bundle
    if !force_offline && is_online() {
        return None;
    }

    // Find project root
    let cwd = std::env::current_dir().ok()?;
    let config_path = find_vx_config(&cwd).ok()?;
    let project_root = config_path.parent()?;

    // Load bundle manifest
    let manifest = load_bundle_manifest(project_root)?;

    // Check if tool is in bundle
    let bundled_tool = manifest.tools.get(tool_name)?;

    // Find executable
    let executable = find_bundle_executable(project_root, tool_name, &bundled_tool.version)?;

    info!(
        "Using bundled {} {} (offline mode)",
        tool_name, bundled_tool.version
    );

    Some(BundleContext {
        project_root: project_root.to_path_buf(),
        tool_name: tool_name.to_string(),
        version: bundled_tool.version.clone(),
        executable,
    })
}

/// Execute a bundled tool directly
///
/// This bypasses the normal resolution/installation flow and runs
/// the executable directly from the bundle.
pub async fn execute_bundle(bundle: &BundleContext, args: &[String]) -> Result<i32> {
    debug!(
        "Executing bundled tool: {} {} ({})",
        bundle.tool_name,
        bundle.version,
        bundle.executable.display()
    );

    // On Windows, .cmd and .bat files need to be executed via cmd.exe
    #[cfg(windows)]
    let mut cmd = {
        let ext = bundle
            .executable
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "cmd" || ext == "bat" {
            let mut c = Command::new("cmd.exe");
            c.arg("/c").arg(&bundle.executable);
            c
        } else {
            Command::new(&bundle.executable)
        }
    };

    #[cfg(not(windows))]
    let mut cmd = Command::new(&bundle.executable);

    cmd.args(args);

    // Set up environment with bundle bin directories in PATH
    let bundle_bin = bundle.executable.parent().map(|p| p.to_path_buf());
    if let Some(bin_dir) = bundle_bin {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = vx_paths::prepend_to_path(&current_path, &[bin_dir.display().to_string()]);
        cmd.env("PATH", new_path);
    }

    // Inherit stdio
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd
        .status()
        .await
        .map_err(|e| ExecuteError::BundleExecutionFailed {
            tool: bundle.tool_name.clone(),
            reason: e.to_string(),
        })?;

    Ok(exit_code_from_status(&status))
}

/// Execute a runtime directly using system PATH (simple fallback)
pub async fn execute_system_runtime(runtime_name: &str, args: &[String]) -> Result<i32> {
    debug!(
        "Executing system runtime: {} {}",
        runtime_name,
        args.join(" ")
    );

    let status = Command::new(runtime_name)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .map_err(|e| ExecuteError::SpawnFailed {
            executable: PathBuf::from(runtime_name),
            reason: e.to_string(),
        })?;

    Ok(exit_code_from_status(&status))
}
