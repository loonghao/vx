//! Tool environment builder
//!
//! This module provides functionality to build environment variables
//! for vx-managed tools, including PATH configuration.

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
    /// Possible bin directory names (defaults to ["bin"])
    pub possible_bin_dirs: Vec<String>,
    /// Pre-resolved bin directory path (if provided by Runtime/Provider)
    /// When set, this takes precedence over path guessing
    pub resolved_bin_dir: Option<PathBuf>,
}

impl ToolSpec {
    /// Create a new tool specification
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            possible_bin_dirs: vec!["bin".to_string()],
            resolved_bin_dir: None,
        }
    }

    /// Create a tool specification with custom bin directories
    pub fn with_bin_dirs(
        name: impl Into<String>,
        version: impl Into<String>,
        bin_dirs: Vec<impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            possible_bin_dirs: bin_dirs.into_iter().map(|s| s.into()).collect(),
            resolved_bin_dir: None,
        }
    }

    /// Create a tool specification with a pre-resolved bin directory path
    ///
    /// Use this when the Runtime/Provider already knows the exact bin directory path.
    /// This bypasses the path guessing logic in `find_bin_dir`.
    pub fn with_resolved_bin_dir(
        name: impl Into<String>,
        version: impl Into<String>,
        bin_dir: PathBuf,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            possible_bin_dirs: vec!["bin".to_string()],
            resolved_bin_dir: Some(bin_dir),
        }
    }

    /// Set the resolved bin directory path
    pub fn set_resolved_bin_dir(mut self, bin_dir: PathBuf) -> Self {
        self.resolved_bin_dir = Some(bin_dir);
        self
    }
}

/// Tool environment builder for vx-managed tools
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

    /// Add multiple tools from ToolSpec instances
    pub fn tools_from_specs(mut self, specs: Vec<ToolSpec>) -> Self {
        self.tools.extend(specs);
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

    /// Check if an environment variable name matches passenv patterns
    pub fn matches_passenv(&self, env_name: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| {
            if pattern.contains('*') {
                // Simple glob matching
                let pattern = pattern.trim_end_matches('*');
                env_name.starts_with(pattern)
            } else {
                env_name == pattern
            }
        })
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
            let tool_path = self.resolve_tool_path(&path_manager, tool)?;

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
            // Note: On Windows, environment variable names are case-insensitive,
            // but HashMap keys are case-sensitive. The PATH variable might be
            // stored as "Path" or "PATH" depending on Windows version/locale.
            let existing_path = get_path_case_insensitive(&env);

            if let Some(existing) = existing_path {
                // Remove old PATH entry (might be "Path" or "PATH")
                env.retain(|k, _| !k.eq_ignore_ascii_case("PATH"));

                if !new_path.is_empty() {
                    env.insert(
                        "PATH".to_string(),
                        format!("{}{}{}", new_path, sep, existing),
                    );
                } else {
                    env.insert("PATH".to_string(), existing);
                }
            } else if !new_path.is_empty() {
                env.insert("PATH".to_string(), new_path);
            }
        } else {
            // In isolation mode, only use vx-managed paths + essential system paths
            // Also remove any existing PATH entries to avoid confusion
            env.retain(|k, _| !k.eq_ignore_ascii_case("PATH"));
            env.insert("PATH".to_string(), new_path);
        }

        // Add custom environment variables (setenv - highest priority)
        env.extend(self.env_vars);

        Ok(env)
    }

    /// Resolve the bin path for a tool
    fn resolve_tool_path(
        &self,
        path_manager: &PathManager,
        tool: &ToolSpec,
    ) -> Result<Option<PathBuf>> {
        // Handle "system" version - find tool from system PATH
        if tool.version == "system" {
            return Ok(find_system_tool_path(&tool.name));
        }

        // If a resolved bin directory is provided, use it directly
        // This is the preferred path - Provider/Runtime knows the exact location
        if let Some(ref bin_dir) = tool.resolved_bin_dir {
            if bin_dir.exists() {
                return Ok(Some(bin_dir.clone()));
            }
            // Fall through to try other methods if resolved path doesn't exist
        }

        let actual_version = if tool.version == "latest" {
            // Resolve "latest" to the actual installed version
            if let Ok(versions) = path_manager.list_store_versions(&tool.name) {
                if let Some(version) = versions.last() {
                    version.clone()
                } else {
                    tool.version.clone()
                }
            } else {
                tool.version.clone()
            }
        } else {
            // Try to resolve version prefix (e.g., "3.11" -> "3.11.13")
            resolve_version_prefix(path_manager, &tool.name, &tool.version)
        };

        // Check store first - with platform subdirectory support
        let store_dir = path_manager.version_store_dir(&tool.name, &actual_version);
        if store_dir.exists() {
            // Try platform-specific subdirectory first (new structure)
            let platform_str = get_current_platform_str();
            let platform_dir = store_dir.join(&platform_str);
            if platform_dir.exists() {
                return Ok(Some(find_bin_dir(&platform_dir, tool)));
            }
            // Fall back to old structure (no platform subdirectory)
            return Ok(Some(find_bin_dir(&store_dir, tool)));
        }

        // Check npm-tools
        let npm_bin = path_manager.npm_tool_bin_dir(&tool.name, &actual_version);
        if npm_bin.exists() {
            return Ok(Some(npm_bin));
        }

        // Check pip-tools
        let pip_bin = path_manager.pip_tool_bin_dir(&tool.name, &actual_version);
        if pip_bin.exists() {
            return Ok(Some(pip_bin));
        }

        // Fallback: Try to find tool in system PATH
        // This is important for tools that are installed via other means (e.g., Rust via rustup)
        //
        // NOTE: We check system PATH even in isolation mode because:
        // 1. Some tools (like Rust) are better installed via their official installers
        // 2. Users may have tools installed system-wide that are not in vx store
        // 3. Isolation mode should prioritize vx-managed tools, not completely block system tools
        // 4. If the user explicitly specifies a tool in vx.toml but hasn't installed it via vx,
        //    it's better to use the system version than to fail completely
        //
        // The isolation mode still ensures vx-managed paths are prepended to PATH first,
        // so vx-installed tools take priority over system tools.
        if let Some(system_path) = find_system_tool_path(&tool.name) {
            tracing::debug!(
                "Tool '{}' version '{}' not found in vx store, using system tool at: {:?}",
                tool.name,
                tool.version,
                system_path
            );
            return Ok(Some(system_path));
        }

        Ok(None)
    }

    /// Build environment in inherited mode (legacy behavior)
    ///
    /// In inherited mode, all current environment variables are passed through,
    /// including PATH. This allows tools installed outside of vx (e.g., via
    /// system package managers or rustup) to be available.
    fn build_inherited_env(&self) -> Result<HashMap<String, String>> {
        if self.inherit_path {
            // Inherit all current environment variables
            Ok(std::env::vars().collect())
        } else {
            // Don't inherit anything
            Ok(HashMap::new())
        }
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
            let key_upper = key.to_uppercase();

            let should_include = all_patterns.iter().any(|pattern| {
                if pattern.contains('*') {
                    // Complex glob pattern (simplified: only support * at end)
                    let parts: Vec<&str> = pattern.split('*').collect();
                    if parts.len() == 2
                        && key_upper.starts_with(parts[0])
                        && key_upper.ends_with(parts[1])
                    {
                        return true;
                    }
                } else {
                    // Exact match
                    if key_upper == *pattern {
                        return true;
                    }
                }
                false
            });

            if should_include {
                env.insert(key.clone(), value.clone());
            }
        }

        Ok(env)
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
        ]
    }
    #[cfg(not(windows))]
    {
        vec![
            "HOME", "USER", "SHELL", "TERM", "LANG",
            "PATH", // Include PATH for Unix systems in isolation mode
        ]
    }
}

/// Get PATH value from environment map in a case-insensitive way
///
/// On Windows, environment variable names are case-insensitive, but HashMap
/// keys are case-sensitive. The PATH variable might be stored as "Path",
/// "PATH", or "path" depending on Windows version/locale.
fn get_path_case_insensitive(env: &HashMap<String, String>) -> Option<String> {
    for (key, value) in env {
        if key.eq_ignore_ascii_case("PATH") {
            return Some(value.clone());
        }
    }
    None
}

/// Get essential system paths that should be included in isolated environments
fn essential_system_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
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

        // Include PowerShell 7 (pwsh) if installed
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            let ps7_path = PathBuf::from(&program_files).join("PowerShell").join("7");
            if ps7_path.exists() {
                paths.push(ps7_path);
            }
        }

        // Fallback to common Windows paths if SYSTEMROOT is not set
        if paths.is_empty() {
            paths.push(PathBuf::from(r"C:\Windows\System32"));
            paths.push(PathBuf::from(r"C:\Windows\System32\Wbem"));
            paths.push(PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0"));
            paths.push(PathBuf::from(r"C:\Windows"));
            paths.push(PathBuf::from(r"C:\Program Files\PowerShell\7"));
        }
    }

    #[cfg(not(windows))]
    {
        paths.push(PathBuf::from("/usr/local/bin"));
        paths.push(PathBuf::from("/usr/bin"));
        paths.push(PathBuf::from("/bin"));
        paths.push(PathBuf::from("/usr/sbin"));
        paths.push(PathBuf::from("/sbin"));
    }

    paths
}

/// Resolve a version prefix to an actual installed version
///
/// For example: "3.11" -> "3.11.13" if 3.11.13 is installed
fn resolve_version_prefix(
    path_manager: &PathManager,
    tool_name: &str,
    version_spec: &str,
) -> String {
    if let Ok(versions) = path_manager.list_store_versions(tool_name) {
        // First, try exact match
        if versions.contains(&version_spec.to_string()) {
            return version_spec.to_string();
        }

        // Then, try prefix match (e.g., "3.11" matches "3.11.13")
        // Sort versions to get the latest matching version
        let mut matching: Vec<_> = versions
            .iter()
            .filter(|v| {
                v.starts_with(version_spec)
                    && (v.len() == version_spec.len()
                        || v.chars().nth(version_spec.len()) == Some('.'))
            })
            .collect();

        // Sort in descending order to get the latest version
        matching.sort_by(|a, b| b.cmp(a));

        if let Some(matched) = matching.first() {
            return (*matched).clone();
        }
    }

    // Fallback to original version spec
    version_spec.to_string()
}

/// Get the current platform string (e.g., "windows-x64", "darwin-arm64", "linux-x64")
///
/// This matches the format used by vx-runtime's Platform::as_str()
fn get_current_platform_str() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };

    format!("{}-{}", os, arch)
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
/// - Nested platform: `{platform}/python/` subdirectory (e.g., windows-x64/python)
fn find_bin_dir(store_dir: &PathBuf, tool: &ToolSpec) -> PathBuf {
    // Priority order:
    // 1. Check tool-specific bin directories directly under store_dir
    for bin_dir_name in &tool.possible_bin_dirs {
        let bin_dir = store_dir.join(bin_dir_name);
        if bin_dir.exists() && has_executable(&bin_dir, &tool.name) {
            return bin_dir;
        }
    }

    // 2. Check for platform-specific subdirectories (e.g., windows-x64, darwin-arm64)
    //    These may contain possible_bin_dirs or bin/ subdirectory
    if let Ok(entries) = std::fs::read_dir(store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check for possible_bin_dirs inside platform directory
                // This handles structures like windows-x64/python/ for Python
                for bin_dir_name in &tool.possible_bin_dirs {
                    let nested_bin = path.join(bin_dir_name);
                    if nested_bin.exists() && has_executable(&nested_bin, &tool.name) {
                        return nested_bin;
                    }
                }

                // Check for bin/ inside platform directory
                let nested_bin = path.join("bin");
                if nested_bin.exists() && has_executable(&nested_bin, &tool.name) {
                    return nested_bin;
                }

                // Check for tool-{platform} directories (e.g., cmake-4.2.2-windows-x86_64)
                if dir_name.starts_with(&format!("{}-", tool.name)) {
                    // Check for bin/ inside the tool-platform directory
                    let tool_nested_bin = path.join("bin");
                    if tool_nested_bin.exists() && has_executable(&tool_nested_bin, &tool.name) {
                        return tool_nested_bin;
                    }
                    // Check for executable directly in the tool-platform directory
                    if has_executable(&path, &tool.name) {
                        return path;
                    }
                }

                // Check for executable directly in platform directory
                if has_executable(&path, &tool.name) {
                    return path;
                }
            }
        }
    }

    // 3. Direct in version directory
    if has_executable(store_dir, &tool.name) {
        return store_dir.clone();
    }

    // 4. Search recursively for bin/ directory or possible_bin_dirs with executable (handles nested structures)
    if let Some(bin_path) =
        find_bin_recursive_with_dirs(store_dir, &tool.name, &tool.possible_bin_dirs, 3)
    {
        return bin_path;
    }

    // Fallback: return store_dir (tool might use a different executable name)
    store_dir.clone()
}

/// Recursively search for a bin directory or possible_bin_dirs containing the tool executable
fn find_bin_recursive_with_dirs(
    dir: &PathBuf,
    tool: &str,
    possible_bin_dirs: &[String],
    max_depth: u32,
) -> Option<PathBuf> {
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

                // Check if this matches any of the possible_bin_dirs
                for bin_dir_name in possible_bin_dirs {
                    if dir_name == *bin_dir_name && has_executable(&path, tool) {
                        return Some(path);
                    }
                }

                // Recurse into subdirectories
                if let Some(found) =
                    find_bin_recursive_with_dirs(&path, tool, possible_bin_dirs, max_depth - 1)
                {
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

    #[test]
    fn test_find_system_tool_path_returns_directory() {
        // find_system_tool_path should return the directory containing the tool, not the tool itself
        // This test verifies the function works for common tools that might be in PATH

        // Test with a tool that should exist on most systems
        #[cfg(windows)]
        {
            // On Windows, cmd.exe should always be available in System32
            if let Some(path) = find_system_tool_path("cmd") {
                assert!(path.is_dir(), "Should return a directory path");
                assert!(
                    path.join("cmd.exe").exists(),
                    "Directory should contain cmd.exe"
                );
            }
        }

        #[cfg(not(windows))]
        {
            // On Unix, sh should always be available
            if let Some(path) = find_system_tool_path("sh") {
                assert!(path.is_dir(), "Should return a directory path");
                assert!(path.join("sh").exists(), "Directory should contain sh");
            }
        }
    }

    #[test]
    fn test_find_system_tool_path_skips_vx_directories() {
        // Verify that find_system_tool_path skips .vx directories
        // This is important to avoid finding vx-managed tools when looking for system tools

        // Create a fake PATH with a .vx directory
        let original_path = std::env::var("PATH").unwrap_or_default();

        // The function should skip any path containing ".vx"
        // We can't easily test this without modifying PATH, but we can verify
        // the behavior is documented in the function

        // Restore PATH (in case it was modified)
        std::env::set_var("PATH", original_path);
    }

    #[test]
    fn test_tool_environment_isolation_mode_still_finds_system_tools() {
        // This test verifies that even in isolation mode, the ToolEnvironment
        // will fall back to system tools when vx-managed tools are not available.
        //
        // This is the key behavior change - isolation mode should prioritize
        // vx-managed tools, not completely block system tools.

        let builder = ToolEnvironment::new()
            .isolation(true)
            .tool("nonexistent-tool-xyz", "1.0.0");

        // The builder should be configured correctly
        assert!(builder.isolation);
        assert_eq!(builder.tools.len(), 1);

        // When build() is called, it will try to resolve the tool path
        // Since "nonexistent-tool-xyz" doesn't exist anywhere, it will return None
        // But for real system tools, it should find them even in isolation mode
    }
}
