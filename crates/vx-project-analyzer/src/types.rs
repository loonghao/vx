//! Core type definitions for project analysis

use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::script_parser::ScriptTool;
use crate::sync::SyncAction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Re-export framework types
pub use crate::frameworks::FrameworkInfo;

/// Complete project analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    /// Project root directory
    pub root: PathBuf,

    /// Detected ecosystems
    pub ecosystems: Vec<Ecosystem>,

    /// Detected application frameworks (Electron, Tauri, etc.)
    pub frameworks: Vec<FrameworkInfo>,

    /// All detected dependencies
    pub dependencies: Vec<Dependency>,

    /// All detected scripts
    pub scripts: Vec<Script>,

    /// Required tools (runtimes that need to be installed)
    pub required_tools: Vec<RequiredTool>,

    /// Suggested sync actions
    pub sync_actions: Vec<SyncAction>,

    /// Audit findings (security, best practices, etc.)
    pub audit_findings: Vec<AuditFinding>,
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

    /// Check if there are any critical audit findings
    pub fn has_critical_audits(&self) -> bool {
        self.audit_findings
            .iter()
            .any(|f| f.severity == AuditSeverity::Critical)
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

    /// Additional metadata for detailed tool configuration.
    /// Used to pass extra info like MSVC components, OS restrictions, etc.
    /// to the sync system for generating rich vx.toml entries.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Vec<String>>,
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
            metadata: HashMap::new(),
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

    /// Add metadata key-value pair (for detailed tool config generation)
    pub fn with_metadata(mut self, key: impl Into<String>, values: Vec<String>) -> Self {
        self.metadata.insert(key.into(), values);
        self
    }

    /// Add OS restrictions
    pub fn with_os(self, os_list: Vec<String>) -> Self {
        self.with_metadata("os", os_list)
    }

    /// Add MSVC components requirement
    pub fn with_components(self, components: Vec<String>) -> Self {
        self.with_metadata("components", components)
    }
}

/// Severity level for audit findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Informational finding
    Info,
    /// Warning - should be addressed
    Warning,
    /// Error - must be addressed
    Error,
    /// Critical - security or stability issue
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Info => write!(f, "info"),
            AuditSeverity::Warning => write!(f, "warning"),
            AuditSeverity::Error => write!(f, "error"),
            AuditSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// An audit finding from project analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    /// Severity of the finding
    pub severity: AuditSeverity,

    /// Short title describing the issue
    pub title: String,

    /// Detailed description of the issue
    pub detail: String,

    /// Suggested fix (if available)
    pub suggestion: Option<String>,

    /// Related file path (if applicable)
    pub file_path: Option<PathBuf>,
}

impl AuditFinding {
    /// Create a new audit finding
    pub fn new(
        severity: AuditSeverity,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            title: title.into(),
            detail: detail.into(),
            suggestion: None,
            file_path: None,
        }
    }

    /// Add a suggestion for fixing the issue
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add a related file path
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}
