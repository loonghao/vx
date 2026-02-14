//! Cross-platform utilities for vx
//!
//! This module provides a unified interface for platform-specific operations,
//! centralizing all platform detection and path handling logic.
//!
//! # Design Principles
//!
//! - **Avoid `Path::new()` for user input**: Use string comparison to prevent
//!   issues with invalid path characters (e.g., `:` in Unix paths from Windows-style inputs)
//! - **Compile-time platform detection**: Use `cfg!()` for efficient platform checks
//! - **Safe PATH handling**: Provide utilities that work correctly with `std::env::join_paths`

use std::ffi::OsString;
use std::path::PathBuf;

/// Operating system type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOS,
    Linux,
    Other,
}

impl Os {
    /// Detect the current operating system at runtime
    #[inline]
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOS
        } else if cfg!(target_os = "linux") {
            Os::Linux
        } else {
            Os::Other
        }
    }

    /// Check if this is a Unix-like OS (Linux, macOS, etc.)
    #[inline]
    pub fn is_unix(&self) -> bool {
        matches!(self, Os::Linux | Os::MacOS)
    }

    /// Check if this is Windows
    #[inline]
    pub fn is_windows(&self) -> bool {
        matches!(self, Os::Windows)
    }
}

/// CPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    Aarch64,
    X86,
    Arm,
    Other,
}

impl Arch {
    /// Detect the current architecture at runtime
    #[inline]
    pub fn current() -> Self {
        if cfg!(target_arch = "x86_64") {
            Arch::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Aarch64
        } else if cfg!(target_arch = "x86") {
            Arch::X86
        } else if cfg!(target_arch = "arm") {
            Arch::Arm
        } else {
            Arch::Other
        }
    }
}

/// Platform information combining OS and architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    /// Get the current platform
    #[inline]
    pub fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
        }
    }

    /// Get the PATH environment variable separator
    ///
    /// - Windows: `;`
    /// - Unix (Linux, macOS): `:`
    #[inline]
    pub fn path_separator(&self) -> char {
        if self.os.is_windows() { ';' } else { ':' }
    }

    /// Get the executable file extension
    ///
    /// - Windows: `.exe`
    /// - Unix: `` (empty string)
    #[inline]
    pub fn executable_extension(&self) -> &'static str {
        if self.os.is_windows() { ".exe" } else { "" }
    }

    /// Get the Python venv bin directory name
    ///
    /// - Windows: `Scripts`
    /// - Unix: `bin`
    #[inline]
    pub fn venv_bin_dir(&self) -> &'static str {
        if self.os.is_windows() {
            "Scripts"
        } else {
            "bin"
        }
    }

    /// Get the platform string used in download URLs and directory names
    ///
    /// Returns strings like "windows-x64", "darwin-arm64", "linux-x64"
    pub fn as_str(&self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Windows, Arch::X86_64) => "windows-x64",
            (Os::Windows, Arch::X86) => "windows-x86",
            (Os::Windows, Arch::Aarch64) => "windows-arm64",
            (Os::MacOS, Arch::X86_64) => "darwin-x64",
            (Os::MacOS, Arch::Aarch64) => "darwin-arm64",
            (Os::Linux, Arch::X86_64) => "linux-x64",
            (Os::Linux, Arch::Aarch64) => "linux-arm64",
            (Os::Linux, Arch::Arm) => "linux-arm",
            _ => "unknown",
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::current()
    }
}

// =============================================================================
// PATH Utilities
// =============================================================================

/// Get the current platform's PATH separator
///
/// This is a convenience function for compile-time platform detection.
#[inline]
pub fn path_separator() -> char {
    if cfg!(windows) { ';' } else { ':' }
}

/// Split a PATH string into individual entries
///
/// Uses the correct separator for the current platform.
///
/// # Example
/// ```
/// use vx_paths::platform::split_path;
///
/// // On Unix: "/usr/bin:/bin" -> ["/usr/bin", "/bin"]
/// // On Windows: "C:\\Windows;C:\\Users" -> ["C:\\Windows", "C:\\Users"]
/// let paths: Vec<&str> = split_path("/usr/bin:/bin").collect();
/// ```
#[inline]
pub fn split_path(path: &str) -> impl Iterator<Item = &str> {
    path.split(path_separator()).filter(|s| !s.is_empty())
}

/// Split a PATH string into owned strings
///
/// This is useful when you need to collect and store the results.
pub fn split_path_owned(path: &str) -> Vec<String> {
    split_path(path).map(|s| s.to_string()).collect()
}

/// Join multiple paths into a single PATH string
///
/// Uses the correct separator for the current platform.
///
/// # Example
/// ```
/// use vx_paths::platform::join_paths_simple;
///
/// let paths = vec!["/usr/bin", "/bin"];
/// let result = join_paths_simple(&paths);
/// // On Unix: "/usr/bin:/bin"
/// // On Windows: "/usr/bin;/bin"
/// ```
pub fn join_paths_simple<S: AsRef<str>>(paths: &[S]) -> String {
    paths
        .iter()
        .map(|s| s.as_ref())
        .collect::<Vec<_>>()
        .join(&path_separator().to_string())
}

/// Join multiple paths using `std::env::join_paths`
///
/// This is the safe way to create PATH strings that will work correctly
/// with `std::env::set_var`. Unlike `join_paths_simple`, this properly
/// handles paths containing special characters.
///
/// # Errors
///
/// Returns an error if any path contains the PATH separator character,
/// which would be invalid.
///
/// # Example
/// ```
/// use vx_paths::platform::join_paths_env;
///
/// let paths = vec!["/usr/bin", "/bin"];
/// let result = join_paths_env(&paths).unwrap();
/// ```
pub fn join_paths_env<S: AsRef<str>>(paths: &[S]) -> Result<OsString, std::env::JoinPathsError> {
    let path_bufs: Vec<PathBuf> = paths.iter().map(|s| PathBuf::from(s.as_ref())).collect();
    std::env::join_paths(path_bufs)
}

/// Prepend entries to an existing PATH string
///
/// # Example
/// ```
/// use vx_paths::platform::prepend_to_path;
///
/// let original = "/usr/bin:/bin";
/// let new_entries = vec!["/my/custom/bin"];
/// let result = prepend_to_path(original, &new_entries);
/// // Result: "/my/custom/bin:/usr/bin:/bin"
/// ```
pub fn prepend_to_path<S: AsRef<str>>(original: &str, entries: &[S]) -> String {
    let mut parts: Vec<String> = entries.iter().map(|s| s.as_ref().to_string()).collect();
    parts.extend(split_path_owned(original));
    join_paths_simple(&parts)
}

/// Append entries to an existing PATH string
///
/// # Example
/// ```
/// use vx_paths::platform::append_to_path;
///
/// let original = "/usr/bin:/bin";
/// let new_entries = vec!["/my/custom/bin"];
/// let result = append_to_path(original, &new_entries);
/// // Result: "/usr/bin:/bin:/my/custom/bin"
/// ```
pub fn append_to_path<S: AsRef<str>>(original: &str, entries: &[S]) -> String {
    let mut parts: Vec<String> = split_path_owned(original);
    parts.extend(entries.iter().map(|s| s.as_ref().to_string()));
    join_paths_simple(&parts)
}

// =============================================================================
// Path String Utilities (avoiding Path::new for untrusted input)
// =============================================================================

/// Check if a path string looks like a Windows path
///
/// Uses string analysis only, avoiding `Path::new()` which can fail
/// on Unix when processing Windows-style paths.
#[inline]
pub fn is_windows_path(path: &str) -> bool {
    // Check for drive letter (C:, D:, etc.) or backslashes
    (path.len() >= 2 && path.chars().nth(1) == Some(':'))
        || path.contains('\\')
        || path.starts_with("\\\\") // UNC path
}

/// Check if a path string looks like a Unix path
///
/// Uses string analysis only.
#[inline]
pub fn is_unix_path(path: &str) -> bool {
    path.starts_with('/')
}

/// Check if a path string matches the current platform's format
#[inline]
pub fn is_native_path(path: &str) -> bool {
    if cfg!(windows) {
        is_windows_path(path)
    } else {
        is_unix_path(path)
    }
}

/// Normalize a path string for case-sensitive or case-insensitive comparison
///
/// - Windows/macOS: lowercase
/// - Linux: unchanged
pub fn normalize_for_comparison(path: &str) -> String {
    if cfg!(any(target_os = "windows", target_os = "macos")) {
        path.to_lowercase()
    } else {
        path.to_string()
    }
}

// =============================================================================
// Executable Utilities
// =============================================================================

/// Get the executable extension for the current platform
///
/// Convenience function using compile-time detection.
#[inline]
pub fn executable_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    }
}

/// Add executable extension to a name if needed
///
/// # Example
/// ```
/// use vx_paths::platform::with_executable_extension;
///
/// let name = with_executable_extension("node");
/// // Windows: "node.exe"
/// // Unix: "node"
/// ```
pub fn with_executable_extension(name: &str) -> String {
    // Don't add extension if already present
    if cfg!(windows)
        && !name.ends_with(".exe")
        && !name.ends_with(".cmd")
        && !name.ends_with(".bat")
    {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

/// Get the venv bin directory name for the current platform
#[inline]
pub fn venv_bin_dir() -> &'static str {
    if cfg!(windows) { "Scripts" } else { "bin" }
}

// =============================================================================
// System Path Detection
// =============================================================================

/// System PATH prefixes that should be inherited in isolated mode.
///
/// These directories contain essential system tools (sh, bash, cat, etc.)
/// that child processes may need.
pub const SYSTEM_PATH_PREFIXES: &[&str] = &[
    // Unix essential directories
    "/bin",
    "/usr/bin",
    "/usr/local/bin",
    "/sbin",
    "/usr/sbin",
    "/usr/local/sbin",
    // macOS Homebrew (Apple Silicon)
    "/opt/homebrew/bin",
    "/opt/homebrew/sbin",
    // macOS Homebrew (Intel)
    "/usr/local/Cellar",
    // Nix
    "/nix/var/nix/profiles/default/bin",
    "/run/current-system/sw/bin",
    // Windows (case-insensitive matching will be used)
    "C:\\Windows\\System32",
    "C:\\Windows\\SysWOW64",
    "C:\\Windows",
    "C:\\Windows\\System32\\Wbem",
    "C:\\Windows\\System32\\WindowsPowerShell",
    "C:\\Windows\\System32\\OpenSSH",
];

/// Check if a path is a system path that should be inherited
///
/// Uses string comparison only, avoiding `Path::new()` to prevent issues
/// with invalid path characters on different platforms.
pub fn is_system_path(path_str: &str) -> bool {
    // Normalize for comparison
    let normalized = if cfg!(windows) {
        path_str.to_lowercase()
    } else {
        path_str.to_string()
    };

    for prefix in SYSTEM_PATH_PREFIXES {
        // Skip prefixes that don't match the current platform
        let is_windows_prefix = is_windows_path(prefix);
        let is_unix_prefix = is_unix_path(prefix);

        if cfg!(windows) {
            if !is_windows_prefix {
                continue;
            }
            // Case-insensitive comparison on Windows
            if normalized.starts_with(&prefix.to_lowercase()) {
                return true;
            }
        } else {
            if !is_unix_prefix {
                continue;
            }
            // Case-sensitive on Unix
            if path_str.starts_with(prefix) || path_str == *prefix {
                return true;
            }
        }
    }

    false
}

/// Filter a PATH string to only include system directories
///
/// This is used in isolated mode to allow access to essential system tools
/// while excluding user-specific directories.
///
/// # Example
/// ```
/// use vx_paths::platform::filter_system_path;
///
/// let full_path = "/home/user/.local/bin:/usr/local/bin:/usr/bin:/bin";
/// let filtered = filter_system_path(full_path);
/// // Result: "/usr/local/bin:/usr/bin:/bin"
/// ```
pub fn filter_system_path(path: &str) -> String {
    let filtered: Vec<&str> = split_path(path)
        .filter(|entry| is_system_path(entry))
        .collect();

    join_paths_simple(&filtered)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_separator() {
        let sep = path_separator();
        if cfg!(windows) {
            assert_eq!(sep, ';');
        } else {
            assert_eq!(sep, ':');
        }
    }

    #[test]
    fn test_split_path() {
        if cfg!(windows) {
            let parts: Vec<_> = split_path("C:\\bin;D:\\tools").collect();
            assert_eq!(parts, vec!["C:\\bin", "D:\\tools"]);
        } else {
            let parts: Vec<_> = split_path("/usr/bin:/bin").collect();
            assert_eq!(parts, vec!["/usr/bin", "/bin"]);
        }
    }

    #[test]
    fn test_split_path_empty_entries() {
        if cfg!(windows) {
            let parts: Vec<_> = split_path("C:\\bin;;D:\\tools;").collect();
            assert_eq!(parts, vec!["C:\\bin", "D:\\tools"]);
        } else {
            let parts: Vec<_> = split_path("/usr/bin::/bin:").collect();
            assert_eq!(parts, vec!["/usr/bin", "/bin"]);
        }
    }

    #[test]
    fn test_join_paths_simple() {
        let paths = vec!["/usr/bin", "/bin"];
        let result = join_paths_simple(&paths);
        if cfg!(windows) {
            assert_eq!(result, "/usr/bin;/bin");
        } else {
            assert_eq!(result, "/usr/bin:/bin");
        }
    }

    #[test]
    fn test_is_windows_path() {
        assert!(is_windows_path("C:\\Windows"));
        assert!(is_windows_path("D:\\Program Files"));
        assert!(is_windows_path("\\\\server\\share"));
        assert!(is_windows_path("path\\with\\backslash"));
        assert!(!is_windows_path("/usr/bin"));
        assert!(!is_windows_path("/home/user"));
    }

    #[test]
    fn test_is_unix_path() {
        assert!(is_unix_path("/usr/bin"));
        assert!(is_unix_path("/home/user"));
        assert!(is_unix_path("/"));
        assert!(!is_unix_path("C:\\Windows"));
        assert!(!is_unix_path("relative/path"));
    }

    #[test]
    fn test_is_system_path_unix() {
        // These should match on Unix, not on Windows
        if !cfg!(windows) {
            assert!(is_system_path("/usr/bin"));
            assert!(is_system_path("/bin"));
            assert!(is_system_path("/usr/local/bin"));
            assert!(is_system_path("/opt/homebrew/bin"));
            assert!(!is_system_path("/home/user/.local/bin"));
            assert!(!is_system_path("/custom/path"));
        }
    }

    #[test]
    fn test_is_system_path_windows() {
        // These should match on Windows, not on Unix
        if cfg!(windows) {
            assert!(is_system_path("C:\\Windows\\System32"));
            assert!(is_system_path("c:\\windows\\system32")); // Case-insensitive
            assert!(is_system_path("C:\\Windows"));
            assert!(!is_system_path("C:\\Users\\test"));
            assert!(!is_system_path("D:\\custom\\path"));
        }
    }

    #[test]
    fn test_filter_system_path_unix() {
        if !cfg!(windows) {
            let path = "/home/user/.local/bin:/usr/local/bin:/usr/bin:/bin:/custom/path";
            let filtered = filter_system_path(path);
            assert_eq!(filtered, "/usr/local/bin:/usr/bin:/bin");
        }
    }

    #[test]
    fn test_filter_system_path_windows() {
        if cfg!(windows) {
            let path = "C:\\Users\\test;C:\\Windows\\System32;C:\\Windows;D:\\custom";
            let filtered = filter_system_path(path);
            assert_eq!(filtered, "C:\\Windows\\System32;C:\\Windows");
        }
    }

    #[test]
    fn test_executable_extension() {
        let ext = executable_extension();
        if cfg!(windows) {
            assert_eq!(ext, ".exe");
        } else {
            assert_eq!(ext, "");
        }
    }

    #[test]
    fn test_with_executable_extension() {
        let name = with_executable_extension("node");
        if cfg!(windows) {
            assert_eq!(name, "node.exe");
        } else {
            assert_eq!(name, "node");
        }

        // Should not double-add extension
        if cfg!(windows) {
            assert_eq!(with_executable_extension("node.exe"), "node.exe");
            assert_eq!(with_executable_extension("script.cmd"), "script.cmd");
        }
    }

    #[test]
    fn test_venv_bin_dir() {
        let dir = venv_bin_dir();
        if cfg!(windows) {
            assert_eq!(dir, "Scripts");
        } else {
            assert_eq!(dir, "bin");
        }
    }

    #[test]
    fn test_prepend_to_path() {
        // Use platform-appropriate separator in input
        let (original, expected) = if cfg!(windows) {
            ("/usr/bin;/bin", "/custom/bin;/usr/bin;/bin")
        } else {
            ("/usr/bin:/bin", "/custom/bin:/usr/bin:/bin")
        };
        let result = prepend_to_path(original, &["/custom/bin"]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_append_to_path() {
        // Use platform-appropriate separator in input
        let (original, expected) = if cfg!(windows) {
            ("/usr/bin;/bin", "/usr/bin;/bin;/custom/bin")
        } else {
            ("/usr/bin:/bin", "/usr/bin:/bin:/custom/bin")
        };
        let result = append_to_path(original, &["/custom/bin"]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();

        // Just verify it returns something sensible
        assert!(matches!(
            platform.os,
            Os::Windows | Os::MacOS | Os::Linux | Os::Other
        ));
        assert!(matches!(
            platform.arch,
            Arch::X86_64 | Arch::Aarch64 | Arch::X86 | Arch::Arm | Arch::Other
        ));
    }

    #[test]
    fn test_platform_as_str() {
        let platform = Platform::current();
        let s = platform.as_str();
        assert!(!s.is_empty());
        // Should contain a dash separator
        if s != "unknown" {
            assert!(s.contains('-'));
        }
    }
}
