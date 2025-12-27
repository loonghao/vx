//! Root VxConfig structure

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    AiConfig, ContainerConfig, DependenciesConfig, DocsConfig, EnvConfig, HooksConfig,
    ProjectConfig, PythonConfig, RemoteConfig, ScriptConfig, SecurityConfig, ServiceConfig,
    SettingsConfig, TeamConfig, TelemetryConfig, TestConfig, ToolVersion, VersioningConfig,
};

/// Root configuration structure for `.vx.toml`
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct VxConfig {
    /// Minimum vx version required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,

    /// Project metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectConfig>,

    /// Tool versions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tools: HashMap<String, ToolVersion>,

    /// Python environment configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<PythonConfig>,

    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<EnvConfig>,

    /// Script definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub scripts: HashMap<String, ScriptConfig>,

    /// Behavior settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<SettingsConfig>,

    // ========== v2 Fields (Phase 1+) ==========
    /// Lifecycle hooks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HooksConfig>,

    /// Service definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub services: HashMap<String, ServiceConfig>,

    /// Dependency management
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DependenciesConfig>,

    // ========== v2 Fields (Phase 2+) ==========
    /// AI integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiConfig>,

    /// Documentation generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<DocsConfig>,

    // ========== v2 Fields (Phase 3+) ==========
    /// Team collaboration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<TeamConfig>,

    /// Remote development
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteConfig>,

    // ========== v2 Fields (Phase 4+) ==========
    /// Security scanning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,

    /// Test pipeline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<TestConfig>,

    /// Telemetry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<TelemetryConfig>,

    // ========== v2 Fields (Phase 5+) ==========
    /// Container deployment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ContainerConfig>,

    /// Versioning strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versioning: Option<VersioningConfig>,
}

// ============================================
// Helper implementations
// ============================================

impl VxConfig {
    /// Get tool version as string
    pub fn get_tool_version(&self, name: &str) -> Option<String> {
        self.tools.get(name).map(|v| match v {
            ToolVersion::Simple(s) => s.clone(),
            ToolVersion::Detailed(d) => d.version.clone(),
        })
    }

    /// Get all tools as simple HashMap (for backward compatibility)
    pub fn tools_as_hashmap(&self) -> HashMap<String, String> {
        self.tools
            .iter()
            .map(|(k, v)| {
                let version = match v {
                    ToolVersion::Simple(s) => s.clone(),
                    ToolVersion::Detailed(d) => d.version.clone(),
                };
                (k.clone(), version)
            })
            .collect()
    }

    /// Get script command
    pub fn get_script_command(&self, name: &str) -> Option<String> {
        self.scripts.get(name).map(|s| match s {
            ScriptConfig::Simple(cmd) => cmd.clone(),
            ScriptConfig::Detailed(d) => d.command.clone(),
        })
    }

    /// Get all scripts as simple HashMap (for backward compatibility)
    pub fn scripts_as_hashmap(&self) -> HashMap<String, String> {
        self.scripts
            .iter()
            .map(|(k, v)| {
                let cmd = match v {
                    ScriptConfig::Simple(s) => s.clone(),
                    ScriptConfig::Detailed(d) => d.command.clone(),
                };
                (k.clone(), cmd)
            })
            .collect()
    }

    /// Get environment variables as HashMap
    pub fn env_as_hashmap(&self) -> HashMap<String, String> {
        self.env
            .as_ref()
            .map(|e| e.vars.clone())
            .unwrap_or_default()
    }

    /// Get settings as HashMap (for backward compatibility)
    pub fn settings_as_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(settings) = &self.settings {
            if let Some(auto_install) = settings.auto_install {
                map.insert("auto_install".to_string(), auto_install.to_string());
            }
            if let Some(parallel) = settings.parallel_install {
                map.insert("parallel_install".to_string(), parallel.to_string());
            }
            if let Some(duration) = &settings.cache_duration {
                map.insert("cache_duration".to_string(), duration.clone());
            }
        }
        map
    }
}
