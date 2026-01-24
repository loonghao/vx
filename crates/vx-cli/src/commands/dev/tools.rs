//! Tool discovery and version detection utilities

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use vx_paths::PathManager;
use vx_runtime::{create_runtime_context, ProviderRegistry};

/// Version parser function type
type VersionParser = fn(&str) -> Option<String>;

/// Version command information: (executable, args, parser)
type VersionCommandInfo = (&'static str, &'static [&'static str], VersionParser);

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

/// Get the status and path of a tool
pub fn get_tool_status(
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

/// Get the vx-managed path for a tool
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

/// Get provider registry and runtime context for dev command
///
/// This function creates a new registry instance for tool installation operations.
/// It's used by dev command to delegate to sync for tool installation.
pub fn get_registry() -> Result<(ProviderRegistry, vx_runtime::RuntimeContext)> {
    let registry = crate::registry::create_registry();
    let context = create_runtime_context()?;
    Ok((registry, context))
}
