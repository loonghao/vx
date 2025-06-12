//! Four-layer tool architecture with environment isolation and configuration management

use crate::{Result, VersionInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Core async tool interface - all tools are environment-isolated with version control
#[async_trait::async_trait]
pub trait AsyncTool: Send + Sync {
    /// Get the name of this tool
    fn name(&self) -> &str;

    /// Get tool description
    fn description(&self) -> &str {
        "A development tool"
    }

    /// Get supported aliases for this tool
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Fetch available versions from the tool's official source
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Install a specific version of the tool
    async fn install_version(&self, version: &str) -> Result<PathBuf>;

    /// Get all installed versions
    async fn get_installed_versions(&self) -> Result<Vec<String>>;

    /// Uninstall a specific version
    async fn uninstall_version(&self, version: &str) -> Result<()>;

    /// Get download URL for a specific version
    async fn get_download_url(&self, version: &str) -> Result<Option<String>>;

    /// Get the executable path for a specific version (vx-managed, environment-isolated)
    fn get_executable_path(&self, version: &str) -> PathBuf;

    /// Execute the tool with given context
    async fn execute(&self, context: &ToolContext) -> Result<i32>;

    /// Check if a specific version is installed
    async fn is_version_installed(&self, version: &str) -> Result<bool> {
        let installed_versions = self.get_installed_versions().await?;
        Ok(installed_versions.contains(&version.to_string()))
    }

    /// Get the base installation directory for this tool
    fn get_base_install_dir(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".vx")
            .join("tools")
            .join(self.name())
    }

    /// Get installation directory for a specific version
    fn get_version_install_dir(&self, version: &str) -> PathBuf {
        self.get_base_install_dir().join(version)
    }
}

/// Virtual environment management interface
pub trait Environment: Send + Sync {
    /// Get environment name
    fn name(&self) -> &str;

    /// Get tool version for this environment
    fn get_tool_version(&self, tool_name: &str) -> Option<String>;

    /// Set tool version for this environment
    fn set_tool_version(&mut self, tool_name: &str, version: String) -> Result<()>;

    /// Get environment directory path
    fn get_environment_path(&self) -> PathBuf;

    /// Activate this environment and return tool context
    fn activate(&self) -> Result<ToolContext>;

    /// List all tools configured in this environment
    fn list_tools(&self) -> Result<HashMap<String, String>>;
}

/// Configuration management interface
pub trait Configuration: Send + Sync {
    /// Get global tool version
    fn get_global_tool_version(&self, tool_name: &str) -> Option<String>;

    /// Set global tool version
    fn set_global_tool_version(&self, tool_name: &str, version: String) -> Result<()>;

    /// Get project-specific tool version
    fn get_project_tool_version(&self, tool_name: &str, project_path: &Path) -> Option<String>;

    /// Set project-specific tool version
    fn set_project_tool_version(
        &self,
        tool_name: &str,
        version: String,
        project_path: &Path,
    ) -> Result<()>;

    /// Get active environment name
    fn get_active_environment(&self) -> Option<String>;

    /// Set active environment
    fn set_active_environment(&self, env_name: Option<String>) -> Result<()>;

    /// Resolve tool version based on priority: CLI > Environment > Project > Global > Latest
    fn resolve_tool_version(
        &self,
        tool_name: &str,
        cli_version: Option<&str>,
        project_path: Option<&Path>,
    ) -> Option<String>;
}

/// Plugin interface - organizes related tools
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin description
    fn description(&self) -> &str {
        "A vx plugin"
    }

    /// Plugin version
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Get all tools provided by this plugin
    fn tools(&self) -> Vec<Box<dyn AsyncTool>>;

    /// Check if this plugin supports a specific tool
    fn supports_tool(&self, tool_name: &str) -> bool;
}

/// Tool execution context with environment isolation
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub tool_name: String,
    pub version: String,
    pub args: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub environment_variables: HashMap<String, String>,
    pub environment: Option<String>, // venv name
    pub use_system_path: bool,       // Whether to use system PATH or vx-managed tools
}

impl ToolContext {
    pub fn new(tool_name: String, version: String, args: Vec<String>) -> Self {
        Self {
            tool_name,
            version,
            args,
            working_directory: None,
            environment_variables: HashMap::new(),
            environment: None,
            use_system_path: false,
        }
    }
}

/// Tool execution result
#[derive(Debug)]
pub struct ToolExecutionResult {
    pub exit_code: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Information about a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub installed_versions: Vec<String>,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
}

/// Status of a tool in a specific environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub installed: bool,
    pub current_version: Option<String>,
    pub installed_versions: Vec<String>,
}

/// Synchronous tool interface for compatibility with legacy code
/// This trait provides a synchronous interface for tools that don't need async operations
pub trait Tool: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Get tool description
    fn description(&self) -> &str {
        "A development tool"
    }

    /// Get tool homepage URL
    fn homepage(&self) -> Option<&str> {
        None
    }

    /// Check if the tool is installed on the system
    fn is_installed(&self) -> Result<bool>;

    /// Get the currently installed version (if any)
    fn get_version(&self) -> Result<Option<String>>;

    /// Get the expected executable path for a given version
    fn get_executable_path(&self, version: &str, install_dir: &Path) -> PathBuf;

    /// Execute the tool with given arguments
    fn execute(&self, args: &[String]) -> Result<i32>;

    /// Check if tool supports auto-installation
    fn supports_auto_install(&self) -> bool {
        true
    }

    /// Get tool information
    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            description: self.description().to_string(),
            aliases: vec![],
            installed_versions: vec![],
            current_version: self.get_version().unwrap_or_default(),
            latest_version: None,
        }
    }
}
