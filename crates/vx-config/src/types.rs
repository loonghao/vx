//! Core types for vx configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main vx configuration structure
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct VxConfig {
    /// Global settings
    pub global: GlobalConfig,
    /// Default configuration settings
    pub defaults: DefaultConfig,
    /// Turbo CDN configuration
    pub turbo_cdn: TurboCdnConfig,
    /// Tool-specific configurations
    pub tools: HashMap<String, ToolConfig>,
    /// Registry configurations
    pub registries: HashMap<String, RegistryConfig>,
    /// Download source configurations
    pub download_sources: HashMap<String, DownloadSourceConfig>,
    /// Platform detection mappings
    pub platform_mappings: HashMap<String, String>,
}

/// Global configuration settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    /// Default home directory for vx (relative to user home)
    pub home_dir: String,
    /// Default tools directory
    pub tools_dir: String,
    /// Default cache directory
    pub cache_dir: String,
    /// Default shims directory
    pub shims_dir: String,
    /// Default configuration directory
    pub config_dir: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            home_dir: ".vx".to_string(),
            tools_dir: "tools".to_string(),
            cache_dir: "cache".to_string(),
            shims_dir: "shims".to_string(),
            config_dir: "config".to_string(),
        }
    }
}

/// Turbo CDN configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurboCdnConfig {
    /// Whether turbo-cdn is enabled
    pub enabled: bool,
    /// Default region
    pub default_region: String,
    /// Maximum concurrent chunks for downloads
    pub max_concurrent_chunks: u32,
    /// Chunk size in bytes
    pub chunk_size: u64,
    /// Maximum retries for failed downloads
    pub max_retries: u32,
    /// Cache configuration
    pub cache_enabled: bool,
    /// Maximum cache size in bytes
    pub cache_max_size: u64,
    /// Enable compression for cached files
    pub cache_compression: bool,
    /// Smart cache configuration
    pub smart_cache: SmartCacheConfig,
}

/// Smart cache configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmartCacheConfig {
    /// Maximum cache size in bytes
    pub max_size: u64,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
    /// Enable deduplication
    pub enable_dedup: bool,
    /// Enable cross-tool sharing
    pub enable_sharing: bool,
    /// Enable cache preheating
    pub enable_preheating: bool,
    /// Cleanup threshold (percentage)
    pub cleanup_threshold: f64,
    /// Minimum file size for deduplication (bytes)
    pub min_dedup_size: u64,
}

impl Default for SmartCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024 * 1024, // 10GB
            ttl_seconds: 7 * 24 * 60 * 60,     // 7 days
            enable_dedup: true,
            enable_sharing: true,
            enable_preheating: false,
            cleanup_threshold: 0.8,      // 80%
            min_dedup_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for TurboCdnConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_region: "Global".to_string(),
            max_concurrent_chunks: 8,
            chunk_size: 2 * 1024 * 1024, // 2MB
            max_retries: 3,
            cache_enabled: true,
            cache_max_size: 5 * 1024 * 1024 * 1024, // 5GB
            cache_compression: true,
            smart_cache: SmartCacheConfig::default(),
        }
    }
}

/// Default configuration settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultConfig {
    /// Whether to automatically install missing tools
    pub auto_install: bool,
    /// Default cache duration for downloads
    pub cache_duration: String,
    /// Whether to fall back to builtin tool configurations
    pub fallback_to_builtin: bool,
    /// Default installation directory
    pub install_dir: Option<String>,
    /// Whether to use system PATH for tools
    pub use_system_path: bool,
    /// Default download timeout in seconds
    pub download_timeout: u64,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: u32,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            cache_duration: "7d".to_string(),
            fallback_to_builtin: true,
            install_dir: None,
            use_system_path: false,
            download_timeout: 300,
            max_concurrent_downloads: 4,
        }
    }
}

/// Configuration for a specific tool
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolConfig {
    /// Tool description
    pub description: Option<String>,
    /// Tool homepage URL
    pub homepage: Option<String>,
    /// Repository identifier (e.g., "owner/repo")
    pub repository: Option<String>,
    /// Version fetcher URL for getting available versions
    pub fetcher_url: Option<String>,
    /// Specific version to use
    pub version: Option<String>,
    /// Installation method (auto, manual, system)
    pub install_method: Option<String>,
    /// Registry to use for this tool
    pub registry: Option<String>,
    /// Custom download sources
    pub custom_sources: Option<Vec<String>>,
    /// Download URL template
    pub download_url_template: Option<String>,
    /// Platform-specific filenames
    pub platforms: Option<HashMap<String, String>>,
    /// Platform-specific executables
    pub executables: Option<HashMap<String, String>>,
    /// Tools this tool depends on
    pub depends_on: Option<Vec<String>>,
    /// Arguments to pass to the executable
    pub exec_args: Option<HashMap<String, Vec<String>>>,
    /// Version parsing configuration
    pub version_parsing: Option<VersionParsingConfig>,
}

/// Version parsing configuration for a tool
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionParsingConfig {
    /// Regex to extract version from GitHub release tags
    pub tag_regex: Option<String>,
    /// Regex to extract version from tool output
    pub output_regex: Option<String>,
    /// Whether to normalize versions to semver format
    pub normalize_semver: Option<bool>,
}

/// Registry configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryConfig {
    /// Registry URL
    pub url: String,
    /// Authentication token if required
    pub token: Option<String>,
    /// Whether this registry is trusted
    pub trusted: bool,
}

/// Download source configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadSourceConfig {
    /// Source name
    pub name: String,
    /// Base URL for the source
    pub base_url: String,
    /// Supported domains
    pub supported_domains: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// URL template for downloads
    pub url_template: String,
    /// Platform-specific URL templates
    pub platform_templates: Option<HashMap<String, String>>,
    /// Whether this source is enabled
    pub enabled: bool,
    /// Priority (higher = preferred)
    pub priority: u32,
}

/// Supported project types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    Python,  // pyproject.toml
    Rust,    // Cargo.toml
    Node,    // package.json
    Go,      // go.mod
    Mixed,   // Multiple project types
    Unknown, // No recognized project files
}

/// Information about a detected project
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Type of project detected
    pub project_type: ProjectType,
    /// Path to the primary configuration file
    pub config_file: PathBuf,
    /// Tool versions detected from project files
    pub tool_versions: HashMap<String, String>,
}

/// Configuration status for diagnostics
#[derive(Debug, Clone)]
pub struct ConfigStatus {
    /// Information about configuration layers
    pub layers: Vec<LayerInfo>,
    /// List of available tools
    pub available_tools: Vec<String>,
    /// Whether fallback to builtin is enabled
    pub fallback_enabled: bool,
    /// Project information if detected
    pub project_info: Option<ProjectInfo>,
}

/// Information about a configuration layer
#[derive(Debug, Clone)]
pub struct LayerInfo {
    /// Name of the layer (builtin, user, project, environment)
    pub name: String,
    /// Whether this layer is available/active
    pub available: bool,
    /// Priority of this layer (higher = more important)
    pub priority: i32,
}

impl ConfigStatus {
    /// Get a summary of the configuration status
    pub fn summary(&self) -> String {
        let active_layers: Vec<&str> = self
            .layers
            .iter()
            .filter(|l| l.available)
            .map(|l| l.name.as_str())
            .collect();

        format!(
            "Configuration layers: {} | Tools: {} | Fallback: {}",
            active_layers.join(", "),
            self.available_tools.len(),
            if self.fallback_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )
    }

    /// Check if the configuration is healthy
    pub fn is_healthy(&self) -> bool {
        // At least one layer should be available
        self.layers.iter().any(|l| l.available) && !self.available_tools.is_empty()
    }
}

/// Project configuration for .vx.toml files
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ProjectConfig {
    /// Tool versions required for this project
    pub tools: HashMap<String, String>,
    /// Project-specific settings
    pub settings: ProjectSettings,
}

/// Settings specific to a project
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSettings {
    /// Whether to auto-install missing tools
    pub auto_install: bool,
    /// Cache duration for this project
    pub cache_duration: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_install: true,
            cache_duration: "7d".to_string(),
        }
    }
}
