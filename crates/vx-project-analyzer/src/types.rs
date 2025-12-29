//! Core type definitions for project analysis

use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::script_parser::ScriptTool;
use crate::sync::SyncAction;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete project analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    /// Project root directory
    pub root: PathBuf,

    /// Detected ecosystems
    pub ecosystems: Vec<Ecosystem>,

    /// All detected dependencies
    pub dependencies: Vec<Dependency>,

    /// All detected scripts
    pub scripts: Vec<Script>,

    /// Required tools (runtimes that need to be installed)
    pub required_tools: Vec<RequiredTool>,

    /// Suggested sync actions
    pub sync_actions: Vec<SyncAction>,
}

impl ProjectAnalysis {
    /// Create a new empty analysis for the given root
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            ..Default::default()
        }
    }

    /// Get missing dependencies
    pub fn missing_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies
            .iter()
            .filter(|d| !d.is_installed)
            .collect()
    }

    /// Get missing tools
    pub fn missing_tools(&self) -> Vec<&RequiredTool> {
        self.required_tools
            .iter()
            .filter(|t| !t.is_available)
            .collect()
    }

    /// Check if the project has any issues
    pub fn has_issues(&self) -> bool {
        !self.missing_dependencies().is_empty() || !self.missing_tools().is_empty()
    }
}

/// Script information detected from project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// Script name
    pub name: String,

    /// Script command
    pub command: String,

    /// Where the script was defined
    pub source: ScriptSource,

    /// Tools used by this script
    pub tools: Vec<ScriptTool>,

    /// Description of the script
    pub description: Option<String>,
}

impl Script {
    /// Create a new script
    pub fn new(name: impl Into<String>, command: impl Into<String>, source: ScriptSource) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            source,
            tools: Vec::new(),
            description: None,
        }
    }

    /// Check if all tools for this script are available
    pub fn all_tools_available(&self) -> bool {
        self.tools.iter().all(|t| t.is_available)
    }

    /// Get missing tools for this script
    pub fn missing_tools(&self) -> Vec<&ScriptTool> {
        self.tools.iter().filter(|t| !t.is_available).collect()
    }
}

/// Source of a script definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptSource {
    /// From vx.toml
    VxConfig,

    /// From pyproject.toml [project.scripts] or [tool.uv.scripts]
    PyprojectToml { section: String },

    /// From package.json scripts
    PackageJson,

    /// From Cargo.toml
    CargoToml,

    /// From Makefile
    Makefile,

    /// From justfile (language-agnostic task runner)
    Justfile,

    /// Auto-detected (e.g., noxfile.py exists)
    Detected { reason: String },
}

impl std::fmt::Display for ScriptSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptSource::VxConfig => write!(f, "vx.toml"),
            ScriptSource::PyprojectToml { section } => write!(f, "pyproject.toml [{}]", section),
            ScriptSource::PackageJson => write!(f, "package.json"),
            ScriptSource::CargoToml => write!(f, "Cargo.toml"),
            ScriptSource::Makefile => write!(f, "Makefile"),
            ScriptSource::Justfile => write!(f, "justfile"),
            ScriptSource::Detected { reason } => write!(f, "detected ({})", reason),
        }
    }
}

/// Required tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredTool {
    /// Tool name
    pub name: String,

    /// Required version (if specified)
    pub version: Option<String>,

    /// Ecosystem this tool belongs to
    pub ecosystem: Ecosystem,

    /// Why this tool is needed
    pub reason: String,

    /// How to install this tool
    pub install_method: InstallMethod,

    /// Whether the tool is currently available
    pub is_available: bool,
}

impl RequiredTool {
    /// Create a new required tool
    pub fn new(
        name: impl Into<String>,
        ecosystem: Ecosystem,
        reason: impl Into<String>,
        install_method: InstallMethod,
    ) -> Self {
        Self {
            name: name.into(),
            version: None,
            ecosystem,
            reason: reason.into(),
            install_method,
            is_available: false,
        }
    }

    /// Set the version requirement
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Mark as available
    pub fn available(mut self) -> Self {
        self.is_available = true;
        self
    }
}
