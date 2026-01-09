//! Provider manifest types

use crate::{Ecosystem, ManifestError, PlatformConstraint, Result, VersionRequest};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Provider manifest - the root structure of provider.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderManifest {
    /// Provider metadata
    pub provider: ProviderMeta,
    /// Runtime definitions
    #[serde(default)]
    pub runtimes: Vec<RuntimeDef>,
}

impl ProviderManifest {
    /// Load a manifest from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse a manifest from TOML string
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: Self = toml::from_str(content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest
    fn validate(&self) -> Result<()> {
        if self.provider.name.is_empty() {
            return Err(ManifestError::MissingField("provider.name".to_string()));
        }

        for runtime in &self.runtimes {
            if runtime.name.is_empty() {
                return Err(ManifestError::MissingField("runtimes[].name".to_string()));
            }
            if runtime.executable.is_empty() {
                return Err(ManifestError::MissingField(format!(
                    "runtimes[{}].executable",
                    runtime.name
                )));
            }
        }

        Ok(())
    }

    /// Get a runtime definition by name
    pub fn get_runtime(&self, name: &str) -> Option<&RuntimeDef> {
        self.runtimes
            .iter()
            .find(|r| r.name == name || r.aliases.iter().any(|a| a == name))
    }

    /// Check if the provider is supported on the current platform
    pub fn is_current_platform_supported(&self) -> bool {
        self.provider.is_current_platform_supported()
    }

    /// Get the platform constraint description for the provider
    pub fn platform_description(&self) -> Option<String> {
        self.provider.platform_description()
    }

    /// Get a short platform label for display
    pub fn platform_label(&self) -> Option<String> {
        self.provider.platform_label()
    }

    /// Get all runtimes supported on the current platform
    pub fn supported_runtimes(&self) -> Vec<&RuntimeDef> {
        // If provider itself is not supported, return empty
        if !self.is_current_platform_supported() {
            return vec![];
        }

        self.runtimes
            .iter()
            .filter(|r| r.is_current_platform_supported())
            .collect()
    }
}

/// Provider metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderMeta {
    /// Provider name (required)
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,
    /// Ecosystem this provider belongs to
    #[serde(default)]
    pub ecosystem: Option<Ecosystem>,
    /// Platform constraints for the entire provider
    #[serde(default, rename = "platforms")]
    pub platform_constraint: Option<PlatformConstraint>,
}

impl ProviderMeta {
    /// Check if the provider is supported on the current platform
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
    pub fn get_dependencies_for_version(&self, version: &str) -> Vec<&DependencyDef> {
        self.get_constraints_for_version(version)
            .into_iter()
            .flat_map(|c| c.requires.iter())
            .collect()
    }

    /// Get all recommended dependencies for a specific version
    pub fn get_recommendations_for_version(&self, version: &str) -> Vec<&DependencyDef> {
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

/// Constraint rule - defines dependencies for a version range
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstraintRule {
    /// Version condition (semver syntax)
    /// Examples: "^1", ">=2, <4", "*"
    pub when: String,
    /// Platform condition (optional)
    #[serde(default)]
    pub platform: Option<String>,
    /// Required dependencies
    #[serde(default)]
    pub requires: Vec<DependencyDef>,
    /// Recommended dependencies (optional, not enforced)
    #[serde(default)]
    pub recommends: Vec<DependencyDef>,
}

impl ConstraintRule {
    /// Check if this rule applies to the given version
    pub fn matches(&self, version: &str) -> bool {
        let req = VersionRequest::parse(&self.when);
        req.satisfies(version)
    }

    /// Check if this rule applies to the given platform
    pub fn matches_platform(&self, platform: &str) -> bool {
        match &self.platform {
            Some(p) => p == platform || p == "*",
            None => true, // No platform restriction
        }
    }
}

/// Dependency definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyDef {
    /// Runtime name of the dependency
    pub runtime: String,
    /// Version constraint (semver syntax)
    pub version: String,
    /// Recommended version to install if none available
    #[serde(default)]
    pub recommended: Option<String>,
    /// Reason for this dependency
    #[serde(default)]
    pub reason: Option<String>,
    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

impl DependencyDef {
    /// Check if a version satisfies this dependency constraint
    pub fn satisfies(&self, version: &str) -> bool {
        let req = VersionRequest::parse(&self.version);
        req.satisfies(version)
    }
}

/// Hooks configuration
///
/// Provides complete lifecycle hooks for runtime operations.
/// All hooks are optional and default to empty vectors.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HooksDef {
    // === Installation lifecycle ===
    /// Hooks to run before installation
    #[serde(default)]
    pub pre_install: Vec<String>,
    /// Hooks to run after installation
    #[serde(default)]
    pub post_install: Vec<String>,
    /// Hooks to run before uninstallation
    #[serde(default)]
    pub pre_uninstall: Vec<String>,
    /// Hooks to run after uninstallation
    #[serde(default)]
    pub post_uninstall: Vec<String>,

    // === Activation lifecycle ===
    /// Hooks to run before activating a runtime version
    #[serde(default)]
    pub pre_activate: Vec<String>,
    /// Hooks to run after activating a runtime version
    #[serde(default)]
    pub post_activate: Vec<String>,
    /// Hooks to run before deactivating a runtime version
    #[serde(default)]
    pub pre_deactivate: Vec<String>,
    /// Hooks to run after deactivating a runtime version
    #[serde(default)]
    pub post_deactivate: Vec<String>,

    // === Execution lifecycle ===
    /// Hooks to run before executing the runtime
    #[serde(default)]
    pub pre_run: Vec<String>,
    /// Hooks to run after executing the runtime
    #[serde(default)]
    pub post_run: Vec<String>,

    // === Error handling hooks ===
    /// Hooks to run when installation fails
    #[serde(default)]
    pub on_install_error: Vec<String>,
    /// Hooks to run when requested version is not found
    #[serde(default)]
    pub on_version_not_found: Vec<String>,
    /// Hooks to run when health check fails
    #[serde(default)]
    pub on_health_check_fail: Vec<String>,

    // === Hook behavior configuration ===
    /// Hook execution configuration
    #[serde(default)]
    pub config: Option<HooksConfig>,
}

/// Hook execution configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HooksConfig {
    /// Whether to fail the operation if a hook fails
    #[serde(default = "default_true")]
    pub fail_on_error: bool,
    /// Timeout for each hook in milliseconds
    #[serde(default = "default_hook_timeout")]
    pub timeout_ms: u64,
    /// Whether to run hooks in parallel
    #[serde(default)]
    pub parallel: bool,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            fail_on_error: true,
            timeout_ms: 30000,
            parallel: false,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_hook_timeout() -> u64 {
    30000
}

/// Platform-specific configurations
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlatformsDef {
    /// Windows-specific configuration
    #[serde(default)]
    pub windows: Option<PlatformConfig>,
    /// macOS-specific configuration
    #[serde(default)]
    pub macos: Option<PlatformConfig>,
    /// Linux-specific configuration
    #[serde(default)]
    pub linux: Option<PlatformConfig>,
    /// Unix (macOS + Linux) configuration
    #[serde(default)]
    pub unix: Option<PlatformConfig>,
}

/// Platform-specific configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlatformConfig {
    /// Executable extensions for this platform
    #[serde(default)]
    pub executable_extensions: Vec<String>,
    /// Download URL pattern for this platform
    #[serde(default)]
    pub download_url_pattern: Option<String>,
}

/// Version source configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionSourceDef {
    /// Source type (e.g., "github-releases", "npm", "pypi")
    pub source: String,
    /// GitHub owner (for github-releases)
    #[serde(default)]
    pub owner: Option<String>,
    /// GitHub repo (for github-releases)
    #[serde(default)]
    pub repo: Option<String>,
    /// Whether to strip 'v' prefix from versions
    #[serde(default)]
    pub strip_v_prefix: bool,
    /// LTS version pattern
    #[serde(default)]
    pub lts_pattern: Option<String>,
}

/// Executable configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExecutableConfig {
    /// Executable extensions (e.g., [".cmd", ".exe"])
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Directory pattern after extraction
    #[serde(default)]
    pub dir_pattern: Option<String>,
}

// ============================================
// RFC 0018: Extended Provider Schema
// Phase 1: Environment, Detection, Health
// ============================================

/// Environment variable configuration
///
/// Supports static, dynamic (template), and conditional environment variables.
/// Template variables:
/// - `{install_dir}` - Installation directory
/// - `{version}` - Current version
/// - `{executable}` - Executable path
/// - `{PATH}` - Original PATH value
/// - `{env:VAR}` - Reference other environment variable
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EnvConfig {
    /// Static environment variables
    #[serde(default)]
    pub vars: std::collections::HashMap<String, String>,

    /// Conditional environment variables (version-based)
    /// Key is version constraint (e.g., ">=18"), value is env vars
    #[serde(default)]
    pub conditional: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

impl EnvConfig {
    /// Get environment variables for a specific version
    pub fn get_vars_for_version(&self, version: &str) -> std::collections::HashMap<String, String> {
        let mut result = self.vars.clone();

        for (constraint, vars) in &self.conditional {
            let req = VersionRequest::parse(constraint);
            if req.satisfies(version) {
                result.extend(vars.clone());
            }
        }

        result
    }

    /// Check if there are any environment variables configured
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.conditional.is_empty()
    }
}

/// Version detection configuration
///
/// Declares how to detect installed versions of a runtime.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectionConfig {
    /// Command to get version (supports {executable} template)
    pub command: String,

    /// Regex pattern to extract version (capture group 1 is version)
    pub pattern: String,

    /// System paths to check for existing installations
    #[serde(default)]
    pub system_paths: Vec<String>,

    /// Environment variable hints (may indicate installation)
    #[serde(default)]
    pub env_hints: Vec<String>,

    /// Windows registry paths to check
    #[serde(default)]
    pub registry_paths: Vec<String>,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            command: "{executable} --version".to_string(),
            pattern: r"v?(\d+\.\d+\.\d+)".to_string(),
            system_paths: Vec::new(),
            env_hints: Vec::new(),
            registry_paths: Vec::new(),
        }
    }
}

/// Health check configuration
///
/// Validates that a runtime installation is working correctly.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthConfig {
    /// Command to check health (supports {executable} template)
    pub check_command: String,

    /// Expected output pattern (regex)
    #[serde(default)]
    pub expected_pattern: Option<String>,

    /// Expected exit code (if not specified, any exit code is accepted)
    #[serde(default)]
    pub exit_code: Option<i32>,

    /// Timeout in milliseconds
    #[serde(default = "default_health_timeout")]
    pub timeout_ms: u64,

    /// Optional verification script path
    #[serde(default)]
    pub verify_script: Option<String>,

    /// When to run health checks
    #[serde(default = "default_check_on")]
    pub check_on: Vec<String>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_command: "{executable} --version".to_string(),
            expected_pattern: None,
            exit_code: Some(0),
            timeout_ms: 5000,
            verify_script: None,
            check_on: vec!["install".to_string()],
        }
    }
}

fn default_health_timeout() -> u64 {
    5000
}

fn default_check_on() -> Vec<String> {
    vec!["install".to_string()]
}

/// Mirror configuration for alternative download sources
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MirrorConfig {
    /// Mirror name (e.g., "taobao", "ustc")
    pub name: String,

    /// Geographic region (e.g., "cn", "us", "eu")
    #[serde(default)]
    pub region: Option<String>,

    /// Mirror URL
    pub url: String,

    /// Priority (higher = preferred)
    #[serde(default)]
    pub priority: i32,

    /// Whether this mirror is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Mirror selection strategy
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MirrorStrategy {
    /// Automatically detect best mirror based on location
    #[serde(default)]
    pub auto_detect: bool,

    /// Fall back to other mirrors on failure
    #[serde(default = "default_true")]
    pub fallback: bool,

    /// Probe mirrors in parallel to find fastest
    #[serde(default)]
    pub parallel_probe: bool,

    /// Probe timeout in milliseconds
    #[serde(default = "default_probe_timeout")]
    pub probe_timeout_ms: u64,
}

fn default_probe_timeout() -> u64 {
    3000
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Version list cache TTL in seconds
    #[serde(default = "default_versions_ttl")]
    pub versions_ttl: u64,

    /// Whether to cache downloads
    #[serde(default = "default_true")]
    pub cache_downloads: bool,

    /// Download retention in days
    #[serde(default = "default_retention_days")]
    pub downloads_retention_days: u32,

    /// Maximum cache size in MB
    #[serde(default)]
    pub max_cache_size_mb: Option<u64>,

    /// Use shared cache across projects
    #[serde(default = "default_true")]
    pub shared_cache: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            versions_ttl: 3600,
            cache_downloads: true,
            downloads_retention_days: 30,
            max_cache_size_mb: None,
            shared_cache: true,
        }
    }
}

fn default_versions_ttl() -> u64 {
    3600
}

fn default_retention_days() -> u32 {
    30
}

// ============================================
// RFC 0018: Phase 2 - Custom Commands
// ============================================

/// Custom command definition
///
/// Allows providers to define additional commands that can be invoked via:
/// `vx <runtime> <command> [args...]`
///
/// Example in provider.toml:
/// ```toml
/// [[runtimes.commands]]
/// name = "doctor"
/// description = "Diagnose installation"
/// command = "{executable} --version && echo OK"
/// category = "maintenance"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandDef {
    /// Command name (required)
    pub name: String,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Command to execute (supports template variables)
    /// Template variables: {executable}, {install_dir}, {version}
    #[serde(default)]
    pub command: Option<String>,

    /// Script file to execute (relative to provider directory)
    /// Alternative to `command` - for complex logic
    #[serde(default)]
    pub script: Option<String>,

    /// Whether to pass user arguments to the command
    #[serde(default)]
    pub pass_args: bool,

    /// Command category for help grouping
    #[serde(default)]
    pub category: Option<String>,

    /// Whether to hide from help output
    #[serde(default)]
    pub hidden: bool,
}

impl CommandDef {
    /// Create a new command definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            command: None,
            script: None,
            pass_args: false,
            category: None,
            hidden: false,
        }
    }

    /// Set the command to execute
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.command = Some(cmd.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Enable argument passing
    pub fn with_pass_args(mut self) -> Self {
        self.pass_args = true;
        self
    }

    /// Check if this command is valid (has either command or script)
    pub fn is_valid(&self) -> bool {
        self.command.is_some() || self.script.is_some()
    }
}

// ============================================
// RFC 0018: Phase 2 - Output Configuration
// ============================================

/// Output format configuration
///
/// Allows providers to customize how version lists, status, and other
/// output is formatted. Follows Unix text stream philosophy.
///
/// Example in provider.toml:
/// ```toml
/// [runtimes.output]
/// list_format = "{version:>12} {lts:>10} {installed:>10}"
/// status_format = "{name} {version} ({path})"
/// default_format = "text"
/// formats = ["text", "json", "csv"]
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Format string for version list display
    /// Template variables: {version}, {lts}, {installed}, {date}, {channel}
    #[serde(default)]
    pub list_format: Option<String>,

    /// Format string for status display
    /// Template variables: {name}, {version}, {path}, {source}
    #[serde(default)]
    pub status_format: Option<String>,

    /// Supported output formats
    #[serde(default)]
    pub formats: Vec<String>,

    /// Default output format (text, json, csv, table)
    #[serde(default)]
    pub default_format: Option<String>,

    /// Machine-readable flags for commands
    #[serde(default)]
    pub machine_flags: Option<MachineFlagsConfig>,

    /// Color configuration for terminal output
    #[serde(default)]
    pub colors: Option<OutputColorConfig>,
}

impl OutputConfig {
    /// Get the list format or a default
    pub fn list_format_or_default(&self) -> &str {
        self.list_format
            .as_deref()
            .unwrap_or("{version:>12} {installed:>10}")
    }

    /// Get the status format or a default
    pub fn status_format_or_default(&self) -> &str {
        self.status_format.as_deref().unwrap_or("{name} {version}")
    }

    /// Check if a format is supported
    pub fn supports_format(&self, format: &str) -> bool {
        if self.formats.is_empty() {
            // Default supported formats
            matches!(format, "text" | "json")
        } else {
            self.formats.iter().any(|f| f == format)
        }
    }
}

/// Machine-readable output flags
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MachineFlagsConfig {
    /// Flag for list command (e.g., "--json")
    #[serde(default)]
    pub list: Option<String>,

    /// Flag for info command
    #[serde(default)]
    pub info: Option<String>,

    /// Flag for status command
    #[serde(default)]
    pub status: Option<String>,
}

/// Color configuration for terminal output
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OutputColorConfig {
    /// Color for LTS versions
    #[serde(default)]
    pub lts: Option<String>,

    /// Color for current/active version
    #[serde(default)]
    pub current: Option<String>,

    /// Color for installed versions
    #[serde(default)]
    pub installed: Option<String>,

    /// Color for outdated versions
    #[serde(default)]
    pub outdated: Option<String>,

    /// Color for error messages
    #[serde(default)]
    pub error: Option<String>,
}

// ============================================
// RFC 0018: Phase 2 - Shell Integration
// ============================================

/// Shell integration configuration
///
/// Supports shell prompt customization, completion scripts, and aliases.
///
/// Example in provider.toml:
/// ```toml
/// [runtimes.shell]
/// prompt_format = "(node-{version})"
/// activate_template = "templates/activate.sh"
///
/// [runtimes.shell.completions]
/// bash = "completions/node.bash"
/// zsh = "completions/_node"
///
/// [runtimes.shell.aliases]
/// n = "node"
/// nr = "npm run"
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ShellConfig {
    /// Prompt format when runtime is activated
    /// Template variables: {name}, {version}
    #[serde(default)]
    pub prompt_format: Option<String>,

    /// Path to activation script template
    #[serde(default)]
    pub activate_template: Option<String>,

    /// Path to deactivation script template
    #[serde(default)]
    pub deactivate_template: Option<String>,

    /// Shell completion scripts
    #[serde(default)]
    pub completions: Option<ShellCompletionsConfig>,

    /// Shell aliases to set when activated
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,
}

impl ShellConfig {
    /// Get the prompt format for a specific version
    pub fn format_prompt(&self, version: &str, name: &str) -> Option<String> {
        self.prompt_format
            .as_ref()
            .map(|fmt| fmt.replace("{version}", version).replace("{name}", name))
    }

    /// Check if there are any shell integrations configured
    pub fn is_empty(&self) -> bool {
        self.prompt_format.is_none()
            && self.activate_template.is_none()
            && self.deactivate_template.is_none()
            && self.completions.is_none()
            && self.aliases.is_empty()
    }
}

/// Shell completion script paths
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ShellCompletionsConfig {
    /// Bash completion script
    #[serde(default)]
    pub bash: Option<String>,

    /// Zsh completion script
    #[serde(default)]
    pub zsh: Option<String>,

    /// Fish completion script
    #[serde(default)]
    pub fish: Option<String>,

    /// PowerShell completion script
    #[serde(default)]
    pub powershell: Option<String>,
}

impl ShellCompletionsConfig {
    /// Get the completion script for a shell type
    pub fn for_shell(&self, shell: &str) -> Option<&str> {
        match shell.to_lowercase().as_str() {
            "bash" => self.bash.as_deref(),
            "zsh" => self.zsh.as_deref(),
            "fish" => self.fish.as_deref(),
            "powershell" | "pwsh" => self.powershell.as_deref(),
            _ => None,
        }
    }

    /// Get all configured shells
    pub fn configured_shells(&self) -> Vec<&str> {
        let mut shells = Vec::new();
        if self.bash.is_some() {
            shells.push("bash");
        }
        if self.zsh.is_some() {
            shells.push("zsh");
        }
        if self.fish.is_some() {
            shells.push("fish");
        }
        if self.powershell.is_some() {
            shells.push("powershell");
        }
        shells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test-runtime"
executable = "test"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        assert_eq!(manifest.provider.name, "test");
        assert_eq!(manifest.runtimes.len(), 1);
        assert_eq!(manifest.runtimes[0].name, "test-runtime");
    }

    #[test]
    fn test_parse_full_manifest() {
        let toml = r#"
[provider]
name = "yarn"
description = "Fast, reliable, and secure dependency management"
homepage = "https://yarnpkg.com"
ecosystem = "nodejs"

[[runtimes]]
name = "yarn"
description = "Yarn package manager"
executable = "yarn"
aliases = ["yarnpkg"]

[[runtimes.constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <23", reason = "Yarn 1.x requires Node.js 12-22" }
]

[[runtimes.constraints]]
when = ">=4"
requires = [
    { runtime = "node", version = ">=18", recommended = "22" }
]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        assert_eq!(manifest.provider.name, "yarn");
        assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));

        let runtime = &manifest.runtimes[0];
        assert_eq!(runtime.name, "yarn");
        assert_eq!(runtime.aliases, vec!["yarnpkg"]);
        assert_eq!(runtime.constraints.len(), 2);

        // Test constraint matching
        let v1_constraints = runtime.get_constraints_for_version("1.22.22");
        assert_eq!(v1_constraints.len(), 1);
        assert_eq!(v1_constraints[0].requires.len(), 1);
        assert_eq!(v1_constraints[0].requires[0].runtime, "node");

        let v4_constraints = runtime.get_constraints_for_version("4.0.0");
        assert_eq!(v4_constraints.len(), 1);
        assert_eq!(v4_constraints[0].requires[0].version, ">=18");
    }

    #[test]
    fn test_parse_manifest_with_platform_constraint() {
        let toml = r#"
[provider]
name = "msvc"
description = "Microsoft Visual C++ Compiler"
ecosystem = "system"

[provider.platforms]
os = ["windows"]

[[runtimes]]
name = "cl"
description = "MSVC C/C++ Compiler"
executable = "cl"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        assert_eq!(manifest.provider.name, "msvc");

        // Check platform constraint
        let platform_constraint = manifest.provider.platform_constraint.as_ref().unwrap();
        assert_eq!(platform_constraint.os.len(), 1);

        // Platform description
        assert_eq!(
            manifest.platform_description(),
            Some("Windows only".to_string())
        );
        assert_eq!(manifest.platform_label(), Some("Windows".to_string()));
    }

    #[test]
    fn test_parse_runtime_with_platform_constraint() {
        let toml = r#"
[provider]
name = "xcode"
description = "Apple Xcode Command Line Tools"

[[runtimes]]
name = "xcodebuild"
executable = "xcodebuild"

[runtimes.platform_constraint]
os = ["macos"]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let runtime = &manifest.runtimes[0];

        // Check runtime platform constraint
        let platform_constraint = runtime.platform_constraint.as_ref().unwrap();
        assert_eq!(platform_constraint.os.len(), 1);
        assert_eq!(
            runtime.platform_description(),
            Some("macOS only".to_string())
        );
    }

    #[test]
    fn test_supported_runtimes() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "cross-platform"
executable = "cross"

[[runtimes]]
name = "windows-only"
executable = "win"

[runtimes.platform_constraint]
os = ["windows"]

[[runtimes]]
name = "macos-only"
executable = "mac"

[runtimes.platform_constraint]
os = ["macos"]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();

        // Get supported runtimes for current platform
        let supported = manifest.supported_runtimes();

        // cross-platform should always be included
        assert!(supported.iter().any(|r| r.name == "cross-platform"));

        // Platform-specific runtimes depend on current OS
        #[cfg(target_os = "windows")]
        {
            assert!(supported.iter().any(|r| r.name == "windows-only"));
            assert!(!supported.iter().any(|r| r.name == "macos-only"));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(!supported.iter().any(|r| r.name == "windows-only"));
            assert!(supported.iter().any(|r| r.name == "macos-only"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(!supported.iter().any(|r| r.name == "windows-only"));
            assert!(!supported.iter().any(|r| r.name == "macos-only"));
        }
    }

    // ============================================
    // RFC 0018: Extended Provider Schema Tests
    // ============================================

    #[test]
    fn test_parse_extended_hooks() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.hooks]
pre_install = ["check-prereqs.sh"]
post_install = ["setup.sh", "verify.sh"]
pre_activate = ["save-env.sh"]
post_activate = ["load-settings.sh"]
on_install_error = ["rollback.sh"]

[runtimes.hooks.config]
fail_on_error = true
timeout_ms = 60000
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let hooks = manifest.runtimes[0].hooks.as_ref().unwrap();

        assert_eq!(hooks.pre_install, vec!["check-prereqs.sh"]);
        assert_eq!(hooks.post_install, vec!["setup.sh", "verify.sh"]);
        assert_eq!(hooks.pre_activate, vec!["save-env.sh"]);
        assert_eq!(hooks.on_install_error, vec!["rollback.sh"]);

        let config = hooks.config.as_ref().unwrap();
        assert!(config.fail_on_error);
        assert_eq!(config.timeout_ms, 60000);
    }

    #[test]
    fn test_parse_detection_config() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.detection]
command = "{executable} --version"
pattern = "v?(\\d+\\.\\d+\\.\\d+)"
system_paths = ["/usr/bin/node", "/usr/local/bin/node"]
env_hints = ["NODE_HOME", "NVM_DIR"]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let detection = manifest.runtimes[0].detection.as_ref().unwrap();

        assert_eq!(detection.command, "{executable} --version");
        assert_eq!(detection.pattern, r"v?(\d+\.\d+\.\d+)");
        assert_eq!(detection.system_paths.len(), 2);
        assert_eq!(detection.env_hints, vec!["NODE_HOME", "NVM_DIR"]);
    }

    #[test]
    fn test_parse_health_config() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.health]
check_command = "{executable} -e 'console.log(process.version)'"
expected_pattern = "v\\d+\\.\\d+\\.\\d+"
exit_code = 0
timeout_ms = 3000
check_on = ["install", "activate"]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let health = manifest.runtimes[0].health.as_ref().unwrap();

        assert_eq!(
            health.check_command,
            "{executable} -e 'console.log(process.version)'"
        );
        assert_eq!(health.expected_pattern, Some(r"v\d+\.\d+\.\d+".to_string()));
        assert_eq!(health.exit_code, Some(0));
        assert_eq!(health.timeout_ms, 3000);
        assert_eq!(health.check_on, vec!["install", "activate"]);
    }

    #[test]
    fn test_parse_mirror_config() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[[runtimes.mirrors]]
name = "taobao"
region = "cn"
url = "https://npmmirror.com/mirrors/node"
priority = 100

[[runtimes.mirrors]]
name = "ustc"
region = "cn"
url = "https://mirrors.ustc.edu.cn/node"
priority = 90
enabled = false
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let mirrors = &manifest.runtimes[0].mirrors;

        assert_eq!(mirrors.len(), 2);
        assert_eq!(mirrors[0].name, "taobao");
        assert_eq!(mirrors[0].region, Some("cn".to_string()));
        assert_eq!(mirrors[0].priority, 100);
        assert!(mirrors[0].enabled); // default true

        assert_eq!(mirrors[1].name, "ustc");
        assert!(!mirrors[1].enabled);
    }

    #[test]
    fn test_parse_cache_config() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.cache]
versions_ttl = 7200
cache_downloads = true
downloads_retention_days = 14
max_cache_size_mb = 1024
shared_cache = false
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let cache = manifest.runtimes[0].cache.as_ref().unwrap();

        assert_eq!(cache.versions_ttl, 7200);
        assert!(cache.cache_downloads);
        assert_eq!(cache.downloads_retention_days, 14);
        assert_eq!(cache.max_cache_size_mb, Some(1024));
        assert!(!cache.shared_cache);
    }

    #[test]
    fn test_parse_priority_and_auto_installable() {
        let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "node"
executable = "node"
priority = 100
auto_installable = true

[[runtimes]]
name = "internal-tool"
executable = "itool"
priority = 50
auto_installable = false
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();

        assert_eq!(manifest.runtimes[0].priority, Some(100));
        assert_eq!(manifest.runtimes[0].auto_installable, Some(true));

        assert_eq!(manifest.runtimes[1].priority, Some(50));
        assert_eq!(manifest.runtimes[1].auto_installable, Some(false));
    }

    #[test]
    fn test_env_config_get_vars_for_version() {
        let mut env_config = EnvConfig::default();
        env_config
            .vars
            .insert("PATH".to_string(), "{install_dir}/bin".to_string());
        env_config.conditional.insert(
            ">=18".to_string(),
            [(
                "NODE_OPTIONS".to_string(),
                "--experimental-vm-modules".to_string(),
            )]
            .into_iter()
            .collect(),
        );
        env_config.conditional.insert(
            "<16".to_string(),
            [(
                "NODE_OPTIONS".to_string(),
                "--experimental-modules".to_string(),
            )]
            .into_iter()
            .collect(),
        );

        // Test version 20 (matches >=18)
        let vars_v20 = env_config.get_vars_for_version("20.0.0");
        assert!(vars_v20.contains_key("PATH"));
        assert_eq!(
            vars_v20.get("NODE_OPTIONS"),
            Some(&"--experimental-vm-modules".to_string())
        );

        // Test version 14 (matches <16)
        let vars_v14 = env_config.get_vars_for_version("14.0.0");
        assert!(vars_v14.contains_key("PATH"));
        assert_eq!(
            vars_v14.get("NODE_OPTIONS"),
            Some(&"--experimental-modules".to_string())
        );
    }

    // ============================================
    // RFC 0018 Phase 2: Custom Commands Tests
    // ============================================

    #[test]
    fn test_parse_custom_commands() {
        let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[[runtimes.commands]]
name = "repl"
description = "Start interactive REPL"
command = "{executable}"
category = "development"

[[runtimes.commands]]
name = "eval"
description = "Evaluate JavaScript expression"
command = "{executable} -e"
pass_args = true

[[runtimes.commands]]
name = "doctor"
description = "Diagnose installation"
script = "scripts/doctor.sh"
category = "maintenance"

[[runtimes.commands]]
name = "internal"
command = "{executable} --internal"
hidden = true
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let runtime = &manifest.runtimes[0];

        assert_eq!(runtime.commands.len(), 4);

        // Check repl command
        let repl = runtime.get_command("repl").unwrap();
        assert_eq!(repl.description, Some("Start interactive REPL".to_string()));
        assert_eq!(repl.command, Some("{executable}".to_string()));
        assert_eq!(repl.category, Some("development".to_string()));
        assert!(!repl.pass_args);
        assert!(!repl.hidden);

        // Check eval command with pass_args
        let eval = runtime.get_command("eval").unwrap();
        assert!(eval.pass_args);

        // Check doctor with script
        let doctor = runtime.get_command("doctor").unwrap();
        assert_eq!(doctor.script, Some("scripts/doctor.sh".to_string()));
        assert!(doctor.command.is_none());

        // Check hidden command
        let internal = runtime.get_command("internal").unwrap();
        assert!(internal.hidden);

        // Test visible_commands (should not include hidden)
        let visible = runtime.visible_commands();
        assert_eq!(visible.len(), 3);
        assert!(visible.iter().all(|c| !c.hidden));

        // Test commands_by_category
        let dev_commands = runtime.commands_by_category("development");
        assert_eq!(dev_commands.len(), 1);
        assert_eq!(dev_commands[0].name, "repl");

        let maint_commands = runtime.commands_by_category("maintenance");
        assert_eq!(maint_commands.len(), 1);
        assert_eq!(maint_commands[0].name, "doctor");
    }

    #[test]
    fn test_command_def_builder() {
        let cmd = CommandDef::new("test")
            .with_command("echo hello")
            .with_description("Test command")
            .with_pass_args();

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.command, Some("echo hello".to_string()));
        assert_eq!(cmd.description, Some("Test command".to_string()));
        assert!(cmd.pass_args);
        assert!(cmd.is_valid());

        // Command without command or script is invalid
        let invalid = CommandDef::new("invalid");
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_parse_output_config() {
        let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.output]
list_format = "{version:>12} {lts:>10} {installed:>10} {date}"
status_format = "{name} {version} ({path})"
formats = ["text", "json", "csv", "table"]
default_format = "text"

[runtimes.output.machine_flags]
list = "--json"
info = "--json"
status = "--json"

[runtimes.output.colors]
lts = "green"
current = "cyan"
installed = "blue"
outdated = "yellow"
error = "red"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let output = manifest.runtimes[0].output.as_ref().unwrap();

        assert_eq!(
            output.list_format,
            Some("{version:>12} {lts:>10} {installed:>10} {date}".to_string())
        );
        assert_eq!(
            output.status_format,
            Some("{name} {version} ({path})".to_string())
        );
        assert_eq!(output.formats, vec!["text", "json", "csv", "table"]);
        assert_eq!(output.default_format, Some("text".to_string()));

        // Test machine flags
        let flags = output.machine_flags.as_ref().unwrap();
        assert_eq!(flags.list, Some("--json".to_string()));
        assert_eq!(flags.info, Some("--json".to_string()));

        // Test colors
        let colors = output.colors.as_ref().unwrap();
        assert_eq!(colors.lts, Some("green".to_string()));
        assert_eq!(colors.current, Some("cyan".to_string()));
        assert_eq!(colors.error, Some("red".to_string()));
    }

    #[test]
    fn test_output_config_defaults() {
        let config = OutputConfig::default();

        assert_eq!(
            config.list_format_or_default(),
            "{version:>12} {installed:>10}"
        );
        assert_eq!(config.status_format_or_default(), "{name} {version}");

        // Default formats supported
        assert!(config.supports_format("text"));
        assert!(config.supports_format("json"));
        assert!(!config.supports_format("yaml"));

        // With explicit formats
        let config_with_formats = OutputConfig {
            formats: vec!["json".to_string(), "yaml".to_string()],
            ..Default::default()
        };
        assert!(config_with_formats.supports_format("json"));
        assert!(config_with_formats.supports_format("yaml"));
        assert!(!config_with_formats.supports_format("text"));
    }

    #[test]
    fn test_parse_shell_config() {
        let toml = r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"

[runtimes.shell]
prompt_format = "(node-{version})"
activate_template = "templates/activate.sh"
deactivate_template = "templates/deactivate.sh"

[runtimes.shell.completions]
bash = "completions/node.bash"
zsh = "completions/_node"
fish = "completions/node.fish"
powershell = "completions/node.ps1"

[runtimes.shell.aliases]
n = "node"
nr = "npm run"
nrd = "npm run dev"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let shell = manifest.runtimes[0].shell.as_ref().unwrap();

        assert_eq!(shell.prompt_format, Some("(node-{version})".to_string()));
        assert_eq!(
            shell.activate_template,
            Some("templates/activate.sh".to_string())
        );
        assert_eq!(
            shell.deactivate_template,
            Some("templates/deactivate.sh".to_string())
        );

        // Check completions
        let completions = shell.completions.as_ref().unwrap();
        assert_eq!(completions.bash, Some("completions/node.bash".to_string()));
        assert_eq!(completions.zsh, Some("completions/_node".to_string()));
        assert_eq!(completions.fish, Some("completions/node.fish".to_string()));
        assert_eq!(
            completions.powershell,
            Some("completions/node.ps1".to_string())
        );

        // Check for_shell method
        assert_eq!(completions.for_shell("bash"), Some("completions/node.bash"));
        assert_eq!(completions.for_shell("ZSH"), Some("completions/_node")); // case insensitive
        assert_eq!(completions.for_shell("pwsh"), Some("completions/node.ps1"));
        assert_eq!(completions.for_shell("tcsh"), None);

        // Check configured_shells
        let shells = completions.configured_shells();
        assert_eq!(shells.len(), 4);
        assert!(shells.contains(&"bash"));
        assert!(shells.contains(&"zsh"));

        // Check aliases
        assert_eq!(shell.aliases.len(), 3);
        assert_eq!(shell.aliases.get("n"), Some(&"node".to_string()));
        assert_eq!(shell.aliases.get("nr"), Some(&"npm run".to_string()));
    }

    #[test]
    fn test_shell_config_format_prompt() {
        let config = ShellConfig {
            prompt_format: Some("({name}-{version})".to_string()),
            ..Default::default()
        };

        let prompt = config.format_prompt("20.0.0", "node");
        assert_eq!(prompt, Some("(node-20.0.0)".to_string()));

        // Empty config
        let empty = ShellConfig::default();
        assert!(empty.format_prompt("1.0.0", "test").is_none());
        assert!(empty.is_empty());
    }
}
