//! Tool environment builder
//!
//! This module provides functionality to build environment variables
//! for vx-managed tools, including PATH configuration.

use crate::EnvBuilder;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use vx_paths::PathManager;

/// Tool specification for environment building
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// Tool name (e.g., "node", "go", "uv")
    pub name: String,
    /// Version specification (e.g., "20.0.0", "latest")
    pub version: String,
}

impl ToolSpec {
    /// Create a new tool specification
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Default environment variables that are always passed through
/// These are essential for the system to function properly
fn default_passenv() -> Vec<&'static str> {
    #[cfg(windows)]
    {
        vec![
            // Essential Windows system variables
            "SYSTEMROOT",
            "SYSTEMDRIVE",
            "WINDIR",
            "TEMP",
            "TMP",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "HOMEDRIVE",
            "HOMEPATH",
            "USERNAME",
            "USERDOMAIN",
            // Windows path handling
            "PATHEXT",
            "COMSPEC",
            // Program Files paths (needed for some tools)
            "PROGRAMFILES",
            "PROGRAMFILES(X86)",
            "PROGRAMDATA",
            "COMMONPROGRAMFILES",
            "COMMONPROGRAMFILES(X86)",
            // Locale
            "LANG",
            "LC_*",
            // VX specific
            "VX_*",
        ]
    }
    #[cfg(not(windows))]
    {
        vec![
            // Essential Unix variables
            "HOME",
            "USER",
            "SHELL",
            "TERM",
            "COLORTERM",
            // Locale settings
            "LANG",
            "LC_*",
            "LANGUAGE",
            // Display (for GUI apps)
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "XDG_*",
            // SSH agent
            "SSH_AUTH_SOCK",
            "SSH_AGENT_PID",
            // VX specific
            "VX_*",
        ]
    }
}

/// Get essential system PATH directories that should always be included
/// even in isolation mode
fn essential_system_paths() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let mut paths = Vec::new();

        // System32 - contains essential Windows commands (where, cmd, etc.)
        if let Ok(system_root) = std::env::var("SYSTEMROOT") {
            let system_root = PathBuf::from(&system_root);
            paths.push(system_root.join("System32"));
            paths.push(system_root.join("System32").join("Wbem"));
            paths.push(
                system_root
                    .join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0"),
            );
            // Also include the root for some edge cases
            paths.push(system_root.clone());
        }

        // Fallback to common Windows paths if SYSTEMROOT is not set
        if paths.is_empty() {
            paths.push(PathBuf::from(r"C:\Windows\System32"));
            paths.push(PathBuf::from(r"C:\Windows\System32\Wbem"));
            paths.push(PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0"));
            paths.push(PathBuf::from(r"C:\Windows"));
        }

        paths
    }
    #[cfg(not(windows))]
    {
        vec![
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
            PathBuf::from("/usr/sbin"),
            PathBuf::from("/sbin"),
        ]
    }
}

/// Builder for tool execution environments
///
/// This struct provides a fluent API for building environment configurations
/// that include vx-managed tools in PATH.
///
/// # Example
///
/// ```rust,no_run
/// use vx_env::ToolEnvironment;
///
/// let env = ToolEnvironment::new()
///     .tool("node", "20.0.0")
///     .tool("go", "1.21.0")
///     .env_var("NODE_ENV", "production")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ToolEnvironment {
    /// Tools to include in the environment
    tools: Vec<ToolSpec>,
    /// Additional environment variables (setenv)
    env_vars: HashMap<String, String>,
    /// Whether to include vx bin directory
    include_vx_bin: bool,
    /// Whether to inherit current PATH
    inherit_path: bool,
    /// Whether to warn about missing tools
    warn_missing: bool,
    /// Isolation mode (default: false for backward compatibility)
    isolation: bool,
    /// Environment variables to pass through (glob patterns)
    passenv: Vec<String>,
}

impl ToolEnvironment {
    /// Create a new tool environment builder
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            env_vars: HashMap::new(),
            include_vx_bin: true,
            inherit_path: true,
            warn_missing: true,
            isolation: false,
            passenv: Vec::new(),
        }
    }

    /// Add a tool to the environment
    pub fn tool(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.tools.push(ToolSpec::new(name, version));
        self
    }

    /// Add multiple tools from a HashMap (e.g., from vx.toml)
    pub fn tools(mut self, tools: &HashMap<String, String>) -> Self {
        for (name, version) in tools {
            self.tools
                .push(ToolSpec::new(name.clone(), version.clone()));
        }
        self
    }

    /// Add an environment variable
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn env_vars(mut self, vars: &HashMap<String, String>) -> Self {
        self.env_vars.extend(vars.clone());
        self
    }

    /// Set whether to include vx bin directory in PATH
    pub fn include_vx_bin(mut self, include: bool) -> Self {
        self.include_vx_bin = include;
        self
    }

    /// Set whether to inherit current PATH
    pub fn inherit_path(mut self, inherit: bool) -> Self {
        self.inherit_path = inherit;
        self
    }

    /// Set whether to warn about missing tools
    pub fn warn_missing(mut self, warn: bool) -> Self {
        self.warn_missing = warn;
        self
    }

    /// Enable isolation mode
    ///
    /// In isolation mode, only explicitly passed environment variables are inherited.
    /// System tools are NOT available unless explicitly added.
    pub fn isolation(mut self, enabled: bool) -> Self {
        self.isolation = enabled;
        self
    }

    /// Set environment variables to pass through (like tox's passenv)
    ///
    /// Supports glob patterns (e.g., "SSH_*", "GITHUB_*")
    pub fn passenv(mut self, patterns: Vec<String>) -> Self {
        self.passenv = patterns;
        self
    }

    /// Build the environment variables
    pub fn build(self) -> Result<HashMap<String, String>> {
        let path_manager = PathManager::new()?;
        let mut env = if self.isolation {
            self.build_isolated_env()?
        } else {
            self.build_inherited_env()?
        };

        let mut path_entries = Vec::new();

        // Add tool bin directories to PATH
        let mut missing_tools = Vec::new();
        for tool in &self.tools {
            let tool_path = self.resolve_tool_path(&path_manager, &tool.name, &tool.version)?;

            match tool_path {
                Some(path) if path.exists() => {
                    path_entries.push(path);
                }
                _ => {
                    missing_tools.push(tool.name.clone());
                }
            }
        }

        // Warn about missing tools
        if self.warn_missing && !missing_tools.is_empty() {
            tracing::warn!(
                "Some tools are not installed: {}. Run 'vx setup' to install them.",
                missing_tools.join(", ")
            );
        }

        // Add vx bin directory
        if self.include_vx_bin {
            let vx_bin = path_manager.bin_dir();
            if vx_bin.exists() {
                path_entries.push(vx_bin.to_path_buf());
            }
        }

        // In isolation mode, add essential system paths at the end
        // This ensures system commands like 'where', 'cmd' are available
        if self.isolation {
            for sys_path in essential_system_paths() {
                if sys_path.exists() && !path_entries.contains(&sys_path) {
                    path_entries.push(sys_path);
                }
            }
        }

        // Build PATH
        let sep = if cfg!(windows) { ";" } else { ":" };
        let new_path = path_entries
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(sep);

        if self.inherit_path && !self.isolation {
            // Prepend to existing PATH
            if let Some(existing_path) = env.get("PATH").cloned() {
                if !new_path.is_empty() {
                    env.insert(
                        "PATH".to_string(),
                        format!("{}{}{}", new_path, sep, existing_path),
                    );
                }
            } else if !new_path.is_empty() {
                env.insert("PATH".to_string(), new_path);
            }
        } else {
            // In isolation mode, only use vx-managed paths + essential system paths
            env.insert("PATH".to_string(), new_path);
        }

        // Add custom environment variables (setenv - highest priority)
        env.extend(self.env_vars);

        Ok(env)
    }

    /// Build environment in inherited mode (legacy behavior)
    fn build_inherited_env(&self) -> Result<HashMap<String, String>> {
        let builder = EnvBuilder::new().inherit(self.inherit_path);
        Ok(builder.build())
    }

    /// Build environment in isolation mode
    fn build_isolated_env(&self) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();

        // Get all current environment variables
        let current_env: HashMap<String, String> = std::env::vars().collect();

        // Always include default passenv patterns
        let mut all_patterns: Vec<String> =
            default_passenv().iter().map(|s| s.to_string()).collect();

        // Add user-specified patterns
        all_patterns.extend(self.passenv.clone());

        // Filter environment variables based on patterns
        for (key, value) in &current_env {
            if self.matches_passenv(key, &all_patterns) {
                env.insert(key.clone(), value.clone());
            }
        }

        Ok(env)
    }

    /// Check if an environment variable name matches any passenv pattern
    fn matches_passenv(&self, name: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if pattern.ends_with('*') {
                // Glob pattern: PREFIX*
                let prefix = &pattern[..pattern.len() - 1];
                if name.starts_with(prefix) {
                    return true;
                }
            } else if pattern.contains('*') {
                // Complex glob pattern (simplified: only support * at end)
                // For more complex patterns, could use glob crate
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 && name.starts_with(parts[0]) && name.ends_with(parts[1]) {
                    return true;
                }
            } else {
                // Exact match
                if name == pattern {
                    return true;
                }
            }
        }
        false
    }

    /// Resolve the bin path for a tool
    fn resolve_tool_path(
        &self,
        path_manager: &PathManager,
        tool: &str,
        version: &str,
    ) -> Result<Option<PathBuf>> {
        // Handle "system" version - find tool from system PATH
        if version == "system" {
            return Ok(find_system_tool_path(tool));
        }

        let actual_version = if version == "latest" {
            // Find the latest installed version
            let versions = path_manager.list_store_versions(tool)?;
            match versions.last() {
                Some(v) => v.clone(),
                None => return Ok(None),
            }
        } else {
            version.to_string()
        };

        // Check store first
        let store_dir = path_manager.version_store_dir(tool, &actual_version);
        if store_dir.exists() {
            return Ok(Some(find_bin_dir(&store_dir, tool)));
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
}

/// Find a tool's directory from the system PATH
/// Returns the parent directory containing the executable
fn find_system_tool_path(tool: &str) -> Option<PathBuf> {
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

    let path_var = std::env::var("PATH").ok()?;
    let sep = if cfg!(windows) { ';' } else { ':' };

    for exe in executables {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", exe)
        } else {
            exe.to_string()
        };

        for dir in path_var.split(sep) {
            // Skip vx directories - we want system tools
            if dir.contains(".vx") {
                continue;
            }

            let exe_path = PathBuf::from(dir).join(&exe_name);
            if exe_path.exists() {
                return Some(PathBuf::from(dir));
            }
        }
    }

    None
}

/// Find the bin directory within a tool installation
///
/// Different tools have different bin directory structures:
/// - Standard: `bin/` subdirectory
/// - Direct: executables in version directory
/// - Platform-specific: `tool-{platform}/bin/` subdirectory (e.g., cmake-4.2.2-windows-x86_64/bin)
fn find_bin_dir(store_dir: &PathBuf, tool: &str) -> PathBuf {
    // Priority order:
    // 1. bin/ subdirectory (standard layout)
    let bin_dir = store_dir.join("bin");
    if bin_dir.exists() && has_executable(&bin_dir, tool) {
        return bin_dir;
    }

    // Special case for python: executable is in python/ subdirectory
    if tool == "python" {
        let python_dir = store_dir.join("python");
        if python_dir.exists() && has_executable(&python_dir, tool) {
            return python_dir;
        }
    }

    // 2. Check for platform-specific subdirectories (e.g., cmake-4.2.2-windows-x86_64)
    //    These may have their own bin/ subdirectory
    if let Ok(entries) = std::fs::read_dir(store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.starts_with(&format!("{}-", tool)) {
                    // Check for bin/ inside the platform-specific directory (e.g., cmake)
                    let nested_bin = path.join("bin");
                    if nested_bin.exists() && has_executable(&nested_bin, tool) {
                        return nested_bin;
                    }
                    // Check for executable directly in the platform-specific directory
                    if has_executable(&path, tool) {
                        return path;
                    }
                }
            }
        }
    }

    // 3. Direct in version directory
    if has_executable(store_dir, tool) {
        return store_dir.clone();
    }

    // 4. Search recursively for bin/ directory with executable (handles nested structures)
    if let Some(bin_path) = find_bin_recursive(store_dir, tool, 2) {
        return bin_path;
    }

    // Fallback: return store_dir (tool might use a different executable name)
    store_dir.clone()
}

/// Recursively search for a bin directory containing the tool executable
fn find_bin_recursive(dir: &PathBuf, tool: &str, max_depth: u32) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check if this is a bin directory
                if dir_name == "bin" && has_executable(&path, tool) {
                    return Some(path);
                }

                // Recurse into subdirectories
                if let Some(found) = find_bin_recursive(&path, tool, max_depth - 1) {
                    return Some(found);
                }
            }
        }
    }

    None
}

/// Check if a directory contains the tool executable
fn has_executable(dir: &std::path::Path, tool: &str) -> bool {
    let exe_name = if cfg!(windows) {
        format!("{}.exe", tool)
    } else {
        tool.to_string()
    };
    dir.join(&exe_name).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_spec_new() {
        let spec = ToolSpec::new("node", "20.0.0");
        assert_eq!(spec.name, "node");
        assert_eq!(spec.version, "20.0.0");
    }

    #[test]
    fn test_tool_environment_builder() {
        let builder = ToolEnvironment::new()
            .tool("node", "20.0.0")
            .tool("go", "1.21.0")
            .env_var("NODE_ENV", "production")
            .include_vx_bin(false)
            .warn_missing(false);

        assert_eq!(builder.tools.len(), 2);
        assert_eq!(
            builder.env_vars.get("NODE_ENV"),
            Some(&"production".to_string())
        );
        assert!(!builder.include_vx_bin);
        assert!(!builder.warn_missing);
    }

    #[test]
    fn test_tool_environment_from_hashmap() {
        let mut tools = HashMap::new();
        tools.insert("node".to_string(), "20.0.0".to_string());
        tools.insert("go".to_string(), "1.21.0".to_string());

        let builder = ToolEnvironment::new().tools(&tools);
        assert_eq!(builder.tools.len(), 2);
    }

    #[test]
    fn test_passenv_exact_match() {
        let builder = ToolEnvironment::new();
        let patterns = vec!["HOME".to_string(), "USER".to_string()];

        assert!(builder.matches_passenv("HOME", &patterns));
        assert!(builder.matches_passenv("USER", &patterns));
        assert!(!builder.matches_passenv("PATH", &patterns));
    }

    #[test]
    fn test_passenv_glob_pattern() {
        let builder = ToolEnvironment::new();
        let patterns = vec!["SSH_*".to_string(), "GITHUB_*".to_string()];

        assert!(builder.matches_passenv("SSH_AUTH_SOCK", &patterns));
        assert!(builder.matches_passenv("SSH_AGENT_PID", &patterns));
        assert!(builder.matches_passenv("GITHUB_TOKEN", &patterns));
        assert!(builder.matches_passenv("GITHUB_ACTIONS", &patterns));
        assert!(!builder.matches_passenv("HOME", &patterns));
        assert!(!builder.matches_passenv("GIT_TOKEN", &patterns));
    }

    #[test]
    fn test_isolation_mode() {
        let builder = ToolEnvironment::new()
            .isolation(true)
            .passenv(vec!["CI".to_string(), "CUSTOM_*".to_string()]);

        assert!(builder.isolation);
        assert_eq!(builder.passenv.len(), 2);
    }

    #[test]
    fn test_default_passenv() {
        let defaults = default_passenv();

        #[cfg(windows)]
        {
            assert!(defaults.contains(&"SYSTEMROOT"));
            assert!(defaults.contains(&"TEMP"));
            assert!(defaults.contains(&"USERPROFILE"));
            assert!(defaults.contains(&"VX_*"));
        }

        #[cfg(not(windows))]
        {
            assert!(defaults.contains(&"HOME"));
            assert!(defaults.contains(&"USER"));
            assert!(defaults.contains(&"SHELL"));
            assert!(defaults.contains(&"VX_*"));
        }
    }

    #[test]
    fn test_essential_system_paths() {
        let paths = essential_system_paths();
        assert!(!paths.is_empty(), "Should have essential system paths");

        #[cfg(windows)]
        {
            // Should contain System32 path
            let has_system32 = paths
                .iter()
                .any(|p| p.to_string_lossy().to_lowercase().contains("system32"));
            assert!(has_system32, "Should include System32 on Windows");
        }

        #[cfg(not(windows))]
        {
            // Should contain /usr/bin
            let has_usr_bin = paths.iter().any(|p| p.to_string_lossy() == "/usr/bin");
            assert!(has_usr_bin, "Should include /usr/bin on Unix");
        }
    }
}
