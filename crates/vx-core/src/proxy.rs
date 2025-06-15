//! Transparent proxy system for vx
//!
//! This module provides the core transparent proxy functionality that allows
//! users to run tools through vx without explicit activation or PATH manipulation.
//!
//! The proxy system:
//! - Automatically detects project configuration (.vx.toml)
//! - Resolves the correct tool version for the current context
//! - Ensures tools are installed (with auto-install if enabled)
//! - Transparently executes the tool with the correct version

use crate::{config_figment::FigmentConfigManager, PluginRegistry, Result, VenvManager, VxError};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Transparent proxy for tool execution
pub struct ToolProxy {
    /// Virtual environment manager for project context
    venv_manager: VenvManager,
    /// Plugin registry for tool resolution
    plugin_registry: PluginRegistry,
    /// Configuration manager for project and global settings
    config_manager: FigmentConfigManager,
}

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ProxyContext {
    /// Tool name being executed
    pub tool_name: String,
    /// Arguments passed to the tool
    pub args: Vec<String>,
    /// Current working directory
    pub working_dir: PathBuf,
    /// Environment variables
    pub env_vars: std::collections::HashMap<String, String>,
}

impl ToolProxy {
    /// Create a new tool proxy
    pub fn new() -> Result<Self> {
        let venv_manager = VenvManager::new()?;
        let plugin_registry = PluginRegistry::new();
        let config_manager = FigmentConfigManager::new()?;

        Ok(Self {
            venv_manager,
            plugin_registry,
            config_manager,
        })
    }

    /// Execute a tool transparently through the proxy
    pub async fn execute_tool(&self, tool_name: &str, args: &[String]) -> Result<i32> {
        // Create execution context
        let context = ProxyContext {
            tool_name: tool_name.to_string(),
            args: args.to_vec(),
            working_dir: env::current_dir().map_err(|e| VxError::Other {
                message: format!("Failed to get current directory: {}", e),
            })?,
            env_vars: env::vars().collect(),
        };

        // Resolve the tool executable path
        let executable_path = self.resolve_tool_executable(&context).await?;

        // Execute the tool
        self.execute_with_path(&executable_path, &context).await
    }

    /// Resolve the executable path for a tool in the current context
    async fn resolve_tool_executable(&self, context: &ProxyContext) -> Result<PathBuf> {
        // First, try to ensure the tool is available through the venv manager
        // This handles project-specific version resolution and auto-installation
        match self
            .venv_manager
            .ensure_tool_available(&context.tool_name)
            .await
        {
            Ok(path) => return Ok(path),
            Err(_) => {
                // If venv manager fails, try to find the tool through plugins
                // This is a fallback for tools not managed by vx
            }
        }

        // Try to find the tool through the plugin registry
        if let Some(tool) = self.plugin_registry.get_tool(&context.tool_name) {
            // Check if any version is installed
            let installed_versions = tool.get_installed_versions().await?;
            if let Some(latest_version) = installed_versions.first() {
                let install_dir = tool.get_version_install_dir(latest_version);
                // Use VxEnvironment to find the executable
                let env = crate::VxEnvironment::new()?;
                return env.find_executable_in_dir(&install_dir, &context.tool_name);
            } else {
                // No version installed, try auto-installation
                return self
                    .auto_install_tool(&context.tool_name, tool.as_ref())
                    .await;
            }
        }

        // Last resort: check system PATH
        if let Ok(path) = which::which(&context.tool_name) {
            return Ok(path);
        }

        Err(VxError::Other {
            message: format!(
                "Tool '{}' not found. Install it with 'vx install {}' or ensure it's available in your PATH.",
                context.tool_name, context.tool_name
            ),
        })
    }

    /// Execute a tool with the resolved executable path
    async fn execute_with_path(
        &self,
        executable_path: &PathBuf,
        context: &ProxyContext,
    ) -> Result<i32> {
        let mut command = Command::new(executable_path);

        // Add arguments
        command.args(&context.args);

        // Set working directory
        command.current_dir(&context.working_dir);

        // Set environment variables
        for (key, value) in &context.env_vars {
            command.env(key, value);
        }

        // Configure stdio to inherit from parent process
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        // Execute the command
        let mut child = command.spawn().map_err(|e| VxError::Other {
            message: format!("Failed to execute tool '{}': {}", context.tool_name, e),
        })?;

        // Wait for the process to complete
        let status = child.wait().map_err(|e| VxError::Other {
            message: format!("Failed to wait for tool '{}': {}", context.tool_name, e),
        })?;

        // Return the exit code
        Ok(status.code().unwrap_or(-1))
    }

    /// Auto-install a tool if auto-installation is enabled
    async fn auto_install_tool(
        &self,
        tool_name: &str,
        tool: &dyn crate::plugin::VxTool,
    ) -> Result<PathBuf> {
        // Check if auto-installation is enabled from configuration
        let auto_install_enabled = self.config_manager.config().defaults.auto_install;

        if !auto_install_enabled {
            return Err(VxError::Other {
                message: format!(
                    "Tool '{}' is not installed and auto-installation is disabled. Install it with 'vx install {}'.",
                    tool_name, tool_name
                ),
            });
        }

        // Get the latest available version
        let available_versions = tool.fetch_versions(false).await?;
        let latest_version = available_versions.first().ok_or_else(|| VxError::Other {
            message: format!("No versions available for tool '{}'", tool_name),
        })?;

        // Install the latest version
        tool.install_version(&latest_version.version, false).await?;

        // Get the installation directory and find the executable
        let install_dir = tool.get_version_install_dir(&latest_version.version);
        let env = crate::VxEnvironment::new()?;
        env.find_executable_in_dir(&install_dir, tool_name)
    }

    /// Check if a tool is available (installed or in PATH)
    pub async fn is_tool_available(&self, tool_name: &str) -> bool {
        // Check through venv manager first
        if self
            .venv_manager
            .ensure_tool_available(tool_name)
            .await
            .is_ok()
        {
            return true;
        }

        // Check through plugin registry
        if let Some(tool) = self.plugin_registry.get_tool(tool_name) {
            if let Ok(versions) = tool.get_installed_versions().await {
                if !versions.is_empty() {
                    return true;
                }
            }
        }

        // Check system PATH
        which::which(tool_name).is_ok()
    }

    /// Get the version of a tool that would be used in the current context
    pub async fn get_effective_version(&self, tool_name: &str) -> Result<String> {
        // Try to get project-specific version from configuration first
        if let Some(version) = self.config_manager.get_project_tool_version(tool_name) {
            return Ok(version);
        }

        // Fallback to venv manager
        if let Ok(Some(version)) = self.venv_manager.get_project_tool_version(tool_name).await {
            return Ok(version);
        }

        // Try to get version from installed tools
        if let Some(tool) = self.plugin_registry.get_tool(tool_name) {
            let versions = tool.get_installed_versions().await?;
            if let Some(latest) = versions.first() {
                return Ok(latest.clone());
            }
        }

        // Try to get version from system tool
        if let Ok(path) = which::which(tool_name) {
            // Try to execute tool --version to get version
            if let Ok(output) = Command::new(&path).arg("--version").output() {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    // Extract version from output (this is a simplified approach)
                    if let Some(line) = version_output.lines().next() {
                        return Ok(line.to_string());
                    }
                }
            }
        }

        Err(VxError::Other {
            message: format!("Could not determine version for tool '{}'", tool_name),
        })
    }

    /// Get the configuration manager
    pub fn config_manager(&self) -> &FigmentConfigManager {
        &self.config_manager
    }

    /// Validate the current configuration
    pub fn validate_config(&self) -> Result<Vec<String>> {
        self.config_manager.validate()
    }

    /// Initialize a new project configuration
    pub fn init_project_config(
        &self,
        tools: Option<std::collections::HashMap<String, String>>,
        interactive: bool,
    ) -> Result<()> {
        self.config_manager.init_project_config(tools, interactive)
    }

    /// Sync project configuration (install all required tools)
    pub async fn sync_project(&self, force: bool) -> Result<Vec<String>> {
        self.config_manager.sync_project(force).await
    }
}

impl Default for ToolProxy {
    fn default() -> Self {
        Self::new().expect("Failed to create ToolProxy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_proxy_creation() {
        let proxy = ToolProxy::new();
        assert!(proxy.is_ok());
    }

    #[tokio::test]
    async fn test_is_tool_available() {
        let proxy = ToolProxy::new().unwrap();

        // Test with a tool that should be available on most systems
        let available = proxy.is_tool_available("echo").await;
        // Note: This might fail on some systems, but it's a basic test
        // In a real test environment, we'd mock the dependencies
        println!("Echo available: {}", available);
    }

    #[tokio::test]
    async fn test_auto_install_functionality() {
        let proxy = ToolProxy::new().unwrap();

        // Test that auto-install logic is properly integrated
        // This test verifies the method exists and can be called
        // In a real environment, this would test with a mock tool

        // For now, just test that the proxy can be created and basic methods work
        assert!(proxy.plugin_registry.get_tool("nonexistent").is_none());

        // Test effective version retrieval for system tools
        if let Ok(version) = proxy.get_effective_version("echo").await {
            println!("Echo version: {}", version);
        }
    }

    #[tokio::test]
    async fn test_config_management() {
        let proxy = ToolProxy::new().unwrap();

        // Test configuration validation
        let validation_result = proxy.validate_config();
        assert!(validation_result.is_ok());

        // Test configuration access
        let config = proxy.config_manager().config();
        assert!(config.defaults.auto_install); // Should be true by default

        // Test project tool version retrieval
        let version = proxy
            .config_manager()
            .get_project_tool_version("nonexistent");
        assert!(version.is_none()); // Should be None for non-existent tools

        println!("Configuration management tests passed");
    }

    #[tokio::test]
    async fn test_proxy_context_creation() {
        let context = ProxyContext {
            tool_name: "test-tool".to_string(),
            args: vec!["--version".to_string()],
            working_dir: std::env::current_dir().unwrap(),
            env_vars: std::env::vars().collect(),
        };

        assert_eq!(context.tool_name, "test-tool");
        assert_eq!(context.args, vec!["--version"]);
        assert!(!context.env_vars.is_empty());
    }

    #[tokio::test]
    async fn test_proxy_initialization() {
        let proxy = ToolProxy::new();
        assert!(proxy.is_ok(), "ToolProxy creation should succeed");

        if let Ok(proxy) = proxy {
            // Test that all components are properly initialized
            let config = proxy.config_manager().config();
            assert!(config.defaults.auto_install); // Default should be true

            // Test validation
            let validation = proxy.validate_config();
            assert!(validation.is_ok(), "Config validation should succeed");
        }
    }

    #[tokio::test]
    async fn test_effective_version_resolution() {
        let proxy = ToolProxy::new().unwrap();

        // Test with non-existent tool
        let result = proxy.get_effective_version("nonexistent-tool").await;
        assert!(result.is_err(), "Should fail for non-existent tool");

        // Test with system tool (if available)
        if let Ok(version) = proxy.get_effective_version("echo").await {
            assert!(!version.is_empty(), "Version should not be empty");
        }
    }

    #[tokio::test]
    async fn test_tool_availability_check() {
        let proxy = ToolProxy::new().unwrap();

        // Test with system tool
        let available = proxy.is_tool_available("echo").await;
        // This might be true or false depending on the system, but should not panic
        println!("Echo available: {}", available);

        // Test with definitely non-existent tool
        let not_available = proxy
            .is_tool_available("definitely-nonexistent-tool-12345")
            .await;
        assert!(!not_available, "Non-existent tool should not be available");
    }

    #[tokio::test]
    async fn test_config_integration() {
        let proxy = ToolProxy::new().unwrap();

        // Test that configuration is properly integrated
        let config = proxy.config_manager().config();

        // Test default values
        assert!(config.defaults.auto_install);
        assert!(!config.defaults.default_registry.is_empty());

        // Test tool configuration access
        let tool_config = proxy.config_manager().get_tool_config("nonexistent");
        assert!(tool_config.is_none());
    }
}
