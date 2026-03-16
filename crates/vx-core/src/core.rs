//! # vx-core - Core abstractions and utilities
//!
//! This module provides essential types and utilities shared across the vx codebase.
//!
//! ## Actively Used Types
//!
//! - `WithDependency` — Runtime dependency spec for `--with` flag
//! - `is_ctrl_c_exit` / `exit_code_from_status` — Process exit status utilities
//! - `is_latest_version` / `resolve_latest_version` — Version resolution helpers
//!
//! ## Design Note
//!
//! Runtime and Provider abstractions are defined in `vx-runtime` (the `Runtime` trait)
//! and `vx-starlark` (the provider.star DSL engine). This crate only contains
//! foundational utilities that have no heavyweight dependencies.

use serde::{Deserialize, Serialize};
use std::process::ExitStatus;

// ============================================================================
// Runtime Dependency (--with flag)
// ============================================================================

/// Runtime dependency specification (used for --with flag)
///
/// This represents a runtime that should be injected into the environment
/// when executing a tool. Similar to uvx --with or rez-env.
///
/// # Example
///
/// ```rust
/// use vx_core::WithDependency;
///
/// // Parse "bun@1.1.0"
/// let dep = WithDependency::parse("bun@1.1.0");
/// assert_eq!(dep.runtime, "bun");
/// assert_eq!(dep.version, Some("1.1.0".to_string()));
///
/// // Parse "deno" (no version)
/// let dep = WithDependency::parse("deno");
/// assert_eq!(dep.runtime, "deno");
/// assert_eq!(dep.version, None);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithDependency {
    /// Runtime name (e.g., "bun", "deno", "node")
    pub runtime: String,
    /// Optional version constraint (e.g., "1.1.0", "latest")
    pub version: Option<String>,
}

impl WithDependency {
    /// Create a new dependency with runtime name and optional version
    pub fn new(runtime: impl Into<String>, version: Option<String>) -> Self {
        Self {
            runtime: runtime.into(),
            version,
        }
    }

    /// Parse a dependency spec from string (e.g., "bun@1.1.0" or "deno")
    pub fn parse(spec: &str) -> Self {
        if let Some((runtime, version)) = spec.split_once('@') {
            Self {
                runtime: runtime.to_string(),
                version: Some(version.to_string()),
            }
        } else {
            Self {
                runtime: spec.to_string(),
                version: None,
            }
        }
    }

    /// Parse multiple dependency specs
    pub fn parse_many(specs: &[String]) -> Vec<Self> {
        specs.iter().map(|s| Self::parse(s)).collect()
    }
}

impl std::fmt::Display for WithDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref version) = self.version {
            write!(f, "{}@{}", self.runtime, version)
        } else {
            write!(f, "{}", self.runtime)
        }
    }
}

// ============================================================================
// Process Exit Status Utilities
// ============================================================================

/// Check if an exit status indicates the process was terminated by Ctrl+C
///
/// On Windows, STATUS_CONTROL_C_EXIT (0xC000013A) indicates Ctrl+C termination.
/// On Unix, signal 2 (SIGINT) indicates Ctrl+C termination.
///
/// # Example
///
/// ```rust,ignore
/// use std::process::Command;
/// use vx_core::is_ctrl_c_exit;
///
/// let status = Command::new("some_command").status().unwrap();
/// if is_ctrl_c_exit(&status) {
///     // Process was terminated by Ctrl+C
/// }
/// ```
pub fn is_ctrl_c_exit(status: &ExitStatus) -> bool {
    #[cfg(windows)]
    {
        // Windows STATUS_CONTROL_C_EXIT = 0xC000013A = 3221225786
        // This is returned as a negative i32 when cast: -1073741510
        if let Some(code) = status.code() {
            // Check both the unsigned and signed representations
            code == -1073741510 || code as u32 == 0xC000013A
        } else {
            false
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        // SIGINT = 2
        status.signal() == Some(2)
    }
}

/// Convert an exit status to an appropriate exit code
///
/// This handles special cases like Ctrl+C termination, returning 130 (128 + SIGINT)
/// which is the standard Unix convention for signal termination.
///
/// # Example
///
/// ```rust,ignore
/// use std::process::Command;
/// use vx_core::exit_code_from_status;
///
/// let status = Command::new("some_command").status().unwrap();
/// let code = exit_code_from_status(&status);
/// std::process::exit(code);
/// ```
pub fn exit_code_from_status(status: &ExitStatus) -> i32 {
    if is_ctrl_c_exit(status) {
        // Return 130 (128 + 2) which is the standard exit code for SIGINT
        // This is recognized by shells as "terminated by signal"
        130
    } else {
        status.code().unwrap_or(1)
    }
}

// ============================================================================
// Version Resolution Utilities
// ============================================================================

/// Check if a version string is "latest" (case insensitive)
///
/// # Example
///
/// ```rust
/// use vx_core::is_latest_version;
///
/// assert!(is_latest_version("latest"));
/// assert!(is_latest_version("LATEST"));
/// assert!(!is_latest_version("1.0.0"));
/// ```
pub fn is_latest_version(version: &str) -> bool {
    version.eq_ignore_ascii_case("latest")
}

/// Resolve "latest" version to an actual version from installed versions
///
/// If the version is "latest", returns the highest version from the provided list
/// using semantic version comparison. Otherwise, returns the version as-is.
///
/// # Example
///
/// ```rust
/// use vx_core::resolve_latest_version;
///
/// let versions = vec!["1.0.0".to_string(), "2.0.0".to_string(), "1.5.0".to_string()];
/// assert_eq!(resolve_latest_version("latest", &versions), Some("2.0.0".to_string()));
/// assert_eq!(resolve_latest_version("1.5.0", &versions), Some("1.5.0".to_string()));
/// assert_eq!(resolve_latest_version("latest", &Vec::new()), None);
/// ```
pub fn resolve_latest_version(version: &str, installed_versions: &[String]) -> Option<String> {
    if is_latest_version(version) {
        crate::version_utils::find_latest_version(installed_versions, false).map(|v| v.to_string())
    } else {
        Some(version.to_string())
    }
}
