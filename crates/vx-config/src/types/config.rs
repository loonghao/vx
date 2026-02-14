//! Root VxConfig structure

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use super::{
    AiConfig, ContainerConfig, DependenciesConfig, DocsConfig, EnvConfig, HooksConfig,
    ProjectConfig, PythonConfig, RemoteConfig, ScriptConfig, SecurityConfig, ServiceConfig,
    SettingsConfig, SetupConfig, TeamConfig, TelemetryConfig, TestConfig, ToolVersion,
    VersioningConfig,
};

/// Tools included/skipped for a platform, with skip reasons.
///
/// `(included_tools, skipped_tools_with_allowed_os)`
type PlatformToolsResult<M> = (M, Vec<(String, Vec<String>)>);

/// Root configuration structure for `vx.toml`
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct VxConfig {
    /// Minimum vx version required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,

    /// Project metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectConfig>,

    /// Tool versions (primary field)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tools: HashMap<String, ToolVersion>,

    /// Tool versions (alias for backward compatibility with [runtimes])
    /// This field is merged into `tools` during deserialization
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub runtimes: HashMap<String, ToolVersion>,

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

    /// Setup pipeline configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<SetupConfig>,

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
        // Check tools first, then runtimes (for backward compatibility)
        self.tools
            .get(name)
            .or_else(|| self.runtimes.get(name))
            .map(|v| match v {
                ToolVersion::Simple(s) => s.clone(),
                ToolVersion::Detailed(d) => d.version.clone(),
            })
    }

    /// Get all tools as simple HashMap (for backward compatibility)
    /// Merges both `tools` and `runtimes` sections, with `tools` taking priority.
    ///
    /// **Note**: This does NOT filter by platform. Use `tools_for_current_platform()`
    /// to get only tools applicable to the current OS.
    pub fn tools_as_hashmap(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        // Add runtimes first (lower priority)
        for (k, v) in &self.runtimes {
            let version = match v {
                ToolVersion::Simple(s) => s.clone(),
                ToolVersion::Detailed(d) => d.version.clone(),
            };
            result.insert(k.clone(), version);
        }

        // Add tools (higher priority, overwrites runtimes)
        for (k, v) in &self.tools {
            let version = match v {
                ToolVersion::Simple(s) => s.clone(),
                ToolVersion::Detailed(d) => d.version.clone(),
            };
            result.insert(k.clone(), version);
        }

        result
    }

    /// Get all tools as BTreeMap (for deterministic ordering in lock files)
    /// Merges both `tools` and `runtimes` sections, with `tools` taking priority.
    ///
    /// **Note**: This does NOT filter by platform. Use `tools_for_current_platform_btree()`
    /// to get only tools applicable to the current OS.
    pub fn tools_as_btreemap(&self) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();

        // Add runtimes first (lower priority)
        for (k, v) in &self.runtimes {
            let version = match v {
                ToolVersion::Simple(s) => s.clone(),
                ToolVersion::Detailed(d) => d.version.clone(),
            };
            result.insert(k.clone(), version);
        }

        // Add tools (higher priority, overwrites runtimes)
        for (k, v) in &self.tools {
            let version = match v {
                ToolVersion::Simple(s) => s.clone(),
                ToolVersion::Detailed(d) => d.version.clone(),
            };
            result.insert(k.clone(), version);
        }

        result
    }

    /// Get tools filtered for the current platform.
    ///
    /// Tools with an `os` constraint that doesn't include the current OS
    /// are skipped. Tools without an `os` constraint (including simple
    /// version strings) are included on all platforms.
    ///
    /// Returns a tuple of (included tools, skipped tools with reason)
    pub fn tools_for_current_platform(&self) -> PlatformToolsResult<HashMap<String, String>> {
        let current_os = Self::current_os_name();
        self.tools_for_platform(current_os)
    }

    /// Get tools filtered for a specific platform (for testing).
    ///
    /// Returns (included_tools, skipped_tools_with_allowed_os)
    pub fn tools_for_platform(&self, os: &str) -> PlatformToolsResult<HashMap<String, String>> {
        let mut included = HashMap::new();
        let mut skipped = Vec::new();

        // Process runtimes first (lower priority)
        for (k, v) in &self.runtimes {
            if let Some((version, skip_reason)) = Self::check_tool_platform(v, os) {
                if let Some(reason) = skip_reason {
                    skipped.push((k.clone(), reason));
                } else {
                    included.insert(k.clone(), version);
                }
            }
        }

        // Process tools (higher priority, overwrites runtimes)
        for (k, v) in &self.tools {
            if let Some((version, skip_reason)) = Self::check_tool_platform(v, os) {
                if let Some(reason) = skip_reason {
                    // Remove from included if it was added from runtimes
                    included.remove(k);
                    skipped.push((k.clone(), reason));
                } else {
                    // Remove from skipped if it was skipped from runtimes
                    skipped.retain(|(name, _)| name != k);
                    included.insert(k.clone(), version);
                }
            }
        }

        (included, skipped)
    }

    /// Get tools filtered for the current platform as BTreeMap.
    ///
    /// Returns a tuple of (included tools, skipped tools with reason)
    pub fn tools_for_current_platform_btree(
        &self,
    ) -> PlatformToolsResult<BTreeMap<String, String>> {
        let (included, skipped) = self.tools_for_current_platform();
        (included.into_iter().collect(), skipped)
    }

    /// Check if a tool version entry is applicable to the given OS.
    ///
    /// Returns Some((version, None)) if the tool should be included,
    /// Some((version, Some(allowed_os_list))) if the tool should be skipped,
    /// None should never happen (always returns Some).
    fn check_tool_platform(tool: &ToolVersion, os: &str) -> Option<(String, Option<Vec<String>>)> {
        match tool {
            ToolVersion::Simple(s) => Some((s.clone(), None)),
            ToolVersion::Detailed(d) => {
                if let Some(os_list) = &d.os
                    && !os_list.is_empty()
                    && !os_list.iter().any(|o| o.eq_ignore_ascii_case(os))
                {
                    return Some((d.version.clone(), Some(os_list.clone())));
                }
                Some((d.version.clone(), None))
            }
        }
    }

    /// Get the current OS name as used in vx.toml `os` field
    pub fn current_os_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        }
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
            if let Some(isolation) = settings.isolation {
                map.insert("isolation".to_string(), isolation.to_string());
            }
        }
        map
    }

    /// Get isolation setting (defaults to true if not specified)
    pub fn is_isolation_mode(&self) -> bool {
        self.settings
            .as_ref()
            .and_then(|s| s.isolation)
            .unwrap_or(true) // Default to isolation mode
    }

    /// Get passenv patterns (environment variables to pass through)
    pub fn get_passenv(&self) -> Vec<String> {
        self.settings
            .as_ref()
            .and_then(|s| s.passenv.clone())
            .unwrap_or_default()
    }

    /// Get setenv variables (environment variables to explicitly set)
    pub fn get_setenv(&self) -> HashMap<String, String> {
        self.settings
            .as_ref()
            .and_then(|s| s.setenv.clone())
            .unwrap_or_default()
    }
}
