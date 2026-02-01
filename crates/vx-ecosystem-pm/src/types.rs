//! Types for ecosystem package installation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents the result of a package installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemInstallResult {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Ecosystem (npm, pip, cargo, uv, bun, go, gem)
    pub ecosystem: String,
    /// Detected executables from the installation
    pub executables: Vec<String>,
    /// The installation directory
    pub install_dir: PathBuf,
    /// The bin directory containing executables
    pub bin_dir: PathBuf,
}

impl EcosystemInstallResult {
    /// Create a new install result
    pub fn new(
        name: String,
        version: String,
        ecosystem: String,
        install_dir: PathBuf,
        bin_dir: PathBuf,
    ) -> Self {
        Self {
            name,
            version,
            ecosystem,
            executables: Vec::new(),
            install_dir,
            bin_dir,
        }
    }

    /// Add detected executables
    pub fn with_executables(mut self, executables: Vec<String>) -> Self {
        self.executables = executables;
        self
    }
}

/// Environment variables for package manager installation
#[derive(Debug, Clone, Default)]
pub struct InstallEnv {
    /// Environment variables to set during installation
    pub vars: HashMap<String, String>,
    /// Additional PATH entries to prepend
    pub path_prepend: Vec<PathBuf>,
}

impl InstallEnv {
    /// Create a new empty InstallEnv
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an environment variable
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Add a PATH entry to prepend
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path_prepend.push(path.into());
        self
    }
}

/// Options for package installation
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Force reinstallation even if package exists
    pub force: bool,
    /// Verbose output
    pub verbose: bool,
    /// Runtime version to use (e.g., node version for npm)
    pub runtime_version: Option<String>,
    /// Additional arguments to pass to the package manager
    pub extra_args: Vec<String>,
}

impl InstallOptions {
    /// Create new install options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set force flag
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Set verbose flag
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set runtime version
    pub fn runtime_version(mut self, version: impl Into<String>) -> Self {
        self.runtime_version = Some(version.into());
        self
    }

    /// Add extra arguments
    pub fn extra_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }
}
