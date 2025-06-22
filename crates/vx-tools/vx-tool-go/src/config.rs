//! Go installation configuration
//!
//! This module provides Go-specific installation configuration,
//! including URL building, platform detection, and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleAction, LifecycleHooks};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for Go tool
#[allow(dead_code)]
pub struct Config;

/// Go URL builder for consistent download URL generation
pub struct GoUrlBuilder;

impl StandardUrlBuilder for GoUrlBuilder {
    /// Generate download URL for Go version - NOW USES CONFIG SYSTEM!
    fn download_url(version: &str) -> Option<String> {
        // Use vx-config system instead of hardcoding
        use vx_config::{get_tool_download_url, ConfigManager, VersionParser};

        // This is a sync wrapper - ideally we'd make this async
        let rt = tokio::runtime::Handle::try_current()
            .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
            .ok()?;

        rt.block_on(async {
            let config_manager = ConfigManager::new().await.ok()?;
            let config = config_manager.config();

            // Use version parser to clean version
            let clean_version = if let Some(tool_config) = config.tools.get("go") {
                if let Some(version_parser) = VersionParser::from_tool_config(tool_config) {
                    version_parser
                        .parse_tag(version)
                        .unwrap_or_else(|_| version.to_string())
                } else {
                    version.to_string()
                }
            } else {
                version.to_string()
            };

            get_tool_download_url(config, "go", &clean_version)
        })
    }

    /// Get platform-specific filename - NOW USES CONFIG SYSTEM!
    fn get_filename(version: &str) -> String {
        // Use vx-config system to get platform-specific filename
        use vx_config::{get_tool_filename, ConfigManager};

        let rt = tokio::runtime::Handle::try_current()
            .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()));

        if let Ok(rt) = rt {
            if let Some(filename) = rt.block_on(async {
                let config_manager = ConfigManager::new().await.ok()?;
                let config = config_manager.config();
                get_tool_filename(config, "go", version)
            }) {
                return filename;
            }
        }

        // Fallback if config system fails
        format!("go{}.linux-amd64.tar.gz", version)
    }

    /// Get platform string - DEPRECATED, USE CONFIG SYSTEM!
    fn get_platform_string() -> String {
        // This method is deprecated - platform detection should use vx-config
        "deprecated-use-config".to_string()
    }
}

/// Implementation of standard tool configuration for Go
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "go"
    }

    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
        // Use simplified sync implementation to avoid runtime conflicts
        create_install_config_sync(version, install_dir)
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
        // Go has no dependencies - it's a standalone runtime
        vec![]
    }

    fn get_default_version() -> &'static str {
        "1.21.6" // Stable version
    }
}

/// Create Go installation configuration (sync version) - USES GLOBAL CONFIG
pub fn create_install_config_sync(version: &str, install_dir: PathBuf) -> InstallConfig {
    // Use global config system for sync access
    use vx_config::{get_global_config, get_tool_download_url, VersionParser};

    let config = get_global_config();

    // Use version parser to clean version properly
    let clean_version = if let Some(tool_config) = config.tools.get("go") {
        if let Some(version_parser) = VersionParser::from_tool_config(tool_config) {
            version_parser
                .parse_tag(version)
                .unwrap_or_else(|_| version.to_string())
        } else {
            version.to_string()
        }
    } else {
        version.to_string()
    };

    let download_url = get_tool_download_url(config, "go", &clean_version).unwrap_or_else(|| {
        // Fallback URL if config system fails
        format!(
            "https://golang.org/dl/go{}.linux-amd64.tar.gz",
            clean_version
        )
    });

    let install_method = InstallMethod::Archive {
        format: ArchiveFormat::TarGz,
    };

    // Create lifecycle hooks to optimize path structure
    let mut hooks = LifecycleHooks::default();
    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: "go".to_string(),
    });

    InstallConfig::builder()
        .tool_name("go")
        .version(clean_version)
        .install_method(install_method)
        .download_url(download_url)
        .install_dir(install_dir)
        .lifecycle_hooks(hooks)
        .build()
}

/// Create Go installation configuration (async version) - USES GLOBAL CONFIG
pub async fn create_install_config(
    version: &str,
    install_dir: PathBuf,
) -> anyhow::Result<InstallConfig> {
    // Use global config system for sync access
    use vx_config::{get_global_config, get_tool_download_url, VersionParser};

    let config = get_global_config();

    // Use version parser to clean version properly
    let clean_version = if let Some(tool_config) = config.tools.get("go") {
        if let Some(version_parser) = VersionParser::from_tool_config(tool_config) {
            version_parser
                .parse_tag(version)
                .unwrap_or_else(|_| version.to_string())
        } else {
            version.to_string()
        }
    } else {
        version.to_string()
    };

    let download_url = get_tool_download_url(config, "go", &clean_version)
        .ok_or_else(|| anyhow::anyhow!("No download URL configured for go"))?;

    let install_method = InstallMethod::Archive {
        format: ArchiveFormat::TarGz, // Config system will handle platform-specific formats
    };

    // Create lifecycle hooks to optimize path structure
    let mut hooks = LifecycleHooks::default();

    // Go archives extract to go/ subdirectory
    // We want to move everything from go/ to the root install directory
    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: "go".to_string(),
    });

    Ok(InstallConfig::builder()
        .tool_name("go")
        .version(clean_version)
        .install_method(install_method)
        .download_url(download_url)
        .install_dir(install_dir)
        .lifecycle_hooks(hooks)
        .build())
}

/// Get available Go installation methods
pub fn get_install_methods() -> Vec<String> {
    vec![
        "Official releases".to_string(),
        if cfg!(target_os = "macos") {
            "Homebrew".to_string()
        } else {
            "Package manager".to_string()
        },
    ]
}

/// Check if Go supports automatic installation
#[allow(dead_code)]
pub fn supports_auto_install() -> bool {
    true
}

/// Get manual installation instructions for Go
pub fn get_manual_instructions() -> String {
    // Use global config system for sync access
    vx_config::get_tool_manual_instructions_sync("go")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_url_builder() {
        let url = GoUrlBuilder::download_url("1.21.6");
        assert!(url.is_some());
        assert!(url.unwrap().contains("golang.org"));
    }

    #[test]
    fn test_platform_string() {
        let platform = GoUrlBuilder::get_platform_string();
        assert!(!platform.is_empty());
    }

    #[tokio::test]
    async fn test_create_install_config() {
        let config = create_install_config("1.21.6", PathBuf::from("/tmp/go"))
            .await
            .unwrap();
        assert_eq!(config.tool_name, "go");
        assert_eq!(config.version, "1.21.6");
        assert!(config.download_url.is_some());
    }

    #[tokio::test]
    async fn test_latest_version_handling() {
        let config = create_install_config("latest", PathBuf::from("/tmp/go"))
            .await
            .unwrap();
        assert_eq!(config.version, "latest");
        // Should use actual version in URL
        assert!(config
            .download_url
            .as_ref()
            .map_or(false, |url| url.contains("golang.org")));
    }
}
