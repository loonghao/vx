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
                let _exe_path = self.default_install_workflow(version, &install_dir).await?;

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
                // Simple implementation - execute the tool directly
                let tool_name = if self.name() == "uvx" {
                    "uv"
                } else {
                    self.name()
                };
                let mut cmd = std::process::Command::new(tool_name);

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
