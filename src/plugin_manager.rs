use crate::plugin::{Plugin, PluginCategory, PluginRegistry};
use crate::plugins;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerConfig {
    pub auto_discover: bool,
    pub plugin_directories: Vec<PathBuf>,
    pub disabled_plugins: Vec<String>,
    pub plugin_settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        let vx_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".vx");

        Self {
            auto_discover: true,
            plugin_directories: vec![vx_dir.join("plugins"), PathBuf::from("./plugins")],
            disabled_plugins: vec![],
            plugin_settings: HashMap::new(),
        }
    }
}

pub struct PluginManager {
    registry: PluginRegistry,
    config: PluginManagerConfig,
    config_path: PathBuf,
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let vx_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".vx");

        fs::create_dir_all(&vx_dir)?;

        let config_path = vx_dir.join("plugin_config.json");
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            PluginManagerConfig::default()
        };

        let mut manager = Self {
            registry: PluginRegistry::new(),
            config,
            config_path,
        };

        // Register built-in plugins
        manager.register_builtin_plugins()?;

        // Auto-discover external plugins if enabled
        if manager.config.auto_discover {
            manager.discover_plugins()?;
        }

        Ok(manager)
    }

    /// Register all built-in plugins
    fn register_builtin_plugins(&mut self) -> Result<()> {
        plugins::register_builtin_plugins(&mut self.registry)?;
        Ok(())
    }

    /// Discover and load external plugins
    fn discover_plugins(&mut self) -> Result<()> {
        let plugin_directories = self.config.plugin_directories.clone();
        for plugin_dir in &plugin_directories {
            if plugin_dir.exists() {
                self.discover_plugins_in_directory(plugin_dir)?;
            }
        }
        Ok(())
    }

    /// Discover plugins in a specific directory
    fn discover_plugins_in_directory(&mut self, dir: &PathBuf) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Look for plugin manifest
                let manifest_path = path.join("plugin.json");
                if manifest_path.exists() {
                    match self.load_external_plugin(&manifest_path) {
                        Ok(_) => println!("📦 Loaded external plugin from {}", path.display()),
                        Err(e) => {
                            eprintln!("⚠️  Failed to load plugin from {}: {}", path.display(), e)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Load an external plugin from manifest
    fn load_external_plugin(&mut self, manifest_path: &PathBuf) -> Result<()> {
        // For now, just log that we found a plugin manifest
        // In a full implementation, this would load dynamic libraries or scripts
        println!("📄 Found plugin manifest: {}", manifest_path.display());
        Ok(())
    }

    /// Save configuration
    pub fn save_config(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.registry.get_plugin(name)
    }

    /// List all plugins
    pub fn list_plugins(&self) -> Vec<&dyn Plugin> {
        self.registry.list_plugins()
    }

    /// List enabled plugins
    pub fn list_enabled_plugins(&self) -> Vec<&dyn Plugin> {
        self.registry.list_enabled_plugins()
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if self.config.disabled_plugins.contains(&name.to_string()) {
            self.config.disabled_plugins.retain(|p| p != name);
        }
        self.registry.enable_plugin(name)?;
        self.save_config()?;
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        if !self.config.disabled_plugins.contains(&name.to_string()) {
            self.config.disabled_plugins.push(name.to_string());
        }
        self.registry.disable_plugin(name)?;
        self.save_config()?;
        Ok(())
    }

    /// Search plugins
    pub fn search_plugins(&self, query: &str) -> Vec<&dyn Plugin> {
        self.registry.search_plugins(query)
    }

    /// Find plugins by category
    pub fn find_plugins_by_category(&self, category: &PluginCategory) -> Vec<&dyn Plugin> {
        self.registry.find_plugins_by_category(category)
    }

    /// Get plugin statistics
    pub fn get_stats(&self) -> crate::plugin::PluginStats {
        self.registry.get_stats()
    }

    /// Install a tool using its plugin
    pub async fn install_tool(&self, tool_name: &str, version: &str) -> Result<PathBuf> {
        let plugin = self
            .get_plugin(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", tool_name))?;

        let install_dir = self.get_tool_install_dir(tool_name, version);
        fs::create_dir_all(&install_dir)?;

        let result = plugin.install(version, &install_dir).await?;
        Ok(result.executable_path)
    }

    /// Uninstall a tool using its plugin
    pub async fn uninstall_tool(&self, tool_name: &str, version: &str) -> Result<()> {
        let plugin = self
            .get_plugin(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", tool_name))?;

        let install_dir = self.get_tool_install_dir(tool_name, version);
        plugin.uninstall(version, &install_dir).await?;
        Ok(())
    }

    /// Execute a tool command using its plugin
    pub async fn execute_tool_command(
        &self,
        tool_name: &str,
        command: &str,
        args: &[String],
    ) -> Result<i32> {
        let plugin = self
            .get_plugin(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", tool_name))?;

        plugin.execute_command(command, args).await
    }

    /// Get tool installation directory
    fn get_tool_install_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".vx")
            .join("tools")
            .join(tool_name)
            .join(version)
    }

    /// Check if a tool is installed
    pub async fn is_tool_installed(&self, tool_name: &str) -> Result<bool> {
        let plugin = self
            .get_plugin(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", tool_name))?;

        plugin.is_installed().await
    }

    /// Get installed version of a tool
    pub async fn get_tool_version(&self, tool_name: &str) -> Result<Option<String>> {
        let plugin = self
            .get_plugin(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", tool_name))?;

        plugin.get_installed_version().await
    }

    /// Get latest version of a tool
    pub async fn get_latest_tool_version(&self, tool_name: &str) -> Result<String> {
        let plugin = self
            .get_plugin(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", tool_name))?;

        plugin.get_latest_version().await
    }

    /// List available commands for a tool
    pub fn get_tool_commands(&self, tool_name: &str) -> Result<Vec<crate::plugin::PluginCommand>> {
        let plugin = self
            .get_plugin(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", tool_name))?;

        Ok(plugin.get_commands())
    }

    /// Show plugin information
    pub fn show_plugin_info(&self, name: &str) -> Result<()> {
        let plugin = self
            .get_plugin(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", name))?;

        let metadata = plugin.metadata();

        println!("📦 Plugin: {}", metadata.name);
        println!("📝 Description: {}", metadata.description);
        println!("👤 Author: {}", metadata.author);
        println!("📄 License: {}", metadata.license);
        println!(
            "🔗 Homepage: {}",
            metadata.homepage.as_deref().unwrap_or("N/A")
        );
        println!(
            "📂 Repository: {}",
            metadata.repository.as_deref().unwrap_or("N/A")
        );
        println!("🏷️  Keywords: {}", metadata.keywords.join(", "));
        println!("📋 Categories: {:?}", metadata.categories);
        println!("💻 Platforms: {:?}", metadata.supported_platforms);

        if !metadata.dependencies.is_empty() {
            println!("📦 Dependencies: {}", metadata.dependencies.join(", "));
        }

        if !metadata.conflicts.is_empty() {
            println!("⚠️  Conflicts: {}", metadata.conflicts.join(", "));
        }

        let commands = plugin.get_commands();
        if !commands.is_empty() {
            println!("\n🔧 Available commands:");
            for cmd in commands {
                println!("  • {} - {}", cmd.name, cmd.description);
            }
        }

        Ok(())
    }
}
