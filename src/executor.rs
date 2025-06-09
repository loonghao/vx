
use crate::package_manager::{Package, PackageManager};
use crate::plugin_manager::PluginManager;
use anyhow::Result;

use std::process::{Command, Stdio};
use which::which;

pub struct Executor {
    plugin_manager: PluginManager,
    package_manager: PackageManager,
}

impl Executor {
    pub fn new() -> Result<Self> {
        let plugin_manager = PluginManager::new()?;
        let package_manager = PackageManager::new()?;

        Ok(Self {
            plugin_manager,
            package_manager,
        })
    }
    
    /// Execute a tool with given arguments
    pub async fn execute(&mut self, tool_name: &str, args: &[String]) -> Result<i32> {
        // Check if tool is supported by plugin system
        if let Some(plugin) = self.plugin_manager.get_plugin(tool_name) {
            // Use plugin to execute directly
            return plugin.execute_command("", args).await;
        }

        // Fallback: check if tool is available in system
        if let Ok(tool_path) = which(tool_name) {
            println!("ℹ️  Using {} (system installed)", tool_name);
            return self.run_command(&tool_path.to_string_lossy(), args).await;
        }

        Err(anyhow::anyhow!("Unsupported tool: {}", tool_name))
    }
    
    /// List available tools from plugins
    pub fn list_tools(&self) -> Vec<String> {
        self.plugin_manager
            .list_enabled_plugins()
            .iter()
            .map(|plugin| plugin.metadata().name.clone())
            .collect()
    }
    

    
    /// Install a tool with the specified version using plugin manager
    pub async fn install_tool(&mut self, tool_name: &str, version: &str) -> Result<std::path::PathBuf> {
        // Use plugin manager for installation
        self.plugin_manager.install_tool(tool_name, version).await
    }



    /// Switch to a different version of a tool
    pub fn switch_version(&mut self, tool_name: &str, version: &str) -> Result<()> {
        self.package_manager.switch_version(tool_name, version)
    }

    /// Remove a specific version of a tool
    pub fn remove_version(&mut self, tool_name: &str, version: &str) -> Result<()> {
        self.package_manager.remove_version(tool_name, version)
    }

    /// Remove all versions of a tool
    pub fn remove_tool(&mut self, tool_name: &str) -> Result<()> {
        let versions: Vec<String> = self.package_manager
            .list_versions(tool_name)
            .iter()
            .map(|pkg| pkg.version.clone())
            .collect();

        for version in versions {
            self.package_manager.remove_version(tool_name, &version)?;
        }

        println!("✅ Removed all versions of {}", tool_name);
        Ok(())
    }

    /// Clean up orphaned packages
    pub fn cleanup(&mut self) -> Result<()> {
        self.package_manager.cleanup()
    }

    /// Get package statistics
    pub fn get_stats(&self) -> crate::package_manager::PackageStats {
        self.package_manager.get_stats()
    }

    /// List all installed packages
    pub fn list_installed_packages(&self) -> Vec<&Package> {
        self.package_manager.list_packages()
    }

    /// Check for updates
    pub async fn check_updates(&self, tool_name: Option<&str>) -> Result<Vec<(String, String, String)>> {
        // This would integrate with the version manager to check for newer versions
        // For now, return empty - this would be implemented with actual version checking
        let _ = tool_name;
        Ok(vec![])
    }

    /// Get package manager reference
    pub fn get_package_manager(&self) -> &PackageManager {
        &self.package_manager
    }
    
    /// Run the actual command
    async fn run_command(&self, tool_path: &str, args: &[String]) -> Result<i32> {
        println!("🚀 Running: {} {}", tool_path, args.join(" "));
        
        let mut command = Command::new(tool_path);
        command.args(args);
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        
        let status = command.status()?;
        Ok(status.code().unwrap_or(1))
    }
    
    /// Check tool status using plugin manager
    pub async fn check_tool_status(&self, tool_name: &str) -> Result<()> {
        if let Some(_plugin) = self.plugin_manager.get_plugin(tool_name) {
            let is_installed = self.plugin_manager.is_tool_installed(tool_name).await?;

            println!("🔍 Checking {} status...", tool_name);

            if is_installed {
                if let Some(version) = self.plugin_manager.get_tool_version(tool_name).await? {
                    println!("✅ {} {} is installed", tool_name, version);
                } else {
                    println!("✅ {} is installed", tool_name);
                }
            } else {
                println!("❌ {} is not installed", tool_name);
                println!("💡 Run 'vx install {}' to install it", tool_name);
            }
        } else {
            return Err(anyhow::anyhow!("Unsupported tool: {}", tool_name));
        }

        Ok(())
    }
}
