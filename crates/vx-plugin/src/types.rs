//! Shared types and data structures for the plugin system
//!
//! This module contains all the common types used across the plugin system,
//! including version information, package specifications, and execution contexts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Information about a tool version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string (e.g., "1.2.3")
    pub version: String,
    /// Whether this is a prerelease version
    pub prerelease: bool,
    /// Release date in ISO format
    pub release_date: Option<String>,
    /// Release notes or description
    pub release_notes: Option<String>,
    /// Download URL for this version
    pub download_url: Option<String>,
    /// Checksum for verification
    pub checksum: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl VersionInfo {
    /// Create a new VersionInfo with minimal information
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            prerelease: false,
            release_date: None,
            release_notes: None,
            download_url: None,
            checksum: None,
            file_size: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new VersionInfo with download URL
    pub fn with_url(version: impl Into<String>, download_url: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            prerelease: false,
            release_date: None,
            release_notes: None,
            download_url: Some(download_url.into()),
            checksum: None,
            file_size: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark this version as a prerelease
    #[allow(clippy::wrong_self_convention)]
    pub fn as_prerelease(mut self) -> Self {
        self.prerelease = true;
        self
    }

    /// Set release date
    pub fn with_release_date(mut self, date: impl Into<String>) -> Self {
        self.release_date = Some(date.into());
        self
    }

    /// Set release notes
    pub fn with_release_notes(mut self, notes: impl Into<String>) -> Self {
        self.release_notes = Some(notes.into());
        self
    }

    /// Set checksum
    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }

    /// Set file size
    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }

    /// Add metadata to this version
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Package specification for installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSpec {
    /// Package name
    pub name: String,
    /// Optional version constraint
    pub version: Option<String>,
    /// Optional features or extras to enable
    pub features: Vec<String>,
    /// Whether this is a development dependency
    pub dev_dependency: bool,
    /// Additional options specific to the package manager
    pub options: HashMap<String, String>,
}

impl PackageSpec {
    /// Create a new package specification
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            features: Vec::new(),
            dev_dependency: false,
            options: HashMap::new(),
        }
    }

    /// Set version constraint
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add a feature
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Mark as development dependency
    #[allow(clippy::wrong_self_convention)]
    pub fn as_dev_dependency(mut self) -> Self {
        self.dev_dependency = true;
        self
    }
}
/// Information about an installed package
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Package description
    pub description: Option<String>,
    /// Whether this is a development dependency
    pub dev_dependency: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Ecosystem that a tool or package manager belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ecosystem {
    /// Node.js ecosystem
    Node,
    /// Python ecosystem
    Python,
    /// Rust ecosystem
    Rust,
    /// Go ecosystem
    Go,
    /// Java ecosystem
    Java,
    /// .NET ecosystem
    DotNet,
    /// Ruby ecosystem
    Ruby,
    /// PHP ecosystem
    Php,
    /// Generic/cross-platform tools
    Generic,
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::Node => write!(f, "node"),
            Ecosystem::Python => write!(f, "python"),
            Ecosystem::Rust => write!(f, "rust"),
            Ecosystem::Go => write!(f, "go"),
            Ecosystem::Java => write!(f, "java"),
            Ecosystem::DotNet => write!(f, "dotnet"),
            Ecosystem::Ruby => write!(f, "ruby"),
            Ecosystem::Php => write!(f, "php"),
            Ecosystem::Generic => write!(f, "generic"),
        }
    }
}

/// Context for tool execution
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// Working directory for the tool
    pub working_directory: Option<PathBuf>,
    /// Environment variables to set
    pub environment_variables: HashMap<String, String>,
    /// Whether to use system PATH instead of vx-managed tools
    pub use_system_path: bool,
    /// Additional execution options
    pub options: HashMap<String, String>,
}

impl ToolContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set working directory
    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = Some(dir);
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment_variables.insert(key.into(), value.into());
        self
    }

    /// Use system PATH instead of vx-managed tools
    pub fn with_system_path(mut self) -> Self {
        self.use_system_path = true;
        self
    }
}
/// Result of tool execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionResult {
    /// Exit code of the process
    pub exit_code: i32,
    /// Standard output (if captured)
    pub stdout: Option<String>,
    /// Standard error (if captured)
    pub stderr: Option<String>,
}

impl ToolExecutionResult {
    /// Create a successful execution result
    pub fn success() -> Self {
        Self {
            exit_code: 0,
            stdout: None,
            stderr: None,
        }
    }

    /// Create a failed execution result
    pub fn failure(exit_code: i32) -> Self {
        Self {
            exit_code,
            stdout: None,
            stderr: None,
        }
    }

    /// Check if the execution was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Add stdout to the result
    pub fn with_stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = Some(stdout.into());
        self
    }

    /// Add stderr to the result
    pub fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = Some(stderr.into());
        self
    }
}

/// Status information for a tool
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolStatus {
    /// Whether the tool is installed
    pub installed: bool,
    /// Currently active version
    pub current_version: Option<String>,
    /// All installed versions
    pub installed_versions: Vec<String>,
}

impl ToolStatus {
    /// Create a new tool status
    pub fn new() -> Self {
        Self {
            installed: false,
            current_version: None,
            installed_versions: Vec::new(),
        }
    }

    /// Create status for an installed tool
    pub fn installed_with_versions(versions: Vec<String>, current: Option<String>) -> Self {
        Self {
            installed: !versions.is_empty(),
            current_version: current,
            installed_versions: versions,
        }
    }
}

impl Default for ToolStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic tool metadata for standard tools
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Supported aliases
    pub aliases: Vec<String>,
    /// Supported platforms
    pub platforms: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ToolMetadata {
    /// Create new tool metadata
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            aliases: Vec::new(),
            platforms: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add a supported platform
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platforms.push(platform.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Package manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    /// Package manager name
    pub name: String,
    /// Package manager version
    pub version: Option<String>,
    /// Path to the executable
    pub executable_path: Option<PathBuf>,
    /// Configuration files used by this package manager
    pub config_files: Vec<PathBuf>,
    /// Cache directory location
    pub cache_directory: Option<PathBuf>,
    /// Whether this package manager supports lock files
    pub supports_lockfiles: bool,
    /// Whether this package manager supports workspaces
    pub supports_workspaces: bool,
    /// Environment isolation level
    pub isolation_level: IsolationLevel,
}

/// Environment isolation level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IsolationLevel {
    /// No isolation, global installation
    Global,
    /// User-level isolation
    User,
    /// Project-level isolation
    #[default]
    Project,
    /// Complete sandboxing
    Sandbox,
}

/// Tool dependency specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDependency {
    /// Name of the dependency tool
    pub tool_name: String,
    /// Human-readable description
    pub description: String,
    /// Whether this dependency is required
    pub required: bool,
    /// Version requirement (optional)
    pub version_requirement: Option<String>,
}

impl ToolDependency {
    /// Create a new required dependency
    pub fn required(tool_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            description: description.into(),
            required: true,
            version_requirement: None,
        }
    }

    /// Create a new optional dependency
    pub fn optional(tool_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            description: description.into(),
            required: false,
            version_requirement: None,
        }
    }

    /// Set version requirement
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version_requirement = Some(version.into());
        self
    }
}
