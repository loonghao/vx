//! UV tool implementations - Python package management tools

use anyhow::Result;
use std::collections::HashMap;
use vx_plugin::{ToolContext, ToolExecutionResult, VersionInfo, VxTool};
use vx_version::{TurboCdnVersionFetcher, VersionFetcher};

/// Macro to generate UV tool implementations using VxTool trait
macro_rules! uv_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            version_fetcher: Option<TurboCdnVersionFetcher>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    version_fetcher: None,
                }
            }

            /// Initialize the tool with turbo-cdn support
            pub async fn init() -> Result<Self> {
                let version_fetcher = TurboCdnVersionFetcher::new("astral-sh", "uv").await?;
                Ok(Self {
                    version_fetcher: Some(version_fetcher),
                })
            }

            /// Get or initialize the version fetcher
            async fn get_version_fetcher(&self) -> Result<TurboCdnVersionFetcher> {
                match &self.version_fetcher {
                    Some(fetcher) => Ok(fetcher.clone()),
                    None => {
                        // Create a new fetcher if not initialized
                        TurboCdnVersionFetcher::new("astral-sh", "uv")
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to create TurboCdnVersionFetcher: {}", e))
                    }
                }
            }
        }

        #[async_trait::async_trait]
        impl VxTool for $name {
            fn name(&self) -> &str {
                $cmd
            }

            fn description(&self) -> &str {
                $desc
            }

            fn aliases(&self) -> Vec<&str> {
                vec![]
            }

            async fn fetch_versions(
                &self,
                include_prerelease: bool,
            ) -> Result<Vec<VersionInfo>, anyhow::Error> {
                // Use TurboCdn for version fetching
                let fetcher = self.get_version_fetcher().await?;
                fetcher
                    .fetch_versions(include_prerelease)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to fetch versions: {}", e))
            }

            async fn install_version(
                &self,
                version: &str,
                force: bool,
            ) -> Result<(), anyhow::Error> {
                // Resolve "latest" to actual version first
                let actual_version = if version == "latest" {
                    let versions = self.fetch_versions(false).await?;
                    if let Some(latest_version) = versions.first() {
                        latest_version.version.clone()
                    } else {
                        return Err(anyhow::anyhow!("No versions found for {}", self.name()));
                    }
                } else {
                    version.to_string()
                };

                // Check if already installed
                let is_installed = self.is_version_installed(&actual_version).await?;

                if is_installed && !force {
                    return Err(anyhow::anyhow!(
                        "Tool {} v{} is already installed. Use --force to reinstall",
                        self.name(),
                        actual_version
                    ));
                }

                // If force=true and already installed, we'll proceed with reinstallation

                let install_dir = self.get_version_install_dir(&actual_version);

                // Use real installation with vx-installer (with resolved version)
                let config = crate::config::create_install_config(&actual_version, install_dir, force);
                let installer = vx_installer::Installer::new().await?;

                let _exe_path = installer
                    .install(&config)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to install UV {}: {}", actual_version, e))?;

                // Verify installation
                if !self.is_version_installed(&actual_version).await? {
                    return Err(anyhow::anyhow!(
                        "Installation verification failed for {} version {}",
                        self.name(),
                        actual_version
                    ));
                }

                Ok(())
            }

            async fn is_version_installed(&self, version: &str) -> Result<bool, anyhow::Error> {
                // Check if version directory exists and contains executable
                let install_dir = self.get_version_install_dir(version);
                if !install_dir.exists() {
                    return Ok(false);
                }

                // Check if executable exists in the install directory
                match self.get_executable_path(&install_dir).await {
                    Ok(exe_path) => Ok(exe_path.exists()),
                    Err(_) => Ok(false),
                }
            }

            async fn get_active_version(&self) -> Result<String, anyhow::Error> {
                // Simple implementation - return a default version
                Ok("latest".to_string())
            }

            async fn get_installed_versions(&self) -> Result<Vec<String>, anyhow::Error> {
                // Use PathManager to scan for installed versions
                let path_manager = vx_paths::PathManager::new().unwrap_or_default();
                let mut versions = path_manager.list_tool_versions(self.name())?;

                // Sort versions (newest first)
                versions.sort_by(|a, b| b.cmp(a));
                Ok(versions)
            }

            async fn execute(
                &self,
                args: &[String],
                context: &ToolContext,
            ) -> Result<ToolExecutionResult, anyhow::Error> {
                let tool_name = if self.name() == "uvx" {
                    "uv"
                } else {
                    self.name()
                };

                // First, try to use vx-managed version
                let path_manager = vx_paths::PathManager::new()
                    .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;

                // Check if we have an installed version
                let installed_versions = self.get_installed_versions().await?;
                let executable_path = if !installed_versions.is_empty() {
                    // Use the latest installed version
                    let latest_version = &installed_versions[0];
                    path_manager.tool_executable_path(tool_name, latest_version)
                } else {
                    // No vx-managed version, check if we should use system PATH
                    if context.use_system_path {
                        // Check if tool is available in system PATH
                        if which::which(tool_name).is_err() {
                            // Try to install tool if not found
                            eprintln!("{} not found, attempting to install...", tool_name);
                            if let Err(e) = self.install_version("latest", false).await {
                                return Err(anyhow::anyhow!("Failed to install {}: {}", tool_name, e));
                            }
                            eprintln!("{} installed successfully", tool_name);
                        }
                        // Use system tool
                        std::path::PathBuf::from(tool_name)
                    } else {
                        return Err(anyhow::anyhow!(
                            "Tool '{}' not found in vx-managed installations and system PATH is disabled",
                            tool_name
                        ));
                    }
                };

                let mut cmd = std::process::Command::new(&executable_path);

                // For uvx, add "tool run" prefix
                if self.name() == "uvx" {
                    cmd.arg("tool");
                    cmd.arg("run");
                }

                cmd.args(args);

                if let Some(cwd) = &context.working_directory {
                    cmd.current_dir(cwd);
                }

                for (key, value) in &context.environment_variables {
                    cmd.env(key, value);
                }

                let status = cmd
                    .status()
                    .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", self.name(), e))?;

                Ok(ToolExecutionResult {
                    exit_code: status.code().unwrap_or(1),
                    stdout: None,
                    stderr: None,
                })
            }

            async fn get_download_url(
                &self,
                version: &str,
            ) -> Result<Option<String>, anyhow::Error> {
                // Handle "latest" version by resolving to actual version first
                let actual_version = if version == "latest" {
                    let versions = self.fetch_versions(false).await?;
                    if let Some(latest_version) = versions.first() {
                        latest_version.version.clone()
                    } else {
                        return Ok(None);
                    }
                } else {
                    version.to_string()
                };

                // Use global config system for sync access
                use vx_config::{get_tool_download_url_sync, get_platform_string};

                // Try to get URL from config first
                if let Some(url) = get_tool_download_url_sync("uv", &actual_version) {
                    return Ok(Some(url));
                }

                // Fallback to hardcoded logic with actual version
                let platform = get_platform_string();
                let filename = if cfg!(windows) {
                    format!("uv-{}.zip", platform)
                } else {
                    format!("uv-{}.tar.gz", platform)
                };

                let url = format!(
                    "https://github.com/astral-sh/uv/releases/download/{}/{}",
                    actual_version, filename
                );

                Ok(Some(url))
            }

            fn metadata(&self) -> HashMap<String, String> {
                // Use global config system for sync access
                use vx_config::get_tool_metadata_sync;

                // Get metadata from global config
                let config_metadata = get_tool_metadata_sync(self.name());
                if !config_metadata.is_empty() {
                    return config_metadata;
                }

                // Fallback to hardcoded metadata if config system fails
                let mut meta = HashMap::new();
                meta.insert("homepage".to_string(), $homepage.unwrap_or("").to_string());
                meta.insert("ecosystem".to_string(), "python".to_string());
                meta.insert(
                    "repository".to_string(),
                    "https://github.com/astral-sh/uv".to_string(),
                );
                meta.insert("license".to_string(), "MIT OR Apache-2.0".to_string());
                meta
            }

            fn get_dependencies(&self) -> Vec<vx_plugin::ToolDependency> {
                // Default implementation for UV tools
                if self.name() == "uvx" {
                    vec![
                        vx_plugin::ToolDependency::required("uv", "uvx is a subcommand of uv")
                    ]
                } else {
                    Vec::new()
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

// Define UV tools using the VxTool macro
uv_vx_tool!(
    UvCommand,
    "uv",
    "An extremely fast Python package installer and resolver",
    Some("https://docs.astral.sh/uv/")
);
uv_vx_tool!(
    UvxTool,
    "uvx",
    "Python application runner",
    Some("https://docs.astral.sh/uv/")
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_tool_creation() {
        let tool = UvCommand::new();
        assert_eq!(tool.name(), "uv");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_uvx_tool_creation() {
        let tool = UvxTool::new();
        assert_eq!(tool.name(), "uvx");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_uv_tool_metadata() {
        let tool = UvCommand::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"python".to_string()));
    }
}
