//! Yarn package manager configuration
//!
//! This module provides Yarn-specific configuration,
//! including dependencies and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleAction, LifecycleHooks};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for Yarn tool
pub struct Config;

/// Yarn URL builder for consistent download URL generation
pub struct YarnUrlBuilder;

impl StandardUrlBuilder for YarnUrlBuilder {
    /// Generate download URL for Yarn version
    fn download_url(version: &str) -> Option<String> {
        Some(format!(
            "https://github.com/yarnpkg/yarn/releases/download/v{}/yarn-v{}.tar.gz",
            version, version
        ))
    }

    /// Get platform-specific filename
    fn get_filename(version: &str) -> String {
        format!("yarn-v{}.tar.gz", version)
    }

    /// Get platform string for Yarn downloads
    fn get_platform_string() -> String {
        // Yarn is platform-independent (JavaScript)
        "universal".to_string()
    }
}

/// Implementation of standard tool configuration for Yarn
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "yarn"
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
        vec![
            ToolDependency::required("node", "Yarn requires Node.js runtime")
                .with_version(">=16.10.0"),
        ]
    }

    fn get_default_version() -> &'static str {
        "1.22.19" // Latest v1.x version
    }
}

/// Create Yarn installation configuration
pub fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "1.22.19" // Default to stable v1.x version
    } else {
        version
    };

    let download_url = YarnUrlBuilder::download_url(actual_version);
    let install_method = InstallMethod::Archive {
        format: ArchiveFormat::TarGz,
    };

    // Create lifecycle hooks to optimize path structure
    let mut hooks = LifecycleHooks::default();

    // Yarn archives extract to yarn-v{version}/ subdirectory
    // We want to move everything to the root install directory
    let archive_subdir = format!("yarn-v{}", actual_version);

    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: archive_subdir,
    });

    // Set executable permissions on Unix systems
    #[cfg(unix)]
    hooks.post_install.push(LifecycleAction::SetExecutable {
        path: "bin/yarn".to_string(),
    });

    // Add health check to verify Yarn installation
    hooks
        .post_install
        .push(LifecycleAction::ValidateInstallation {
            command: if cfg!(windows) {
                "bin/yarn.cmd --version".to_string()
            } else {
                "bin/yarn --version".to_string()
            },
            expected_output: Some("1.".to_string()), // Yarn v1.x version starts with '1.'
        });

    InstallConfig::builder()
        .tool_name("yarn")
        .version(actual_version.to_string())
        .install_method(install_method)
        .download_url(download_url.unwrap_or_default())
        .install_dir(install_dir)
        .lifecycle_hooks(hooks)
        .build()
}

/// Get available Yarn installation methods
pub fn get_install_methods() -> Vec<String> {
    vec![
        "GitHub releases (recommended)".to_string(),
        "NPM global install".to_string(),
        if cfg!(target_os = "macos") {
            "Homebrew".to_string()
        } else {
            "Package manager".to_string()
        },
    ]
}

/// Get manual installation instructions for Yarn
pub fn get_manual_instructions() -> String {
    "To install Yarn manually:\n\
     • npm install -g yarn\n\
     • Or visit: https://yarnpkg.com/getting-started/install\n\
     • Requires Node.js >=16.10.0"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yarn_dependencies() {
        let deps = Config::get_dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].tool_name, "node");
        assert!(deps[0].required);
        assert_eq!(deps[0].version_requirement, Some(">=16.10.0".to_string()));
    }

    #[test]
    fn test_yarn_config() {
        assert_eq!(Config::tool_name(), "yarn");
        assert!(Config::supports_auto_install());
        assert_eq!(Config::get_default_version(), "1.22.19");
    }

    #[test]
    fn test_yarn_url_builder() {
        let url = YarnUrlBuilder::download_url("1.22.19");
        assert!(url.is_some());
        assert!(url
            .expect("URL should be generated")
            .contains("github.com/yarnpkg/yarn"));
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("latest", PathBuf::from("/tmp/yarn"));
        assert_eq!(config.tool_name, "yarn");
        assert_eq!(config.version, "1.22.19"); // Should resolve "latest" to actual version
    }
}
