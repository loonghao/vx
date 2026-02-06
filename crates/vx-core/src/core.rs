//! # vx-core - Core abstractions and interfaces
//!
//! This module provides the essential abstractions for the vx tool ecosystem.
//! Following SOLID principles, it defines interfaces without implementations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitStatus;

/// Core result type for vx operations
pub type VxResult<T> = Result<T, VxError>;

/// Core error types for vx operations
#[derive(thiserror::Error, Debug)]
pub enum VxError {
    #[error("Tool '{tool}' not found")]
    ToolNotFound { tool: String },

    #[error("Version '{version}' not found for tool '{tool}'")]
    VersionNotFound { tool: String, version: String },

    #[error("Installation failed for '{tool}': {reason}")]
    InstallationFailed { tool: String, reason: String },

    #[error("Execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Platform information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Platform {
    /// Operating system (windows, macos, linux)
    pub os: String,
    /// Architecture (x86_64, aarch64, etc.)
    pub arch: String,
}

impl Platform {
    /// Get current platform
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let platform_str = match (self.os.as_str(), self.arch.as_str()) {
            ("windows", "x86_64") => "win-x64".to_string(),
            ("windows", "aarch64") => "win-arm64".to_string(),
            ("macos", "x86_64") => "darwin-x64".to_string(),
            ("macos", "aarch64") => "darwin-arm64".to_string(),
            ("linux", "x86_64") => "linux-x64".to_string(),
            ("linux", "aarch64") => "linux-arm64".to_string(),
            _ => format!("{}-{}", self.os, self.arch),
        };
        write!(f, "{}", platform_str)
    }
}

/// Version information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    /// Version string
    pub version: String,
    /// Whether this is a prerelease
    pub prerelease: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Version {
    /// Create a new version
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            prerelease: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a prerelease version
    pub fn prerelease(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            prerelease: true,
            metadata: HashMap::new(),
        }
    }
}

/// Tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Supported platforms
    pub platforms: Vec<Platform>,
    /// Available versions
    pub versions: Vec<Version>,
    /// Installation methods
    pub install_methods: Vec<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Tool installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Tool name
    pub tool: String,
    /// Version to install
    pub version: String,
    /// Target platform
    pub platform: Platform,
    /// Installation directory
    pub install_dir: PathBuf,
    /// Download URL
    pub download_url: Option<String>,
    /// Installation method
    pub method: InstallMethod,
}

/// Installation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Download and extract archive
    Archive { format: ArchiveFormat },
    /// Download binary directly
    Binary,
    /// Use package manager
    PackageManager { manager: String },
    /// Custom installation script
    Custom { script: String },
}

/// Archive formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
    TarXz,
}

/// Runtime dependency specification (used for --with flag)
///
/// This represents a runtime that should be injected into the environment
/// when executing a tool. Similar to uvx --with or rez-env.
///
/// # Example
///
/// ```rust
/// use vx_core::WithDependency;
///
/// // Parse "bun@1.1.0"
/// let dep = WithDependency::parse("bun@1.1.0");
/// assert_eq!(dep.runtime, "bun");
/// assert_eq!(dep.version, Some("1.1.0".to_string()));
///
/// // Parse "deno" (no version)
/// let dep = WithDependency::parse("deno");
/// assert_eq!(dep.runtime, "deno");
/// assert_eq!(dep.version, None);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithDependency {
    /// Runtime name (e.g., "bun", "deno", "node")
    pub runtime: String,
    /// Optional version constraint (e.g., "1.1.0", "latest")
    pub version: Option<String>,
}

impl WithDependency {
    /// Create a new dependency with runtime name and optional version
    pub fn new(runtime: impl Into<String>, version: Option<String>) -> Self {
        Self {
            runtime: runtime.into(),
            version,
        }
    }

    /// Parse a dependency spec from string (e.g., "bun@1.1.0" or "deno")
    pub fn parse(spec: &str) -> Self {
        if let Some((runtime, version)) = spec.split_once('@') {
            Self {
                runtime: runtime.to_string(),
                version: Some(version.to_string()),
            }
        } else {
            Self {
                runtime: spec.to_string(),
                version: None,
            }
        }
    }

    /// Parse multiple dependency specs
    pub fn parse_many(specs: &[String]) -> Vec<Self> {
        specs.iter().map(|s| Self::parse(s)).collect()
    }
}

impl std::fmt::Display for WithDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref version) = self.version {
            write!(f, "{}@{}", self.runtime, version)
        } else {
            write!(f, "{}", self.runtime)
        }
    }
}

/// Execution context for tools
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Working directory
    pub working_dir: PathBuf,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Tool arguments
    pub args: Vec<String>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env_vars: HashMap::new(),
            args: Vec::new(),
        }
    }
}

/// Execution result
#[derive(Debug)]
pub struct ExecutionResult {
    /// Exit code
    pub exit_code: i32,
    /// Execution time
    pub duration: std::time::Duration,
    /// Whether the execution was successful
    pub success: bool,
}

/// Core trait for tool management
#[async_trait]
pub trait ToolManager: Send + Sync {
    /// Check if a tool is available
    async fn is_available(&self, tool: &str) -> VxResult<bool>;

    /// Get installed version of a tool
    async fn get_version(&self, tool: &str) -> VxResult<Option<Version>>;

    /// Install a tool
    async fn install(&self, config: &InstallConfig) -> VxResult<()>;

    /// Execute a tool
    async fn execute(&self, tool: &str, context: &ExecutionContext) -> VxResult<ExecutionResult>;

    /// List available tools
    async fn list_tools(&self) -> VxResult<Vec<String>>;
}

/// Core trait for tool resolution
#[async_trait]
pub trait ToolResolver: Send + Sync {
    /// Resolve tool specification
    async fn resolve(&self, tool: &str) -> VxResult<ToolSpec>;

    /// Get installation configuration
    async fn get_install_config(&self, tool: &str, version: &str) -> VxResult<InstallConfig>;
}

/// Core trait for version management
#[async_trait]
pub trait VersionManager: Send + Sync {
    /// List available versions for a tool
    async fn list_versions(&self, tool: &str) -> VxResult<Vec<Version>>;

    /// Get latest version for a tool
    async fn get_latest(&self, tool: &str) -> VxResult<Version>;

    /// Check if a version satisfies a constraint
    fn satisfies(&self, version: &Version, constraint: &str) -> bool;
}

/// Configuration for vx operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VxConfig {
    /// Installation directory
    pub install_dir: PathBuf,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Default platform
    pub platform: Platform,
    /// Registry URLs
    pub registries: Vec<String>,
    /// Tool-specific configurations
    pub tools: HashMap<String, serde_json::Value>,
    /// CDN acceleration settings
    #[serde(default)]
    pub cdn: CdnSettings,
}

/// CDN acceleration settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CdnSettings {
    /// Whether CDN acceleration is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Preferred region (auto-detected if not set)
    #[serde(default)]
    pub region: Option<String>,
}

impl Default for VxConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let vx_dir = home_dir.join(".vx");

        Self {
            install_dir: vx_dir.join("tools"),
            cache_dir: vx_dir.join("cache"),
            platform: Platform::current(),
            registries: vec!["https://registry.vx.dev".to_string()],
            tools: HashMap::new(),
            cdn: CdnSettings::default(),
        }
    }
}

// ============================================================================
// Process Exit Status Utilities
// ============================================================================

/// Check if an exit status indicates the process was terminated by Ctrl+C
///
/// On Windows, STATUS_CONTROL_C_EXIT (0xC000013A) indicates Ctrl+C termination.
/// On Unix, signal 2 (SIGINT) indicates Ctrl+C termination.
///
/// # Example
///
/// ```rust,ignore
/// use std::process::Command;
/// use vx_core::is_ctrl_c_exit;
///
/// let status = Command::new("some_command").status().unwrap();
/// if is_ctrl_c_exit(&status) {
///     // Process was terminated by Ctrl+C
/// }
/// ```
pub fn is_ctrl_c_exit(status: &ExitStatus) -> bool {
    #[cfg(windows)]
    {
        // Windows STATUS_CONTROL_C_EXIT = 0xC000013A = 3221225786
        // This is returned as a negative i32 when cast: -1073741510
        if let Some(code) = status.code() {
            // Check both the unsigned and signed representations
            code == -1073741510 || code as u32 == 0xC000013A
        } else {
            false
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        // SIGINT = 2
        status.signal() == Some(2)
    }
}

/// Convert an exit status to an appropriate exit code
///
/// This handles special cases like Ctrl+C termination, returning 130 (128 + SIGINT)
/// which is the standard Unix convention for signal termination.
///
/// # Example
///
/// ```rust,ignore
/// use std::process::Command;
/// use vx_core::exit_code_from_status;
///
/// let status = Command::new("some_command").status().unwrap();
/// let code = exit_code_from_status(&status);
/// std::process::exit(code);
/// ```
pub fn exit_code_from_status(status: &ExitStatus) -> i32 {
    if is_ctrl_c_exit(status) {
        // Return 130 (128 + 2) which is the standard exit code for SIGINT
        // This is recognized by shells as "terminated by signal"
        130
    } else {
        status.code().unwrap_or(1)
    }
}

// ============================================================================
// Version Resolution Utilities
// ============================================================================

/// Check if a version string is "latest" (case insensitive)
///
/// # Example
///
/// ```rust
/// use vx_core::is_latest_version;
///
/// assert!(is_latest_version("latest"));
/// assert!(is_latest_version("LATEST"));
/// assert!(!is_latest_version("1.0.0"));
/// ```
pub fn is_latest_version(version: &str) -> bool {
    version.eq_ignore_ascii_case("latest")
}

/// Resolve "latest" version to an actual version from installed versions
///
/// If the version is "latest", returns the highest version from the provided list
/// using semantic version comparison. Otherwise, returns the version as-is.
///
/// # Example
///
/// ```rust
/// use vx_core::resolve_latest_version;
///
/// let versions = vec!["1.0.0".to_string(), "2.0.0".to_string(), "1.5.0".to_string()];
/// assert_eq!(resolve_latest_version("latest", &versions), Some("2.0.0".to_string()));
/// assert_eq!(resolve_latest_version("1.5.0", &versions), Some("1.5.0".to_string()));
/// assert_eq!(resolve_latest_version("latest", &Vec::new()), None);
/// ```
pub fn resolve_latest_version(version: &str, installed_versions: &[String]) -> Option<String> {
    if is_latest_version(version) {
        crate::version_utils::find_latest_version(installed_versions, false).map(|v| v.to_string())
    } else {
        Some(version.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
    }

    #[test]
    fn test_platform_to_string() {
        let platform = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(platform.to_string(), "linux-x64");
    }

    #[test]
    fn test_version_creation() {
        let version = Version::new("1.0.0");
        assert_eq!(version.version, "1.0.0");
        assert!(!version.prerelease);

        let prerelease = Version::prerelease("2.0.0-beta.1");
        assert!(prerelease.prerelease);
    }

    #[test]
    fn test_vx_config_default() {
        let config = VxConfig::default();
        assert!(config.install_dir.to_string_lossy().contains(".vx"));
        assert!(!config.registries.is_empty());
    }

    #[test]
    fn test_with_dependency_parse() {
        // With version
        let dep = WithDependency::parse("bun@1.1.0");
        assert_eq!(dep.runtime, "bun");
        assert_eq!(dep.version, Some("1.1.0".to_string()));

        // Without version
        let dep = WithDependency::parse("deno");
        assert_eq!(dep.runtime, "deno");
        assert_eq!(dep.version, None);

        // Display format
        let dep = WithDependency::new("node", Some("20.0.0".to_string()));
        assert_eq!(dep.to_string(), "node@20.0.0");

        let dep = WithDependency::new("bun", None);
        assert_eq!(dep.to_string(), "bun");
    }

    #[test]
    fn test_with_dependency_parse_many() {
        let specs = vec![
            "bun@1.1".to_string(),
            "deno".to_string(),
            "node@20".to_string(),
        ];
        let deps = WithDependency::parse_many(&specs);
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].runtime, "bun");
        assert_eq!(deps[0].version, Some("1.1".to_string()));
        assert_eq!(deps[1].runtime, "deno");
        assert_eq!(deps[1].version, None);
        assert_eq!(deps[2].runtime, "node");
        assert_eq!(deps[2].version, Some("20".to_string()));
    }
}
