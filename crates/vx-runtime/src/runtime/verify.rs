//! Installation verification logic for the `Runtime` trait.
//!
//! This module provides [`VerificationResult`] and the default implementation
//! of `verify_installation()` used by the `Runtime` trait's `install()` method.

use std::path::Path;

use crate::platform::Platform;

/// Installation verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the installation is valid
    pub valid: bool,
    /// Path to the executable if found
    pub executable_path: Option<std::path::PathBuf>,
    /// List of issues found during verification
    pub issues: Vec<String>,
    /// Suggested fixes for the issues
    pub suggestions: Vec<String>,
}

impl VerificationResult {
    /// Create a successful verification result
    pub fn success(executable_path: std::path::PathBuf) -> Self {
        Self {
            valid: true,
            executable_path: Some(executable_path),
            issues: vec![],
            suggestions: vec![],
        }
    }

    /// Create a failed verification result
    pub fn failure(issues: Vec<String>, suggestions: Vec<String>) -> Self {
        Self {
            valid: false,
            executable_path: None,
            issues,
            suggestions,
        }
    }

    /// Create a successful verification result for system-installed tools.
    ///
    /// Used for tools installed via system package managers (winget, brew, apt, etc.)
    /// where the executable is available in the system PATH rather than a specific path.
    pub fn success_system_installed() -> Self {
        Self {
            valid: true,
            executable_path: None,
            issues: vec![],
            suggestions: vec![],
        }
    }
}

/// Default implementation of `verify_installation` for the `Runtime` trait.
///
/// Checks that the executable exists at the expected relative path (supports
/// glob patterns) and is executable on Unix.
pub fn verify_installation_default(
    exe_relative: &str,
    install_path: &Path,
    exe_name: &str,
    exe_extensions: &[&str],
) -> VerificationResult {
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();

    // Handle glob patterns in executable path (e.g., "*/bin/java.exe")
    let exe_path = if exe_relative.contains('*') {
        let pattern = install_path.join(exe_relative);
        let pattern_str = pattern.to_string_lossy();
        match glob::glob(&pattern_str) {
            Ok(paths) => {
                let matches: Vec<_> = paths.filter_map(|p| p.ok()).collect();
                if matches.is_empty() {
                    None
                } else {
                    Some(matches[0].clone())
                }
            }
            Err(_) => None,
        }
    } else {
        let path = install_path.join(exe_relative);
        if path.exists() { Some(path) } else { None }
    };

    // Check if executable exists
    let exe_path = match exe_path {
        Some(path) if path.exists() => path,
        _ => {
            issues.push(format!(
                "Executable not found at expected path: {}",
                install_path.join(exe_relative).display()
            ));

            // Try to find the executable in the install directory
            if let Some(found_path) =
                find_executable_in_install_dir(install_path, exe_name, exe_extensions)
            {
                suggestions.push(format!(
                    "Found executable at: {}. Consider overriding executable_relative_path() \
                     to return the correct relative path.",
                    found_path.display()
                ));
                if let Ok(relative) = found_path.strip_prefix(install_path) {
                    suggestions.push(format!(
                        "Suggested executable_relative_path: \"{}\"",
                        relative.display()
                    ));
                }
            } else {
                // List top-level contents for debugging
                if let Ok(entries) = std::fs::read_dir(install_path) {
                    let contents: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| {
                            let path = e.path();
                            let is_dir = path.is_dir();
                            format!(
                                "{}{}",
                                e.file_name().to_string_lossy(),
                                if is_dir { "/" } else { "" }
                            )
                        })
                        .collect();
                    suggestions.push(format!(
                        "Install directory contents: [{}]",
                        contents.join(", ")
                    ));
                }
            }

            return VerificationResult::failure(issues, suggestions);
        }
    };

    // Check if file is executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&exe_path) {
            let mode = metadata.permissions().mode();
            if mode & 0o111 == 0 {
                issues.push(format!(
                    "File exists but is not executable: {}",
                    exe_path.display()
                ));
                suggestions.push("Try: chmod +x <path>".to_string());
                return VerificationResult::failure(issues, suggestions);
            }
        }
    }

    VerificationResult::success(exe_path)
}

/// Search for an executable in the install directory (up to 3 levels deep).
pub fn find_executable_in_install_dir(
    install_path: &Path,
    exe_name: &str,
    exe_extensions: &[&str],
) -> Option<std::path::PathBuf> {
    let platform = Platform::current();
    let exe_names = platform.all_executable_names(exe_name, exe_extensions);

    for name in &exe_names {
        if let Some(path) = search_for_executable(install_path, name, exe_name, 0, 3) {
            return Some(path);
        }
    }
    None
}

/// Recursively search for an executable.
pub fn search_for_executable(
    dir: &Path,
    exe_name: &str,
    runtime_name: &str,
    current_depth: usize,
    max_depth: usize,
) -> Option<std::path::PathBuf> {
    if current_depth > max_depth || !dir.exists() {
        return None;
    }

    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            if name == exe_name || name == runtime_name {
                return Some(path);
            }
        } else if path.is_dir()
            && let Some(found) =
                search_for_executable(&path, exe_name, runtime_name, current_depth + 1, max_depth)
        {
            return Some(found);
        }
    }

    None
}
