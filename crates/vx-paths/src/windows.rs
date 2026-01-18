//! Windows-specific path handling for long paths
//!
//! Windows has a historical `MAX_PATH` limit of 260 characters. This module provides
//! utilities to work around this limitation by using the extended-length path prefix `\\?\`.
//!
//! ## Background
//! - Windows traditional `MAX_PATH`: 260 characters
//! - Extended-length path prefix `\\?\` allows up to 32,767 characters
//! - Windows 10 1607+ can enable long path support via registry/group policy
//!
//! ## Usage
//! ```rust
//! use vx_paths::windows::{to_long_path, check_path_length, is_long_path_enabled};
//!
//! // Convert path to extended-length format on Windows
//! let long_path = to_long_path(&my_path);
//!
//! // Check if path exceeds safe length
//! if let Err(warning) = check_path_length(&path) {
//!     eprintln!("{}", warning);
//! }
//! ```

use std::path::{Path, PathBuf};

/// Windows MAX_PATH limit (260 characters including null terminator)
pub const WINDOWS_MAX_PATH: usize = 260;

/// Warning threshold for path length (leave some margin)
pub const WINDOWS_PATH_WARN_THRESHOLD: usize = 200;

/// Extended-length path prefix for Windows
pub const EXTENDED_PATH_PREFIX: &str = r"\\?\";

/// UNC extended-length path prefix
pub const EXTENDED_UNC_PREFIX: &str = r"\\?\UNC\";

/// Result of path length check
#[derive(Debug, Clone)]
pub enum PathLengthStatus {
    /// Path is within safe limits
    Safe,
    /// Path is approaching the limit (warning)
    Warning {
        length: usize,
        path: PathBuf,
    },
    /// Path exceeds MAX_PATH limit
    TooLong {
        length: usize,
        path: PathBuf,
    },
}

impl PathLengthStatus {
    /// Returns true if the path is safe (not too long)
    pub fn is_safe(&self) -> bool {
        matches!(self, PathLengthStatus::Safe)
    }

    /// Returns true if the path is too long
    pub fn is_too_long(&self) -> bool {
        matches!(self, PathLengthStatus::TooLong { .. })
    }

    /// Get a human-readable message
    pub fn message(&self) -> Option<String> {
        match self {
            PathLengthStatus::Safe => None,
            PathLengthStatus::Warning { length, path } => Some(format!(
                "Path length ({}) approaching Windows limit ({}): {}",
                length,
                WINDOWS_MAX_PATH,
                path.display()
            )),
            PathLengthStatus::TooLong { length, path } => Some(format!(
                "Path length ({}) exceeds Windows limit ({}): {}",
                length,
                WINDOWS_MAX_PATH,
                path.display()
            )),
        }
    }
}

/// Convert a path to extended-length format on Windows.
///
/// This function adds the `\\?\` prefix to absolute paths on Windows,
/// allowing paths longer than MAX_PATH (260 characters).
///
/// On non-Windows platforms, this function returns the path unchanged.
///
/// # Example
/// ```
/// use std::path::PathBuf;
/// use vx_paths::windows::to_long_path;
///
/// let path = PathBuf::from(r"C:\Users\name\.vx\store\node\20.0.0\bin\node.exe");
/// let long_path = to_long_path(&path);
/// // On Windows: \\?\C:\Users\name\.vx\store\node\20.0.0\bin\node.exe
/// // On other platforms: C:\Users\name\.vx\store\node\20.0.0\bin\node.exe
/// ```
#[cfg(windows)]
pub fn to_long_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();

    // Already has extended-length prefix
    if path_str.starts_with(EXTENDED_PATH_PREFIX) {
        return path.to_path_buf();
    }

    // Handle UNC paths (\\server\share)
    if path_str.starts_with(r"\\") && !path_str.starts_with(r"\\?") {
        // Convert \\server\share to \\?\UNC\server\share
        let unc_path = &path_str[2..]; // Remove leading \\
        return PathBuf::from(format!("{}{}", EXTENDED_UNC_PREFIX, unc_path));
    }

    // Only apply to absolute paths
    if path.is_absolute() {
        PathBuf::from(format!("{}{}", EXTENDED_PATH_PREFIX, path_str))
    } else {
        path.to_path_buf()
    }
}

/// Convert a path to extended-length format (no-op on non-Windows)
#[cfg(not(windows))]
pub fn to_long_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

/// Remove the extended-length prefix from a path.
///
/// This is useful for display purposes, as paths with the `\\?\` prefix
/// can be confusing to users.
#[cfg(windows)]
pub fn from_long_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();

    if path_str.starts_with(EXTENDED_UNC_PREFIX) {
        // Convert \\?\UNC\server\share back to \\server\share
        let unc_path = &path_str[EXTENDED_UNC_PREFIX.len()..];
        return PathBuf::from(format!(r"\\{}", unc_path));
    }

    if path_str.starts_with(EXTENDED_PATH_PREFIX) {
        return PathBuf::from(&path_str[EXTENDED_PATH_PREFIX.len()..]);
    }

    path.to_path_buf()
}

/// Remove the extended-length prefix from a path (no-op on non-Windows)
#[cfg(not(windows))]
pub fn from_long_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

/// Check if a path length is safe for Windows.
///
/// Returns a status indicating whether the path is safe, approaching the limit,
/// or too long.
pub fn check_path_length(path: &Path) -> PathLengthStatus {
    let path_len = path.to_string_lossy().len();

    if path_len >= WINDOWS_MAX_PATH {
        PathLengthStatus::TooLong {
            length: path_len,
            path: path.to_path_buf(),
        }
    } else if path_len >= WINDOWS_PATH_WARN_THRESHOLD {
        PathLengthStatus::Warning {
            length: path_len,
            path: path.to_path_buf(),
        }
    } else {
        PathLengthStatus::Safe
    }
}

/// Check if Windows long path support is enabled via registry.
///
/// Returns `true` if long path support is enabled, `false` otherwise.
/// On non-Windows platforms, this always returns `true`.
#[cfg(windows)]
pub fn is_long_path_enabled() -> bool {
    use std::process::Command;

    // Try to read the registry key
    // HKLM\SYSTEM\CurrentControlSet\Control\FileSystem\LongPathsEnabled
    let output = Command::new("reg")
        .args([
            "query",
            r"HKLM\SYSTEM\CurrentControlSet\Control\FileSystem",
            "/v",
            "LongPathsEnabled",
        ])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Look for "LongPathsEnabled    REG_DWORD    0x1" in output
            stdout.contains("0x1")
        }
        Err(_) => false,
    }
}

/// Check if Windows long path support is enabled (always true on non-Windows)
#[cfg(not(windows))]
pub fn is_long_path_enabled() -> bool {
    true
}

/// Get a message about enabling Windows long path support
pub fn get_long_path_enable_instructions() -> &'static str {
    r#"
To enable Windows long path support:

Option 1: Run this PowerShell command (requires Administrator):
  New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
      -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force

Option 2: Via Group Policy (Windows 10 Pro/Enterprise):
  1. Open gpedit.msc
  2. Navigate to: Computer Configuration > Administrative Templates > System > Filesystem
  3. Enable "Enable Win32 long paths"

Option 3: Set VX_HOME to a shorter path:
  $env:VX_HOME = "C:\vx"

After enabling, restart your terminal or reboot Windows.
"#
}

/// Log a warning about path length if necessary
pub fn warn_if_path_too_long(path: &Path) {
    #[cfg(windows)]
    {
        let status = check_path_length(path);
        if let Some(message) = status.message() {
            match status {
                PathLengthStatus::TooLong { .. } => {
                    tracing::error!("{}", message);
                    if !is_long_path_enabled() {
                        tracing::error!(
                            "Consider enabling Windows long path support or using a shorter VX_HOME path"
                        );
                    }
                }
                PathLengthStatus::Warning { .. } => {
                    tracing::warn!("{}", message);
                }
                PathLengthStatus::Safe => {}
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = path; // Suppress unused warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_path_length_safe() {
        let path = PathBuf::from(r"C:\short\path");
        let status = check_path_length(&path);
        assert!(status.is_safe());
        assert!(!status.is_too_long());
        assert!(status.message().is_none());
    }

    #[test]
    fn test_check_path_length_warning() {
        // Create a path just over the warning threshold
        let long_segment = "a".repeat(WINDOWS_PATH_WARN_THRESHOLD);
        let path = PathBuf::from(format!(r"C:\{}", long_segment));
        let status = check_path_length(&path);

        match status {
            PathLengthStatus::Warning { length, .. } => {
                assert!(length >= WINDOWS_PATH_WARN_THRESHOLD);
            }
            PathLengthStatus::TooLong { .. } => {
                // Also acceptable if it exceeds MAX_PATH
            }
            _ => panic!("Expected Warning or TooLong status"),
        }
    }

    #[test]
    fn test_check_path_length_too_long() {
        // Create a path over MAX_PATH
        let long_segment = "a".repeat(WINDOWS_MAX_PATH);
        let path = PathBuf::from(format!(r"C:\{}", long_segment));
        let status = check_path_length(&path);
        assert!(status.is_too_long());
        assert!(status.message().is_some());
    }

    #[cfg(windows)]
    #[test]
    fn test_to_long_path() {
        // Regular absolute path
        let path = PathBuf::from(r"C:\Users\name\file.txt");
        let long_path = to_long_path(&path);
        assert_eq!(
            long_path.to_string_lossy(),
            r"\\?\C:\Users\name\file.txt"
        );

        // Already has prefix
        let path = PathBuf::from(r"\\?\C:\Users\name\file.txt");
        let long_path = to_long_path(&path);
        assert_eq!(
            long_path.to_string_lossy(),
            r"\\?\C:\Users\name\file.txt"
        );

        // UNC path
        let path = PathBuf::from(r"\\server\share\file.txt");
        let long_path = to_long_path(&path);
        assert_eq!(
            long_path.to_string_lossy(),
            r"\\?\UNC\server\share\file.txt"
        );

        // Relative path (should not be modified)
        let path = PathBuf::from(r"relative\path");
        let long_path = to_long_path(&path);
        assert_eq!(long_path.to_string_lossy(), r"relative\path");
    }

    #[cfg(windows)]
    #[test]
    fn test_from_long_path() {
        // Extended path
        let path = PathBuf::from(r"\\?\C:\Users\name\file.txt");
        let short_path = from_long_path(&path);
        assert_eq!(short_path.to_string_lossy(), r"C:\Users\name\file.txt");

        // Extended UNC path
        let path = PathBuf::from(r"\\?\UNC\server\share\file.txt");
        let short_path = from_long_path(&path);
        assert_eq!(short_path.to_string_lossy(), r"\\server\share\file.txt");

        // Regular path (should not be modified)
        let path = PathBuf::from(r"C:\Users\name\file.txt");
        let short_path = from_long_path(&path);
        assert_eq!(short_path.to_string_lossy(), r"C:\Users\name\file.txt");
    }

    #[cfg(not(windows))]
    #[test]
    fn test_to_long_path_non_windows() {
        let path = PathBuf::from("/home/user/file.txt");
        let long_path = to_long_path(&path);
        assert_eq!(long_path, path);
    }

    #[cfg(not(windows))]
    #[test]
    fn test_is_long_path_enabled_non_windows() {
        // Always true on non-Windows
        assert!(is_long_path_enabled());
    }
}
