//! Dependency detection and management

use crate::ecosystem::Ecosystem;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,

    /// Version constraint (if specified)
    pub version: Option<String>,

    /// Ecosystem this dependency belongs to
    pub ecosystem: Ecosystem,

    /// Where this dependency was found
    pub source: DependencySource,

    /// Whether this is a development dependency
    pub is_dev: bool,

    /// Whether this dependency is currently installed
    pub is_installed: bool,
}

impl Dependency {
    /// Create a new dependency
    pub fn new(name: impl Into<String>, ecosystem: Ecosystem, source: DependencySource) -> Self {
        Self {
            name: name.into(),
            version: None,
            ecosystem,
            source,
            is_dev: false,
            is_installed: false,
        }
    }

    /// Set version constraint
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Mark as development dependency
    pub fn as_dev(mut self) -> Self {
        self.is_dev = true;
        self
    }

    /// Mark as installed
    pub fn installed(mut self) -> Self {
        self.is_installed = true;
        self
    }
}

/// Source of a dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencySource {
    /// From a configuration file
    ConfigFile {
        /// Path to the config file
        path: PathBuf,
        /// Section in the config (e.g., "dependencies", "dev-dependencies")
        section: String,
    },

    /// Detected from a script command
    Script {
        /// Script name that uses this dependency
        script_name: String,
        /// The command that references this dependency
        command: String,
    },

    /// From a lock file
    LockFile {
        /// Path to the lock file
        path: PathBuf,
    },

    /// Detected from imports in source files
    SourceImport {
        /// Path to the source file
        path: PathBuf,
    },
}

impl std::fmt::Display for DependencySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencySource::ConfigFile { path, section } => {
                write!(f, "{} [{}]", path.display(), section)
            }
            DependencySource::Script {
                script_name,
                command: _,
            } => {
                write!(f, "script '{}'", script_name)
            }
            DependencySource::LockFile { path } => {
                write!(f, "{}", path.display())
            }
            DependencySource::SourceImport { path } => {
                write!(f, "import in {}", path.display())
            }
        }
    }
}

/// Method to install a dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Install via uv (Python)
    Uv {
        /// Command to run (e.g., "uv add nox")
        command: String,
    },

    /// Install via pip (Python)
    Pip {
        /// Command to run
        command: String,
    },

    /// Install via npm (Node.js)
    Npm {
        /// Command to run
        command: String,
    },

    /// Install via cargo (Rust)
    Cargo {
        /// Command to run
        command: String,
    },

    /// Install via vx (managed tool)
    Vx {
        /// Tool name
        tool: String,
        /// Version (if specified)
        version: Option<String>,
    },

    /// Manual installation required
    Manual {
        /// Instructions
        instructions: String,
    },
}

impl InstallMethod {
    /// Get the install command as a string
    pub fn command(&self) -> String {
        match self {
            InstallMethod::Uv { command } => command.clone(),
            InstallMethod::Pip { command } => command.clone(),
            InstallMethod::Npm { command } => command.clone(),
            InstallMethod::Cargo { command } => command.clone(),
            InstallMethod::Vx { tool, version } => {
                if let Some(v) = version {
                    format!("vx install {}@{}", tool, v)
                } else {
                    format!("vx install {}", tool)
                }
            }
            InstallMethod::Manual { instructions } => instructions.clone(),
        }
    }

    /// Create a uv install method for a dev dependency
    pub fn uv_dev(package: &str) -> Self {
        InstallMethod::Uv {
            command: format!("uv add --group dev {}", package),
        }
    }

    /// Create a uv install method for a regular dependency
    pub fn uv(package: &str) -> Self {
        InstallMethod::Uv {
            command: format!("uv add {}", package),
        }
    }

    /// Create an npm install method for a dev dependency
    pub fn npm_dev(package: &str) -> Self {
        InstallMethod::Npm {
            command: format!("npm install --save-dev {}", package),
        }
    }

    /// Create an npm install method for a regular dependency
    pub fn npm(package: &str) -> Self {
        InstallMethod::Npm {
            command: format!("npm install {}", package),
        }
    }

    /// Create a vx install method
    pub fn vx(tool: &str) -> Self {
        InstallMethod::Vx {
            tool: tool.to_string(),
            version: None,
        }
    }

    /// Create a vx install method with version
    pub fn vx_versioned(tool: &str, version: &str) -> Self {
        InstallMethod::Vx {
            tool: tool.to_string(),
            version: Some(version.to_string()),
        }
    }
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command())
    }
}
