//! Common types used across the runtime system

use crate::ecosystem::Ecosystem;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Version information for a runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string (e.g., "20.0.0")
    pub version: String,
    /// Release date
    pub released_at: Option<DateTime<Utc>>,
    /// Whether this is a prerelease
    pub prerelease: bool,
    /// Whether this is an LTS version
    pub lts: bool,
    /// Download URL for current platform
    pub download_url: Option<String>,
    /// Checksum (SHA256)
    pub checksum: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl VersionInfo {
    /// Create a new version info with just the version string
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            released_at: None,
            prerelease: false,
            lts: false,
            download_url: None,
            checksum: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the download URL
    pub fn with_download_url(mut self, url: impl Into<String>) -> Self {
        self.download_url = Some(url.into());
        self
    }

    /// Set as prerelease
    pub fn with_prerelease(mut self, prerelease: bool) -> Self {
        self.prerelease = prerelease;
        self
    }

    /// Set as LTS
    pub fn with_lts(mut self, lts: bool) -> Self {
        self.lts = lts;
        self
    }

    /// Set the release date from a string
    pub fn with_release_date(mut self, date: impl Into<String>) -> Self {
        // Store date string in metadata for now
        self.metadata
            .insert("release_date".to_string(), date.into());
        self
    }

    /// Set release notes
    pub fn with_release_notes(mut self, notes: impl Into<String>) -> Self {
        self.metadata
            .insert("release_notes".to_string(), notes.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Runtime dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDependency {
    /// Name of the required runtime
    pub name: String,
    /// Version requirement (e.g., ">=18.0.0", "^20.0.0")
    pub version_req: Option<String>,
    /// Minimum version required (semver constraint)
    pub min_version: Option<String>,
    /// Maximum version allowed (semver constraint)
    pub max_version: Option<String>,
    /// Recommended version for this dependency
    pub recommended_version: Option<String>,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Reason for this dependency
    pub reason: Option<String>,
}

impl RuntimeDependency {
    /// Create a new required dependency
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_req: None,
            min_version: None,
            max_version: None,
            recommended_version: None,
            optional: false,
            reason: None,
        }
    }

    /// Create a new optional dependency
    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_req: None,
            min_version: None,
            max_version: None,
            recommended_version: None,
            optional: true,
            reason: None,
        }
    }

    /// Set version requirement
    pub fn with_version(mut self, version_req: impl Into<String>) -> Self {
        self.version_req = Some(version_req.into());
        self
    }

    /// Set minimum version constraint
    pub fn with_min_version(mut self, version: impl Into<String>) -> Self {
        self.min_version = Some(version.into());
        self
    }

    /// Set maximum version constraint
    pub fn with_max_version(mut self, version: impl Into<String>) -> Self {
        self.max_version = Some(version.into());
        self
    }

    /// Set recommended version
    pub fn with_recommended_version(mut self, version: impl Into<String>) -> Self {
        self.recommended_version = Some(version.into());
        self
    }

    /// Set reason for this dependency
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Check if a version satisfies this dependency's constraints
    pub fn is_version_compatible(&self, version: &str) -> bool {
        // Parse version for comparison
        let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();

        if parts.is_empty() {
            return true; // Can't parse, assume compatible
        }

        // Check minimum version
        if let Some(ref min) = self.min_version {
            let min_parts: Vec<u32> = min.split('.').filter_map(|s| s.parse().ok()).collect();
            if !Self::version_gte(&parts, &min_parts) {
                return false;
            }
        }

        // Check maximum version
        if let Some(ref max) = self.max_version {
            let max_parts: Vec<u32> = max.split('.').filter_map(|s| s.parse().ok()).collect();
            if !Self::version_lte(&parts, &max_parts) {
                return false;
            }
        }

        true
    }

    /// Compare version parts: a >= b
    fn version_gte(a: &[u32], b: &[u32]) -> bool {
        for i in 0..std::cmp::max(a.len(), b.len()) {
            let av = a.get(i).copied().unwrap_or(0);
            let bv = b.get(i).copied().unwrap_or(0);
            if av > bv {
                return true;
            }
            if av < bv {
                return false;
            }
        }
        true // Equal
    }

    /// Compare version parts: a <= b
    fn version_lte(a: &[u32], b: &[u32]) -> bool {
        for i in 0..std::cmp::max(a.len(), b.len()) {
            let av = a.get(i).copied().unwrap_or(0);
            let bv = b.get(i).copied().unwrap_or(0);
            if av < bv {
                return true;
            }
            if av > bv {
                return false;
            }
        }
        true // Equal
    }
}

/// Runtime specification (metadata about a runtime)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSpec {
    /// Runtime name
    pub name: String,
    /// Aliases for this runtime
    pub aliases: Vec<String>,
    /// Ecosystem this runtime belongs to
    pub ecosystem: Ecosystem,
    /// Dependencies on other runtimes
    pub dependencies: Vec<RuntimeDependency>,
    /// Description
    pub description: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
}

impl RuntimeSpec {
    /// Create a new runtime spec
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aliases: vec![],
            ecosystem: Ecosystem::Unknown,
            dependencies: vec![],
            description: String::new(),
            homepage: None,
            repository: None,
        }
    }

    /// Set ecosystem
    pub fn with_ecosystem(mut self, ecosystem: Ecosystem) -> Self {
        self.ecosystem = ecosystem;
        self
    }

    /// Add an alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dep: RuntimeDependency) -> Self {
        self.dependencies.push(dep);
        self
    }
}

/// Result of an installation operation
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Whether the installation was successful
    pub success: bool,
    /// Path where the runtime was installed
    pub install_path: PathBuf,
    /// Path to the main executable
    pub executable_path: PathBuf,
    /// Version that was installed
    pub version: String,
    /// Whether this was a fresh install or already existed
    pub already_installed: bool,
}

impl InstallResult {
    /// Create a successful install result
    pub fn success(install_path: PathBuf, executable_path: PathBuf, version: String) -> Self {
        Self {
            success: true,
            install_path,
            executable_path,
            version,
            already_installed: false,
        }
    }

    /// Create an already-installed result
    pub fn already_installed(
        install_path: PathBuf,
        executable_path: PathBuf,
        version: String,
    ) -> Self {
        Self {
            success: true,
            install_path,
            executable_path,
            version,
            already_installed: true,
        }
    }

    /// Create a result for system-installed tools (via package manager)
    ///
    /// For system-installed tools, install_path is set to a placeholder
    /// since the tool is managed by the system package manager.
    pub fn system_installed(version: String, executable_path: Option<PathBuf>) -> Self {
        Self {
            success: true,
            install_path: PathBuf::from("system"),
            executable_path: executable_path.unwrap_or_else(|| PathBuf::from("system")),
            version,
            already_installed: false,
        }
    }

    /// Create a result for proxy-managed runtimes (RFC 0028)
    ///
    /// Proxy-managed runtimes are not directly installed; the prepare stage
    /// handles proxy execution setup. No executable_path is available.
    pub fn proxy(version: String) -> Self {
        Self {
            success: true,
            install_path: PathBuf::new(),
            executable_path: PathBuf::new(),
            version,
            already_installed: true,
        }
    }

    /// Create a result for a runtime that is already installed,
    /// with an optional executable path resolved from the store.
    pub fn already_installed_with(version: String, executable_path: Option<PathBuf>) -> Self {
        Self {
            success: true,
            install_path: PathBuf::new(),
            executable_path: executable_path.unwrap_or_default(),
            version,
            already_installed: true,
        }
    }
}

/// Execution preparation result (RFC 0028)
///
/// This struct is returned by `Runtime::prepare_execution()` to configure
/// how a proxy-managed or bundled tool should be executed.
///
/// # Examples
///
/// ## Proxy-managed tool (Yarn 2.x+ via corepack)
/// ```rust,ignore
/// ExecutionPrep {
///     use_system_path: true,  // Use corepack's yarn from PATH
///     proxy_ready: true,
///     ..Default::default()
/// }
/// ```
///
/// ## Bundled tool (msbuild with dotnet)
/// ```rust,ignore
/// ExecutionPrep {
///     executable_override: Some(PathBuf::from("/path/to/msbuild.dll")),
///     command_prefix: vec!["dotnet".to_string()],
///     proxy_ready: true,
///     ..Default::default()
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ExecutionPrep {
    /// Use system PATH instead of vx-managed path
    ///
    /// When true, the executor will look for the executable in the system PATH
    /// rather than in vx's store directory. This is used for proxy-managed tools
    /// like Yarn 2.x+ which are executed via corepack.
    pub use_system_path: bool,

    /// Override the executable path directly
    ///
    /// Used when the executable is discovered dynamically (e.g., bundled tools).
    /// If set, this path will be used instead of the normal resolution.
    pub executable_override: Option<PathBuf>,

    /// Additional environment variables to set before execution
    pub env_vars: HashMap<String, String>,

    /// Command prefix to add before user arguments
    ///
    /// For example, `["dotnet", "msbuild"]` for running msbuild via dotnet.
    /// The final command would be: `dotnet msbuild <user_args>`
    pub command_prefix: Vec<String>,

    /// Whether the proxy/bundled tool is ready for execution
    ///
    /// If false after `prepare_execution()`, the executor should report an error.
    pub proxy_ready: bool,

    /// Additional PATH entries to prepend
    ///
    /// These paths will be prepended to the PATH environment variable
    /// before executing the command.
    pub path_prepend: Vec<PathBuf>,

    /// Message to display to the user (for setup instructions, etc.)
    pub message: Option<String>,
}

impl ExecutionPrep {
    /// Create a new ExecutionPrep for a ready proxy-managed tool
    pub fn proxy_ready() -> Self {
        Self {
            use_system_path: true,
            proxy_ready: true,
            ..Default::default()
        }
    }

    /// Create a new ExecutionPrep with an executable override
    pub fn with_executable(path: PathBuf) -> Self {
        Self {
            executable_override: Some(path),
            proxy_ready: true,
            ..Default::default()
        }
    }

    /// Set an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add a command prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.command_prefix.push(prefix.into());
        self
    }

    /// Add a PATH entry to prepend
    pub fn with_path_prepend(mut self, path: PathBuf) -> Self {
        self.path_prepend.push(path);
        self
    }

    /// Set a message for the user
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Check if this is a no-op preparation (default behavior)
    pub fn is_default(&self) -> bool {
        !self.use_system_path
            && self.executable_override.is_none()
            && self.env_vars.is_empty()
            && self.command_prefix.is_empty()
            && !self.proxy_ready
            && self.path_prepend.is_empty()
            && self.message.is_none()
    }
}

/// Result of a command execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code
    pub exit_code: i32,
    /// Captured stdout (if capture_output was enabled)
    pub stdout: Option<String>,
    /// Captured stderr (if capture_output was enabled)
    pub stderr: Option<String>,
}

impl ExecutionResult {
    /// Create a successful execution result
    pub fn success() -> Self {
        Self {
            exit_code: 0,
            stdout: None,
            stderr: None,
        }
    }

    /// Create a result with exit code
    pub fn with_exit_code(exit_code: i32) -> Self {
        Self {
            exit_code,
            stdout: None,
            stderr: None,
        }
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Set stdout
    pub fn with_stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = Some(stdout.into());
        self
    }

    /// Set stderr
    pub fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = Some(stderr.into());
        self
    }
}
