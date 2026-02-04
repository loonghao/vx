use crate::PlatformConstraint;
use serde::{Deserialize, Serialize};

use super::{
    command::CommandDef,
    constraint::ConstraintRule,
    detection::DetectionConfig,
    download::DownloadConfig,
    env::EnvConfig,
    executable::ExecutableConfig,
    health::HealthConfig,
    hooks::HooksDef,
    layout::LayoutConfig,
    mirror::{CacheConfig, MirrorConfig, MirrorStrategy},
    normalize::NormalizeConfig,
    output::OutputConfig,
    platform_config::PlatformsDef,
    shell::ShellConfig,
    system_deps::{SystemDepsConfigDef, SystemInstallConfigDef},
    test_config::TestConfig,
    version_range::VersionRangeConfig,
    version_source::VersionSourceDef,
};

/// Bundled runtime configuration (RFC 0028)
///
/// For runtimes that are bundled with another runtime (e.g., MSBuild with .NET SDK)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BundledConfig {
    /// Parent runtime that provides this bundled runtime
    pub parent_runtime: String,
    /// Command prefix to use when executing (e.g., ["dotnet", "msbuild"])
    pub command_prefix: Vec<String>,
    /// Whether to fallback to system detection if parent is not installed
    #[serde(default)]
    pub fallback_detection: bool,
}

/// Runtime definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeDef {
    /// Runtime name (required)
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Executable name (required)
    pub executable: String,
    /// Aliases for this runtime
    #[serde(default)]
    pub aliases: Vec<String>,
    /// If this runtime is bundled with another
    #[serde(default)]
    pub bundled_with: Option<String>,
    /// If this runtime is managed by another (e.g., rustc managed by rustup)
    #[serde(default)]
    pub managed_by: Option<String>,
    /// Command prefix to add when executing (e.g., ["x"] for bunx -> bun x)
    #[serde(default)]
    pub command_prefix: Vec<String>,
    /// Dependency constraints
    #[serde(default)]
    pub constraints: Vec<ConstraintRule>,
    /// Hooks configuration
    #[serde(default)]
    pub hooks: Option<HooksDef>,
    /// Platform-specific configuration (download URLs, extensions, etc.)
    #[serde(default)]
    pub platforms: Option<PlatformsDef>,
    /// Platform constraints for this runtime
    #[serde(default, rename = "platform_constraint")]
    pub platform_constraint: Option<PlatformConstraint>,
    /// Version source configuration
    #[serde(default)]
    pub versions: Option<VersionSourceDef>,
    /// Executable configuration
    #[serde(default, rename = "executable_config")]
    pub executable_config: Option<ExecutableConfig>,

    // === RFC 0019: Executable Layout Configuration ===
    /// Layout configuration for downloaded archives
    #[serde(default)]
    pub layout: Option<LayoutConfig>,

    // === RFC 0020: Download Configuration ===
    /// Download configuration (timeout, retries, etc.)
    #[serde(default)]
    pub download: Option<DownloadConfig>,

    // === RFC 0018: Extended fields ===
    /// Installation priority (higher = install first)
    #[serde(default)]
    pub priority: Option<i32>,
    /// Whether this runtime can be auto-installed
    #[serde(default)]
    pub auto_installable: Option<bool>,
    /// Environment variable configuration
    #[serde(default, rename = "env")]
    pub env_config: Option<EnvConfig>,
    /// Version detection configuration
    #[serde(default)]
    pub detection: Option<DetectionConfig>,
    /// Health check configuration
    #[serde(default)]
    pub health: Option<HealthConfig>,
    /// Cache configuration
    #[serde(default)]
    pub cache: Option<CacheConfig>,
    /// Mirror configurations
    #[serde(default)]
    pub mirrors: Vec<MirrorConfig>,
    /// Mirror selection strategy
    #[serde(default, rename = "mirrors.strategy")]
    pub mirror_strategy: Option<MirrorStrategy>,

    // === RFC 0018 Phase 2: Custom Commands ===
    /// Custom commands provided by this runtime
    #[serde(default)]
    pub commands: Vec<CommandDef>,

    /// Output format configuration
    #[serde(default)]
    pub output: Option<OutputConfig>,

    /// Shell integration configuration
    #[serde(default)]
    pub shell: Option<ShellConfig>,

    // === RFC 0020: Test Configuration ===
    /// Test configuration for this runtime
    #[serde(default)]
    pub test: Option<TestConfig>,

    // === RFC 0021: System Dependencies ===
    /// System-level dependencies (VCRedist, KB updates, etc.)
    #[serde(default)]
    pub system_deps: Option<SystemDepsConfigDef>,

    /// System installation configuration (package manager strategies)
    #[serde(default)]
    pub system_install: Option<SystemInstallConfigDef>,

    // === RFC 0022: Install Normalize ===
    /// Post-install normalization configuration
    #[serde(default)]
    pub normalize: Option<NormalizeConfig>,

    // === RFC 0023: Version Range Locking ===
    /// Version range configuration for this runtime
    ///
    /// This allows providers to define:
    /// - Default version range for "latest" requests
    /// - Maximum/minimum allowed versions
    /// - Deprecated version ranges
    /// - Versions with known issues
    /// - Recommended stable version ranges
    #[serde(default)]
    pub version_ranges: Option<VersionRangeConfig>,

    // === RFC 0028: Bundled Runtime Configuration ===
    /// Bundled runtime configuration (for tools bundled with another runtime)
    #[serde(default)]
    pub bundled: Option<BundledConfig>,
}

impl RuntimeDef {
    /// Get constraints that apply to a specific version
    pub fn get_constraints_for_version(&self, version: &str) -> Vec<&ConstraintRule> {
        self.constraints
            .iter()
            .filter(|c| c.matches(version))
            .collect()
    }

    /// Get all required dependencies for a specific version
    pub fn get_dependencies_for_version(
        &self,
        version: &str,
    ) -> Vec<&super::constraint::DependencyDef> {
        self.get_constraints_for_version(version)
            .into_iter()
            .flat_map(|c| c.requires.iter())
            .collect()
    }

    /// Get all recommended dependencies for a specific version
    pub fn get_recommendations_for_version(
        &self,
        version: &str,
    ) -> Vec<&super::constraint::DependencyDef> {
        self.get_constraints_for_version(version)
            .into_iter()
            .flat_map(|c| c.recommends.iter())
            .collect()
    }

    /// Check if the runtime is supported on the current platform
    pub fn is_current_platform_supported(&self) -> bool {
        self.platform_constraint
            .as_ref()
            .is_none_or(|c| c.is_current_platform_supported())
    }

    /// Get the platform constraint description
    pub fn platform_description(&self) -> Option<String> {
        self.platform_constraint
            .as_ref()
            .and_then(|c| c.description())
    }

    /// Get a short platform label for display
    pub fn platform_label(&self) -> Option<String> {
        self.platform_constraint
            .as_ref()
            .and_then(|c| c.short_label())
    }

    /// Get a custom command by name
    pub fn get_command(&self, name: &str) -> Option<&CommandDef> {
        self.commands.iter().find(|c| c.name == name)
    }

    /// Get all visible (non-hidden) commands
    pub fn visible_commands(&self) -> Vec<&CommandDef> {
        self.commands.iter().filter(|c| !c.hidden).collect()
    }

    /// Get commands by category
    pub fn commands_by_category(&self, category: &str) -> Vec<&CommandDef> {
        self.commands
            .iter()
            .filter(|c| c.category.as_deref() == Some(category))
            .collect()
    }
}
