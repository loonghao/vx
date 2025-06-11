use crate::package_manager::{Package, PackageManager};
use crate::tool_manager::ToolManager;
use crate::ui::UI;
use anyhow::Result;

use std::process::{Command, Stdio};
use which::which;

pub struct Executor {
    tool_manager: Option<ToolManager>,
    package_manager: Option<PackageManager>,
}

impl Executor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            tool_manager: None,
            package_manager: None,
        })
    }

    /// Lazy initialization of tool manager
    fn ensure_tool_manager(&mut self) -> Result<&ToolManager> {
        if self.tool_manager.is_none() {
            self.tool_manager = Some(ToolManager::new().or_else(|_| ToolManager::minimal())?);
        }
        Ok(self.tool_manager.as_ref().unwrap())
    }

    /// Lazy initialization of package manager
    fn ensure_package_manager(&mut self) -> Result<&PackageManager> {
        if self.package_manager.is_none() {
            self.package_manager = Some(PackageManager::new()?);
        }
        Ok(self.package_manager.as_ref().unwrap())
    }

    /// Lazy initialization of package manager (mutable)
    fn ensure_package_manager_mut(&mut self) -> Result<&mut PackageManager> {
        if self.package_manager.is_none() {
            self.package_manager = Some(PackageManager::new()?);
        }
        Ok(self.package_manager.as_mut().unwrap())
    }

    /// Execute a tool with given arguments
    pub async fn execute(
        &mut self,
        tool_name: &str,
        args: &[String],
        use_system_path: bool,
    ) -> Result<i32> {
        if use_system_path {
            // When --use-system-path is specified, ONLY use system tools
            if let Ok(tool_path) = which(tool_name) {
                UI::info(&format!("Using {} (system installed)", tool_name));
                return self.run_command(&tool_path.to_string_lossy(), args).await;
            } else {
                return Err(anyhow::anyhow!(
                    "Tool '{}' not found in system PATH",
                    tool_name
                ));
            }
        }

        // Default behavior: prioritize vx-managed tools, with system fallback
        let tool_manager = self.ensure_tool_manager()?;

        // First, check if we have a tool for this
        if tool_manager.has_tool(tool_name) {
            // Try to execute using tool manager (handles auto-installation)
            match tool_manager.execute_tool(tool_name, args) {
                Ok(exit_code) => return Ok(exit_code),
                Err(e) => {
                    UI::warning(&format!("vx-managed execution failed: {}", e));
                    // Fallback to system tool if available
                    if let Ok(tool_path) = which(tool_name) {
                        UI::info(&format!("Using {} (system installed)", tool_name));
                        return self.run_command(&tool_path.to_string_lossy(), args).await;
                    }
                }
            }
        } else {
            // Tool is not supported, check system
            if let Ok(tool_path) = which(tool_name) {
                UI::info(&format!("Using {} (system installed)", tool_name));
                return self.run_command(&tool_path.to_string_lossy(), args).await;
            }
        }

        // Provide helpful error message with available tools
        let available_tools = self.list_tools()?;
        let mut error_msg = format!("Tool '{}' not found.", tool_name);

        if !available_tools.is_empty() {
            error_msg.push_str("\n\nSupported tools:");
            for tool in &available_tools {
                error_msg.push_str(&format!("\n  * {}", tool));
            }

            // Check if this tool is supported
            if available_tools.contains(&tool_name.to_string()) {
                error_msg.push_str(&format!(
                    "\n\nTo install {}, run: vx install {}",
                    tool_name, tool_name
                ));
            }
        }

        Err(anyhow::anyhow!(error_msg))
    }

    /// List available tools
    pub fn list_tools(&mut self) -> Result<Vec<String>> {
        let tool_manager = self.ensure_tool_manager()?;
        Ok(tool_manager.get_tool_names())
    }

    /// Install a tool with the specified version using tool manager
    pub async fn install_tool(
        &mut self,
        tool_name: &str,
        _version: &str,
    ) -> Result<std::path::PathBuf> {
        // Use tool manager for installation
        let tool_manager = self.ensure_tool_manager()?;
        tool_manager.install_tool(tool_name)?;

        // Return a dummy path for now - this should be improved
        Ok(std::path::PathBuf::from(format!("/tmp/{}", tool_name)))
    }

    /// Switch to a different version of a tool
    pub fn switch_version(&mut self, tool_name: &str, version: &str) -> Result<()> {
        let package_manager = self.ensure_package_manager_mut()?;
        package_manager.switch_version(tool_name, version)
    }

    /// Remove a specific version of a tool
    pub fn remove_version(&mut self, tool_name: &str, version: &str) -> Result<()> {
        let package_manager = self.ensure_package_manager_mut()?;
        package_manager.remove_version(tool_name, version)
    }

    /// Remove all versions of a tool
    pub fn remove_tool(&mut self, tool_name: &str) -> Result<()> {
        let package_manager = self.ensure_package_manager_mut()?;
        let versions: Vec<String> = package_manager
            .list_versions(tool_name)
            .iter()
            .map(|pkg| pkg.version.clone())
            .collect();

        for version in versions {
            package_manager.remove_version(tool_name, &version)?;
        }

        UI::success(&format!("Removed all versions of {}", tool_name));
        Ok(())
    }

    /// Clean up orphaned packages
    pub fn cleanup(&mut self) -> Result<()> {
        let package_manager = self.ensure_package_manager_mut()?;
        package_manager.cleanup()
    }

    /// Get package statistics
    pub fn get_stats(&mut self) -> Result<crate::package_manager::PackageStats> {
        let package_manager = self.ensure_package_manager()?;
        Ok(package_manager.get_stats())
    }

    /// List all installed packages
    pub fn list_installed_packages(&mut self) -> Result<Vec<&Package>> {
        let package_manager = self.ensure_package_manager()?;
        Ok(package_manager.list_packages())
    }

    /// Check for updates
    pub async fn check_updates(
        &self,
        tool_name: Option<&str>,
    ) -> Result<Vec<(String, String, String)>> {
        // This would integrate with the version manager to check for newer versions
        // For now, return empty - this would be implemented with actual version checking
        let _ = tool_name;
        Ok(vec![])
    }

    /// Get package manager reference
    pub fn get_package_manager(&mut self) -> Result<&PackageManager> {
        self.ensure_package_manager()
    }

    /// Run the actual command (synchronous to avoid runtime nesting)
    fn run_command_sync(&self, tool_path: &str, args: &[String]) -> Result<i32> {
        UI::show_command_execution(tool_path, args);

        let mut command = Command::new(tool_path);
        command.args(args);
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        let status = command.status()?;
        Ok(status.code().unwrap_or(1))
    }

    /// Run the actual command (async wrapper)
    async fn run_command(&self, tool_path: &str, args: &[String]) -> Result<i32> {
        // Use blocking call to avoid runtime nesting
        self.run_command_sync(tool_path, args)
    }

    /// Find vx-managed tool executable path
    #[allow(dead_code)]
    fn find_vx_managed_tool(&mut self, tool_name: &str) -> Result<std::path::PathBuf> {
        let package_manager = self.ensure_package_manager()?;

        // Check if tool is installed via vx package manager
        if let Some(active_package) = package_manager.get_active_version(tool_name) {
            if active_package.executable_path.exists() {
                return Ok(active_package.executable_path.clone());
            }
        }

        Err(anyhow::anyhow!(
            "Tool '{}' not found in vx-managed installations",
            tool_name
        ))
    }

    /// Check tool status using tool manager
    pub async fn check_tool_status(&mut self, tool_name: &str) -> Result<()> {
        let tool_manager = self.ensure_tool_manager()?;

        if tool_manager.has_tool(tool_name) {
            let spinner = UI::new_spinner(&format!("Checking {} status...", tool_name));
            let status = tool_manager.check_tool_status(tool_name)?;
            spinner.finish_and_clear();

            if status.installed {
                if let Some(version) = &status.current_version {
                    UI::success(&format!("{} {} is installed", tool_name, version));
                } else {
                    UI::success(&format!("{} is installed", tool_name));
                }
            } else {
                UI::error(&format!("{} is not installed", tool_name));
                if status.supports_auto_install {
                    UI::hint(&format!("Run 'vx install {}' to install it", tool_name));
                }
            }
        } else {
            return Err(anyhow::anyhow!("Unsupported tool: {}", tool_name));
        }

        Ok(())
    }
}

/// Standalone function for executing tools (used by CLI)
pub async fn execute_tool(tool_name: &str, args: &[String], use_system_path: bool) -> Result<i32> {
    let mut executor = Executor::new()?;
    executor.execute(tool_name, args, use_system_path).await
}
