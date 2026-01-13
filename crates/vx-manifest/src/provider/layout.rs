//! Executable Layout Configuration (RFC 0019)
//!
//! This module defines the layout configuration for how executables are
//! organized within downloaded archives.
//!
//! ## Example provider.toml (Archive)
//!
//! ```toml
//! [runtimes.layout]
//! download_type = "archive"
//!
//! [runtimes.layout.archive]
//! strip_prefix = "node-v{version}-{platform}-{arch}"
//! executable_paths = [
//!     "bin/node.exe",  # Windows
//!     "bin/node"       # Unix
//! ]
//! ```
//!
//! ## Example provider.toml (Binary with platform-specific config)
//!
//! ```toml
//! [runtimes.layout]
//! download_type = "binary"
//!
//! [runtimes.layout.binary."windows-x86_64"]
//! source_name = "ninja.exe"
//! target_name = "ninja.exe"
//! target_dir = "bin"
//!
//! [runtimes.layout.binary."linux-x86_64"]
//! source_name = "ninja"
//! target_name = "ninja"
//! target_dir = "bin"
//! target_permissions = "755"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Download type for the runtime
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadType {
    /// Archive file (tar.gz, zip, etc.)
    #[default]
    Archive,
    /// Single binary file
    Binary,
    /// Installer (msi, pkg, etc.) - not recommended
    Installer,
}

/// Archive-specific layout configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiveLayoutConfig {
    /// Prefix to strip from archive paths
    /// Supports placeholders: {version}, {platform}, {arch}, {os}
    #[serde(default)]
    pub strip_prefix: Option<String>,

    /// Paths to executables within the archive (after stripping prefix)
    /// First matching path is used
    /// Supports placeholders: {version}, {platform}, {arch}, {os}
    #[serde(default)]
    pub executable_paths: Vec<String>,

    /// Additional files/directories to preserve
    #[serde(default)]
    pub preserve_paths: Vec<String>,
}

impl ArchiveLayoutConfig {
    /// Get the executable path for the current platform
    pub fn get_executable_path(&self) -> Option<&str> {
        if self.executable_paths.is_empty() {
            return None;
        }

        // On Windows, prefer .exe paths
        #[cfg(windows)]
        {
            for path in &self.executable_paths {
                if path.ends_with(".exe") || path.ends_with(".cmd") || path.ends_with(".bat") {
                    return Some(path);
                }
            }
        }

        // On Unix, prefer paths without extension
        #[cfg(not(windows))]
        {
            for path in &self.executable_paths {
                if !path.contains('.') || path.ends_with('/') {
                    return Some(path);
                }
            }
        }

        // Fallback to first path
        self.executable_paths.first().map(|s| s.as_str())
    }

    /// Expand placeholders in a path
    pub fn expand_path(&self, path: &str, version: &str, platform: &str, arch: &str) -> String {
        path.replace("{version}", version)
            .replace("{platform}", platform)
            .replace("{arch}", arch)
            .replace("{os}", std::env::consts::OS)
    }
}

/// Platform-specific binary layout configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformBinaryConfig {
    /// Source filename in the download
    #[serde(default)]
    pub source_name: Option<String>,

    /// Target filename after installation
    #[serde(default)]
    pub target_name: Option<String>,

    /// Target directory (relative to install path)
    #[serde(default)]
    pub target_dir: Option<String>,

    /// File permissions (Unix only, e.g., "755")
    #[serde(default)]
    pub target_permissions: Option<String>,
}

impl PlatformBinaryConfig {
    /// Get the executable path for this platform config
    pub fn get_executable_path(&self) -> Option<String> {
        let name = self.target_name.as_ref()?;
        if let Some(dir) = &self.target_dir {
            Some(format!("{}/{}", dir, name))
        } else {
            Some(name.clone())
        }
    }
}

/// Binary-specific layout configuration
/// Can be either a simple config or a map of platform-specific configs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BinaryLayoutConfig {
    /// Simple binary config (same for all platforms)
    #[default]
    Simple,

    /// Platform-specific binary configs
    /// Keys are like "windows-x86_64", "linux-x86_64", "macos-aarch64"
    PlatformSpecific(HashMap<String, PlatformBinaryConfig>),
}

impl BinaryLayoutConfig {
    /// Get the current platform key
    fn current_platform_key() -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        format!("{}-{}", os, arch)
    }

    /// Get the executable path for the current platform
    pub fn get_executable_path(&self) -> Option<String> {
        match self {
            BinaryLayoutConfig::Simple => None,
            BinaryLayoutConfig::PlatformSpecific(configs) => {
                let key = Self::current_platform_key();
                configs.get(&key).and_then(|c| c.get_executable_path())
            }
        }
    }

    /// Get the platform-specific config for the current platform
    pub fn get_current_platform_config(&self) -> Option<&PlatformBinaryConfig> {
        match self {
            BinaryLayoutConfig::Simple => None,
            BinaryLayoutConfig::PlatformSpecific(configs) => {
                let key = Self::current_platform_key();
                configs.get(&key)
            }
        }
    }
}

/// Layout configuration for a runtime
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Download type
    #[serde(default)]
    pub download_type: DownloadType,

    /// Archive-specific configuration
    #[serde(default)]
    pub archive: Option<ArchiveLayoutConfig>,

    /// Binary-specific configuration
    #[serde(default)]
    pub binary: Option<BinaryLayoutConfig>,
}

impl LayoutConfig {
    /// Check if this is an archive layout
    pub fn is_archive(&self) -> bool {
        matches!(self.download_type, DownloadType::Archive)
    }

    /// Check if this is a binary layout
    pub fn is_binary(&self) -> bool {
        matches!(self.download_type, DownloadType::Binary)
    }

    /// Get the executable path for the current platform
    pub fn get_executable_path(&self) -> Option<&str> {
        // First try archive config
        if let Some(archive) = &self.archive {
            if let Some(path) = archive.get_executable_path() {
                return Some(path);
            }
        }

        // Binary config returns owned String, so we can't return a reference
        // This is a limitation - callers should use get_executable_path_owned() for binary
        None
    }

    /// Get the executable path as an owned String (works for both archive and binary)
    pub fn get_executable_path_owned(&self) -> Option<String> {
        // First try archive config
        if let Some(archive) = &self.archive {
            if let Some(path) = archive.get_executable_path() {
                return Some(path.to_string());
            }
        }

        // Then try binary config
        if let Some(binary) = &self.binary {
            if let Some(path) = binary.get_executable_path() {
                return Some(path);
            }
        }

        None
    }

    /// Get the strip prefix (if any)
    pub fn get_strip_prefix(&self) -> Option<&str> {
        self.archive
            .as_ref()
            .and_then(|a| a.strip_prefix.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_layout_executable_path() {
        let config = ArchiveLayoutConfig {
            strip_prefix: Some("node-v{version}-{platform}-{arch}".to_string()),
            executable_paths: vec!["bin/node.exe".to_string(), "bin/node".to_string()],
            preserve_paths: vec![],
        };

        let path = config.get_executable_path();
        assert!(path.is_some());

        #[cfg(windows)]
        assert_eq!(path, Some("bin/node.exe"));

        #[cfg(not(windows))]
        assert_eq!(path, Some("bin/node"));
    }

    #[test]
    fn test_expand_path() {
        let config = ArchiveLayoutConfig::default();
        let expanded = config.expand_path("node-v{version}-{platform}-{arch}", "20.0.0", "linux", "x64");
        assert_eq!(expanded, "node-v20.0.0-linux-x64");
    }

    #[test]
    fn test_layout_config_defaults() {
        let config = LayoutConfig::default();
        assert!(config.is_archive());
        assert!(!config.is_binary());
    }

    #[test]
    fn test_platform_binary_config() {
        let mut configs = HashMap::new();
        configs.insert(
            "windows-x86_64".to_string(),
            PlatformBinaryConfig {
                source_name: Some("ninja.exe".to_string()),
                target_name: Some("ninja.exe".to_string()),
                target_dir: Some("bin".to_string()),
                target_permissions: None,
            },
        );

        let binary = BinaryLayoutConfig::PlatformSpecific(configs);

        #[cfg(all(windows, target_arch = "x86_64"))]
        {
            let path = binary.get_executable_path();
            assert_eq!(path, Some("bin/ninja.exe".to_string()));
        }
    }
}
