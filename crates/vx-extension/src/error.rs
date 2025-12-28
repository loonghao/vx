//! Extension system error types and diagnostics
//!
//! This module provides structured error types for the extension system,
//! with detailed diagnostic information to help users understand and fix issues.

use std::path::PathBuf;
use thiserror::Error;

/// Extension system errors
#[derive(Debug, Error)]
pub enum ExtensionError {
    // ============ Configuration Errors ============
    /// Extension configuration file not found
    #[error("Extension configuration not found")]
    ConfigNotFound {
        /// Path where the config was expected
        path: PathBuf,
        /// Suggestion for the user
        suggestion: String,
    },

    /// Invalid extension configuration
    #[error("Invalid extension configuration in '{path}'")]
    ConfigInvalid {
        /// Path to the invalid config file
        path: PathBuf,
        /// The parsing error message
        reason: String,
        /// Line number if available
        line: Option<usize>,
        /// Column number if available
        column: Option<usize>,
    },

    /// Missing required field in configuration
    #[error("Missing required field '{field}' in extension configuration")]
    ConfigMissingField {
        /// The missing field name
        field: String,
        /// Path to the config file
        path: PathBuf,
    },

    // ============ Discovery Errors ============
    /// Extension not found
    #[error("Extension '{name}' not found")]
    ExtensionNotFound {
        /// Name of the extension
        name: String,
        /// Available extensions
        available: Vec<String>,
        /// Searched locations
        searched_paths: Vec<PathBuf>,
    },

    /// Multiple extensions with the same name
    #[error("Multiple extensions named '{name}' found")]
    DuplicateExtension {
        /// Name of the extension
        name: String,
        /// Paths where the extension was found
        paths: Vec<PathBuf>,
    },

    // ============ Execution Errors ============
    /// Subcommand not found in extension
    #[error("Subcommand '{subcommand}' not found in extension '{extension}'")]
    SubcommandNotFound {
        /// The extension name
        extension: String,
        /// The subcommand that was not found
        subcommand: String,
        /// Available subcommands
        available: Vec<String>,
    },

    /// No entrypoint defined for extension
    #[error("Extension '{name}' has no entrypoint defined")]
    NoEntrypoint {
        /// Extension name
        name: String,
        /// Available commands if any
        available_commands: Vec<String>,
    },

    /// Script file not found
    #[error("Script '{script}' not found for extension '{extension}'")]
    ScriptNotFound {
        /// Extension name
        extension: String,
        /// Script path
        script: PathBuf,
        /// Extension directory
        extension_dir: PathBuf,
    },

    /// Runtime not available
    #[error("Runtime '{runtime}' required by extension '{extension}' is not available")]
    RuntimeNotAvailable {
        /// Extension name
        extension: String,
        /// Required runtime
        runtime: String,
        /// Version constraint if any
        version_constraint: Option<String>,
    },

    /// Script execution failed
    #[error("Extension '{extension}' script execution failed")]
    ExecutionFailed {
        /// Extension name
        extension: String,
        /// Exit code
        exit_code: Option<i32>,
        /// Error message from stderr if available
        stderr: Option<String>,
    },

    // ============ Link/Unlink Errors ============
    /// Failed to link development extension
    #[error("Failed to link extension from '{source_path}'")]
    LinkFailed {
        /// Source path
        source_path: PathBuf,
        /// Target path
        target_path: PathBuf,
        /// Reason for failure
        reason: String,
    },

    /// Extension is not a symlink (cannot unlink)
    #[error("Extension '{name}' is not a development link")]
    NotADevLink {
        /// Extension name
        name: String,
        /// Path to the extension
        path: PathBuf,
    },

    // ============ IO Errors ============
    /// IO error
    #[error("IO error: {message}")]
    Io {
        /// Error message
        message: String,
        /// Path involved if any
        path: Option<PathBuf>,
        /// Source error
        #[source]
        source: std::io::Error,
    },

    // ============ Permission Errors ============
    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied {
        /// Error message
        message: String,
        /// Path involved
        path: PathBuf,
    },
}

impl ExtensionError {
    // ============ Constructor helpers ============

    /// Create a ConfigNotFound error
    pub fn config_not_found(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self::ConfigNotFound {
            suggestion: format!(
                "Create a 'vx-extension.toml' file in '{}' with at least:\n\n\
                 [extension]\n\
                 name = \"your-extension-name\"",
                path.display()
            ),
            path,
        }
    }

    /// Create a ConfigInvalid error from a toml parse error
    pub fn config_invalid(path: impl Into<PathBuf>, error: &toml::de::Error) -> Self {
        let path = path.into();
        let span = error.span();
        Self::ConfigInvalid {
            path,
            reason: error.message().to_string(),
            line: span.map(|s| s.start),
            column: None,
        }
    }

    /// Create an ExtensionNotFound error
    pub fn extension_not_found(
        name: impl Into<String>,
        available: Vec<String>,
        searched_paths: Vec<PathBuf>,
    ) -> Self {
        Self::ExtensionNotFound {
            name: name.into(),
            available,
            searched_paths,
        }
    }

    /// Create a SubcommandNotFound error
    pub fn subcommand_not_found(
        extension: impl Into<String>,
        subcommand: impl Into<String>,
        available: Vec<String>,
    ) -> Self {
        Self::SubcommandNotFound {
            extension: extension.into(),
            subcommand: subcommand.into(),
            available,
        }
    }

    /// Create a NoEntrypoint error
    pub fn no_entrypoint(name: impl Into<String>, available_commands: Vec<String>) -> Self {
        Self::NoEntrypoint {
            name: name.into(),
            available_commands,
        }
    }

    /// Create a ScriptNotFound error
    pub fn script_not_found(
        extension: impl Into<String>,
        script: impl Into<PathBuf>,
        extension_dir: impl Into<PathBuf>,
    ) -> Self {
        Self::ScriptNotFound {
            extension: extension.into(),
            script: script.into(),
            extension_dir: extension_dir.into(),
        }
    }

    /// Create a RuntimeNotAvailable error
    pub fn runtime_not_available(
        extension: impl Into<String>,
        runtime: impl Into<String>,
        version_constraint: Option<String>,
    ) -> Self {
        Self::RuntimeNotAvailable {
            extension: extension.into(),
            runtime: runtime.into(),
            version_constraint,
        }
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>, path: Option<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            path,
            source,
        }
    }

    /// Create a LinkFailed error
    pub fn link_failed(
        source_path: impl Into<PathBuf>,
        target_path: impl Into<PathBuf>,
        reason: impl Into<String>,
    ) -> Self {
        Self::LinkFailed {
            source_path: source_path.into(),
            target_path: target_path.into(),
            reason: reason.into(),
        }
    }

    // ============ Diagnostic helpers ============

    /// Get a user-friendly diagnostic message with suggestions
    pub fn diagnostic(&self) -> String {
        match self {
            Self::ConfigNotFound { path, suggestion } => {
                format!(
                    "Extension configuration not found at: {}\n\n\
                     Suggestion:\n{}",
                    path.display(),
                    suggestion
                )
            }

            Self::ConfigInvalid {
                path,
                reason,
                line,
                column,
            } => {
                let location = match (line, column) {
                    (Some(l), Some(c)) => format!(" at line {}, column {}", l, c),
                    (Some(l), None) => format!(" at position {}", l),
                    _ => String::new(),
                };
                format!(
                    "Invalid configuration in '{}'{}\n\n\
                     Error: {}\n\n\
                     Tip: Validate your TOML syntax at https://www.toml-lint.com/",
                    path.display(),
                    location,
                    reason
                )
            }

            Self::ConfigMissingField { field, path } => {
                format!(
                    "Missing required field '{}' in {}\n\n\
                     Add the field to your vx-extension.toml:\n\n\
                     [extension]\n\
                     {} = \"value\"",
                    field,
                    path.display(),
                    field
                )
            }

            Self::ExtensionNotFound {
                name,
                available,
                searched_paths,
            } => {
                let mut msg = format!("Extension '{}' not found.\n\n", name);

                if !available.is_empty() {
                    msg.push_str("Available extensions:\n");
                    for ext in available {
                        msg.push_str(&format!("  - {}\n", ext));
                    }
                    msg.push('\n');
                }

                msg.push_str("Searched in:\n");
                for path in searched_paths {
                    msg.push_str(&format!("  - {}\n", path.display()));
                }

                msg.push_str("\nTo install an extension:\n");
                msg.push_str("  vx ext install <extension-name>\n\n");
                msg.push_str("To create a local extension:\n");
                msg.push_str("  mkdir -p ~/.vx/extensions/my-extension\n");
                msg.push_str("  # Create vx-extension.toml in that directory");

                msg
            }

            Self::SubcommandNotFound {
                extension,
                subcommand,
                available,
            } => {
                let mut msg = format!(
                    "Subcommand '{}' not found in extension '{}'.\n\n",
                    subcommand, extension
                );

                if !available.is_empty() {
                    msg.push_str("Available commands:\n");
                    for cmd in available {
                        msg.push_str(&format!("  vx x {} {}\n", extension, cmd));
                    }
                } else {
                    msg.push_str(&format!(
                        "This extension has no subcommands. Try:\n  vx x {}",
                        extension
                    ));
                }

                msg
            }

            Self::NoEntrypoint {
                name,
                available_commands,
            } => {
                let mut msg = format!("Extension '{}' has no main entrypoint defined.\n\n", name);

                if !available_commands.is_empty() {
                    msg.push_str("Use one of the available commands:\n");
                    for cmd in available_commands {
                        msg.push_str(&format!("  vx x {} {}\n", name, cmd));
                    }
                } else {
                    msg.push_str("Add an entrypoint to vx-extension.toml:\n\n");
                    msg.push_str("[entrypoint]\n");
                    msg.push_str("main = \"main.py\"");
                }

                msg
            }

            Self::ScriptNotFound {
                extension,
                script,
                extension_dir,
            } => {
                format!(
                    "Script '{}' not found for extension '{}'.\n\n\
                     Expected at: {}\n\n\
                     Make sure the script file exists and the path in vx-extension.toml is correct.",
                    script.display(),
                    extension,
                    extension_dir.join(script).display()
                )
            }

            Self::RuntimeNotAvailable {
                extension,
                runtime,
                version_constraint,
            } => {
                let constraint = version_constraint
                    .as_ref()
                    .map(|c| format!(" {}", c))
                    .unwrap_or_default();

                format!(
                    "Runtime '{}{}' required by extension '{}' is not available.\n\n\
                     Install it with:\n  vx install {}{}",
                    runtime, constraint, extension, runtime, constraint
                )
            }

            Self::ExecutionFailed {
                extension,
                exit_code,
                stderr,
            } => {
                let mut msg = format!("Extension '{}' execution failed", extension);

                if let Some(code) = exit_code {
                    msg.push_str(&format!(" with exit code {}", code));
                }
                msg.push_str(".\n");

                if let Some(err) = stderr {
                    msg.push_str(&format!("\nError output:\n{}", err));
                }

                msg
            }

            Self::LinkFailed {
                source_path,
                target_path,
                reason,
            } => {
                format!(
                    "Failed to link extension.\n\n\
                     Source: {}\n\
                     Target: {}\n\
                     Reason: {}\n\n\
                     Make sure you have write permissions and the source directory exists.",
                    source_path.display(),
                    target_path.display(),
                    reason
                )
            }

            Self::NotADevLink { name, path } => {
                format!(
                    "Extension '{}' at '{}' is not a development link.\n\n\
                     Only symlinked extensions (created with 'vx ext dev') can be unlinked.\n\
                     To remove a regular extension, delete its directory manually.",
                    name,
                    path.display()
                )
            }

            Self::Io {
                message,
                path,
                source: _,
            } => {
                let path_info = path
                    .as_ref()
                    .map(|p| format!("\nPath: {}", p.display()))
                    .unwrap_or_default();

                format!("IO error: {}{}", message, path_info)
            }

            Self::PermissionDenied { message, path } => {
                format!(
                    "Permission denied: {}\n\
                     Path: {}\n\n\
                     Try running with appropriate permissions or check file ownership.",
                    message,
                    path.display()
                )
            }

            Self::DuplicateExtension { name, paths } => {
                let mut msg = format!("Multiple extensions named '{}' found:\n\n", name);

                for (i, path) in paths.iter().enumerate() {
                    msg.push_str(&format!("  {}. {}\n", i + 1, path.display()));
                }

                msg.push_str("\nThe extension with highest priority will be used.\n");
                msg.push_str("Priority order: dev > project > user > builtin");

                msg
            }
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ExtensionNotFound { .. }
                | Self::SubcommandNotFound { .. }
                | Self::RuntimeNotAvailable { .. }
        )
    }

    /// Get the error code for CLI exit
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ConfigNotFound { .. } => 64,      // EX_USAGE
            Self::ConfigInvalid { .. } => 65,       // EX_DATAERR
            Self::ConfigMissingField { .. } => 65,  // EX_DATAERR
            Self::ExtensionNotFound { .. } => 66,   // EX_NOINPUT
            Self::DuplicateExtension { .. } => 65,  // EX_DATAERR
            Self::SubcommandNotFound { .. } => 64,  // EX_USAGE
            Self::NoEntrypoint { .. } => 78,        // EX_CONFIG
            Self::ScriptNotFound { .. } => 66,      // EX_NOINPUT
            Self::RuntimeNotAvailable { .. } => 69, // EX_UNAVAILABLE
            Self::ExecutionFailed { exit_code, .. } => exit_code.unwrap_or(1),
            Self::LinkFailed { .. } => 73,       // EX_CANTCREAT
            Self::NotADevLink { .. } => 64,      // EX_USAGE
            Self::Io { .. } => 74,               // EX_IOERR
            Self::PermissionDenied { .. } => 77, // EX_NOPERM
        }
    }
}

/// Result type alias for extension operations
pub type ExtensionResult<T> = Result<T, ExtensionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_not_found_error() {
        let err = ExtensionError::config_not_found("/path/to/ext");
        assert!(matches!(err, ExtensionError::ConfigNotFound { .. }));
        assert!(err.diagnostic().contains("vx-extension.toml"));
    }

    #[test]
    fn test_extension_not_found_error() {
        let err = ExtensionError::extension_not_found(
            "my-ext",
            vec!["ext1".to_string(), "ext2".to_string()],
            vec![PathBuf::from("/path1"), PathBuf::from("/path2")],
        );
        let diag = err.diagnostic();
        assert!(diag.contains("my-ext"));
        assert!(diag.contains("ext1"));
        assert!(diag.contains("ext2"));
    }

    #[test]
    fn test_subcommand_not_found_error() {
        let err = ExtensionError::subcommand_not_found(
            "docker-compose",
            "invalid",
            vec!["up".to_string(), "down".to_string()],
        );
        let diag = err.diagnostic();
        assert!(diag.contains("invalid"));
        assert!(diag.contains("up"));
        assert!(diag.contains("down"));
    }

    #[test]
    fn test_exit_codes() {
        let err = ExtensionError::config_not_found("/path");
        assert_eq!(err.exit_code(), 64);

        let err = ExtensionError::extension_not_found("test", vec![], vec![]);
        assert_eq!(err.exit_code(), 66);
    }

    #[test]
    fn test_is_recoverable() {
        let err = ExtensionError::extension_not_found("test", vec![], vec![]);
        assert!(err.is_recoverable());

        let err = ExtensionError::config_not_found("/path");
        assert!(!err.is_recoverable());
    }
}
