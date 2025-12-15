//! Tool resolver - detects and resolves tool dependencies
//!
//! This module handles:
//! - Detecting installed tools (both vx-managed and system)
//! - Resolving tool dependencies
//! - Determining what needs to be installed

use crate::{DependencyMap, ExecutorConfig, Result, ToolSpec};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, trace};
use vx_paths::PathManager;

/// Status of a tool
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool is managed by vx and installed
    VxManaged { version: String, path: PathBuf },
    /// Tool is available in system PATH
    SystemAvailable { path: PathBuf },
    /// Tool is not installed
    NotInstalled,
    /// Tool is unknown (not in dependency map)
    Unknown,
}

impl ToolStatus {
    /// Check if the tool is available (either vx-managed or system)
    pub fn is_available(&self) -> bool {
        matches!(
            self,
            ToolStatus::VxManaged { .. } | ToolStatus::SystemAvailable { .. }
        )
    }

    /// Get the executable path if available
    pub fn executable_path(&self) -> Option<&PathBuf> {
        match self {
            ToolStatus::VxManaged { path, .. } => Some(path),
            ToolStatus::SystemAvailable { path } => Some(path),
            _ => None,
        }
    }
}

/// Resolution result for a tool execution request
#[derive(Debug)]
pub struct ResolutionResult {
    /// The primary tool to execute
    pub tool: String,

    /// The actual executable to run
    pub executable: PathBuf,

    /// Command prefix to add before user arguments
    pub command_prefix: Vec<String>,

    /// Tools that need to be installed before execution
    pub missing_dependencies: Vec<String>,

    /// Installation order for missing dependencies
    pub install_order: Vec<String>,

    /// Whether the tool itself needs installation
    pub tool_needs_install: bool,
}

/// Tool resolver that handles dependency detection and resolution
pub struct ToolResolver {
    /// Dependency map for tool specifications
    dependency_map: DependencyMap,

    /// Path manager for vx-managed tools
    path_manager: PathManager,

    /// Configuration
    config: ExecutorConfig,
}

impl ToolResolver {
    /// Create a new tool resolver
    pub fn new(config: ExecutorConfig) -> Result<Self> {
        let path_manager = PathManager::new()?;
        Ok(Self {
            dependency_map: DependencyMap::new(),
            path_manager,
            config,
        })
    }

    /// Create a resolver with a custom dependency map (for testing)
    pub fn with_dependency_map(config: ExecutorConfig, dep_map: DependencyMap) -> Result<Self> {
        let path_manager = PathManager::new()?;
        Ok(Self {
            dependency_map: dep_map,
            path_manager,
            config,
        })
    }

    /// Check the status of a tool
    pub fn check_tool_status(&self, tool_name: &str) -> ToolStatus {
        // First, check if it's a known tool
        let spec = match self.dependency_map.get(tool_name) {
            Some(spec) => spec,
            None => {
                // Unknown tool - check system PATH only
                return self.check_system_path(tool_name);
            }
        };

        let executable_name = spec.get_executable();

        // Check vx-managed installation first if preferred
        if self.config.prefer_vx_managed {
            if let Some(status) = self.check_vx_managed(executable_name) {
                return status;
            }
        }

        // Check system PATH
        if self.config.fallback_to_system {
            let system_status = self.check_system_path(executable_name);
            if system_status.is_available() {
                return system_status;
            }
        }

        // Check vx-managed if not preferred but fallback enabled
        if !self.config.prefer_vx_managed {
            if let Some(status) = self.check_vx_managed(executable_name) {
                return status;
            }
        }

        ToolStatus::NotInstalled
    }

    /// Check if a tool is installed via vx
    fn check_vx_managed(&self, tool_name: &str) -> Option<ToolStatus> {
        // Get installed versions
        let versions = self.path_manager.list_tool_versions(tool_name).ok()?;

        if let Some(version) = versions.first() {
            let path = self.path_manager.tool_executable_path(tool_name, version);
            if path.exists() {
                debug!("Found vx-managed {} version {}", tool_name, version);
                return Some(ToolStatus::VxManaged {
                    version: version.clone(),
                    path,
                });
            }
        }
        None
    }

    /// Check if a tool is available in system PATH
    fn check_system_path(&self, tool_name: &str) -> ToolStatus {
        match which::which(tool_name) {
            Ok(path) => {
                trace!("Found {} in system PATH: {:?}", tool_name, path);
                ToolStatus::SystemAvailable { path }
            }
            Err(_) => ToolStatus::NotInstalled,
        }
    }

    /// Resolve a tool for execution
    ///
    /// This method:
    /// 1. Looks up the tool specification
    /// 2. Checks if the tool and its dependencies are installed
    /// 3. Returns a resolution result with execution details
    pub fn resolve(&self, tool_name: &str) -> Result<ResolutionResult> {
        debug!("Resolving tool: {}", tool_name);

        // Get the tool specification
        let spec = self.dependency_map.get(tool_name);

        // Check tool status
        let tool_status = self.check_tool_status(tool_name);

        // Collect missing dependencies
        let mut missing_deps = Vec::new();
        let mut install_order = Vec::new();

        if let Some(spec) = spec {
            // Check each required dependency
            for dep in spec.required_dependencies() {
                let dep_name = dep.provided_by.as_deref().unwrap_or(&dep.tool_name);
                let dep_status = self.check_tool_status(dep_name);

                if !dep_status.is_available() {
                    debug!("Missing dependency: {} (for {})", dep_name, tool_name);
                    missing_deps.push(dep_name.to_string());
                }
            }

            // Get installation order
            if !missing_deps.is_empty() {
                install_order = self.get_install_order(&missing_deps);
            }
        }

        // Determine if tool itself needs installation
        let tool_needs_install = !tool_status.is_available();
        if tool_needs_install {
            // Add tool to install order if not already there
            let resolved_name = self
                .dependency_map
                .resolve_name(tool_name)
                .unwrap_or(tool_name);
            if !install_order.contains(&resolved_name.to_string()) {
                install_order.push(resolved_name.to_string());
            }
        }

        // Get executable path
        let executable = match &tool_status {
            ToolStatus::VxManaged { path, .. } => path.clone(),
            ToolStatus::SystemAvailable { path } => path.clone(),
            _ => {
                // Tool not installed - use the expected executable name
                let exe_name = spec.map(|s| s.get_executable()).unwrap_or(tool_name);
                PathBuf::from(exe_name)
            }
        };

        // Get command prefix if applicable
        let command_prefix = spec.map(|s| s.command_prefix.clone()).unwrap_or_default();

        Ok(ResolutionResult {
            tool: tool_name.to_string(),
            executable,
            command_prefix,
            missing_dependencies: missing_deps,
            install_order,
            tool_needs_install,
        })
    }

    /// Get the installation order for a set of tools
    fn get_install_order(&self, tools: &[String]) -> Vec<String> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();

        for tool in tools {
            let tool_order = self.dependency_map.get_install_order(tool);
            for t in tool_order {
                if !visited.contains(t) {
                    visited.insert(t);
                    order.push(t.to_string());
                }
            }
        }

        order
    }

    /// Get the tool specification
    pub fn get_spec(&self, tool_name: &str) -> Option<&ToolSpec> {
        self.dependency_map.get(tool_name)
    }

    /// Check if a tool is known
    pub fn is_known_tool(&self, tool_name: &str) -> bool {
        self.dependency_map.contains(tool_name)
    }

    /// Get all known tool names
    pub fn known_tools(&self) -> Vec<&str> {
        self.dependency_map.tool_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_status_is_available() {
        assert!(ToolStatus::VxManaged {
            version: "1.0.0".into(),
            path: PathBuf::from("/usr/bin/node")
        }
        .is_available());

        assert!(ToolStatus::SystemAvailable {
            path: PathBuf::from("/usr/bin/node")
        }
        .is_available());

        assert!(!ToolStatus::NotInstalled.is_available());
        assert!(!ToolStatus::Unknown.is_available());
    }

    #[test]
    fn test_resolver_creation() {
        let config = ExecutorConfig::default();
        let resolver = ToolResolver::new(config);
        assert!(resolver.is_ok());
    }

    #[test]
    fn test_known_tools() {
        let config = ExecutorConfig::default();
        let resolver = ToolResolver::new(config).unwrap();

        assert!(resolver.is_known_tool("node"));
        assert!(resolver.is_known_tool("npm"));
        assert!(resolver.is_known_tool("uv"));
        assert!(resolver.is_known_tool("cargo"));
    }
}
