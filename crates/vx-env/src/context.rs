//! Environment context detection and management
//!
//! This module provides context-aware environment detection for vx,
//! allowing commands to automatically use project-specific or global settings.

use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vx_config::parse_config;
use vx_paths::project::{find_vx_config, LOCK_FILE_NAME};
use vx_resolver::LockFile;

/// Environment context type
#[derive(Debug, Clone)]
pub enum EnvContext {
    /// Global environment (no project configuration)
    Global,
    /// Project environment with configuration
    Project {
        /// Project root directory
        root: PathBuf,
        /// Path to vx.toml/vx.toml config file
        config_path: PathBuf,
        /// Parsed tools from config
        tools: HashMap<String, String>,
        /// Lock file path (if exists)
        lock_path: Option<PathBuf>,
        /// Locked versions (if lock file exists)
        locked_versions: HashMap<String, String>,
    },
}

impl EnvContext {
    /// Detect environment context from the current directory
    pub fn detect() -> Result<Self> {
        Self::detect_from(&std::env::current_dir()?)
    }

    /// Detect environment context from a specific directory
    pub fn detect_from(start_dir: &Path) -> Result<Self> {
        match find_vx_config(start_dir) {
            Ok(config_path) => {
                let project_root = config_path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Invalid config path"))?
                    .to_path_buf();

                // Parse config to get tools
                let tools = parse_tools_from_config(&config_path)?;

                // Check for lock file
                let lock_path = project_root.join(LOCK_FILE_NAME);
                let (lock_path, locked_versions) = if lock_path.exists() {
                    let versions = parse_locked_versions(&lock_path)?;
                    (Some(lock_path), versions)
                } else {
                    (None, HashMap::new())
                };

                Ok(EnvContext::Project {
                    root: project_root,
                    config_path,
                    tools,
                    lock_path,
                    locked_versions,
                })
            }
            Err(_) => Ok(EnvContext::Global),
        }
    }

    /// Check if this is a project context
    pub fn is_project(&self) -> bool {
        matches!(self, EnvContext::Project { .. })
    }

    /// Check if this is a global context
    pub fn is_global(&self) -> bool {
        matches!(self, EnvContext::Global)
    }

    /// Get project root (if in project context)
    pub fn project_root(&self) -> Option<&Path> {
        match self {
            EnvContext::Project { root, .. } => Some(root),
            EnvContext::Global => None,
        }
    }

    /// Get tool version for a specific tool
    ///
    /// Priority:
    /// 1. Locked version (from vx.lock)
    /// 2. Config version (from vx.toml)
    /// 3. None (for global context or unknown tool)
    pub fn get_tool_version(&self, tool: &str) -> Option<String> {
        match self {
            EnvContext::Global => None,
            EnvContext::Project {
                tools,
                locked_versions,
                ..
            } => {
                // Priority: locked version > config version
                locked_versions
                    .get(tool)
                    .cloned()
                    .or_else(|| tools.get(tool).cloned())
            }
        }
    }

    /// Get all configured tools
    pub fn tools(&self) -> HashMap<String, String> {
        match self {
            EnvContext::Global => HashMap::new(),
            EnvContext::Project { tools, .. } => tools.clone(),
        }
    }

    /// Get effective tools (locked versions if available, otherwise config versions)
    pub fn effective_tools(&self) -> HashMap<String, String> {
        match self {
            EnvContext::Global => HashMap::new(),
            EnvContext::Project {
                tools,
                locked_versions,
                ..
            } => {
                let mut effective = tools.clone();
                // Override with locked versions
                for (name, version) in locked_versions {
                    effective.insert(name.clone(), version.clone());
                }
                effective
            }
        }
    }

    /// Check if lock file exists and is consistent
    pub fn has_valid_lock(&self) -> bool {
        match self {
            EnvContext::Global => false,
            EnvContext::Project {
                lock_path,
                locked_versions,
                tools,
                ..
            } => {
                if lock_path.is_none() {
                    return false;
                }
                // Check if all config tools are locked
                tools.keys().all(|name| locked_versions.contains_key(name))
            }
        }
    }

    /// Check if lock file needs update
    pub fn needs_lock_update(&self) -> bool {
        match self {
            EnvContext::Global => false,
            EnvContext::Project {
                lock_path,
                locked_versions,
                tools,
                ..
            } => {
                if lock_path.is_none() {
                    // No lock file exists
                    return !tools.is_empty();
                }
                // Check if any config tools are not locked
                tools.keys().any(|name| !locked_versions.contains_key(name))
            }
        }
    }

    /// Get context description for display
    pub fn description(&self) -> String {
        match self {
            EnvContext::Global => "global".to_string(),
            EnvContext::Project { root, lock_path, .. } => {
                let lock_status = if lock_path.is_some() {
                    "with lock"
                } else {
                    "no lock"
                };
                format!(
                    "project: {} ({})",
                    root.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| root.to_string_lossy().to_string()),
                    lock_status
                )
            }
        }
    }
}

/// Parse tools from vx.toml config using vx_config
fn parse_tools_from_config(config_path: &Path) -> Result<HashMap<String, String>> {
    let config = parse_config(config_path)
        .with_context(|| format!("Failed to parse config: {}", config_path.display()))?;
    
    Ok(config.tools_as_hashmap())
}

/// Parse locked versions from vx.lock using vx_resolver::LockFile
fn parse_locked_versions(lock_path: &Path) -> Result<HashMap<String, String>> {
    let lockfile = LockFile::load(lock_path)
        .with_context(|| format!("Failed to load lock file: {}", lock_path.display()))?;
    
    let mut versions = HashMap::new();
    for (name, tool) in &lockfile.tools {
        versions.insert(name.clone(), tool.version.clone());
    }
    
    Ok(versions)
}

/// Context override for explicit context selection
#[derive(Debug, Clone, Default)]
pub struct ContextOverride {
    /// Force global context
    pub force_global: bool,
    /// Force specific project root
    pub force_project_root: Option<PathBuf>,
    /// Override tool version
    pub tool_version_overrides: HashMap<String, String>,
}

impl ContextOverride {
    /// Create a new context override
    pub fn new() -> Self {
        Self::default()
    }

    /// Force global context
    pub fn global(mut self) -> Self {
        self.force_global = true;
        self
    }

    /// Force specific project root
    pub fn project_root(mut self, root: PathBuf) -> Self {
        self.force_project_root = Some(root);
        self
    }

    /// Override a tool version
    pub fn tool_version(mut self, tool: impl Into<String>, version: impl Into<String>) -> Self {
        self.tool_version_overrides
            .insert(tool.into(), version.into());
        self
    }

    /// Apply override to detected context
    pub fn apply(&self, context: EnvContext) -> EnvContext {
        if self.force_global {
            return EnvContext::Global;
        }

        match context {
            EnvContext::Global => {
                if let Some(ref root) = self.force_project_root {
                    // Try to detect from forced root
                    EnvContext::detect_from(root).unwrap_or(EnvContext::Global)
                } else {
                    EnvContext::Global
                }
            }
            EnvContext::Project {
                root,
                config_path,
                mut tools,
                lock_path,
                mut locked_versions,
            } => {
                // Apply tool version overrides
                for (tool, version) in &self.tool_version_overrides {
                    tools.insert(tool.clone(), version.clone());
                    // Also update locked versions to ensure override takes precedence
                    locked_versions.insert(tool.clone(), version.clone());
                }

                EnvContext::Project {
                    root,
                    config_path,
                    tools,
                    lock_path,
                    locked_versions,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(temp: &TempDir, tools: &[(&str, &str)], with_lock: bool) {
        // Create vx.toml
        let mut config_content = "[tools]\n".to_string();
        for (name, version) in tools {
            config_content.push_str(&format!("{} = \"{}\"\n", name, version));
        }
        fs::write(temp.path().join("vx.toml"), &config_content).unwrap();

        // Create vx.lock if requested
        if with_lock {
            let mut lock_content = "version = 1\n\n[metadata]\ngenerated_at = \"2025-01-01T00:00:00Z\"\nvx_version = \"0.8.0\"\nplatform = \"test\"\n\n".to_string();
            for (name, version) in tools {
                // Add .0 to make it look like a resolved version
                lock_content.push_str(&format!(
                    "[tools.{}]\nversion = \"{}.0\"\nsource = \"test\"\nresolved_from = \"{}\"\n\n",
                    name, version, version
                ));
            }
            fs::write(temp.path().join("vx.lock"), &lock_content).unwrap();
        }
    }

    #[test]
    fn test_detect_global_context() {
        let temp = TempDir::new().unwrap();
        // Don't create any config file
        let context = EnvContext::detect_from(temp.path()).unwrap();
        assert!(context.is_global());
    }

    #[test]
    fn test_detect_project_context() {
        let temp = TempDir::new().unwrap();
        create_test_project(&temp, &[("python", "3.11"), ("node", "22")], false);

        let context = EnvContext::detect_from(temp.path()).unwrap();
        assert!(context.is_project());

        let tools = context.tools();
        assert_eq!(tools.get("python"), Some(&"3.11".to_string()));
        assert_eq!(tools.get("node"), Some(&"22".to_string()));
    }

    #[test]
    fn test_detect_project_with_lock() {
        let temp = TempDir::new().unwrap();
        create_test_project(&temp, &[("python", "3.11")], true);

        let context = EnvContext::detect_from(temp.path()).unwrap();
        assert!(context.is_project());
        assert!(context.has_valid_lock());

        // Should return locked version (3.11.0), not config version (3.11)
        let version = context.get_tool_version("python");
        assert_eq!(version, Some("3.11.0".to_string()));
    }

    #[test]
    fn test_needs_lock_update() {
        let temp = TempDir::new().unwrap();
        create_test_project(&temp, &[("python", "3.11")], false);

        let context = EnvContext::detect_from(temp.path()).unwrap();
        assert!(context.needs_lock_update());

        // With lock file
        let temp2 = TempDir::new().unwrap();
        create_test_project(&temp2, &[("python", "3.11")], true);

        let context2 = EnvContext::detect_from(temp2.path()).unwrap();
        assert!(!context2.needs_lock_update());
    }

    #[test]
    fn test_context_override_global() {
        let temp = TempDir::new().unwrap();
        create_test_project(&temp, &[("python", "3.11")], false);

        let context = EnvContext::detect_from(temp.path()).unwrap();
        assert!(context.is_project());

        let override_ctx = ContextOverride::new().global();
        let overridden = override_ctx.apply(context);
        assert!(overridden.is_global());
    }

    #[test]
    fn test_context_override_tool_version() {
        let temp = TempDir::new().unwrap();
        create_test_project(&temp, &[("python", "3.11")], false);

        let context = EnvContext::detect_from(temp.path()).unwrap();

        let override_ctx = ContextOverride::new().tool_version("python", "3.14");
        let overridden = override_ctx.apply(context);

        assert_eq!(
            overridden.get_tool_version("python"),
            Some("3.14".to_string())
        );
    }
}
