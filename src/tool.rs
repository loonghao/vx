// Simplified tool system for vx
// Replaces the complex plugin system with a simple tool abstraction

use anyhow::Result;
use std::path::PathBuf;

/// Tool trait - simplified interface for managing development tools
pub trait Tool: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Check if the tool is installed on the system
    fn is_installed(&self) -> Result<bool>;

    /// Get the currently installed version (if any)
    fn get_version(&self) -> Result<Option<String>>;

    /// Get the expected executable path for a given version
    fn get_executable_path(&self, version: &str, install_dir: &std::path::Path) -> PathBuf;

    /// Execute the tool with given arguments
    fn execute(&self, args: &[String]) -> Result<i32>;

    /// Get tool description
    fn description(&self) -> &str {
        "A development tool"
    }

    /// Get tool homepage URL
    fn homepage(&self) -> Option<&str> {
        None
    }

    /// Check if tool supports auto-installation
    fn supports_auto_install(&self) -> bool {
        true
    }
}

/// Tool execution result
#[derive(Debug)]
pub struct ToolResult {
    pub exit_code: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Tool metadata for display purposes
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub installed: bool,
    pub version: Option<String>,
    pub supports_auto_install: bool,
}

impl ToolInfo {
    /// Create ToolInfo from a Tool implementation
    pub fn from_tool(tool: &dyn Tool) -> Self {
        let installed = tool.is_installed().unwrap_or(false);
        let version = if installed {
            tool.get_version().unwrap_or(None)
        } else {
            None
        };

        Self {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            homepage: tool.homepage().map(|s| s.to_string()),
            installed,
            version,
            supports_auto_install: tool.supports_auto_install(),
        }
    }
}
