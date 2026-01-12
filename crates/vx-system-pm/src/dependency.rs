//! System dependency definitions

use serde::{Deserialize, Serialize};

/// System dependency definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemDependency {
    /// Dependency type
    #[serde(rename = "type")]
    pub dep_type: SystemDepType,

    /// Dependency identifier (e.g., "kb2919355", "vcredist140", "git")
    pub id: String,

    /// Version constraint (optional, semver syntax)
    #[serde(default)]
    pub version: Option<String>,

    /// Reason for this dependency
    #[serde(default)]
    pub reason: Option<String>,

    /// Platform conditions (e.g., ["windows"], ["linux", "macos"])
    #[serde(default)]
    pub platforms: Vec<String>,

    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

impl SystemDependency {
    /// Create a new system dependency
    pub fn new(dep_type: SystemDepType, id: impl Into<String>) -> Self {
        Self {
            dep_type,
            id: id.into(),
            version: None,
            reason: None,
            platforms: Vec::new(),
            optional: false,
        }
    }

    /// Set version constraint
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Set platforms
    pub fn with_platforms(mut self, platforms: Vec<String>) -> Self {
        self.platforms = platforms;
        self
    }

    /// Set as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Check if this dependency applies to the current platform
    pub fn matches_current_platform(&self) -> bool {
        if self.platforms.is_empty() {
            return true;
        }

        let current_os = std::env::consts::OS;
        self.platforms.iter().any(|p| {
            p == current_os || p == "*" || (p == "unix" && (current_os == "linux" || current_os == "macos"))
        })
    }
}

/// System dependency types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemDepType {
    /// Windows KB update (e.g., KB2919355)
    WindowsKb,

    /// Windows Feature (DISM feature)
    WindowsFeature,

    /// Visual C++ Redistributable
    VcRedist,

    /// .NET Framework or .NET Runtime
    DotNet,

    /// System package (installed via package manager)
    Package,

    /// Another vx-managed runtime
    Runtime,
}

impl std::fmt::Display for SystemDepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowsKb => write!(f, "Windows KB"),
            Self::WindowsFeature => write!(f, "Windows Feature"),
            Self::VcRedist => write!(f, "VC++ Redistributable"),
            Self::DotNet => write!(f, ".NET"),
            Self::Package => write!(f, "Package"),
            Self::Runtime => write!(f, "Runtime"),
        }
    }
}

/// System dependencies configuration for a runtime
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SystemDepsConfig {
    /// Pre-dependencies (must be satisfied before installation)
    #[serde(default)]
    pub pre_depends: Vec<SystemDependency>,

    /// Runtime dependencies
    #[serde(default)]
    pub depends: Vec<SystemDependency>,

    /// Recommended dependencies
    #[serde(default)]
    pub recommends: Vec<SystemDependency>,

    /// Optional/suggested dependencies
    #[serde(default)]
    pub suggests: Vec<SystemDependency>,
}

impl SystemDepsConfig {
    /// Get all required dependencies (pre_depends + depends)
    pub fn all_required(&self) -> Vec<&SystemDependency> {
        self.pre_depends
            .iter()
            .chain(self.depends.iter())
            .filter(|d| !d.optional)
            .collect()
    }

    /// Get all dependencies for the current platform
    pub fn for_current_platform(&self) -> SystemDepsConfig {
        SystemDepsConfig {
            pre_depends: self
                .pre_depends
                .iter()
                .filter(|d| d.matches_current_platform())
                .cloned()
                .collect(),
            depends: self
                .depends
                .iter()
                .filter(|d| d.matches_current_platform())
                .cloned()
                .collect(),
            recommends: self
                .recommends
                .iter()
                .filter(|d| d.matches_current_platform())
                .cloned()
                .collect(),
            suggests: self
                .suggests
                .iter()
                .filter(|d| d.matches_current_platform())
                .cloned()
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_dependency_creation() {
        let dep = SystemDependency::new(SystemDepType::VcRedist, "vcredist140")
            .with_version(">=14.0")
            .with_reason("Required for C++ runtime")
            .with_platforms(vec!["windows".to_string()]);

        assert_eq!(dep.id, "vcredist140");
        assert_eq!(dep.dep_type, SystemDepType::VcRedist);
        assert_eq!(dep.version, Some(">=14.0".to_string()));
        assert!(!dep.optional);
    }

    #[test]
    fn test_platform_matching() {
        let dep = SystemDependency::new(SystemDepType::Package, "git")
            .with_platforms(vec!["windows".to_string()]);

        #[cfg(windows)]
        assert!(dep.matches_current_platform());

        #[cfg(not(windows))]
        assert!(!dep.matches_current_platform());

        // Empty platforms means all platforms
        let universal_dep = SystemDependency::new(SystemDepType::Package, "curl");
        assert!(universal_dep.matches_current_platform());
    }
}
