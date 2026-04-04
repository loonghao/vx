//! Common utilities shared across CLI commands
//!
//! This module provides shared functionality to avoid code duplication
//! and ensure consistency across commands.
//!
//! ## Design Principles (Unix Philosophy)
//!
//! - **DRY**: Extract repeated patterns into reusable functions
//! - **Single Responsibility**: Each function does one thing well
//! - **Composability**: Functions can be combined for complex operations

use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use vx_config::{VxConfig, parse_config};
use vx_paths::{PathManager, find_vx_config as find_vx_config_path};

// =============================================================================
// Configuration Loading
// =============================================================================

/// Find vx.toml in current directory or parent directories
///
/// This is the standard way to locate project configuration.
/// Returns an error if no configuration file is found.
pub fn find_project_config(start_dir: &Path) -> Result<PathBuf> {
    find_vx_config_path(start_dir).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Find vx.toml starting from current working directory
pub fn find_project_config_cwd() -> Result<PathBuf> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    find_project_config(&current_dir)
}

/// Load and parse VxConfig (full typed configuration)
///
/// This is the recommended way to load configuration for new code.
/// For backward compatibility, see load_config_view().
pub fn load_full_config(path: &Path) -> Result<VxConfig> {
    parse_config(path)
        .with_context(|| format!("Failed to parse configuration file: {}", path.display()))
}

/// Find and load VxConfig from current directory
///
/// This combines find_project_config_cwd() and load_full_config().
pub fn load_full_config_cwd() -> Result<(PathBuf, VxConfig)> {
    let path = find_project_config_cwd()?;
    let config = load_full_config(&path)?;
    Ok((path, config))
}

/// Find and parse VxConfig, then convert to ConfigView (backward-compatible)
///
/// This provides backward compatibility with code that uses ConfigView.
/// New code should prefer load_full_config() or load_full_config_cwd().
///
/// Note: This re-imports setup::ConfigView for compatibility.
pub fn load_config_view(path: &Path) -> Result<(PathBuf, crate::commands::setup::ConfigView)> {
    let config = load_full_config(path)?;
    let view = crate::commands::setup::ConfigView::from(config);
    Ok((path.to_path_buf(), view))
}

/// Find and load ConfigView from current working directory
///
/// This is the backward-compatible version of load_full_config_cwd().
pub fn load_config_view_cwd() -> Result<(PathBuf, crate::commands::setup::ConfigView)> {
    let path = find_project_config_cwd()?;
    load_config_view(&path)
}

// =============================================================================
// Shell Detection
// =============================================================================

/// Detected shell type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Nushell,
    Unknown,
}

impl ShellType {
    /// Get shell name as string
    pub fn name(&self) -> &'static str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::PowerShell => "powershell",
            ShellType::Cmd => "cmd",
            ShellType::Nushell => "nushell",
            ShellType::Unknown => "unknown",
        }
    }

    /// Check if this is a POSIX-compatible shell
    pub fn is_posix(&self) -> bool {
        matches!(self, ShellType::Bash | ShellType::Zsh)
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Detect current shell from environment
pub fn detect_shell() -> ShellType {
    // Check SHELL environment variable (Unix)
    if let Ok(shell) = env::var("SHELL") {
        let shell_lower = shell.to_lowercase();
        if shell_lower.contains("zsh") {
            return ShellType::Zsh;
        }
        if shell_lower.contains("bash") {
            return ShellType::Bash;
        }
        if shell_lower.contains("fish") {
            return ShellType::Fish;
        }
        if shell_lower.contains("nu") {
            return ShellType::Nushell;
        }
    }

    // Check for PowerShell (Windows and cross-platform)
    if env::var("PSModulePath").is_ok() {
        return ShellType::PowerShell;
    }

    // Check for Windows CMD
    if env::var("COMSPEC").is_ok() && env::var("SHELL").is_err() {
        return ShellType::Cmd;
    }

    // Check NU_VERSION for nushell
    if env::var("NU_VERSION").is_ok() {
        return ShellType::Nushell;
    }

    ShellType::Unknown
}

/// Get shell name as string (for backward compatibility)
pub fn detect_shell_name() -> String {
    detect_shell().name().to_string()
}

// =============================================================================
// Size Formatting
// =============================================================================

/// Format byte size to human-readable string
///
/// Examples:
/// - `format_size(512)` -> "512 B"
/// - `format_size(1536)` -> "1.5 KB"
/// - `format_size(1048576)` -> "1.0 MB"
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

// =============================================================================
// Directory Size Calculation
// =============================================================================

/// Calculate total size of a directory recursively
pub fn calculate_directory_size(path: &Path) -> Result<u64> {
    if path.is_file() {
        Ok(path.metadata()?.len())
    } else if path.is_dir() {
        let mut size = 0;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                size += entry.metadata()?.len();
            }
        }
        Ok(size)
    } else {
        Ok(0)
    }
}

// =============================================================================
// Tool Version Parsing
// =============================================================================

/// Parse tool@version format
///
/// Returns (tool_name, version) tuple.
///
/// # Examples
/// ```ignore
/// parse_tool_version("node@20.10.0") // Ok(("node", "20.10.0"))
/// parse_tool_version("python@3.11") // Ok(("python", "3.11"))
/// parse_tool_version("node") // Err - missing version
/// ```
pub fn parse_tool_version(tool_version: &str) -> Result<(String, String)> {
    let request = vx_resolver::RuntimeRequest::parse(tool_version);
    if request.name.is_empty() {
        return Err(anyhow::anyhow!(
            "Invalid format: {}. Expected format: tool@version (e.g., node@20.10.0)",
            tool_version
        ));
    }
    match request.version {
        Some(version) if !version.is_empty() => Ok((request.name, version)),
        _ => Err(anyhow::anyhow!(
            "Invalid format: {}. Expected format: tool@version (e.g., node@20.10.0)",
            tool_version
        )),
    }
}

// =============================================================================
// Tool Status Checking
// =============================================================================

/// Tool installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool is installed by vx
    Installed,
    /// Tool is not installed
    NotInstalled,
    /// Tool is available from system PATH (not vx managed)
    SystemFallback,
}

/// Tool status tuple: (name, version, status, path, detected_version)
pub type ToolStatusTuple = (String, String, ToolStatus, Option<PathBuf>, Option<String>);

/// Check the installation status of a single tool
///
/// This function checks if a tool is installed via vx, available in system PATH,
/// or not available at all. It returns detailed status information.
///
/// For tools like Rust where the store uses a different version scheme (rustup versions)
/// than what vx.toml records (rustc versions), this function also checks whether
/// any installed store version contains the tool with the matching detected version.
///
/// # Arguments
/// * `path_manager` - PathManager instance for checking store
/// * `tool` - Tool name
/// * `version` - Tool version (can be "latest", "system", or a specific version)
///
/// # Returns
/// Tuple containing (status, path, detected_version)
///
/// # Examples
/// ```ignore
/// let path_manager = PathManager::new()?;
/// let (status, path, version) = check_tool_status(&path_manager, "node", "20.0.0")?;
/// ```
pub fn check_tool_status(
    path_manager: &PathManager,
    tool: &str,
    version: &str,
) -> Result<(ToolStatus, Option<PathBuf>, Option<String>)> {
    // Handle "system" version specially - use system-installed tool
    if version == "system" {
        if let Some(system_path) = find_system_tool(tool) {
            let detected_version = get_system_tool_version(tool);
            return Ok((
                ToolStatus::SystemFallback,
                Some(system_path),
                detected_version,
            ));
        }
        return Ok((ToolStatus::NotInstalled, None, None));
    }

    let actual_version = if version == "latest" {
        path_manager
            .list_store_versions(tool)?
            .last()
            .cloned()
            .unwrap_or_else(|| version.to_string())
    } else {
        version.to_string()
    };

    // Check store first (exact version match)
    let store_dir = path_manager.version_store_dir(tool, &actual_version);
    if store_dir.exists() {
        let bin_path = find_tool_bin_dir(&store_dir, tool);
        return Ok((ToolStatus::Installed, Some(bin_path), Some(actual_version)));
    }

    // For tools that use a different version scheme in the store (e.g., Rust stores
    // rustup versions like 1.28.1 but vx.toml records rustc versions like 1.93.1),
    // check if any installed store version contains the tool with the correct detected version.
    if let Some((bin_path, detected_ver)) =
        find_tool_in_store_by_detected_version(path_manager, tool, &actual_version)?
    {
        return Ok((ToolStatus::Installed, Some(bin_path), Some(detected_ver)));
    }

    // Check npm-tools
    let npm_bin = path_manager.npm_tool_bin_dir(tool, &actual_version);
    if npm_bin.exists() {
        return Ok((ToolStatus::Installed, Some(npm_bin), Some(actual_version)));
    }

    // Check pip-tools
    let pip_bin = path_manager.pip_tool_bin_dir(tool, &actual_version);
    if pip_bin.exists() {
        return Ok((ToolStatus::Installed, Some(pip_bin), Some(actual_version)));
    }

    // Check if available in system PATH as fallback
    if let Some(system_path) = find_system_tool(tool) {
        let detected_version = get_system_tool_version(tool);

        // If the system-detected version matches the requested version,
        // treat it as effectively installed (not just a fallback).
        // This handles cases where the tool is managed outside the vx store
        // (e.g., rustup installed Rust at ~/.cargo/bin/) but the correct
        // version is available.
        if let Some(ref detected) = detected_version
            && versions_match(detected, &actual_version)
        {
            return Ok((ToolStatus::Installed, Some(system_path), detected_version));
        }

        return Ok((
            ToolStatus::SystemFallback,
            Some(system_path),
            detected_version,
        ));
    }

    Ok((ToolStatus::NotInstalled, None, None))
}

/// Check if two version strings match, handling partial versions.
///
/// For example:
/// - "1.93.1" matches "1.93.1" (exact)
/// - "1.93.1" matches "1.93" (partial, 2-component match)
/// - "1.93.0" matches "1.93" (partial, 2-component with .0 patch)
fn versions_match(detected: &str, requested: &str) -> bool {
    if detected == requested {
        return true;
    }

    // Try parsing both as semver-like versions
    let det_parts: Vec<&str> = detected.split('.').collect();
    let req_parts: Vec<&str> = requested.split('.').collect();

    // If requested has fewer parts, it's a partial version
    // e.g., "1.93" should match "1.93.1"
    if req_parts.len() <= det_parts.len() {
        let all_match = req_parts.iter().zip(det_parts.iter()).all(|(r, d)| r == d);
        if all_match && req_parts.len() >= 2 {
            return true;
        }
    }

    // Check if detected matches requested with .0 patch implied
    // e.g., requested "1.93.1" matches detected "1.93.1"
    if det_parts.len() >= 2 && req_parts.len() >= 2 {
        // Normalize both to 3 components
        let det_major = det_parts[0];
        let det_minor = det_parts[1];
        let det_patch = det_parts.get(2).copied().unwrap_or("0");

        let req_major = req_parts[0];
        let req_minor = req_parts[1];
        let req_patch = req_parts.get(2).copied().unwrap_or("0");

        return det_major == req_major && det_minor == req_minor && det_patch == req_patch;
    }

    false
}

/// Search for a tool in the vx store by running the tool's version command
/// from each installed store version. This handles tools like Rust where
/// the store uses rustup versions but vx.toml records rustc versions.
///
/// Only checks tools that have known version commands (rust, go, node, python, etc.)
fn find_tool_in_store_by_detected_version(
    path_manager: &PathManager,
    tool: &str,
    requested_version: &str,
) -> Result<Option<(PathBuf, String)>> {
    // Only attempt this for tools with known alternative store layouts
    let store_bin_subdirs = get_tool_store_bin_subdirs(tool);
    if store_bin_subdirs.is_empty() {
        return Ok(None);
    }

    let (exe_name, args, parser) = match get_version_command(tool) {
        Some(cmd) => cmd,
        None => return Ok(None),
    };

    // List all installed versions in the store for this tool
    let installed_versions = path_manager.list_store_versions(tool)?;
    if installed_versions.is_empty() {
        return Ok(None);
    }

    let platform_dir_name = path_manager.platform_dir_name();

    for store_version in &installed_versions {
        let version_dir = path_manager.version_store_dir(tool, store_version);
        let platform_dir = version_dir.join(&platform_dir_name);

        // Check each possible bin subdirectory
        for subdir in &store_bin_subdirs {
            let bin_dir = platform_dir.join(subdir);
            if !bin_dir.exists() {
                continue;
            }

            let exe_path = if cfg!(windows) {
                bin_dir.join(format!("{}.exe", exe_name))
            } else {
                bin_dir.join(exe_name)
            };

            if !exe_path.exists() {
                continue;
            }

            // Run the version command to detect the actual version
            if let Ok(output) = Command::new(&exe_path)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                && output.status.success()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if let Some(detected) = parser(&stdout).or_else(|| parser(&stderr))
                    && versions_match(&detected, requested_version)
                {
                    return Ok(Some((bin_dir, detected)));
                }
            }
        }
    }

    Ok(None)
}

/// Get the known bin subdirectory patterns for a tool within the store.
///
/// Returns alternative subdirectories where executables might be found.
/// This is needed for tools that don't follow the standard bin/ layout.
fn get_tool_store_bin_subdirs(tool: &str) -> Vec<&'static str> {
    match tool {
        // Rust: executables in cargo/bin/ (via rustup)
        "rust" | "rustc" | "cargo" | "rustfmt" | "rustup" => vec!["cargo/bin"],
        _ => vec![],
    }
}

/// Internal implementation: check status for any iterable of (name, version) pairs.
fn check_tools_status_impl<'a, I>(
    path_manager: &PathManager,
    tools: I,
) -> Result<Vec<ToolStatusTuple>>
where
    I: IntoIterator<Item = (&'a String, &'a String)>,
{
    let mut statuses = Vec::new();

    for (name, version) in tools {
        let (status, path, detected_version) = check_tool_status(path_manager, name, version)?;
        statuses.push((
            name.clone(),
            version.clone(),
            status,
            path,
            detected_version,
        ));
    }

    statuses.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(statuses)
}

/// Check the installation status of multiple tools (HashMap variant)
///
/// # Arguments
/// * `tools` - HashMap of tool names to versions
///
/// # Returns
/// Sorted vector of tool status tuples
pub fn check_tools_status(tools: &HashMap<String, String>) -> Result<Vec<ToolStatusTuple>> {
    let path_manager = PathManager::new()?;
    check_tools_status_impl(&path_manager, tools)
}

/// Check the installation status of multiple tools (BTreeMap variant)
///
/// Using BTreeMap ensures deterministic ordering for lock file operations.
///
/// # Arguments
/// * `tools` - BTreeMap of tool names to versions
///
/// # Returns
/// Sorted vector of tool status tuples
pub fn check_tools_status_ordered(
    tools: &BTreeMap<String, String>,
) -> Result<Vec<ToolStatusTuple>> {
    let path_manager = PathManager::new()?;
    check_tools_status_impl(&path_manager, tools)
}

/// Get the vx-managed path for a tool
///
/// Returns None if the tool is not managed by vx (e.g., system tools).
pub fn get_vx_tool_path(
    path_manager: &PathManager,
    tool: &str,
    version: &str,
) -> Result<Option<PathBuf>> {
    let actual_version = if version == "latest" {
        path_manager
            .list_store_versions(tool)?
            .last()
            .cloned()
            .unwrap_or_else(|| version.to_string())
    } else {
        version.to_string()
    };

    // Check store
    let store_dir = path_manager.version_store_dir(tool, &actual_version);
    if store_dir.exists() {
        return Ok(Some(find_tool_bin_dir(&store_dir, tool)));
    }

    // Check npm-tools
    let npm_bin = path_manager.npm_tool_bin_dir(tool, &actual_version);
    if npm_bin.exists() {
        return Ok(Some(npm_bin));
    }

    // Check pip-tools
    let pip_bin = path_manager.pip_tool_bin_dir(tool, &actual_version);
    if pip_bin.exists() {
        return Ok(Some(pip_bin));
    }

    Ok(None)
}

/// Version parser function type
type VersionParser = fn(&str) -> Option<String>;

/// Version command information: (executable, args, parser)
type VersionCommandInfo = (&'static str, &'static [&'static str], VersionParser);

/// Get the version command and parser for a tool
fn get_version_command(tool: &str) -> Option<VersionCommandInfo> {
    match tool {
        "rust" => Some(("cargo", &["--version"][..], |output| {
            // "cargo 1.91.1 (ea2d97820 2025-10-10)" -> "1.91.1"
            output.split_whitespace().nth(1).map(|s| s.to_string())
        })),
        "go" | "golang" => Some(("go", &["version"][..], |output| {
            // "go version go1.21.0 linux/amd64" -> "1.21.0"
            output
                .split_whitespace()
                .find(|s| s.starts_with("go"))
                .and_then(|s| s.strip_prefix("go"))
                .map(|s| s.to_string())
        })),
        "node" | "nodejs" => Some(("node", &["--version"][..], |output| {
            // "v20.0.0" -> "20.0.0"
            output.trim().strip_prefix('v').map(|s| s.to_string())
        })),
        "python" => Some(("python", &["--version"][..], |output| {
            // "Python 3.11.0" -> "3.11.0"
            output.split_whitespace().nth(1).map(|s| s.to_string())
        })),
        "uv" => Some(("uv", &["--version"][..], |output| {
            // "uv 0.5.0" -> "0.5.0"
            output.split_whitespace().nth(1).map(|s| s.to_string())
        })),
        "deno" => Some(("deno", &["--version"][..], |output| {
            // "deno 1.40.0 ..." -> "1.40.0"
            output
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .map(|s| s.to_string())
        })),
        "bun" => Some(("bun", &["--version"][..], |output| {
            // "1.0.0" -> "1.0.0"
            Some(output.trim().to_string())
        })),
        // For unknown tools, we can't get version without knowing the executable
        _ => None,
    }
}

/// Get the version of a system-installed tool
pub fn get_system_tool_version(tool: &str) -> Option<String> {
    let (exe, args, parser) = get_version_command(tool)?;

    let output = Command::new(exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Some tools output version to stderr
    parser(&stdout).or_else(|| parser(&stderr))
}

/// Find a tool in the system PATH (excluding vx paths)
pub fn find_system_tool(tool: &str) -> Option<PathBuf> {
    // Map tool names to their actual executables
    // Some tools have different names for the provider vs the executable
    let executables: Vec<&str> = match tool {
        "rust" => vec!["cargo", "rustc"],
        "go" | "golang" => vec!["go"],
        "node" | "nodejs" => vec!["node"],
        "python" => vec!["python", "python3"],
        "uv" => vec!["uv"],
        _ => vec![tool],
    };

    let path_var = env::var("PATH").ok()?;
    let sep = if cfg!(windows) { ';' } else { ':' };

    for exe in executables {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", exe)
        } else {
            exe.to_string()
        };

        for dir in path_var.split(sep) {
            // Skip vx directories
            if dir.contains(".vx") {
                continue;
            }

            let exe_path = PathBuf::from(dir).join(&exe_name);
            if exe_path.exists() {
                return Some(exe_path);
            }
        }
    }

    None
}

/// Find the bin directory within a tool installation
pub fn find_tool_bin_dir(store_dir: &std::path::Path, tool: &str) -> PathBuf {
    // Check bin/ subdirectory
    let bin_dir = store_dir.join("bin");
    if bin_dir.exists() {
        return bin_dir;
    }

    // Check cargo/bin/ subdirectory (Rust tools via rustup)
    let cargo_bin_dir = store_dir.join("cargo").join("bin");
    if cargo_bin_dir.exists() {
        return cargo_bin_dir;
    }

    // Check for platform-specific subdirectories
    if let Ok(entries) = std::fs::read_dir(store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.starts_with(&format!("{}-", tool)) {
                    return path;
                }
            }
        }
    }

    // Return store_dir as fallback
    store_dir.to_path_buf()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_parse_tool_version() {
        // Valid cases
        assert_eq!(
            parse_tool_version("node@20.10.0").unwrap(),
            ("node".to_string(), "20.10.0".to_string())
        );
        assert_eq!(
            parse_tool_version("python@3.11.0").unwrap(),
            ("python".to_string(), "3.11.0".to_string())
        );

        // Invalid cases
        assert!(parse_tool_version("node").is_err());
        assert!(parse_tool_version("@20.10.0").is_err());
        assert!(parse_tool_version("node@").is_err());
        assert!(parse_tool_version("").is_err());
    }

    #[test]
    fn test_shell_type_name() {
        assert_eq!(ShellType::Bash.name(), "bash");
        assert_eq!(ShellType::Zsh.name(), "zsh");
        assert_eq!(ShellType::PowerShell.name(), "powershell");
    }

    #[test]
    fn test_shell_type_is_posix() {
        assert!(ShellType::Bash.is_posix());
        assert!(ShellType::Zsh.is_posix());
        assert!(!ShellType::Fish.is_posix());
        assert!(!ShellType::PowerShell.is_posix());
    }
}
