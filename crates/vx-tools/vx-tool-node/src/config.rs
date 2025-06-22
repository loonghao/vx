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
    /// Generate download URL for Node.js version - USES GLOBAL CONFIG
    fn download_url(version: &str) -> Option<String> {
        // IMPORTANT: This method should NOT receive "latest" as version
        // The caller should resolve "latest" to actual version first
        if version == "latest" {
            eprintln!(
                "Warning: download_url received 'latest' version, this should be resolved first"
            );
            return None;
        }

        // Use global config system for sync access
        use vx_config::get_tool_download_url_sync;

        // Try to get URL from config first
        if let Some(url) = get_tool_download_url_sync("node", version) {
            return Some(url);
        }

        // Fallback to hardcoded logic with actual version
        let filename = if cfg!(windows) {
            format!("node-v{}-win-x64.zip", version)
        } else if cfg!(target_os = "macos") {
            format!("node-v{}-darwin-x64.tar.gz", version)
        } else {
            format!("node-v{}-linux-x64.tar.xz", version)
        };

        Some(format!("https://nodejs.org/dist/v{}/{}", version, filename))
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
                get_tool_filename(config, "node", version)
            }) {
                return filename;
            }
        }

        // Fallback if config system fails
        format!("node-v{}-linux-x64.tar.gz", version)
    }

    /// Get platform string - DEPRECATED, USE CONFIG SYSTEM!
    fn get_platform_string() -> String {
        // This method is deprecated - platform detection should use vx-config
        "deprecated-use-config".to_string()
    }
}

/// Implementation of standard tool configuration for Node.js
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "node"
    }

    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
        // This is a sync wrapper - in practice, this should be avoided
        // and the async version should be used directly
        tokio::runtime::Handle::current()
            .block_on(create_install_config(version, install_dir))
            .expect("Failed to create install config")
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

/// Create Node.js installation configuration - USES GLOBAL CONFIG
pub async fn create_install_config(
    version: &str,
    install_dir: PathBuf,
) -> anyhow::Result<InstallConfig> {
    // IMPORTANT: This function should NOT receive "latest" as version
    // The caller should resolve "latest" to actual version first
    if version == "latest" {
        return Err(anyhow::anyhow!(
            "create_install_config received 'latest' version, this should be resolved first"
        ));
    }

    // Use global config system for sync access
    use vx_config::{get_global_config, get_tool_download_url, VersionParser};

    let config = get_global_config();

    // Use version parser to clean version properly
    let clean_version = if let Some(tool_config) = config.tools.get("node") {
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

    let download_url = get_tool_download_url(config, "node", &clean_version)
        .ok_or_else(|| anyhow::anyhow!("No download URL configured for node"))?;

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
    // Use platform-specific directory names
    let archive_subdir = if cfg!(windows) {
        format!("node-v{}-win-x64", clean_version)
    } else if cfg!(target_os = "macos") {
        format!("node-v{}-darwin-x64", clean_version)
    } else {
        format!("node-v{}-linux-x64", clean_version)
    };

    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: archive_subdir,
    });

    // Add health check to verify Node.js installation
    // Use config system to get executable name
    let exe_name = if cfg!(windows) { "node.exe" } else { "node" };

    hooks
        .post_install
        .push(LifecycleAction::ValidateInstallation {
            command: format!("{} --version", exe_name),
            expected_output: Some("v".to_string()), // Node.js version starts with 'v'
        });

    // Cleanup any temporary files
    hooks.post_install.push(LifecycleAction::CleanupTemp {
        pattern: ".tmp".to_string(),
    });

    Ok(InstallConfig::builder()
        .tool_name("node")
        .version(clean_version)
        .install_method(install_method)
        .download_url(download_url)
        .install_dir(install_dir)
        .lifecycle_hooks(hooks)
        .build())
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

    #[tokio::test]
    async fn test_create_install_config() {
        let config = create_install_config("18.17.0", PathBuf::from("/tmp/node"))
            .await
            .unwrap();
        assert_eq!(config.tool_name, "node");
        assert_eq!(config.version, "18.17.0");
        assert!(config.download_url.is_some());
    }

    #[tokio::test]
    async fn test_latest_version_handling() {
        let config = create_install_config("latest", PathBuf::from("/tmp/node"))
            .await
            .unwrap();
        assert_eq!(config.version, "latest");
        // Should use actual version in URL
        assert!(config
            .download_url
            .as_ref()
            .map_or(false, |url| url.contains("nodejs.org")));
    }
}
