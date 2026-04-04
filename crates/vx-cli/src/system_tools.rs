//! System tool discovery module
//!
//! This module provides functionality to discover system tools that are
//! already installed on the user's system. Unlike managed runtimes,
//! system tools are not installed by vx but are discovered from the
//! system PATH or known locations.
//!
//! # Performance Design
//!
//! The discovery uses a **batch PATH indexing** strategy to avoid N×M complexity
//! where N = number of tools and M = number of PATH directories. Instead of
//! calling `which::which()` for each tool separately (N separate scans of all
//! PATH directories), we:
//!
//! 1. Scan all PATH directories **once** → build a `HashMap<filename, PathBuf>` index
//! 2. Do O(1) lookups per tool against the index
//! 3. For tools not on PATH, fall back to provider-defined `system_paths` (glob patterns)
//!
//! This reduces discovery from O(N×M) to O(M+N), with an additional speedup from
//! using the correct `executable` name from `ManifestDrivenRuntime` (e.g. `7z`
//! instead of `7zip`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
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

/// Build a lookup index of all executable files on the system PATH.
///
/// Scans every directory in `PATH` once and records each filename → full path.
/// On Windows, filenames are stored **lowercased** so that lookups are
/// case-insensitive (matching Windows semantics).
///
/// Duplicate filenames are resolved by PATH priority: the **first** occurrence wins.
fn build_path_index() -> HashMap<String, PathBuf> {
    let mut index: HashMap<String, PathBuf> = HashMap::new();
    let path_var = std::env::var_os("PATH").unwrap_or_default();

    for dir in std::env::split_paths(&path_var) {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(fname) = path.file_name().and_then(OsStr::to_str) {
                let key = if cfg!(windows) {
                    fname.to_lowercase()
                } else {
                    fname.to_string()
                };
                // First occurrence in PATH wins
                index.entry(key).or_insert(path);
            }
        }
    }

    index
}

/// Look up an executable name in the pre-built PATH index.
///
/// On Windows, tries `name`, `name.exe`, `name.cmd`, `name.bat` (all lowercased).
/// On Unix, tries the exact name only.
fn lookup_in_index(index: &HashMap<String, PathBuf>, exe_name: &str) -> Option<PathBuf> {
    if cfg!(windows) {
        let lower = exe_name.to_lowercase();
        // Try the exact name first, then common Windows extensions
        for candidate in &[
            lower.clone(),
            format!("{}.exe", lower),
            format!("{}.cmd", lower),
            format!("{}.bat", lower),
        ] {
            if let Some(path) = index.get(candidate) {
                return Some(path.clone());
            }
        }
        None
    } else {
        index.get(exe_name).cloned()
    }
}

/// Search provider-defined `system_paths` glob patterns for an executable.
///
/// These are well-known installation locations defined in `provider.star`
/// (e.g. `C:/Program Files/7-Zip/7z.exe`, MSVC paths with `*` globs).
/// Only returns paths that are regular files.
fn search_system_paths(search_paths_json: &str) -> Option<PathBuf> {
    let paths: Vec<String> = serde_json::from_str(search_paths_json).unwrap_or_default();
    for pattern in &paths {
        // Check if the pattern contains glob wildcards
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            if let Ok(mut matches) = glob::glob(pattern)
                && let Some(Ok(path)) = matches.next()
                && path.is_file()
            {
                return Some(path);
            }
        } else {
            // Direct path — just check existence
            let path = PathBuf::from(pattern);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

/// Discover all system tools from the registry
///
/// When `detect_versions` is false (default for `list --system`), only checks
/// whether tools exist via batch PATH scanning — much faster (~0.1-0.5s vs 40-60s).
/// Pass `detect_versions: true` (via `--version-check`) to also run each tool
/// with `--version` / `-V` to capture installed version strings.
///
/// # Performance
///
/// Uses batch PATH indexing: scans all PATH directories once (O(M)), then does
/// O(1) lookups per tool. Falls back to provider-defined `system_paths` glob
/// patterns for tools not on PATH (e.g. MSVC cl.exe in Visual Studio dirs).
pub fn discover_system_tools(
    registry: &ProviderRegistry,
    detect_versions: bool,
) -> SystemToolDiscoveryResult {
    let current_platform = Platform::current();
    let mut available = Vec::new();
    let mut unavailable = Vec::new();

    // Phase 1: build a PATH index once, then look up each tool in O(1)
    let path_index = build_path_index();
    tracing::debug!(
        "PATH index built: {} executables across all PATH directories",
        path_index.len()
    );

    let mut found_tools: Vec<(String, String, PathBuf, String, Option<String>)> = Vec::new();

    for runtime_name in registry.runtime_names() {
        if let Some(runtime) = registry.get_runtime(&runtime_name) {
            // Only process system ecosystem tools
            if runtime.ecosystem() != Ecosystem::System {
                continue;
            }

            let is_platform_supported = runtime.is_platform_supported(&current_platform);
            let category = get_tool_category(&runtime_name);

            if !is_platform_supported {
                unavailable.push(DiscoveredSystemTool {
                    name: runtime_name.clone(),
                    description: runtime.description().to_string(),
                    path: None,
                    version: None,
                    category,
                    platform: Some(get_platform_label(&runtime)),
                    available: false,
                });
                continue;
            }

            // Look up using the correct executable name (e.g. "7z" not "7zip")
            let exe_name = runtime.executable_name();
            let path = lookup_in_index(&path_index, exe_name).or_else(|| {
                // Fall back to provider-defined system_paths (glob patterns)
                runtime
                    .metadata()
                    .get("search_paths")
                    .and_then(|sp| search_system_paths(sp))
            });

            if let Some(path) = path {
                found_tools.push((
                    runtime_name.clone(),
                    runtime.description().to_string(),
                    path,
                    category,
                    None, // platform
                ));
            } else {
                unavailable.push(DiscoveredSystemTool {
                    name: runtime_name.clone(),
                    description: runtime.description().to_string(),
                    path: None,
                    version: None,
                    category,
                    platform: None,
                    available: false,
                });
            }
        }
    }

    // Phase 2: detect versions (optionally, in parallel via thread pool)
    if detect_versions && !found_tools.is_empty() {
        // Use scoped threads to detect versions in parallel
        let versions: Vec<Option<String>> = {
            let handles: Vec<_> = found_tools
                .iter()
                .map(|(_, _, path, _, _)| {
                    let path = path.clone();
                    std::thread::spawn(move || get_tool_version(&path))
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap_or(None))
                .collect()
        };

        for ((name, description, path, category, platform), version) in
            found_tools.into_iter().zip(versions)
        {
            available.push(DiscoveredSystemTool {
                name,
                description,
                path: Some(path),
                version,
                category,
                platform,
                available: true,
            });
        }
    } else {
        // Skip version detection — just report paths
        for (name, description, path, category, platform) in found_tools {
            available.push(DiscoveredSystemTool {
                name,
                description,
                path: Some(path),
                version: None,
                category,
                platform,
                available: true,
            });
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

/// Timeout for version detection commands (per invocation)
const VERSION_DETECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Try to get the version of a tool by running it with --version.
///
/// Each subprocess is given a strict timeout to avoid hanging on slow tools
/// (e.g., Python/Java wrappers on Windows). Only `--version` and `-V` are
/// attempted — `-v` (often "verbose") and bare `version` sub-commands are
/// omitted to prevent unexpected behaviour.
fn get_tool_version(path: &PathBuf) -> Option<String> {
    // Only try the two most reliable flags; `-v` and `version` cause issues
    // with many tools (verbose mode, GUI prompts, etc.)
    for flag in &["--version", "-V"] {
        if let Some(version) = try_version_flag(path, flag) {
            return Some(version);
        }
    }
    None
}

/// Run a single version-detection command with a timeout.
fn try_version_flag(path: &PathBuf, flag: &str) -> Option<String> {
    let mut cmd = std::process::Command::new(path);
    cmd.arg(flag)
        .stdin(std::process::Stdio::null()) // prevent interactive prompts
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    // On Windows, prevent console window pop-ups for GUI-less tools
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn().ok()?;

    // Poll with timeout instead of blocking indefinitely
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process exited
                if status.success() {
                    let stdout = child
                        .stdout
                        .take()
                        .and_then(|mut s| {
                            let mut buf = String::new();
                            std::io::Read::read_to_string(&mut s, &mut buf).ok()?;
                            Some(buf)
                        })
                        .unwrap_or_default();
                    let stderr = child
                        .stderr
                        .take()
                        .and_then(|mut s| {
                            let mut buf = String::new();
                            std::io::Read::read_to_string(&mut s, &mut buf).ok()?;
                            Some(buf)
                        })
                        .unwrap_or_default();
                    let combined = format!("{}{}", stdout, stderr);
                    return extract_version(&combined);
                }
                return None;
            }
            Ok(None) => {
                // Still running — check timeout
                if start.elapsed() > VERSION_DETECT_TIMEOUT {
                    tracing::debug!("version check timed out for {} {}", path.display(), flag);
                    let _ = child.kill();
                    let _ = child.wait(); // reap the zombie
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }
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
        "gpg" | "codesign" | "signtool" | "certutil" | "security" => "security",

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
        "podman" | "kubectl" | "helm" | "minikube" => "container",
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
