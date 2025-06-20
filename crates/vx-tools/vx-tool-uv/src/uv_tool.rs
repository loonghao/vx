//! UV tool implementations - Python package management tools

use crate::config::UvUrlBuilder;
use anyhow::Result;
use std::collections::HashMap;
use vx_plugin::{ToolContext, ToolExecutionResult, VersionInfo, VxTool};
use vx_version::{GitHubVersionFetcher, VersionFetcher};

/// Macro to generate UV tool implementations using VxTool trait
macro_rules! uv_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            version_fetcher: GitHubVersionFetcher,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    version_fetcher: GitHubVersionFetcher::new("astral-sh", "uv"),
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
                // For UV, fetch from GitHub releases
                self.version_fetcher
                    .fetch_versions(include_prerelease)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to fetch versions: {}", e))
            }

            async fn install_version(
                &self,
                version: &str,
                force: bool,
            ) -> Result<(), anyhow::Error> {
                if !force && self.is_version_installed(version).await? {
                    return Err(anyhow::anyhow!(
                        "Version {} already installed for {}",
                        version,
                        self.name()
                    ));
                }

                let install_dir = self.get_version_install_dir(version);

                // Use real installation with vx-installer
                let config = crate::config::create_install_config(version, install_dir);
                let installer = vx_installer::Installer::new().await?;

                let _exe_path = installer
                    .install(&config)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to install UV {}: {}", version, e))?;

                // Verify installation
                if !self.is_version_installed(version).await? {
                    return Err(anyhow::anyhow!(
                        "Installation verification failed for {} version {}",
                        self.name(),
                        version
                    ));
                }

                Ok(())
            }

            async fn is_version_installed(&self, version: &str) -> Result<bool, anyhow::Error> {
                // Simple implementation - check if version directory exists
                let install_dir = self.get_version_install_dir(version);
                Ok(install_dir.exists())
            }

            async fn get_active_version(&self) -> Result<String, anyhow::Error> {
                // Simple implementation - return a default version
                Ok("latest".to_string())
            }

            async fn get_installed_versions(&self) -> Result<Vec<String>, anyhow::Error> {
                // Simple implementation - return empty list
                Ok(vec![])
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
                use vx_tool_standard::StandardUrlBuilder;
                if version == "latest" {
                    // For latest, get the actual latest version first
                    let versions = self.fetch_versions(false).await?;
                    if let Some(latest_version) = versions.first() {
                        return Ok(UvUrlBuilder::download_url(&latest_version.version));
                    }
                    return Ok(None);
                }
                Ok(UvUrlBuilder::download_url(version))
            }

            fn metadata(&self) -> HashMap<String, String> {
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
