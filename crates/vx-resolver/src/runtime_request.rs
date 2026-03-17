//! Runtime request parsing
//!
//! This module provides parsing for runtime specifications with optional version constraints.
//! Supports formats like:
//! - `yarn` - runtime name only, use latest/default version
//! - `yarn@1.21.1` - runtime with exact version
//! - `node@20` - runtime with major version constraint
//! - `node@^18.0.0` - runtime with semver constraint
//! - `msvc::cl` - runtime with executable override
//! - `msvc@14.42::cl` - runtime with version and executable override
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
    /// - `runtime@version::executable` - runtime with version and executable override
    /// - `runtime::shell` - runtime with shell launch (if shell is a known shell name)
    /// - `runtime@version::shell` - runtime with version and shell launch
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
    /// // Canonical format: runtime@version::executable
    /// let req = RuntimeRequest::parse("msvc@14.42::cl");
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
        if let Some((runtime_part, exe_or_shell)) = spec.split_once("::") {
            // Parse version from runtime part (canonical: runtime@version::executable)
            let (name, version, exe_or_shell) =
                if let Some((name, version)) = runtime_part.split_once('@') {
                    let version = if version.is_empty() {
                        None
                    } else {
                        Some(version.to_string())
                    };
                    (name, version, exe_or_shell)
                } else if let Some((exe_or_shell, version)) = exe_or_shell.rsplit_once('@') {
                    // Compatibility form: runtime::executable@version
                    let version = if version.is_empty() {
                        None
                    } else {
                        Some(version.to_string())
                    };
                    (runtime_part, version, exe_or_shell)
                } else {
                    (runtime_part, None, exe_or_shell)
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
                name: name.to_string(),
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
        // Output canonical format: runtime[@version][::executable_or_shell]
        write!(f, "{}", self.name)?;
        if let Some(ref version) = self.version {
            write!(f, "@{}", version)?;
        }
        if let Some(ref shell) = self.shell {
            write!(f, "::{}", shell)?;
        } else if let Some(ref exe) = self.executable {
            write!(f, "::{}", exe)?;
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
