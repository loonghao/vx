use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Plugin trait that all tool plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Check if the tool is installed
    fn is_installed(&self) -> Result<bool>;

    /// Get installed version
    fn get_installed_version(&self) -> Result<Option<String>>;

    /// Get latest available version
    fn get_latest_version(&self) -> Result<String>;

    /// Install a specific version
    fn install(&self, version: &str, install_dir: &PathBuf) -> Result<InstallResult>;

    /// Uninstall a specific version
    fn uninstall(&self, version: &str, install_dir: &PathBuf) -> Result<()>;

    /// Get executable path for a version
    fn get_executable_path(&self, version: &str, install_dir: &PathBuf) -> PathBuf;

    /// Validate installation
    fn validate_installation(&self, install_dir: &PathBuf) -> Result<bool>;

    /// Get tool-specific commands
    fn get_commands(&self) -> Vec<PluginCommand>;

    /// Execute a tool-specific command
    fn execute_command(&self, command: &str, args: &[String]) -> Result<i32>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub keywords: Vec<String>,
    pub categories: Vec<PluginCategory>,
    pub supported_platforms: Vec<Platform>,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCategory {
    Language,       // Programming languages (Go, Rust, Python)
    Runtime,        // Runtime environments (Node.js, JVM)
    PackageManager, // Package managers (npm, pip, cargo)
    BuildTool,      // Build tools (make, cmake, gradle)
    VersionControl, // Version control (git, svn)
    Database,       // Databases (postgres, mysql, redis)
    Cloud,          // Cloud tools (aws-cli, gcloud, kubectl)
    DevOps,         // DevOps tools (docker, terraform, ansible)
    Editor,         // Editors and IDEs (vim, vscode)
    Utility,        // General utilities
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    FreeBSD,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub executable_path: PathBuf,
    pub installed_files: Vec<PathBuf>,
    pub size: u64,
    pub checksum: Option<String>,
}

/// Plugin registry for managing all plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_configs: HashMap<String, PluginConfig>,
    enabled_plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub auto_update: bool,
    pub preferred_version: Option<String>,
    pub install_method: Option<String>,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_update: false,
            preferred_version: None,
            install_method: None,
            custom_settings: HashMap::new(),
        }
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_configs: HashMap::new(),
            enabled_plugins: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let name = plugin.metadata().name.clone();

        // Check for conflicts
        for conflict in &plugin.metadata().conflicts {
            if self.plugins.contains_key(conflict) {
                return Err(anyhow::anyhow!(
                    "Plugin {} conflicts with already registered plugin {}",
                    name,
                    conflict
                ));
            }
        }

        // Add default config if not exists
        if !self.plugin_configs.contains_key(&name) {
            self.plugin_configs
                .insert(name.clone(), PluginConfig::default());
        }

        // Register plugin
        self.plugins.insert(name.clone(), plugin);

        // Enable if configured
        if self.plugin_configs.get(&name).unwrap().enabled {
            self.enabled_plugins.push(name.clone());
        }

        println!("📦 Registered plugin: {}", name);
        Ok(())
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }

    /// List enabled plugins
    pub fn list_enabled_plugins(&self) -> Vec<&dyn Plugin> {
        self.enabled_plugins
            .iter()
            .filter_map(|name| self.plugins.get(name).map(|p| p.as_ref()))
            .collect()
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if !self.plugins.contains_key(name) {
            return Err(anyhow::anyhow!("Plugin {} not found", name));
        }

        if !self.enabled_plugins.contains(&name.to_string()) {
            self.enabled_plugins.push(name.to_string());
        }

        if let Some(config) = self.plugin_configs.get_mut(name) {
            config.enabled = true;
        }

        println!("✅ Enabled plugin: {}", name);
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        self.enabled_plugins.retain(|p| p != name);

        if let Some(config) = self.plugin_configs.get_mut(name) {
            config.enabled = false;
        }

        println!("❌ Disabled plugin: {}", name);
        Ok(())
    }

    /// Get plugin configuration
    pub fn get_plugin_config(&self, name: &str) -> Option<&PluginConfig> {
        self.plugin_configs.get(name)
    }

    /// Update plugin configuration
    pub fn update_plugin_config(&mut self, name: &str, config: PluginConfig) -> Result<()> {
        if !self.plugins.contains_key(name) {
            return Err(anyhow::anyhow!("Plugin {} not found", name));
        }

        self.plugin_configs.insert(name.to_string(), config);
        Ok(())
    }

    /// Find plugins by category
    pub fn find_plugins_by_category(&self, category: &PluginCategory) -> Vec<&dyn Plugin> {
        self.plugins
            .values()
            .filter(|plugin| plugin.metadata().categories.contains(category))
            .map(|p| p.as_ref())
            .collect()
    }

    /// Search plugins by keyword
    pub fn search_plugins(&self, keyword: &str) -> Vec<&dyn Plugin> {
        let keyword = keyword.to_lowercase();
        self.plugins
            .values()
            .filter(|plugin| {
                let metadata = plugin.metadata();
                metadata.name.to_lowercase().contains(&keyword)
                    || metadata.description.to_lowercase().contains(&keyword)
                    || metadata
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&keyword))
            })
            .map(|p| p.as_ref())
            .collect()
    }

    /// Get plugin statistics
    pub fn get_stats(&self) -> PluginStats {
        let total_plugins = self.plugins.len();
        let enabled_plugins = self.enabled_plugins.len();
        let categories: HashMap<PluginCategory, usize> = self
            .plugins
            .values()
            .flat_map(|p| &p.metadata().categories)
            .fold(HashMap::new(), |mut acc, cat| {
                *acc.entry(cat.clone()).or_insert(0) += 1;
                acc
            });

        PluginStats {
            total_plugins,
            enabled_plugins,
            disabled_plugins: total_plugins - enabled_plugins,
            categories,
        }
    }
}

#[derive(Debug)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub enabled_plugins: usize,
    pub disabled_plugins: usize,
    pub categories: HashMap<PluginCategory, usize>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
