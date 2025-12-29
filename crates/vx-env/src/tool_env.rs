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
    /// Additional environment variables
    env_vars: HashMap<String, String>,
    /// Whether to include vx bin directory
    include_vx_bin: bool,
    /// Whether to inherit current PATH
    inherit_path: bool,
    /// Whether to warn about missing tools
    warn_missing: bool,
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

    /// Build the environment variables
    pub fn build(self) -> Result<HashMap<String, String>> {
        let path_manager = PathManager::new()?;
        let mut builder = EnvBuilder::new().inherit(self.inherit_path);
        let mut missing_tools = Vec::new();

        // Add tool bin directories to PATH
        for tool in &self.tools {
            let tool_path = self.resolve_tool_path(&path_manager, &tool.name, &tool.version)?;

            match tool_path {
                Some(path) if path.exists() => {
                    builder = builder.path_prepend(path);
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
                builder = builder.path_prepend(vx_bin);
            }
        }

        // Build the base environment
        let mut env = builder.build();

        // Add custom environment variables
        env.extend(self.env_vars);

        Ok(env)
    }

    /// Resolve the bin path for a tool
    fn resolve_tool_path(
        &self,
        path_manager: &PathManager,
        tool: &str,
        version: &str,
    ) -> Result<Option<PathBuf>> {
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

/// Find the bin directory within a tool installation
///
/// Different tools have different bin directory structures:
/// - Standard: `bin/` subdirectory
/// - Direct: executables in version directory
/// - Platform-specific: `tool-{platform}/` subdirectory
fn find_bin_dir(store_dir: &PathBuf, tool: &str) -> PathBuf {
    // Priority order:
    // 1. bin/ subdirectory (standard layout)
    let bin_dir = store_dir.join("bin");
    if bin_dir.exists() && has_executable(&bin_dir, tool) {
        return bin_dir;
    }

    // 2. Check for platform-specific subdirectories (e.g., uv-x86_64-unknown-linux-gnu)
    if let Ok(entries) = std::fs::read_dir(store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.starts_with(&format!("{}-", tool)) && has_executable(&path, tool) {
                    return path;
                }
            }
        }
    }

    // 3. Direct in version directory
    if has_executable(store_dir, tool) {
        return store_dir.clone();
    }

    // Fallback: return store_dir (tool might use a different executable name)
    store_dir.clone()
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
}
