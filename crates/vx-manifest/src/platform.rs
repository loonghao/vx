//! Platform constraint types for Provider/Runtime platform awareness
//!
//! This module provides types for declaring platform constraints in `provider.toml`,
//! enabling vx to:
//! - Display platform compatibility information in `vx list`
//! - Provide friendly error messages on unsupported platforms
//! - Optionally filter providers at compile time

use serde::{Deserialize, Serialize};
use std::fmt;

/// Operating system types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    /// Microsoft Windows
    Windows,
    /// Apple macOS
    #[serde(alias = "darwin")]
    MacOS,
    /// Linux distributions
    Linux,
}

impl Os {
    /// Get the current operating system
    #[cfg(target_os = "windows")]
    pub fn current() -> Self {
        Os::Windows
    }

    /// Get the current operating system
    #[cfg(target_os = "macos")]
    pub fn current() -> Self {
        Os::MacOS
    }

    /// Get the current operating system
    #[cfg(target_os = "linux")]
    pub fn current() -> Self {
        Os::Linux
    }

    /// Get the current operating system (fallback for other platforms)
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn current() -> Self {
        // Default to Linux for Unix-like systems
        Os::Linux
    }

    /// Get the OS name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Os::Windows => "Windows",
            Os::MacOS => "macOS",
            Os::Linux => "Linux",
        }
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// CPU architecture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    /// 64-bit x86 (AMD64/Intel 64)
    #[serde(alias = "amd64", alias = "x64")]
    X86_64,
    /// 64-bit ARM (Apple Silicon, ARM64)
    #[serde(alias = "arm64")]
    Aarch64,
    /// 32-bit x86
    #[serde(alias = "i686", alias = "i386")]
    X86,
}

impl Arch {
    /// Get the current architecture
    #[cfg(target_arch = "x86_64")]
    pub fn current() -> Self {
        Arch::X86_64
    }

    /// Get the current architecture
    #[cfg(target_arch = "aarch64")]
    pub fn current() -> Self {
        Arch::Aarch64
    }

    /// Get the current architecture
    #[cfg(target_arch = "x86")]
    pub fn current() -> Self {
        Arch::X86
    }

    /// Get the current architecture (fallback)
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
    pub fn current() -> Self {
        Arch::X86_64
    }

    /// Get the architecture name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
            Arch::X86 => "x86",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Platform exclusion rule
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct PlatformExclusion {
    /// Operating system to exclude
    #[serde(default)]
    pub os: Option<Os>,
    /// Architecture to exclude
    #[serde(default)]
    pub arch: Option<Arch>,
}

/// Platform constraint definition
///
/// Used to declare which platforms a Provider or Runtime supports.
/// If no constraints are specified, all platforms are supported.
///
/// # Example
///
/// ```toml
/// [provider.platforms]
/// os = ["windows"]  # Windows only
///
/// # Or with architecture constraints
/// [provider.platforms]
/// os = ["windows", "linux"]
/// arch = ["x86_64", "aarch64"]
/// exclude = [{ os = "linux", arch = "x86" }]
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct PlatformConstraint {
    /// Supported operating systems (empty = all)
    #[serde(default)]
    pub os: Vec<Os>,

    /// Supported architectures (empty = all)
    #[serde(default)]
    pub arch: Vec<Arch>,

    /// Excluded platform combinations
    #[serde(default)]
    pub exclude: Vec<PlatformExclusion>,
}

impl PlatformConstraint {
    /// Create a new empty constraint (all platforms supported)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a constraint for Windows only
    pub fn windows_only() -> Self {
        Self {
            os: vec![Os::Windows],
            ..Default::default()
        }
    }

    /// Create a constraint for macOS only
    pub fn macos_only() -> Self {
        Self {
            os: vec![Os::MacOS],
            ..Default::default()
        }
    }

    /// Create a constraint for Linux only
    pub fn linux_only() -> Self {
        Self {
            os: vec![Os::Linux],
            ..Default::default()
        }
    }

    /// Create a constraint for Unix (macOS + Linux)
    pub fn unix_only() -> Self {
        Self {
            os: vec![Os::MacOS, Os::Linux],
            ..Default::default()
        }
    }

    /// Check if the constraint is empty (all platforms supported)
    pub fn is_empty(&self) -> bool {
        self.os.is_empty() && self.arch.is_empty() && self.exclude.is_empty()
    }

    /// Check if the current platform is supported
    pub fn is_current_platform_supported(&self) -> bool {
        self.is_platform_supported(Os::current(), Arch::current())
    }

    /// Check if a specific platform is supported
    pub fn is_platform_supported(&self, os: Os, arch: Arch) -> bool {
        // If no constraints specified, all platforms are supported
        if self.is_empty() {
            return true;
        }

        // Check OS constraint
        if !self.os.is_empty() && !self.os.contains(&os) {
            return false;
        }

        // Check architecture constraint
        if !self.arch.is_empty() && !self.arch.contains(&arch) {
            return false;
        }

        // Check exclusion list
        for exclusion in &self.exclude {
            let os_match = exclusion.os.is_none_or(|o| o == os);
            let arch_match = exclusion.arch.is_none_or(|a| a == arch);
            if os_match && arch_match {
                return false;
            }
        }

        true
    }

    /// Generate a human-readable platform description
    ///
    /// Returns `None` if all platforms are supported.
    pub fn description(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        // Single OS case
        if self.os.len() == 1 && self.arch.is_empty() && self.exclude.is_empty() {
            return Some(format!("{} only", self.os[0]));
        }

        // Multiple OS case
        if !self.os.is_empty() && self.arch.is_empty() && self.exclude.is_empty() {
            let names: Vec<_> = self.os.iter().map(|o| o.as_str()).collect();
            return Some(format!("{} only", names.join("/")));
        }

        // Complex case with architecture or exclusions
        let mut parts = Vec::new();

        if !self.os.is_empty() {
            let names: Vec<_> = self.os.iter().map(|o| o.as_str()).collect();
            parts.push(format!("OS: {}", names.join(", ")));
        }

        if !self.arch.is_empty() {
            let names: Vec<_> = self.arch.iter().map(|a| a.as_str()).collect();
            parts.push(format!("Arch: {}", names.join(", ")));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join("; "))
        }
    }

    /// Get a short label for display (e.g., "Windows", "macOS/Linux")
    pub fn short_label(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        if self.os.len() == 1 {
            return Some(self.os[0].as_str().to_string());
        }

        if !self.os.is_empty() {
            let names: Vec<_> = self.os.iter().map(|o| o.as_str()).collect();
            return Some(names.join("/"));
        }

        None
    }
}

/// Current platform information
#[derive(Debug, Clone, Copy)]
pub struct Platform {
    /// Current operating system
    pub os: Os,
    /// Current architecture
    pub arch: Arch,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.os.as_str().to_lowercase(), self.arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_current() {
        let os = Os::current();
        // Should return one of the valid OS types
        assert!(matches!(os, Os::Windows | Os::MacOS | Os::Linux));
    }

    #[test]
    fn test_arch_current() {
        let arch = Arch::current();
        // Should return one of the valid arch types
        assert!(matches!(arch, Arch::X86_64 | Arch::Aarch64 | Arch::X86));
    }

    #[test]
    fn test_empty_constraint_supports_all() {
        let constraint = PlatformConstraint::new();
        assert!(constraint.is_empty());
        assert!(constraint.is_platform_supported(Os::Windows, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::MacOS, Arch::Aarch64));
        assert!(constraint.is_platform_supported(Os::Linux, Arch::X86));
        assert!(constraint.description().is_none());
    }

    #[test]
    fn test_windows_only() {
        let constraint = PlatformConstraint::windows_only();
        assert!(constraint.is_platform_supported(Os::Windows, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::Windows, Arch::Aarch64));
        assert!(!constraint.is_platform_supported(Os::MacOS, Arch::X86_64));
        assert!(!constraint.is_platform_supported(Os::Linux, Arch::X86_64));
        assert_eq!(constraint.description(), Some("Windows only".to_string()));
    }

    #[test]
    fn test_macos_only() {
        let constraint = PlatformConstraint::macos_only();
        assert!(!constraint.is_platform_supported(Os::Windows, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::MacOS, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::MacOS, Arch::Aarch64));
        assert!(!constraint.is_platform_supported(Os::Linux, Arch::X86_64));
        assert_eq!(constraint.description(), Some("macOS only".to_string()));
    }

    #[test]
    fn test_unix_only() {
        let constraint = PlatformConstraint::unix_only();
        assert!(!constraint.is_platform_supported(Os::Windows, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::MacOS, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::Linux, Arch::X86_64));
        assert_eq!(
            constraint.description(),
            Some("macOS/Linux only".to_string())
        );
    }

    #[test]
    fn test_arch_constraint() {
        let constraint = PlatformConstraint {
            os: vec![],
            arch: vec![Arch::X86_64, Arch::Aarch64],
            exclude: vec![],
        };
        assert!(constraint.is_platform_supported(Os::Windows, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::Linux, Arch::Aarch64));
        assert!(!constraint.is_platform_supported(Os::Windows, Arch::X86));
    }

    #[test]
    fn test_exclusion() {
        let constraint = PlatformConstraint {
            os: vec![Os::Windows, Os::Linux],
            arch: vec![],
            exclude: vec![PlatformExclusion {
                os: Some(Os::Linux),
                arch: Some(Arch::X86),
            }],
        };
        assert!(constraint.is_platform_supported(Os::Windows, Arch::X86_64));
        assert!(constraint.is_platform_supported(Os::Windows, Arch::X86));
        assert!(constraint.is_platform_supported(Os::Linux, Arch::X86_64));
        assert!(!constraint.is_platform_supported(Os::Linux, Arch::X86)); // Excluded
        assert!(!constraint.is_platform_supported(Os::MacOS, Arch::X86_64)); // Not in OS list
    }

    #[test]
    fn test_serde_os() {
        let os: Os = serde_json::from_str(r#""windows""#).unwrap();
        assert_eq!(os, Os::Windows);

        let os: Os = serde_json::from_str(r#""macos""#).unwrap();
        assert_eq!(os, Os::MacOS);

        let os: Os = serde_json::from_str(r#""darwin""#).unwrap();
        assert_eq!(os, Os::MacOS);

        let os: Os = serde_json::from_str(r#""linux""#).unwrap();
        assert_eq!(os, Os::Linux);
    }

    #[test]
    fn test_serde_arch() {
        let arch: Arch = serde_json::from_str(r#""x86_64""#).unwrap();
        assert_eq!(arch, Arch::X86_64);

        let arch: Arch = serde_json::from_str(r#""amd64""#).unwrap();
        assert_eq!(arch, Arch::X86_64);

        let arch: Arch = serde_json::from_str(r#""aarch64""#).unwrap();
        assert_eq!(arch, Arch::Aarch64);

        let arch: Arch = serde_json::from_str(r#""arm64""#).unwrap();
        assert_eq!(arch, Arch::Aarch64);
    }

    #[test]
    fn test_platform_constraint_toml() {
        let toml = r#"
            os = ["windows"]
        "#;
        let constraint: PlatformConstraint = toml::from_str(toml).unwrap();
        assert_eq!(constraint.os, vec![Os::Windows]);
        assert!(constraint.arch.is_empty());
    }

    #[test]
    fn test_short_label() {
        assert_eq!(
            PlatformConstraint::windows_only().short_label(),
            Some("Windows".to_string())
        );
        assert_eq!(
            PlatformConstraint::macos_only().short_label(),
            Some("macOS".to_string())
        );
        assert_eq!(
            PlatformConstraint::unix_only().short_label(),
            Some("macOS/Linux".to_string())
        );
        assert_eq!(PlatformConstraint::new().short_label(), None);
    }
}
