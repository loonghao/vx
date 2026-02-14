//! System tool discovery module
//!
//! This module provides functionality to discover system tools that are
//! already installed on the user's system. Unlike managed runtimes,
//! system tools are not installed by vx but are discovered from the
//! system PATH or known locations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use vx_runtime::{Ecosystem, Platform, ProviderRegistry, Runtime};

/// Information about a discovered system tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSystemTool {
    /// Tool name
    pub name: String,
    /// Description
    pub description: String,
    /// Path to the executable (if found)
    pub path: Option<PathBuf>,
    /// Version (if detected)
    pub version: Option<String>,
    /// Category (build, network, security, etc.)
    pub category: String,
    /// Platform constraint (if any)
    pub platform: Option<String>,
    /// Whether the tool is available on the current system
    pub available: bool,
}

/// System tool discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemToolDiscoveryResult {
    /// Tools that are available on the current system
    pub available: Vec<DiscoveredSystemTool>,
    /// Tools that are not available (not installed or wrong platform)
    pub unavailable: Vec<DiscoveredSystemTool>,
}

/// Discover all system tools from the registry
pub fn discover_system_tools(registry: &ProviderRegistry) -> SystemToolDiscoveryResult {
    let current_platform = Platform::current();
    let mut available = Vec::new();
    let mut unavailable = Vec::new();

    for runtime_name in registry.runtime_names() {
        if let Some(runtime) = registry.get_runtime(&runtime_name) {
            // Only process system ecosystem tools
            if runtime.ecosystem() != Ecosystem::System {
                continue;
            }

            let is_platform_supported = runtime.is_platform_supported(&current_platform);
            let category = get_tool_category(&runtime_name);

            // Try to find the tool
            let (path, version) = if is_platform_supported {
                find_system_tool(&runtime)
            } else {
                (None, None)
            };

            let tool = DiscoveredSystemTool {
                name: runtime_name.clone(),
                description: runtime.description().to_string(),
                path: path.clone(),
                version,
                category,
                platform: if !is_platform_supported {
                    Some(get_platform_label(&runtime))
                } else {
                    None
                },
                available: path.is_some(),
            };

            if path.is_some() {
                available.push(tool);
            } else {
                unavailable.push(tool);
            }
        }
    }

    // Sort by name
    available.sort_by(|a, b| a.name.cmp(&b.name));
    unavailable.sort_by(|a, b| a.name.cmp(&b.name));

    SystemToolDiscoveryResult {
        available,
        unavailable,
    }
}

/// Find a system tool by checking PATH and known locations
fn find_system_tool(runtime: &std::sync::Arc<dyn Runtime>) -> (Option<PathBuf>, Option<String>) {
    let exe_name = runtime.executable_name();

    // First, try to find in PATH
    if let Ok(path) = which::which(exe_name) {
        // Try to get version
        let version = get_tool_version(&path);
        return (Some(path), version);
    }

    // Check known search paths from metadata
    if let Some(search_paths) = runtime.metadata().get("search_paths") {
        // Parse search paths (could be JSON array or comma-separated)
        let paths: Vec<&str> = if search_paths.starts_with('[') {
            // JSON array format
            serde_json::from_str(search_paths).unwrap_or_default()
        } else {
            search_paths.split(',').map(|s| s.trim()).collect()
        };

        for search_path in paths {
            let mut full_path = PathBuf::from(search_path);

            // Add executable name with platform-specific extension
            #[cfg(windows)]
            {
                full_path.push(format!("{}.exe", exe_name));
            }
            #[cfg(not(windows))]
            {
                full_path.push(exe_name);
            }

            if full_path.exists() {
                let version = get_tool_version(&full_path);
                return (Some(full_path), version);
            }
        }
    }

    (None, None)
}

/// Try to get the version of a tool by running it with --version
fn get_tool_version(path: &PathBuf) -> Option<String> {
    // Try common version flags
    for flag in &["--version", "-V", "-v", "version"] {
        if let Ok(output) = std::process::Command::new(path).arg(flag).output()
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);

            // Try to extract version number
            if let Some(version) = extract_version(&combined) {
                return Some(version);
            }
        }
    }
    None
}

/// Extract version number from output string
fn extract_version(output: &str) -> Option<String> {
    // Common patterns for version strings
    let patterns = [
        r"(\d+\.\d+\.\d+)",       // 1.2.3
        r"(\d+\.\d+)",            // 1.2
        r"version\s+(\d+[\d.]+)", // version 1.2.3
        r"v(\d+[\d.]+)",          // v1.2.3
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern)
            && let Some(caps) = re.captures(output)
            && let Some(m) = caps.get(1)
        {
            return Some(m.as_str().to_string());
        }
    }
    None
}

/// Get tool category based on runtime name
fn get_tool_category(runtime_name: &str) -> String {
    match runtime_name {
        // Build tools
        "cmake" | "make" | "ninja" | "msbuild" | "xcodebuild" | "bazel" | "gradle" | "maven" => {
            "build"
        }
        // Compilers
        "cl" | "nmake" | "gcc" | "clang" | "swift" | "swiftc" | "javac" => "compiler",
        // Security
        "openssl" | "gpg" | "codesign" | "signtool" | "certutil" | "security" => "security",
        // Network
        "curl" | "wget" | "ssh" | "scp" | "rsync" | "netstat" | "tcpdump" => "network",
        // System
        "systemctl" | "journalctl" | "launchctl" | "loginctl" | "systemd-analyze" => "system",
        // Package managers
        "choco" | "brew" | "apt" | "apt-get" | "yum" | "dnf" | "pacman" | "winget" | "scoop" => {
            "package"
        }
        // Version control
        "git" | "svn" | "hg" | "git-lfs" => "vcs",
        // Container
        "docker" | "podman" | "kubectl" | "helm" | "minikube" => "container",
        // Cloud
        "aws" | "az" | "gcloud" | "terraform" | "pulumi" | "ansible" => "cloud",
        // Archive
        "tar" | "zip" | "unzip" | "7z" => "archive",
        // Filesystem
        "robocopy" | "xcopy" | "find" | "fd" | "rg" => "filesystem",
        // MLOps
        "nvidia-smi" | "nvcc" | "mlflow" | "dvc" | "wandb" => "mlops",
        _ => "other",
    }
    .to_string()
}

/// Get platform label for a runtime
fn get_platform_label(runtime: &std::sync::Arc<dyn Runtime>) -> String {
    let platforms = runtime.supported_platforms();
    if platforms.is_empty() {
        return "universal".to_string();
    }

    platforms
        .iter()
        .map(|p| p.as_str())
        .collect::<Vec<_>>()
        .join("/")
}

/// Group system tools by category
pub fn group_by_category(
    tools: &[DiscoveredSystemTool],
) -> HashMap<String, Vec<&DiscoveredSystemTool>> {
    let mut groups: HashMap<String, Vec<&DiscoveredSystemTool>> = HashMap::new();

    for tool in tools {
        groups.entry(tool.category.clone()).or_default().push(tool);
    }

    // Sort tools within each category
    for tools in groups.values_mut() {
        tools.sort_by(|a, b| a.name.cmp(&b.name));
    }

    groups
}
