//! Node.js tool implementations - JavaScript runtime and package management tools

use crate::config::NodeUrlBuilder;
use anyhow::Result;
use std::collections::HashMap;
use vx_plugin::{ToolContext, ToolExecutionResult, VersionInfo, VxTool};
use vx_tool_standard::StandardUrlBuilder;
use vx_version::{NodeVersionFetcher, VersionFetcher};
// use vx_core::{UrlBuilder, VersionParser};

/// Macro to generate Node.js tool implementations using VxTool trait
macro_rules! node_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            version_fetcher: NodeVersionFetcher,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    version_fetcher: NodeVersionFetcher::new(),
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
                match $cmd {
                    "node" => vec!["nodejs"],
                    "npm" => vec![],
                    "npx" => vec![],
                    _ => vec![],
                }
            }

            async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
                // For Node.js, fetch from official API
                self.version_fetcher
                    .fetch_versions(include_prerelease)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to fetch versions: {}", e))
            }

            async fn install_version(&self, version: &str, force: bool) -> Result<()> {
                if !force && self.is_version_installed(version).await? {
                    return Err(anyhow::anyhow!(
                        "Version {} of {} is already installed",
                        version,
                        self.name()
                    ));
                }

                let install_dir = self.get_version_install_dir(version);

                // Use real installation with vx-installer
                let mut config = crate::config::create_install_config(version, install_dir);
                config.force = force; // Set the force flag
                let installer = vx_installer::Installer::new().await?;

                let _exe_path = installer
                    .install(&config)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to install Node.js {}: {}", version, e))?;

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

            async fn is_version_installed(&self, version: &str) -> Result<bool> {
                // Simple implementation - check if version directory exists
                let install_dir = self.get_version_install_dir(version);
                Ok(install_dir.exists())
            }

            async fn execute(
                &self,
                args: &[String],
                context: &ToolContext,
            ) -> Result<ToolExecutionResult> {
                // Check if tool is available in system PATH
                if which::which($cmd).is_err() {
                    // Try to install tool if not found
                    eprintln!("{} not found, attempting to install...", $cmd);
                    if let Err(e) = self.install_version("latest", false).await {
                        return Err(anyhow::anyhow!("Failed to install {}: {}", $cmd, e));
                    }
                    eprintln!("{} installed successfully", $cmd);
                }

                let mut cmd = std::process::Command::new($cmd);
                cmd.args(args);

                if let Some(cwd) = &context.working_directory {
                    cmd.current_dir(cwd);
                }

                for (key, value) in &context.environment_variables {
                    cmd.env(key, value);
                }

                let status = cmd
                    .status()
                    .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", $cmd, e))?;

                Ok(ToolExecutionResult {
                    exit_code: status.code().unwrap_or(1),
                    stdout: None,
                    stderr: None,
                })
            }

            async fn get_active_version(&self) -> Result<String> {
                // Simple implementation - return a default version
                Ok("latest".to_string())
            }

            async fn get_installed_versions(&self) -> Result<Vec<String>> {
                // Simple implementation - return empty list
                Ok(vec![])
            }

            async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
                Ok(NodeUrlBuilder::download_url(version))
            }

            fn metadata(&self) -> HashMap<String, String> {
                let mut meta = HashMap::new();
                meta.insert("homepage".to_string(), $homepage.unwrap_or("").to_string());
                meta.insert("ecosystem".to_string(), "javascript".to_string());
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

// Define Node.js tools using the VxTool macro
node_vx_tool!(
    NodeTool,
    "node",
    "Node.js JavaScript runtime",
    Some("https://nodejs.org/")
);
node_vx_tool!(
    NpmTool,
    "npm",
    "Node.js package manager",
    Some("https://www.npmjs.com/")
);
node_vx_tool!(
    NpxTool,
    "npx",
    "Node.js package runner",
    Some("https://www.npmjs.com/package/npx")
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_tool_creation() {
        let tool = NodeTool::new();
        assert_eq!(tool.name(), "node");
        assert!(!tool.description().is_empty());
        assert!(tool.aliases().contains(&"nodejs"));
    }

    #[test]
    fn test_npm_tool_creation() {
        let tool = NpmTool::new();
        assert_eq!(tool.name(), "npm");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_npx_tool_creation() {
        let tool = NpxTool::new();
        assert_eq!(tool.name(), "npx");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_node_tool_metadata() {
        let tool = NodeTool::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"javascript".to_string()));
    }
}
