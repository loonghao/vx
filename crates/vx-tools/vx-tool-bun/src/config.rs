//! Bun runtime configuration
//!
//! This module provides Bun-specific configuration,
//! including dependencies and installation methods.

use std::path::PathBuf;
use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleAction, LifecycleHooks};
use vx_tool_standard::{StandardToolConfig, StandardUrlBuilder, ToolDependency};

/// Standard configuration for Bun tool
pub struct Config;

/// Bun URL builder for consistent download URL generation
pub struct BunUrlBuilder;

impl StandardUrlBuilder for BunUrlBuilder {
    /// Generate download URL for Bun version
    fn download_url(version: &str) -> Option<String> {
        let filename = Self::get_filename(version);
        Some(format!(
            "https://github.com/oven-sh/bun/releases/download/bun-v{}/{}",
            version, filename
        ))
    }

    /// Get platform-specific filename
    fn get_filename(_version: &str) -> String {
        let platform = Self::get_platform_string();
        format!("bun-{}.zip", platform)
    }

    /// Get platform string for Bun downloads - USES BUN-SPECIFIC FORMAT
    fn get_platform_string() -> String {
        // Bun uses a different platform naming convention
        // Convert from standard platform string to Bun format
        let standard_platform = vx_config::get_platform_string();

        // Map standard platform strings to Bun format
        match standard_platform.as_str() {
            "x86_64-pc-windows-msvc" => "windows-x64".to_string(),
            "aarch64-pc-windows-msvc" => "windows-aarch64".to_string(),
            "x86_64-apple-darwin" => "darwin-x64".to_string(),
            "aarch64-apple-darwin" => "darwin-aarch64".to_string(),
            "x86_64-unknown-linux-gnu" => "linux-x64".to_string(),
            "aarch64-unknown-linux-gnu" => "linux-aarch64".to_string(),
            _ => "linux-x64".to_string(), // Default fallback
        }
    }
}

/// Implementation of standard tool configuration for Bun
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "bun"
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
        // Bun is a standalone runtime with no dependencies
        vec![]
    }

    fn get_default_version() -> &'static str {
        "1.0.0" // Latest stable version
    }
}

/// Create Bun installation configuration
pub async fn create_install_config(
    version: &str,
    install_dir: PathBuf,
) -> anyhow::Result<InstallConfig> {
    let actual_version = if version == "latest" {
        "1.2.9" // Default to stable version
    } else {
        version
    };

    // Use vx-config system to get download URL - NO MORE HARDCODING!
    use vx_config::{get_tool_download_url, ConfigManager, VersionParser};

    let config_manager = ConfigManager::new().await?;
    let config = config_manager.config();

    // Use version parser to clean version properly
    let clean_version = if let Some(tool_config) = config.tools.get("bun") {
        if let Some(version_parser) = VersionParser::from_tool_config(tool_config) {
            version_parser
                .parse_tag(actual_version)
                .unwrap_or_else(|_| actual_version.to_string())
        } else {
            actual_version.to_string()
        }
    } else {
        actual_version.to_string()
    };

    let download_url = get_tool_download_url(config, "bun", &clean_version)
        .ok_or_else(|| anyhow::anyhow!("No download URL configured for bun"))?;
    let install_method = InstallMethod::Archive {
        format: ArchiveFormat::Zip,
    };

    // Create lifecycle hooks to optimize path structure
    let mut hooks = LifecycleHooks::default();

    // Bun archives have nested structure: bun-v{version}/bun-{platform}/bun.exe
    // We need to flatten this to just have bun.exe in the root
    let platform_dir = if cfg!(windows) {
        "bun-windows-x64"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "bun-darwin-aarch64"
        } else {
            "bun-darwin-x64"
        }
    } else if cfg!(target_arch = "aarch64") {
        "bun-linux-aarch64"
    } else {
        "bun-linux-x64"
    };

    // First flatten the version directory (bun-v{version})
    let version_dir = format!("bun-v{}", clean_version);
    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: version_dir,
    });

    // Then flatten the platform directory
    hooks.post_install.push(LifecycleAction::FlattenDirectory {
        source_pattern: platform_dir.to_string(),
    });

    // Set executable permissions on Unix systems
    #[cfg(unix)]
    hooks.post_install.push(LifecycleAction::SetExecutable {
        path: "bun".to_string(),
    });

    // Add health check to verify Bun installation
    let exe_name = if cfg!(windows) { "bun.exe" } else { "bun" };
    hooks
        .post_install
        .push(LifecycleAction::ValidateInstallation {
            command: format!("{} --version", exe_name),
            expected_output: None, // Just check that the command runs successfully
        });

    Ok(InstallConfig::builder()
        .tool_name("bun")
        .version(clean_version)
        .install_method(install_method)
        .download_url(download_url)
        .install_dir(install_dir)
        .lifecycle_hooks(hooks)
        .build())
}

/// Get available Bun installation methods
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

/// Get manual installation instructions for Bun
pub fn get_manual_instructions() -> String {
    "To install Bun manually:\n\
     • curl -fsSL https://bun.sh/install | bash\n\
     • Or visit: https://bun.sh/docs/installation\n\
     • No dependencies required"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bun_dependencies() {
        let deps = Config::get_dependencies();
        assert_eq!(deps.len(), 0); // Bun has no dependencies
    }

    #[test]
    fn test_bun_config() {
        assert_eq!(Config::tool_name(), "bun");
        assert!(Config::supports_auto_install());
        assert_eq!(Config::get_default_version(), "1.0.0");
    }

    #[test]
    fn test_bun_url_builder() {
        let url = BunUrlBuilder::download_url("1.0.0");
        assert!(url.is_some());
        assert!(url
            .expect("URL should be generated")
            .contains("github.com/oven-sh/bun"));
    }

    #[tokio::test]
    async fn test_create_install_config() {
        let config = create_install_config("latest", PathBuf::from("/tmp/bun"))
            .await
            .expect("Should create install config");
        assert_eq!(config.tool_name, "bun");
        assert_eq!(config.version, "1.2.9"); // Should use actual version for consistency
    }
}
