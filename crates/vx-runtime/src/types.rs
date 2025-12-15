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
}

/// Runtime dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDependency {
    /// Name of the required runtime
    pub name: String,
    /// Version requirement (e.g., ">=18.0.0", "^20.0.0")
    pub version_req: Option<String>,
    /// Whether this dependency is optional
    pub optional: bool,
}

impl RuntimeDependency {
    /// Create a new required dependency
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_req: None,
            optional: false,
        }
    }

    /// Create a new optional dependency
    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_req: None,
            optional: true,
        }
    }

    /// Set version requirement
    pub fn with_version(mut self, version_req: impl Into<String>) -> Self {
        self.version_req = Some(version_req.into());
        self
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
