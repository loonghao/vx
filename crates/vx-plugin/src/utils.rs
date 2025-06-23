//! Utility functions and helpers for plugin development
//!
//! This module provides common utilities that plugin developers can use
//! to simplify their implementations.

use crate::{Result, VersionInfo};
use std::path::Path;

/// Check if a command is available in the system PATH
///
/// This is useful for checking if a tool or package manager is installed
/// on the system before attempting to use it.
pub fn is_command_available(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Get the platform-specific executable extension
///
/// Returns ".exe" on Windows, empty string on other platforms.
pub fn get_exe_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    }
}

/// Get the platform-specific executable name
///
/// Adds the appropriate extension for the current platform.
pub fn get_exe_name(base_name: &str) -> String {
    format!("{}{}", base_name, get_exe_extension())
}

/// Check if a path exists and is executable
///
/// This function checks if a file exists and has execute permissions
/// (on Unix-like systems).
pub fn is_executable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(not(unix))]
    {
        // On Windows, if the file exists, assume it's executable
        // A more sophisticated check could look at file extensions
        true
    }
}

/// Find an executable in a directory
///
/// Searches for an executable with the given name in the specified directory,
/// trying common subdirectories like "bin", "Scripts", etc.
pub fn find_executable_in_dir(dir: &Path, exe_name: &str) -> Option<std::path::PathBuf> {
    let exe_name_with_ext = get_exe_name(exe_name);

    // Try common locations
    let candidates = vec![
        dir.join(&exe_name_with_ext),
        dir.join("bin").join(&exe_name_with_ext),
        dir.join("Scripts").join(&exe_name_with_ext), // Windows Python-style
        dir.join("sbin").join(&exe_name_with_ext),    // System binaries
    ];

    candidates
        .into_iter()
        .find(|candidate| is_executable(candidate))
}
/// Parse version string into components
///
/// Attempts to parse a semantic version string into major, minor, and patch components.
/// Returns None if the version string is not in a recognizable format.
pub fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.trim_start_matches('v').split('.').collect();

    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        // Handle patch versions that might have additional suffixes (e.g., "1-beta")
        let patch_str = parts[2].split('-').next().unwrap_or(parts[2]);
        let patch = patch_str.parse().ok()?;

        Some((major, minor, patch))
    } else {
        None
    }
}

/// Compare two version strings
///
/// Returns:
/// - `std::cmp::Ordering::Less` if `a < b`
/// - `std::cmp::Ordering::Equal` if `a == b`
/// - `std::cmp::Ordering::Greater` if `a > b`
/// - `None` if versions cannot be compared
pub fn compare_versions(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let (a_major, a_minor, a_patch) = parse_version(a)?;
    let (b_major, b_minor, b_patch) = parse_version(b)?;

    Some((a_major, a_minor, a_patch).cmp(&(b_major, b_minor, b_patch)))
}

/// Sort versions in descending order (newest first)
///
/// This function sorts a vector of version strings, placing the newest
/// versions first. Versions that cannot be parsed are placed at the end.
pub fn sort_versions_desc(versions: &mut [String]) {
    versions.sort_by(|a, b| {
        match compare_versions(a, b) {
            Some(ordering) => ordering.reverse(), // Reverse for descending order
            None => std::cmp::Ordering::Equal,
        }
    });
}

/// Check if a version is a prerelease
///
/// Returns true if the version string contains prerelease indicators
/// like "alpha", "beta", "rc", etc.
pub fn is_prerelease(version: &str) -> bool {
    let version_lower = version.to_lowercase();
    version_lower.contains("alpha")
        || version_lower.contains("beta")
        || version_lower.contains("rc")
        || version_lower.contains("pre")
        || version_lower.contains("dev")
        || version_lower.contains("snapshot")
}

/// Create a VersionInfo from a simple version string
///
/// This is a convenience function for creating VersionInfo objects
/// with automatic prerelease detection.
pub fn create_version_info(version: &str, download_url: Option<String>) -> VersionInfo {
    VersionInfo {
        version: version.to_string(),
        prerelease: is_prerelease(version),
        release_date: None,
        release_notes: None,
        download_url,
        checksum: None,
        file_size: None,
        metadata: std::collections::HashMap::new(),
    }
}
/// Validate a tool name
///
/// Checks if a tool name follows the expected conventions:
/// - Contains only alphanumeric characters, hyphens, and underscores
/// - Starts with a letter
/// - Is not empty and not too long
pub fn validate_tool_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Tool name cannot be empty"));
    }

    if name.len() > 64 {
        return Err(anyhow::anyhow!(
            "Tool name cannot be longer than 64 characters"
        ));
    }

    if !name.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(anyhow::anyhow!("Tool name must start with a letter"));
    }

    for ch in name.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' {
            return Err(anyhow::anyhow!(
                "Tool name can only contain letters, numbers, hyphens, and underscores"
            ));
        }
    }

    Ok(())
}

/// Validate a version string
///
/// Checks if a version string is in a valid format.
/// Accepts semantic versioning and other common version formats.
pub fn validate_version(version: &str) -> Result<()> {
    if version.is_empty() {
        return Err(anyhow::anyhow!("Version cannot be empty"));
    }

    // Allow 'v' prefix
    let version = version.strip_prefix('v').unwrap_or(version);

    // Check for basic version pattern (at least one number)
    if !version.chars().any(|c| c.is_ascii_digit()) {
        return Err(anyhow::anyhow!("Version must contain at least one number"));
    }

    // More sophisticated validation could be added here
    Ok(())
}

/// Get the default vx directory
///
/// Returns the default directory where vx stores its data.
/// This is typically `~/.vx` on Unix-like systems and `%USERPROFILE%\.vx` on Windows.
pub fn get_vx_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".vx")
}

/// Get the tools directory
///
/// Returns the directory where vx stores installed tools.
pub fn get_tools_dir() -> std::path::PathBuf {
    get_vx_dir().join("tools")
}

/// Get the plugins directory
///
/// Returns the directory where vx looks for plugins.
pub fn get_plugins_dir() -> std::path::PathBuf {
    get_vx_dir().join("plugins")
}

/// Resolve version string to actual version
///
/// If the version is "latest", this function will fetch the latest version
/// from the tool's version list. Otherwise, it returns the version as-is.
pub async fn resolve_version<T: crate::VxTool>(tool: &T, version: &str) -> Result<String> {
    if version == "latest" {
        let versions = tool.fetch_versions(false).await?;
        if versions.is_empty() {
            return Err(anyhow::anyhow!("No versions found for {}", tool.name()));
        }
        Ok(versions[0].version.clone())
    } else {
        Ok(version.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2.3-beta"), Some((1, 2, 3)));
        assert_eq!(parse_version("invalid"), None);
    }

    #[test]
    fn test_is_prerelease() {
        assert!(is_prerelease("1.0.0-alpha"));
        assert!(is_prerelease("1.0.0-beta.1"));
        assert!(is_prerelease("1.0.0-rc.1"));
        assert!(!is_prerelease("1.0.0"));
    }

    #[test]
    fn test_validate_tool_name() {
        assert!(validate_tool_name("node").is_ok());
        assert!(validate_tool_name("my-tool").is_ok());
        assert!(validate_tool_name("tool_name").is_ok());
        assert!(validate_tool_name("").is_err());
        assert!(validate_tool_name("123tool").is_err());
        assert!(validate_tool_name("tool@name").is_err());
    }
}
