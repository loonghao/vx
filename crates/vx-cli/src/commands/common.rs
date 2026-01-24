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
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use vx_config::{
    parse_config, ScriptConfig, ToolVersion, VxConfig,
};
use vx_paths::{find_vx_config as find_vx_config_path, PathManager};

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
    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();
    
    let view = crate::commands::setup::ConfigView {
        tools: config.tools.iter().map(|(k, v)| {
            let version = match v {
                ToolVersion::Simple(v_str) => v_str.clone(),
                ToolVersion::Detailed(details) => details.version.clone(),
            };
            (k.clone(), version)
        }).collect(),
        settings: config.settings.as_ref().map(|s| {
            // Convert SettingsConfig to HashMap for backward compatibility
            let mut map = std::collections::HashMap::new();
            if let Some(v) = s.auto_install {
                map.insert("auto_install".to_string(), if v { "true".to_string() } else { "false".to_string() });
            }
            if let Some(isolation) = s.isolation {
                map.insert("isolation".to_string(), if isolation { "true".to_string() } else { "false".to_string() });
            }
            if let Some(ref setenv) = s.setenv {
                for (k, v) in setenv {
                    map.insert(k.clone(), v.clone());
                }
            }
            if let Some(ref passenv) = s.passenv {
                let passenv_str = passenv.join(",");
                map.insert("passenv".to_string(), passenv_str);
            }
            map
        }).unwrap_or_default(),
        env: config.env.as_ref().map(|e| {
            // Convert EnvConfig to HashMap for backward compatibility
            let mut map = std::collections::HashMap::new();
            for (k, v) in &e.vars {
                map.insert(k.clone(), v.clone());
            }
            map
        }).unwrap_or_default(),
        scripts: config.scripts.iter().map(|(k, v)| {
            let command = match v {
                ScriptConfig::Simple(cmd) => cmd.clone(),
                ScriptConfig::Detailed(details) => details.command.clone(),
            };
            (k.clone(), command)
        }).collect(),
        project_name,
        isolation: config.settings.as_ref()
            .and_then(|s| s.isolation)
            .unwrap_or(true),
        setenv: config.settings.as_ref()
            .and_then(|s| s.setenv.as_ref())
            .cloned()
            .unwrap_or_default(),
        passenv: config.settings.as_ref()
            .and_then(|s| s.passenv.as_ref())
            .cloned()
            .unwrap_or_default(),
    };
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
    if let Some((tool, version)) = tool_version.split_once('@') {
        if tool.is_empty() || version.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid tool@version format: {}",
                tool_version
            ));
        }
        Ok((tool.to_string(), version.to_string()))
    } else {
        Err(anyhow::anyhow!(
            "Invalid format: {}. Expected format: tool@version (e.g., node@20.10.0)",
            tool_version
        ))
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

    // Check store first
    let store_dir = path_manager.version_store_dir(tool, &actual_version);
    if store_dir.exists() {
        let bin_path = find_tool_bin_dir(&store_dir, tool);
        return Ok((ToolStatus::Installed, Some(bin_path), Some(actual_version)));
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
        return Ok((
            ToolStatus::SystemFallback,
            Some(system_path),
            detected_version,
        ));
    }

    Ok((ToolStatus::NotInstalled, None, None))
}

/// Check the installation status of multiple tools
///
/// This is a convenience function that checks all tools in a HashMap
/// and returns a sorted list of status tuples.
///
/// # Arguments
/// * `tools` - HashMap of tool names to versions
///
/// # Returns
/// Sorted vector of tool status tuples
pub fn check_tools_status(
    tools: &HashMap<String, String>,
) -> Result<Vec<ToolStatusTuple>> {
    let path_manager = PathManager::new()?;
    let mut statuses = Vec::new();

    for (name, version) in tools {
        let (status, path, detected_version) = check_tool_status(&path_manager, name, version)?;
        statuses.push((
            name.clone(),
            version.clone(),
            status,
            path,
            detected_version,
        ));
    }

    // Sort by name for consistent output
    statuses.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(statuses)
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
pub fn find_tool_bin_dir(store_dir: &PathBuf, tool: &str) -> PathBuf {
    // Check bin/ subdirectory
    let bin_dir = store_dir.join("bin");
    if bin_dir.exists() {
        return bin_dir;
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
    store_dir.clone()
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
