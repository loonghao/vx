//! Type definitions for manifest-driven runtimes.
//!
//! This module contains all the data types used by [`super::ManifestDrivenRuntime`]:
//! installation strategies, detection configuration, shell definitions, and
//! system dependency declarations.

use crate::platform::Platform;

/// Source of a provider
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderSource {
    /// Built-in provider (compiled into vx)
    BuiltIn,
    /// User local provider (~/.vx/providers/)
    UserLocal(std::path::PathBuf),
    /// Environment variable specified path
    EnvPath(std::path::PathBuf),
}

impl std::fmt::Display for ProviderSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderSource::BuiltIn => write!(f, "built-in"),
            ProviderSource::UserLocal(p) => write!(f, "{}", p.display()),
            ProviderSource::EnvPath(p) => write!(f, "{} (env)", p.display()),
        }
    }
}

/// Installation strategy for system tools
#[derive(Debug, Clone)]
pub enum InstallStrategy {
    /// Use a system package manager
    PackageManager {
        /// Package manager name (choco, winget, brew, apt, etc.)
        manager: String,
        /// Package name
        package: String,
        /// Installation parameters (Chocolatey --params)
        params: Option<String>,
        /// Native installer arguments
        install_args: Option<String>,
        /// Priority (higher = preferred)
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Direct download
    DirectDownload {
        /// URL template (supports {version}, {platform}, {arch})
        url: String,
        /// Archive format (zip, tar.gz, etc.)
        format: Option<String>,
        /// Executable path within archive
        executable_path: Option<String>,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Run a script
    Script {
        /// Script URL
        url: String,
        /// Script type (powershell, bash, cmd)
        script_type: ScriptType,
        /// Script arguments
        args: Vec<String>,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Provided by another runtime
    ProvidedBy {
        /// Provider runtime name
        provider: String,
        /// Relative path to executable
        relative_path: String,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
}

impl InstallStrategy {
    /// Get the priority of this strategy
    pub fn priority(&self) -> i32 {
        match self {
            InstallStrategy::PackageManager { priority, .. } => *priority,
            InstallStrategy::DirectDownload { priority, .. } => *priority,
            InstallStrategy::Script { priority, .. } => *priority,
            InstallStrategy::ProvidedBy { priority, .. } => *priority,
        }
    }

    /// Check if this strategy matches the current platform
    pub fn matches_platform(&self, platform: &Platform) -> bool {
        let platforms = match self {
            InstallStrategy::PackageManager { platforms, .. } => platforms,
            InstallStrategy::DirectDownload { platforms, .. } => platforms,
            InstallStrategy::Script { platforms, .. } => platforms,
            InstallStrategy::ProvidedBy { platforms, .. } => platforms,
        };

        if platforms.is_empty() {
            return true; // No filter = all platforms
        }

        let current = platform.os_name();
        platforms.iter().any(|p| p.eq_ignore_ascii_case(current))
    }
}

/// Script type for installation
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
    PowerShell,
    Bash,
    Cmd,
}

/// Tool provided by a runtime
#[derive(Debug, Clone)]
pub struct ProvidedTool {
    /// Tool name
    pub name: String,
    /// Relative path to executable
    pub relative_path: String,
    /// Supported platforms
    pub platforms: Vec<String>,
}

/// Detection configuration for version detection
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Command to run (e.g., "{executable} --version")
    pub command: String,
    /// Regex pattern to extract version
    pub pattern: String,
    /// System paths to search
    pub system_paths: Vec<String>,
    /// Environment variable hints
    pub env_hints: Vec<String>,
}

/// System dependencies configuration
#[derive(Debug, Clone, Default)]
pub struct SystemDepsConfig {
    /// Pre-installation dependencies
    pub pre_depends: Vec<SystemDependency>,
    /// Runtime dependencies
    pub depends: Vec<SystemDependency>,
    /// Recommended dependencies
    pub recommends: Vec<SystemDependency>,
    /// Optional dependencies
    pub suggests: Vec<SystemDependency>,
}

/// A system-level dependency
#[derive(Debug, Clone)]
pub struct SystemDependency {
    /// Dependency type
    pub dep_type: SystemDepType,
    /// Dependency identifier
    pub id: String,
    /// Version constraint
    pub version: Option<String>,
    /// Reason for dependency
    pub reason: Option<String>,
    /// Platform filter
    pub platforms: Vec<String>,
    /// Whether this is optional
    pub optional: bool,
}

/// Type of system dependency
#[derive(Debug, Clone, PartialEq)]
pub enum SystemDepType {
    /// Windows KB update
    WindowsKb,
    /// Windows Feature (DISM)
    WindowsFeature,
    /// Visual C++ Redistributable
    VcRedist,
    /// .NET Framework / Runtime
    DotNet,
    /// System package
    Package,
    /// Another vx runtime
    Runtime,
}

/// Shell definition for runtime-provided shells (RFC 0038)
#[derive(Debug, Clone)]
pub struct ShellDefinition {
    /// Shell name (e.g., "git-bash", "cmd")
    pub name: String,
    /// Relative path from install directory (e.g., "git-bash.exe", "bin/bash.exe")
    pub path: String,
}
