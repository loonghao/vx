//! Shim management for vx tools using shimexe-core
//!
//! This module provides functionality to create and manage executable shims
//! that allow transparent version switching for tools.

#[cfg(test)]
use crate::with_executable_extension;
use crate::PathManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use shimexe_core::ShimConfig as ShimexeConfig;
use std::path::{Path, PathBuf};

/// Configuration for a tool shim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimConfig {
    /// Path to the actual executable
    pub path: PathBuf,
    /// Additional arguments to pass to the executable
    pub args: Vec<String>,
    /// Working directory for the executable
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set
    pub env: std::collections::HashMap<String, String>,
    /// Signal handling configuration
    pub signal_handling: SignalHandling,
}

/// Signal handling configuration for shims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalHandling {
    /// Whether to forward signals to the child process
    pub forward_signals: bool,
    /// Whether to kill child process on exit
    pub kill_on_exit: bool,
}

impl Default for SignalHandling {
    fn default() -> Self {
        Self {
            forward_signals: true,
            kill_on_exit: true,
        }
    }
}

impl Default for ShimConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            args: Vec::new(),
            working_dir: None,
            env: std::collections::HashMap::new(),
            signal_handling: SignalHandling::default(),
        }
    }
}

/// Manager for creating and managing tool shims
pub struct ShimManager {
    path_manager: PathManager,
}

impl ShimManager {
    /// Create a new ShimManager
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }

    /// Create a shim for a tool pointing to a specific version
    pub async fn create_tool_shim(
        &self,
        tool_name: &str,
        target_executable: &Path,
        version: &str,
    ) -> Result<()> {
        // Create shimexe-core compatible configuration
        let shimexe_config =
            ShimexeConfig::new(tool_name, target_executable.to_string_lossy().to_string());

        // Write shim configuration file
        let config_path = self.path_manager.tool_current_shim_config_path(tool_name);

        // Ensure the directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        shimexe_config.to_file(&config_path)?;

        println!(
            "Created shim config for {} (v{}) -> {} at {}",
            tool_name,
            version,
            target_executable.display(),
            config_path.display()
        );

        Ok(())
    }

    /// Switch a tool to a different version by updating the shim
    pub async fn switch_tool_version(&self, tool_name: &str, version: &str) -> Result<()> {
        // Get the target executable path for the new version
        let target_executable = self.path_manager.tool_executable_path(tool_name, version);

        if !target_executable.exists() {
            return Err(anyhow::anyhow!(
                "Version {} of {} is not installed at {}",
                version,
                tool_name,
                target_executable.display()
            ));
        }

        // Create/update the shim
        self.create_tool_shim(tool_name, &target_executable, version)
            .await?;

        Ok(())
    }

    /// Get the current version of a tool (if any)
    pub fn get_current_version(&self, tool_name: &str) -> Result<Option<String>> {
        let config_path = self.path_manager.tool_current_shim_config_path(tool_name);

        if !config_path.exists() {
            return Ok(None);
        }

        // Load shimexe-core config
        let shimexe_config = ShimexeConfig::from_file(&config_path)?;
        let target_path = shimexe_config.get_executable_path()?;

        // Extract version from the target path
        // Expected format: ~/.vx/tools/<tool>/<version>/...
        // We need to find the version directory in the path
        let path_components: Vec<_> = target_path.components().collect();

        // Look for the pattern: tools/<tool>/<version>
        for (i, component) in path_components.iter().enumerate() {
            if let Some(name) = component.as_os_str().to_str() {
                if name == "tools" && i + 2 < path_components.len() {
                    // Check if the next component is our tool name OR a parent tool name
                    if let Some(tool_component) = path_components.get(i + 1) {
                        if let Some(tool_name_in_path) = tool_component.as_os_str().to_str() {
                            if tool_name_in_path == tool_name {
                                // Direct match: tools/<tool>/<version>
                                if let Some(version_component) = path_components.get(i + 2) {
                                    if let Some(version_str) =
                                        version_component.as_os_str().to_str()
                                    {
                                        return Ok(Some(version_str.to_string()));
                                    }
                                }
                            } else {
                                // Possible dependent tool: tools/<parent_tool>/<version>
                                // For dependent tools, we return the version from the parent tool's path
                                if let Some(version_component) = path_components.get(i + 2) {
                                    if let Some(version_str) =
                                        version_component.as_os_str().to_str()
                                    {
                                        // Return the version found in the path, regardless of tool name mismatch
                                        // This handles cases like uvx -> uv/0.5.10/uv.bat
                                        return Ok(Some(version_str.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Remove a tool's shim
    pub fn remove_tool_shim(&self, tool_name: &str) -> Result<()> {
        let current_dir = self.path_manager.tool_current_dir(tool_name);

        if current_dir.exists() {
            std::fs::remove_dir_all(&current_dir)?;
        }

        Ok(())
    }

    /// Check if a specific version of a tool is installed
    pub fn is_version_installed(&self, tool_name: &str, version: &str) -> bool {
        let target_executable = self.path_manager.tool_executable_path(tool_name, version);
        target_executable.exists()
    }

    /// Get the path manager (for advanced usage)
    pub fn path_manager(&self) -> &PathManager {
        &self.path_manager
    }

    /// List all tools that have shims
    pub fn list_shimmed_tools(&self) -> Result<Vec<String>> {
        let tools_dir = self.path_manager.tools_dir();
        let mut shimmed_tools = Vec::new();

        if !tools_dir.exists() {
            return Ok(shimmed_tools);
        }

        for entry in std::fs::read_dir(tools_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(tool_name) = entry.file_name().to_str() {
                    if self.path_manager.has_current_version(tool_name) {
                        shimmed_tools.push(tool_name.to_string());
                    }
                }
            }
        }

        shimmed_tools.sort();
        Ok(shimmed_tools)
    }

    /// Execute a tool using its shim configuration
    pub fn execute_tool_shim(&self, tool_name: &str, args: &[String]) -> Result<i32> {
        let config_path = self.path_manager.tool_current_shim_config_path(tool_name);

        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "No shim configuration found for tool: {}",
                tool_name
            ));
        }

        // Use ShimRunner to execute the tool
        let runner = shimexe_core::ShimRunner::from_file(&config_path)?;
        let exit_code = runner.execute(args)?;

        Ok(exit_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_shim_creation() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join(".vx");
        let path_manager = PathManager::with_base_dir(&base_dir).unwrap();
        let shim_manager = ShimManager::new(path_manager);

        // Create a fake target executable
        let version_dir = temp_dir.path().join(".vx/tools/node/18.17.0");
        std::fs::create_dir_all(&version_dir).unwrap();
        let target_exe = version_dir.join(with_executable_extension("node"));
        std::fs::write(&target_exe, "fake node executable").unwrap();

        // Create shim
        shim_manager
            .create_tool_shim("node", &target_exe, "18.17.0")
            .await
            .unwrap();

        // Verify shim config file was created
        let shim_config = base_dir.join("tools/node/current/node.shim.toml");
        assert!(shim_config.exists());

        // Verify current version detection
        let current_version = shim_manager.get_current_version("node").unwrap();
        assert_eq!(current_version, Some("18.17.0".to_string()));
    }
}
