//! Node.js installation configuration
//!
//! This module provides Node.js-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleAction, LifecycleHooks};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for Node.js tool
pub struct Config;

/// Node.js URL builder for consistent download URL generation
pub struct NodeUrlBuilder;

impl StandardUrlBuilder for NodeUrlBuilder {
    /// Generate download URL for Node.js version
    fn download_url(version: &str) -> Option<String> {
        let _platform = Self::get_platform_string();
        let filename = Self::get_filename(version);

        Some(format!("https://nodejs.org/dist/v{}/{}", version, filename))
    }

    /// Get platform-specific filename
    fn get_filename(version: &str) -> String {
        let platform = Self::get_platform_string();
        if cfg!(windows) {
            format!("node-v{}-{}.zip", version, platform)
        } else {
            format!("node-v{}-{}.tar.gz", version, platform)
        }
    }

    /// Get platform string for Node.js downloads
    fn get_platform_string() -> String {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "win-x64".to_string(),
            ("windows", "x86") => "win-x86".to_string(),
            ("windows", "aarch64") => "win-arm64".to_string(),
            ("macos", "x86_64") => "darwin-x64".to_string(),
            ("macos", "aarch64") => "darwin-arm64".to_string(),
            ("linux", "x86_64") => "linux-x64".to_string(),
            ("linux", "x86") => "linux-x86".to_string(),
            ("linux", "aarch64") => "linux-arm64".to_string(),
            ("linux", "arm") => "linux-armv7l".to_string(),
            _ => "linux-x64".to_string(), // Default fallback
        }
    }
}

/// Implementation of standard tool configuration for Node.js
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "node"
    }

    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
        create_install_config(version, install_dir)
    }

    fn get_install_methods() -> Vec<String> {
        get_install_methods()
    }

    fn supports_auto_install() -> bool {
        true
    }

    fn get_manual_instructions() -> String {
        get_manual_instructions()
    }

    fn get_dependencies() -> Vec<ToolDependency> {
        // Node.js has no dependencies - it's a base runtime
        vec![]
    }

    fn get_default_version() -> &'static str {
        "20.11.0" // LTS version
    }
}

/// Create Node.js installation configuration
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "20.11.0" // Default to LTS version
    } else {
        version
    };

    let download_url = NodeUrlBuilder::download_url(actual_version);
    let install_method = InstallMethod::Archive {
        format: if cfg!(windows) {
            ArchiveFormat::Zip
        } else {
            ArchiveFormat::TarGz
        },
    };

    // Create lifecycle hooks to optimize path structure
    let mut hooks = LifecycleHooks::default();

    // Post-install action to flatten the directory structure
    // Node.js archives extract to node-v{version}-{platform}/ subdirectory
    // We want to move everything to the root install directory
    let platform = NodeUrlBuilder::get_platform_string();
    let archive_subdir = format!("node-v{}-{}", actual_version, platform);

    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: archive_subdir,
    });

    // Add health check to verify Node.js installation
    // Use --version flag to avoid hanging and get a quick response
    hooks
        .post_install
        .push(LifecycleAction::ValidateInstallation {
            command: if cfg!(windows) {
                "node.exe --version".to_string()
            } else {
                "node --version".to_string()
            },
            expected_output: Some("v".to_string()), // Node.js version starts with 'v'
        });

    // Cleanup any temporary files
    hooks.post_install.push(LifecycleAction::CleanupTemp {
        pattern: ".tmp".to_string(),
    });

    InstallConfig::builder()
        .tool_name("node")
        .version(version.to_string())
        .install_method(install_method)
        .download_url(download_url.unwrap_or_default())
        .install_dir(install_dir)
        .lifecycle_hooks(hooks)
        .build()
}

/// Get available Node.js installation methods
pub fn get_install_methods() -> Vec<String> {
    vec![
        "Official releases".to_string(),
        if cfg!(target_os = "macos") {
            "Homebrew".to_string()
        } else {
            "Package manager".to_string()
        },
        "Node Version Manager (nvm)".to_string(),
    ]
}

/// Check if Node.js supports automatic installation
pub fn supports_auto_install() -> bool {
    true
}

/// Get manual installation instructions for Node.js
pub fn get_manual_instructions() -> String {
    "To install Node.js manually:\n\
     • Visit: https://nodejs.org/\n\
     • Or use a version manager like nvm"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_url_builder() {
        let url = NodeUrlBuilder::download_url("18.17.0");
        assert!(url.is_some());
        assert!(url.unwrap().contains("nodejs.org"));
    }

    #[test]
    fn test_platform_string() {
        let platform = NodeUrlBuilder::get_platform_string();
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("18.17.0", PathBuf::from("/tmp/node"));
        assert_eq!(config.tool_name, "node");
        assert_eq!(config.version, "18.17.0");
        assert!(config.download_url.is_some());
    }

    #[test]
    fn test_latest_version_handling() {
        let config = create_install_config("latest", PathBuf::from("/tmp/node"));
        assert_eq!(config.version, "latest");
        // Should use actual version in URL
        assert!(config.download_url.unwrap().contains("20.11.0"));
    }
}
