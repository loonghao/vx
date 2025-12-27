//! Configuration type definitions
//!
//! This module defines all configuration structures for `.vx.toml` files.
//! All fields are optional to maintain backward compatibility.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
// Core Types (v1 compatible)
// ============================================

/// Project metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ProjectConfig {
    /// Project name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Project version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// License
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

/// Tool version specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ToolVersion {
    /// Simple version string
    Simple(String),
    /// Detailed tool configuration
    Detailed(ToolConfig),
}

impl Default for ToolVersion {
    fn default() -> Self {
        ToolVersion::Simple("latest".to_string())
    }
}

/// Detailed tool configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ToolConfig {
    /// Version string
    pub version: String,

    /// Post-install command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postinstall: Option<String>,

    /// OS restrictions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Vec<String>>,

    /// Install environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_env: Option<HashMap<String, String>>,
}

/// Python environment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PythonConfig {
    /// Python version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Virtual environment path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venv: Option<String>,

    /// Package manager (uv, pip, poetry)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    /// Dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<PythonDependencies>,
}

/// Python dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PythonDependencies {
    /// Requirements files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,

    /// Direct packages
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,

    /// Git dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub git: Vec<String>,

    /// Dev dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dev: Vec<String>,
}

/// Environment variable configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct EnvConfig {
    /// Static environment variables
    #[serde(flatten)]
    pub vars: HashMap<String, String>,

    /// Required environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<HashMap<String, String>>,

    /// Optional environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<HashMap<String, String>>,

    /// Secret variables (loaded from secure storage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<SecretsConfig>,
}

/// Secrets configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SecretsConfig {
    /// Provider (auto, 1password, vault, aws-secrets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Secret items to load
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
}

/// Script configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ScriptConfig {
    /// Simple command string
    Simple(String),
    /// Detailed script configuration
    Detailed(ScriptDetails),
}

impl Default for ScriptConfig {
    fn default() -> Self {
        ScriptConfig::Simple(String::new())
    }
}

/// Detailed script configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ScriptDetails {
    /// Command to run
    pub command: String,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default arguments
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Dependencies (other scripts to run first)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends: Vec<String>,
}

/// Settings configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SettingsConfig {
    /// Auto-install missing tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_install: Option<bool>,

    /// Parallel installation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_install: Option<bool>,

    /// Cache duration (e.g., "7d")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_duration: Option<String>,

    /// Shell to use (auto, bash, zsh, fish, pwsh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    /// Log level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,

    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalConfig>,
}

/// Experimental features
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ExperimentalConfig {
    /// Monorepo support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monorepo: Option<bool>,

    /// Workspaces support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<bool>,
}

// ============================================
// Phase 1: Hooks and Services
// ============================================

/// Lifecycle hooks configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct HooksConfig {
    /// Pre-setup hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_setup: Option<HookCommand>,

    /// Post-setup hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_setup: Option<HookCommand>,

    /// Pre-commit hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_commit: Option<HookCommand>,

    /// Directory enter hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enter: Option<HookCommand>,

    /// Custom hooks
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, HookCommand>,
}

/// Hook command (string or array)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum HookCommand {
    /// Single command
    Single(String),
    /// Multiple commands
    Multiple(Vec<String>),
}

impl Default for HookCommand {
    fn default() -> Self {
        HookCommand::Single(String::new())
    }
}

/// Service configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ServiceConfig {
    /// Docker image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Command to run (for non-container services)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Port mappings
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Environment file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,

    /// Volume mounts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<String>,

    /// Dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,

    /// Health check command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<String>,

    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}

/// Dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DependenciesConfig {
    /// Generate lockfile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile: Option<bool>,

    /// Run audit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<bool>,

    /// Auto-update strategy (none, patch, minor, major)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<String>,

    /// Node.js dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeDependenciesConfig>,

    /// Python dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<PythonDependenciesConfig>,

    /// Dependency constraints
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub constraints: HashMap<String, ConstraintValue>,
}

/// Node.js dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct NodeDependenciesConfig {
    /// Package manager (npm, yarn, pnpm, bun)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    /// Registry URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
}

/// Python dependencies configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PythonDependenciesConfig {
    /// Index URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_url: Option<String>,

    /// Extra index URLs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_index_urls: Vec<String>,
}

/// Constraint value
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ConstraintValue {
    /// Version constraint
    Version(String),
    /// Detailed constraint
    Detailed(ConstraintDetails),
}

/// Detailed constraint
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ConstraintDetails {
    /// Allowed licenses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<String>,
}

// ============================================
// Phase 2+: Placeholder types
// ============================================

/// AI configuration (Phase 2)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct AiConfig {
    /// Enable AI integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// AI provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Documentation configuration (Phase 2)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DocsConfig {
    /// Enable documentation generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Output directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Team configuration (Phase 3)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TeamConfig {
    /// Extends from URL (remote preset)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Code owners configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_owners: Option<CodeOwnersConfig>,

    /// Review rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewConfig>,

    /// Conventions to enforce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conventions: Option<ConventionsConfig>,
}

/// Code owners configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CodeOwnersConfig {
    /// Enable CODEOWNERS generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Default owners for all files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_owners: Vec<String>,

    /// Path-specific owners
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub paths: HashMap<String, Vec<String>>,

    /// Output file path (default: .github/CODEOWNERS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Review configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ReviewConfig {
    /// Minimum required approvals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_approvals: Option<u32>,

    /// Require review from code owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_code_owner: Option<bool>,

    /// Dismiss stale reviews on new commits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismiss_stale: Option<bool>,

    /// Protected branches
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected_branches: Vec<String>,

    /// Auto-assign reviewers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_assign: Option<AutoAssignConfig>,
}

/// Auto-assign reviewers configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct AutoAssignConfig {
    /// Enable auto-assign
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Number of reviewers to assign
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,

    /// Reviewer pool
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviewers: Vec<String>,
}

/// Conventions configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ConventionsConfig {
    /// Commit message format (conventional, angular, custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_format: Option<String>,

    /// Custom commit pattern (regex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_pattern: Option<String>,

    /// Branch naming convention
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_pattern: Option<String>,

    /// PR title format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_title_pattern: Option<String>,

    /// Enforce linear history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linear_history: Option<bool>,

    /// Allowed merge strategies (merge, squash, rebase)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merge_strategies: Vec<String>,
}

/// Remote development configuration (Phase 3)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct RemoteConfig {
    /// Enable remote development config generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// GitHub Codespaces configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codespaces: Option<CodespacesConfig>,

    /// GitPod configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitpod: Option<GitpodConfig>,

    /// DevContainer configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devcontainer: Option<DevContainerConfig>,
}

/// GitHub Codespaces configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CodespacesConfig {
    /// Enable Codespaces config generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Machine type (basicLinux32gb, standardLinux32gb, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,

    /// Prebuild configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuild: Option<PrebuildConfig>,

    /// VS Code extensions to install
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    /// Forwarded ports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<PortForward>,
}

/// Prebuild configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PrebuildConfig {
    /// Enable prebuilds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Branches to prebuild
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<String>,
}

/// Port forwarding configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PortForward {
    /// Port number
    pub port: u16,

    /// Port label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Visibility (private, org, public)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// On auto-forward action (notify, openBrowser, ignore)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_auto_forward: Option<String>,
}

/// GitPod configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodConfig {
    /// Enable GitPod config generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Docker image to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Init tasks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<GitpodTask>,

    /// VS Code extensions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    /// Ports configuration
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<GitpodPort>,

    /// Prebuilds configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilds: Option<GitpodPrebuilds>,
}

/// GitPod task
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodTask {
    /// Task name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Init command (runs during prebuild)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<String>,

    /// Command (runs on workspace start)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Before command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
}

/// GitPod port configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodPort {
    /// Port number
    pub port: u16,

    /// Visibility (private, public)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// On open action (notify, open-browser, open-preview, ignore)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_open: Option<String>,
}

/// GitPod prebuilds configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct GitpodPrebuilds {
    /// Enable prebuilds for default branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<bool>,

    /// Enable prebuilds for branches
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches: Option<bool>,

    /// Enable prebuilds for PRs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<bool>,

    /// Add check to PRs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_check: Option<bool>,
}

/// DevContainer configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DevContainerConfig {
    /// Enable devcontainer.json generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Container name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Docker image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Dockerfile path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,

    /// Docker build context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Features to install
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub features: HashMap<String, serde_json::Value>,

    /// Post-create command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_create_command: Option<String>,

    /// Post-start command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_start_command: Option<String>,

    /// VS Code customizations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customizations: Option<DevContainerCustomizations>,

    /// Forwarded ports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forward_ports: Vec<u16>,

    /// Remote user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_user: Option<String>,

    /// Container environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub container_env: HashMap<String, String>,

    /// Mounts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mounts: Vec<String>,
}

/// DevContainer customizations
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DevContainerCustomizations {
    /// VS Code customizations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vscode: Option<VsCodeCustomizations>,
}

/// VS Code customizations
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct VsCodeCustomizations {
    /// Extensions to install
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    /// Settings
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Security configuration (Phase 4)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SecurityConfig {
    /// Enable security scanning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Fail on severity level (critical, high, medium, low)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_on: Option<String>,

    /// Dependency vulnerability scanning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<SecurityAuditConfig>,

    /// Secret detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<SecretDetectionConfig>,

    /// SAST (Static Application Security Testing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sast: Option<SastConfig>,

    /// Allowed licenses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_licenses: Vec<String>,

    /// Denied licenses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub denied_licenses: Vec<String>,
}

/// Security audit configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SecurityAuditConfig {
    /// Enable dependency audit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Ignore specific CVEs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,

    /// Audit on install
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_install: Option<bool>,

    /// Audit on CI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_ci: Option<bool>,
}

/// Secret detection configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SecretDetectionConfig {
    /// Enable secret detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Patterns to detect
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,

    /// Files to exclude
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    /// Pre-commit hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_commit: Option<bool>,

    /// Baseline file (known secrets to ignore)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
}

/// SAST configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SastConfig {
    /// Enable SAST
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// SAST tool (semgrep, codeql, snyk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,

    /// Ruleset to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruleset: Option<String>,

    /// Custom rules path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_path: Option<String>,

    /// Paths to exclude
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// Test configuration (Phase 4)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TestConfig {
    /// Test framework (auto, jest, pytest, cargo-test, go-test)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,

    /// Run tests in parallel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel: Option<bool>,

    /// Number of parallel workers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<u32>,

    /// Coverage configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageConfig>,

    /// Test hooks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<TestHooksConfig>,

    /// Test environments
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environments: HashMap<String, TestEnvironment>,

    /// Test timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,

    /// Retry failed tests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
}

/// Coverage configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CoverageConfig {
    /// Enable coverage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Coverage threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<u32>,

    /// Coverage tool (auto, lcov, cobertura, jacoco)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,

    /// Output format (html, lcov, json, cobertura)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formats: Vec<String>,

    /// Output directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Paths to exclude from coverage
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    /// Fail if coverage drops
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_on_decrease: Option<bool>,
}

/// Test hooks configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TestHooksConfig {
    /// Before all tests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_all: Option<String>,

    /// After all tests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_all: Option<String>,

    /// Before each test file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_each: Option<String>,

    /// After each test file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_each: Option<String>,
}

/// Test environment
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TestEnvironment {
    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Services to start
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,

    /// Setup command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,

    /// Teardown command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teardown: Option<String>,
}

/// Telemetry configuration (Phase 4)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Enable telemetry (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Anonymous mode (no identifiable data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous: Option<bool>,

    /// Build time tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_tracking: Option<BuildTrackingConfig>,

    /// OTLP export configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otlp: Option<OtlpConfig>,

    /// Metrics to collect
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<String>,
}

/// Build time tracking configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct BuildTrackingConfig {
    /// Enable build tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Track tool install times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_times: Option<bool>,

    /// Track script execution times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_times: Option<bool>,

    /// Track service startup times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_times: Option<bool>,

    /// Output file for local tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// OTLP export configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct OtlpConfig {
    /// Enable OTLP export
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// OTLP endpoint URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Headers for authentication
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// Service name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,

    /// Export interval in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
}

/// Container configuration (Phase 5)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ContainerConfig {
    /// Enable container support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Container runtime (docker, podman)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,

    /// Dockerfile generation configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<DockerfileConfig>,

    /// Multi-stage build configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<ContainerBuildConfig>,

    /// Registry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryConfig>,

    /// Image tags configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<ImageTagsConfig>,

    /// Container targets (for multi-image projects)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub targets: HashMap<String, ContainerTarget>,
}

/// Dockerfile generation configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DockerfileConfig {
    /// Output path (default: Dockerfile)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Base image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_image: Option<String>,

    /// Working directory in container
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,

    /// User to run as
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Exposed ports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expose: Vec<u16>,

    /// Labels
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Additional packages to install
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,

    /// Copy instructions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copy: Vec<CopyInstruction>,

    /// Run commands
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run: Vec<String>,

    /// Entrypoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,

    /// Default command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,

    /// Healthcheck configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<ContainerHealthcheck>,

    /// Files/directories to ignore (.dockerignore)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
}

/// Copy instruction for Dockerfile
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CopyInstruction {
    /// Source path
    pub src: String,

    /// Destination path
    pub dest: String,

    /// Owner (user:group)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chown: Option<String>,

    /// From stage (for multi-stage builds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

/// Container healthcheck configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ContainerHealthcheck {
    /// Command to run
    pub cmd: String,

    /// Interval between checks (e.g., "30s")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,

    /// Timeout for each check (e.g., "10s")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,

    /// Start period (e.g., "5s")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,

    /// Number of retries before unhealthy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
}

/// Multi-stage build configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ContainerBuildConfig {
    /// Enable multi-stage build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_stage: Option<bool>,

    /// Build stages
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<BuildStage>,

    /// Build arguments
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, String>,

    /// Target stage to build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// Cache configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<BuildCacheConfig>,

    /// Build context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Platform(s) to build for
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<String>,
}

/// Build stage configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct BuildStage {
    /// Stage name
    pub name: String,

    /// Base image for this stage
    pub base_image: String,

    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,

    /// Copy instructions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copy: Vec<CopyInstruction>,

    /// Run commands
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run: Vec<String>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Build arguments used in this stage
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// Build cache configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct BuildCacheConfig {
    /// Enable BuildKit cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Cache type (inline, registry, local, gha)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_type: Option<String>,

    /// Cache from locations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cache_from: Vec<String>,

    /// Cache to location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_to: Option<String>,
}

/// Registry configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct RegistryConfig {
    /// Registry URL (e.g., docker.io, ghcr.io, gcr.io)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Registry username (or env var reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Registry password/token (env var reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Image name (without tag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Push after build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,

    /// Additional registries for multi-registry push
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mirrors: Vec<RegistryMirror>,
}

/// Registry mirror configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct RegistryMirror {
    /// Mirror registry URL
    pub url: String,

    /// Mirror image name (optional, defaults to main image name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Image tags configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ImageTagsConfig {
    /// Tag strategy (semver, git-sha, branch, custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,

    /// Include latest tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<bool>,

    /// Include git SHA tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<bool>,

    /// SHA length (default: 7)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha_length: Option<u32>,

    /// Include branch name tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<bool>,

    /// Include timestamp tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<bool>,

    /// Timestamp format (default: %Y%m%d%H%M%S)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_format: Option<String>,

    /// Custom tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,

    /// Tag prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Tag suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

/// Container target (for multi-image projects)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ContainerTarget {
    /// Dockerfile path (relative to project root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,

    /// Build context path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Image name override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Target-specific build args
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, String>,

    /// Target-specific tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Depends on other targets
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

/// Versioning configuration (Phase 5)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct VersioningConfig {
    /// Versioning strategy (semver, calver)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,

    /// Auto-bump version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_bump: Option<bool>,
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
