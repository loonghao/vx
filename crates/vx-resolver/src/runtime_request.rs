//! Runtime request parsing
//!
//! This module provides parsing for runtime specifications with optional version constraints.
//! Supports formats like:
//! - `yarn` - runtime name only, use latest/default version
//! - `yarn@1.21.1` - runtime with exact version
//! - `node@20` - runtime with major version constraint
//! - `node@^18.0.0` - runtime with semver constraint
//! - `msvc::cl` - runtime with executable override
//! - `msvc::cl@14.42` - runtime with executable override and version
//! - `git::git-bash` - runtime with shell launch (launches Git Bash with git's environment)

use std::fmt;

/// Known shell executables that can be launched with runtime environment
const KNOWN_SHELLS: &[&str] = &[
    "cmd",
    "powershell",
    "pwsh",
    "bash",
    "sh",
    "zsh",
    "fish",
    "dash",
    "ksh",
    "csh",
    "tcsh",
    // Platform-specific shells
    "git-bash",
    "git-cmd",
    "cmd.exe",
    "powershell.exe",
];

/// Check if a name is a known shell executable
fn is_known_shell(name: &str) -> bool {
    KNOWN_SHELLS.contains(&name.to_lowercase().as_str())
}

/// A parsed runtime request with optional version constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequest {
    /// The runtime name (e.g., "yarn", "node", "npm", "msvc")
    pub name: String,

    /// Optional version constraint (e.g., "1.21.1", "20", "^18.0.0")
    pub version: Option<String>,

    /// Optional executable override (e.g., "cl" for `msvc::cl`)
    ///
    /// When set, the resolver will search for this executable name instead of
    /// the runtime's default executable. The runtime name is still used for
    /// store directory lookup, dependency resolution, and installation.
    pub executable: Option<String>,

    /// Optional shell to launch with runtime environment
    /// When set, instead of running an executable, we launch a shell
    /// with the runtime's environment configured.
    pub shell: Option<String>,
}

impl RuntimeRequest {
    /// Create a new runtime request with just a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            executable: None,
            shell: None,
        }
    }

    /// Create a new runtime request with a specific version
    pub fn with_version(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: Some(version.into()),
            executable: None,
            shell: None,
        }
    }

    /// Parse a runtime request from a string
    ///
    /// Supports formats:
    /// - `runtime` - just the runtime name
    /// - `runtime@version` - runtime with version constraint
    /// - `runtime::executable` - runtime with executable override
    /// - `runtime::executable@version` - runtime with executable override and version
    /// - `runtime::shell` - runtime with shell launch (if shell is a known shell name)
    ///
    /// # Examples
    ///
    /// ```
    /// use vx_resolver::RuntimeRequest;
    ///
    /// let req = RuntimeRequest::parse("yarn");
    /// assert_eq!(req.name, "yarn");
    /// assert_eq!(req.version, None);
    /// assert_eq!(req.executable, None);
    /// assert_eq!(req.shell, None);
    ///
    /// let req = RuntimeRequest::parse("yarn@1.21.1");
    /// assert_eq!(req.name, "yarn");
    /// assert_eq!(req.version, Some("1.21.1".to_string()));
    ///
    /// let req = RuntimeRequest::parse("msvc::cl");
    /// assert_eq!(req.name, "msvc");
    /// assert_eq!(req.executable, Some("cl".to_string()));
    /// assert_eq!(req.version, None);
    ///
    /// let req = RuntimeRequest::parse("msvc::cl@14.42");
    /// assert_eq!(req.name, "msvc");
    /// assert_eq!(req.executable, Some("cl".to_string()));
    /// assert_eq!(req.version, Some("14.42".to_string()));
    ///
    /// let req = RuntimeRequest::parse("git::git-bash");
    /// assert_eq!(req.name, "git");
    /// assert_eq!(req.shell, Some("git-bash".to_string()));
    /// ```
    pub fn parse(spec: &str) -> Self {
        // Check for `::` executable/shell override syntax first
        // Format: runtime::executable_or_shell[@version]
        if let Some((runtime_part, exe_and_version)) = spec.split_once("::") {
            // Parse version from exe part: executable_or_shell[@version]
            let (exe_or_shell, version) =
                if let Some((exe, version)) = exe_and_version.split_once('@') {
                    (
                        exe,
                        if version.is_empty() {
                            None
                        } else {
                            Some(version.to_string())
                        },
                    )
                } else {
                    (exe_and_version, None)
                };

            // Determine if this is a shell or an executable
            let (executable, shell) = if exe_or_shell.is_empty() {
                (None, None)
            } else if is_known_shell(exe_or_shell) {
                // It's a shell
                (None, Some(exe_or_shell.to_string()))
            } else {
                // It's an executable
                (Some(exe_or_shell.to_string()), None)
            };

            return Self {
                name: runtime_part.to_string(),
                version,
                executable,
                shell,
            };
        }

        // Standard format: runtime[@version]
        if let Some((name, version)) = spec.split_once('@') {
            Self {
                name: name.to_string(),
                version: if version.is_empty() {
                    None
                } else {
                    Some(version.to_string())
                },
                executable: None,
                shell: None,
            }
        } else {
            Self {
                name: spec.to_string(),
                version: None,
                executable: None,
                shell: None,
            }
        }
    }

    /// Check if a specific version is requested
    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    /// Get the version constraint or "latest" as default
    pub fn version_or_latest(&self) -> &str {
        self.version.as_deref().unwrap_or("latest")
    }

    /// Check if this request wants to launch a shell with runtime environment
    pub fn is_shell_request(&self) -> bool {
        self.shell.is_some()
    }

    /// Get the shell name if this is a shell request
    pub fn shell_name(&self) -> Option<&str> {
        self.shell.as_deref()
    }
}

impl fmt::Display for RuntimeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ref shell) = self.shell {
            write!(f, "::{}", shell)?;
        } else if let Some(ref exe) = self.executable {
            write!(f, "::{}", exe)?;
        }
        if let Some(ref version) = self.version {
            write!(f, "@{}", version)?;
        }
        Ok(())
    }
}

impl From<&str> for RuntimeRequest {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl From<String> for RuntimeRequest {
    fn from(s: String) -> Self {
        Self::parse(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_only() {
        let req = RuntimeRequest::parse("yarn");
        assert_eq!(req.name, "yarn");
        assert_eq!(req.version, None);
        assert_eq!(req.executable, None);
    }

    #[test]
    fn test_parse_with_exact_version() {
        let req = RuntimeRequest::parse("yarn@1.21.1");
        assert_eq!(req.name, "yarn");
        assert_eq!(req.version, Some("1.21.1".to_string()));
        assert_eq!(req.executable, None);
    }

    #[test]
    fn test_parse_with_major_version() {
        let req = RuntimeRequest::parse("node@20");
        assert_eq!(req.name, "node");
        assert_eq!(req.version, Some("20".to_string()));
    }

    #[test]
    fn test_parse_with_semver_constraint() {
        let req = RuntimeRequest::parse("node@^18.0.0");
        assert_eq!(req.name, "node");
        assert_eq!(req.version, Some("^18.0.0".to_string()));
    }

    #[test]
    fn test_parse_empty_version() {
        let req = RuntimeRequest::parse("yarn@");
        assert_eq!(req.name, "yarn");
        assert_eq!(req.version, None);
    }

    #[test]
    fn test_parse_executable_override() {
        let req = RuntimeRequest::parse("msvc::cl");
        assert_eq!(req.name, "msvc");
        assert_eq!(req.executable, Some("cl".to_string()));
        assert_eq!(req.version, None);
    }

    #[test]
    fn test_parse_executable_override_with_version() {
        let req = RuntimeRequest::parse("msvc::cl@14.42");
        assert_eq!(req.name, "msvc");
        assert_eq!(req.executable, Some("cl".to_string()));
        assert_eq!(req.version, Some("14.42".to_string()));
    }

    #[test]
    fn test_parse_executable_override_empty_exe() {
        let req = RuntimeRequest::parse("msvc::@14.42");
        assert_eq!(req.name, "msvc");
        assert_eq!(req.executable, None);
        assert_eq!(req.version, Some("14.42".to_string()));
    }

    #[test]
    fn test_parse_executable_override_empty_all() {
        let req = RuntimeRequest::parse("msvc::");
        assert_eq!(req.name, "msvc");
        assert_eq!(req.executable, None);
        assert_eq!(req.version, None);
    }

    #[test]
    fn test_display() {
        let req = RuntimeRequest::with_version("yarn", "1.21.1");
        assert_eq!(format!("{}", req), "yarn@1.21.1");

        let req = RuntimeRequest::new("yarn");
        assert_eq!(format!("{}", req), "yarn");

        // With executable override
        let req = RuntimeRequest {
            name: "msvc".to_string(),
            executable: Some("cl".to_string()),
            version: Some("14.42".to_string()),
            shell: None,
        };
        assert_eq!(format!("{}", req), "msvc::cl@14.42");

        let req = RuntimeRequest {
            name: "msvc".to_string(),
            executable: Some("cl".to_string()),
            version: None,
            shell: None,
        };
        assert_eq!(format!("{}", req), "msvc::cl");
    }

    #[test]
    fn test_version_or_latest() {
        let req = RuntimeRequest::new("yarn");
        assert_eq!(req.version_or_latest(), "latest");

        let req = RuntimeRequest::with_version("yarn", "1.21.1");
        assert_eq!(req.version_or_latest(), "1.21.1");
    }

    // Tests for shell syntax (runtime::shell)

    #[test]
    fn test_parse_git_bash_shell() {
        // git::git-bash - launch Git Bash with git's environment
        let req = RuntimeRequest::parse("git::git-bash");
        assert_eq!(req.name, "git");
        assert_eq!(req.shell, Some("git-bash".to_string()));
        assert_eq!(req.executable, None);
        assert_eq!(req.version, None);
        assert!(req.is_shell_request());
        assert_eq!(req.shell_name(), Some("git-bash"));
    }

    #[test]
    fn test_parse_cmd_shell() {
        // git::cmd - launch cmd with git's environment
        let req = RuntimeRequest::parse("git::cmd");
        assert_eq!(req.name, "git");
        assert_eq!(req.shell, Some("cmd".to_string()));
        assert_eq!(req.executable, None);
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_parse_powershell_shell() {
        // node::powershell - launch powershell with node's environment
        let req = RuntimeRequest::parse("node::powershell");
        assert_eq!(req.name, "node");
        assert_eq!(req.shell, Some("powershell".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_parse_bash_shell() {
        // go::bash - launch bash with go's environment
        let req = RuntimeRequest::parse("go::bash");
        assert_eq!(req.name, "go");
        assert_eq!(req.shell, Some("bash".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_parse_shell_with_version() {
        // git::git-bash@2.43 - specific git version with shell
        let req = RuntimeRequest::parse("git::git-bash@2.43");
        assert_eq!(req.name, "git");
        assert_eq!(req.shell, Some("git-bash".to_string()));
        assert_eq!(req.version, Some("2.43".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_executable_vs_shell_distinction() {
        // "cl" is NOT a known shell, so it should be treated as executable
        let req = RuntimeRequest::parse("msvc::cl");
        assert_eq!(req.executable, Some("cl".to_string()));
        assert_eq!(req.shell, None);
        assert!(!req.is_shell_request());

        // "cmd" IS a known shell
        let req = RuntimeRequest::parse("msvc::cmd");
        assert_eq!(req.executable, None);
        assert_eq!(req.shell, Some("cmd".to_string()));
        assert!(req.is_shell_request());
    }

    #[test]
    fn test_all_known_shells() {
        for shell in KNOWN_SHELLS {
            let req = RuntimeRequest::parse(&format!("test::{}", shell));
            assert_eq!(req.shell, Some(shell.to_string()));
            assert!(req.is_shell_request());
        }
    }

    #[test]
    fn test_display_with_shell() {
        let req = RuntimeRequest {
            name: "git".to_string(),
            shell: Some("git-bash".to_string()),
            version: Some("2.43".to_string()),
            executable: None,
        };
        assert_eq!(format!("{}", req), "git::git-bash@2.43");

        let req = RuntimeRequest {
            name: "git".to_string(),
            shell: Some("git-bash".to_string()),
            version: None,
            executable: None,
        };
        assert_eq!(format!("{}", req), "git::git-bash");
    }
}
