// Figment-based configuration system for vx
// Leverages the excellent figment crate for layered configuration
// Supports reading from existing project configuration files

use anyhow::Result;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main vx configuration structure
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VxConfig {
    /// Global default settings
    pub defaults: DefaultConfig,
    /// Tool-specific configurations
    pub tools: HashMap<String, ToolConfig>,
    /// Registry configurations
    pub registries: HashMap<String, RegistryConfig>,
}

/// Default configuration settings
#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    /// Automatically install missing tools
    pub auto_install: bool,
    /// Check for updates periodically
    pub check_updates: bool,
    /// Update check interval
    pub update_interval: String,
    /// Default registry to use
    pub default_registry: String,
    /// Whether to fall back to builtin configuration
    pub fallback_to_builtin: bool,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            check_updates: true,
            update_interval: "24h".to_string(),
            default_registry: "official".to_string(),
            fallback_to_builtin: true,
        }
    }
}

/// Tool-specific configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolConfig {
    /// Preferred version (latest, lts, specific version)
    pub version: Option<String>,
    /// Installation method preference
    pub install_method: Option<String>,
    /// Registry to use for this tool
    pub registry: Option<String>,
    /// Custom download sources
    pub custom_sources: Option<HashMap<String, String>>,
}

/// Registry configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryConfig {
    /// Registry name
    pub name: String,
    /// Base URL for the registry
    pub base_url: String,
    /// API URL (optional)
    pub api_url: Option<String>,
    /// Authentication token (optional)
    pub auth_token: Option<String>,
    /// Registry priority (higher = more preferred)
    pub priority: i32,
    /// Whether this registry is enabled
    pub enabled: bool,
}

/// Project configuration detection result
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub config_file: PathBuf,
    pub tool_versions: HashMap<String, String>,
}

/// Supported project types
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Python,  // pyproject.toml
    Rust,    // Cargo.toml
    Node,    // package.json
    Go,      // go.mod
    Mixed,   // Multiple project types
    Unknown, // No recognized project files
}

/// Configuration manager using Figment
pub struct FigmentConfigManager {
    figment: Figment,
    config: VxConfig,
    project_info: Option<ProjectInfo>,
}

impl FigmentConfigManager {
    /// Create a new configuration manager with full layered configuration
    pub fn new() -> Result<Self> {
        let project_info = Self::detect_project_info()?;
        let figment = Self::build_figment(&project_info)?;
        let config = figment.extract()?;

        Ok(Self {
            figment,
            config,
            project_info,
        })
    }

    /// Create a minimal configuration manager (builtin defaults only)
    pub fn minimal() -> Result<Self> {
        let figment = Figment::from(Serialized::defaults(VxConfig::default()));
        let config = figment.extract()?;

        Ok(Self {
            figment,
            config,
            project_info: None,
        })
    }

    /// Detect project information and configuration files
    fn detect_project_info() -> Result<Option<ProjectInfo>> {
        let current_dir = std::env::current_dir()?;
        let mut detected_projects = Vec::new();
        let mut all_tool_versions = HashMap::new();

        // Check for Python project (pyproject.toml)
        let pyproject_path = current_dir.join("pyproject.toml");
        if pyproject_path.exists() {
            if let Ok(versions) = Self::parse_pyproject_toml(&pyproject_path) {
                detected_projects.push(ProjectType::Python);
                all_tool_versions.extend(versions);
            }
        }

        // Check for Rust project (Cargo.toml)
        let cargo_path = current_dir.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(versions) = Self::parse_cargo_toml(&cargo_path) {
                detected_projects.push(ProjectType::Rust);
                all_tool_versions.extend(versions);
            }
        }

        // Check for Node.js project (package.json)
        let package_path = current_dir.join("package.json");
        if package_path.exists() {
            if let Ok(versions) = Self::parse_package_json(&package_path) {
                detected_projects.push(ProjectType::Node);
                all_tool_versions.extend(versions);
            }
        }

        // Check for Go project (go.mod)
        let gomod_path = current_dir.join("go.mod");
        if gomod_path.exists() {
            if let Ok(versions) = Self::parse_go_mod(&gomod_path) {
                detected_projects.push(ProjectType::Go);
                all_tool_versions.extend(versions);
            }
        }

        if detected_projects.is_empty() {
            return Ok(None);
        }

        let project_type = if detected_projects.len() == 1 {
            detected_projects[0].clone()
        } else {
            ProjectType::Mixed
        };

        // Use the first detected config file as primary
        let config_file = match project_type {
            ProjectType::Python => pyproject_path,
            ProjectType::Rust => cargo_path,
            ProjectType::Node => package_path,
            ProjectType::Go => gomod_path,
            ProjectType::Mixed => {
                // Prefer pyproject.toml for mixed projects
                if pyproject_path.exists() {
                    pyproject_path
                } else if cargo_path.exists() {
                    cargo_path
                } else if package_path.exists() {
                    package_path
                } else {
                    gomod_path
                }
            }
            ProjectType::Unknown => return Ok(None),
        };

        Ok(Some(ProjectInfo {
            project_type,
            config_file,
            tool_versions: all_tool_versions,
        }))
    }

    /// Build the complete figment with all configuration layers
    fn build_figment(project_info: &Option<ProjectInfo>) -> Result<Figment> {
        let mut figment = Figment::new();

        // Layer 1: Built-in defaults (lowest priority)
        figment = figment.merge(Serialized::defaults(VxConfig::default()));

        // Layer 2: Global user configuration
        if let Some(config_dir) = dirs::config_dir() {
            let global_config = config_dir.join("vx").join("config.toml");
            if global_config.exists() {
                figment = figment.merge(Toml::file(global_config));
            }
        }

        // Layer 3: Project-specific tool versions (from project config files)
        if let Some(project_info) = project_info {
            let project_config = Self::create_project_config_from_info(project_info)?;
            figment = figment.merge(Serialized::defaults(project_config));
        }

        // Layer 4: vx-specific project configuration (.vx.toml)
        let vx_project_config = PathBuf::from(".vx.toml");
        if vx_project_config.exists() {
            figment = figment.merge(Toml::file(vx_project_config));
        }

        // Layer 5: Environment variables (highest priority)
        figment = figment.merge(Env::prefixed("VX_"));

        Ok(figment)
    }

    /// Get download URL for a tool and version
    pub fn get_download_url(&self, tool_name: &str, version: &str) -> Result<String> {
        // First try to get from configuration
        if let Some(tool_config) = self.config.tools.get(tool_name) {
            if let Some(custom_sources) = &tool_config.custom_sources {
                if let Some(url_template) = custom_sources.get("default") {
                    return Ok(self.expand_url_template(url_template, tool_name, version));
                }
            }
        }

        // Fall back to builtin configuration
        if self.config.defaults.fallback_to_builtin {
            crate::install_configs::get_install_config(tool_name, version)
                .and_then(|config| config.download_url)
                .ok_or_else(|| {
                    anyhow::anyhow!("No download URL available for {} {}", tool_name, version)
                })
        } else {
            Err(anyhow::anyhow!(
                "Tool {} not configured and fallback disabled",
                tool_name
            ))
        }
    }

    /// Expand URL template with variables
    fn expand_url_template(&self, template: &str, tool_name: &str, version: &str) -> String {
        let platform = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86"
        };

        let ext = if cfg!(windows) { "zip" } else { "tar.gz" };

        template
            .replace("{tool}", tool_name)
            .replace("{version}", version)
            .replace("{platform}", platform)
            .replace("{arch}", arch)
            .replace("{ext}", ext)
    }

    /// Get tool configuration
    pub fn get_tool_config(&self, tool_name: &str) -> Option<&ToolConfig> {
        self.config.tools.get(tool_name)
    }

    /// Get available tools (from configuration + builtin)
    pub fn get_available_tools(&self) -> Vec<String> {
        let mut tools = std::collections::HashSet::new();

        // Add configured tools
        for tool in self.config.tools.keys() {
            tools.insert(tool.clone());
        }

        // Add builtin tools if fallback is enabled
        if self.config.defaults.fallback_to_builtin {
            for tool in &["uv", "node", "go", "rust"] {
                if crate::install_configs::supports_auto_install(tool) {
                    tools.insert(tool.to_string());
                }
            }
        }

        let mut result: Vec<String> = tools.into_iter().collect();
        result.sort();
        result
    }

    /// Check if a tool is supported
    pub fn supports_tool(&self, tool_name: &str) -> bool {
        // Check if configured
        if self.config.tools.contains_key(tool_name) {
            return true;
        }

        // Check builtin if fallback enabled
        if self.config.defaults.fallback_to_builtin {
            return crate::install_configs::supports_auto_install(tool_name);
        }

        false
    }

    /// Get configuration status for diagnostics
    pub fn get_status(&self) -> ConfigStatus {
        let mut layers = Vec::new();

        // Check which layers are active
        layers.push(LayerInfo {
            name: "builtin".to_string(),
            available: true,
            priority: 10,
        });

        if let Some(config_dir) = dirs::config_dir() {
            let global_config = config_dir.join("vx").join("config.toml");
            layers.push(LayerInfo {
                name: "user".to_string(),
                available: global_config.exists(),
                priority: 50,
            });
        }

        let project_config = PathBuf::from(".vx.toml");
        layers.push(LayerInfo {
            name: "project".to_string(),
            available: project_config.exists(),
            priority: 80,
        });

        layers.push(LayerInfo {
            name: "environment".to_string(),
            available: std::env::vars().any(|(k, _)| k.starts_with("VX_")),
            priority: 100,
        });

        ConfigStatus {
            layers,
            available_tools: self.get_available_tools(),
            fallback_enabled: self.config.defaults.fallback_to_builtin,
            project_info: self.project_info.clone(),
        }
    }

    /// Reload configuration
    pub fn reload(&mut self) -> Result<()> {
        self.project_info = Self::detect_project_info()?;
        self.figment = Self::build_figment(&self.project_info)?;
        self.config = self.figment.extract()?;
        Ok(())
    }

    /// Get the underlying figment for advanced usage
    pub fn figment(&self) -> &Figment {
        &self.figment
    }

    /// Get the current configuration
    pub fn config(&self) -> &VxConfig {
        &self.config
    }

    /// Get project information
    pub fn project_info(&self) -> &Option<ProjectInfo> {
        &self.project_info
    }

    /// Parse pyproject.toml for tool version requirements
    fn parse_pyproject_toml(path: &PathBuf) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(path)?;
        let parsed: toml::Value = toml::from_str(&content)?;
        let mut versions = HashMap::new();

        // Check for Python version requirement
        if let Some(project) = parsed.get("project") {
            if let Some(requires_python) = project.get("requires-python") {
                if let Some(version_str) = requires_python.as_str() {
                    // Parse version requirement like ">=3.8" to "3.8"
                    let version = Self::parse_version_requirement(version_str);
                    versions.insert("python".to_string(), version);
                }
            }
        }

        // Check for tool.uv configuration
        if let Some(tool) = parsed.get("tool") {
            if let Some(uv) = tool.get("uv") {
                if let Some(version) = uv.get("version") {
                    if let Some(version_str) = version.as_str() {
                        versions.insert("uv".to_string(), version_str.to_string());
                    }
                }
            }
        }

        Ok(versions)
    }

    /// Parse Cargo.toml for tool version requirements
    fn parse_cargo_toml(path: &PathBuf) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(path)?;
        let parsed: toml::Value = toml::from_str(&content)?;
        let mut versions = HashMap::new();

        // Check for Rust version requirement
        if let Some(package) = parsed.get("package") {
            if let Some(rust_version) = package.get("rust-version") {
                if let Some(version_str) = rust_version.as_str() {
                    versions.insert("rust".to_string(), version_str.to_string());
                }
            }
        }

        Ok(versions)
    }

    /// Parse package.json for tool version requirements
    fn parse_package_json(path: &PathBuf) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(path)?;
        let parsed: JsonValue = serde_json::from_str(&content)?;
        let mut versions = HashMap::new();

        // Check for Node.js version requirement in engines
        if let Some(engines) = parsed.get("engines") {
            if let Some(node_version) = engines.get("node") {
                if let Some(version_str) = node_version.as_str() {
                    let version = Self::parse_version_requirement(version_str);
                    versions.insert("node".to_string(), version);
                }
            }
            if let Some(npm_version) = engines.get("npm") {
                if let Some(version_str) = npm_version.as_str() {
                    let version = Self::parse_version_requirement(version_str);
                    versions.insert("npm".to_string(), version);
                }
            }
        }

        Ok(versions)
    }

    /// Parse go.mod for Go version requirement
    fn parse_go_mod(path: &PathBuf) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(path)?;
        let mut versions = HashMap::new();

        // Parse go.mod format: "go 1.21"
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("go ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    versions.insert("go".to_string(), parts[1].to_string());
                }
                break;
            }
        }

        Ok(versions)
    }

    /// Parse version requirement string to extract version number
    fn parse_version_requirement(requirement: &str) -> String {
        // Handle common version requirement formats:
        // ">=3.8" -> "3.8"
        // "^18.0.0" -> "18.0.0"
        // "~1.70" -> "1.70"
        let cleaned = requirement
            .trim_start_matches(">=")
            .trim_start_matches("^")
            .trim_start_matches("~")
            .trim_start_matches("=")
            .trim();

        cleaned.to_string()
    }

    /// Create VxConfig from project information
    fn create_project_config_from_info(project_info: &ProjectInfo) -> Result<VxConfig> {
        let mut config = VxConfig::default();

        // Convert detected tool versions to tool configurations
        for (tool_name, version) in &project_info.tool_versions {
            let tool_config = ToolConfig {
                version: Some(version.clone()),
                install_method: None,
                registry: None,
                custom_sources: None,
            };
            config.tools.insert(tool_name.clone(), tool_config);
        }

        Ok(config)
    }
}

/// Configuration status information
#[derive(Debug)]
pub struct ConfigStatus {
    pub layers: Vec<LayerInfo>,
    pub available_tools: Vec<String>,
    pub fallback_enabled: bool,
    pub project_info: Option<ProjectInfo>,
}

/// Information about a configuration layer
#[derive(Debug)]
pub struct LayerInfo {
    pub name: String,
    pub available: bool,
    pub priority: u8,
}

impl ConfigStatus {
    /// Get a summary of the configuration status
    pub fn summary(&self) -> String {
        let active_layers = self.layers.iter().filter(|l| l.available).count();
        let project_type = self
            .project_info
            .as_ref()
            .map(|p| format!("{:?}", p.project_type))
            .unwrap_or_else(|| "None".to_string());

        format!(
            "{} layers active, {} tools available, project: {}, fallback {}",
            active_layers,
            self.available_tools.len(),
            project_type,
            if self.fallback_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )
    }

    /// Check if configuration is healthy
    pub fn is_healthy(&self) -> bool {
        !self.available_tools.is_empty() && self.layers.iter().any(|l| l.available)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config() {
        let manager = FigmentConfigManager::minimal().expect("Minimal config should work");
        assert!(!manager.get_available_tools().is_empty());
        assert!(manager.supports_tool("uv"));
    }

    #[test]
    fn test_url_template_expansion() {
        let manager = FigmentConfigManager::minimal().expect("Minimal config should work");
        let template = "https://example.com/{tool}/{version}/{tool}-{platform}-{arch}.{ext}";
        let expanded = manager.expand_url_template(template, "uv", "0.1.0");

        assert!(expanded.contains("uv"));
        assert!(expanded.contains("0.1.0"));
        assert!(expanded.contains("example.com"));
    }

    #[test]
    fn test_config_status() {
        let manager = FigmentConfigManager::minimal().expect("Minimal config should work");
        let status = manager.get_status();

        assert!(!status.layers.is_empty());
        assert!(status.is_healthy());
        assert!(!status.summary().is_empty());
    }
}
