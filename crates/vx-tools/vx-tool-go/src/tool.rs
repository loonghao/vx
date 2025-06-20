//! Go tool implementation

use crate::config::GoUrlBuilder;
use anyhow::Result;
use std::collections::HashMap;
use vx_plugin::{ToolContext, ToolExecutionResult, VersionInfo, VxTool};
use vx_tool_standard::StandardUrlBuilder;
use vx_version::{GitHubVersionFetcher, VersionFetcher};

/// Go tool implementation
#[derive(Debug, Clone)]
pub struct GoTool {
    version_fetcher: GitHubVersionFetcher,
}

impl GoTool {
    pub fn new() -> Self {
        Self {
            version_fetcher: GitHubVersionFetcher::new("golang", "go"),
        }
    }
}

impl Default for GoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl VxTool for GoTool {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["golang"]
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // For Go, fetch from GitHub releases
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

    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        // Determine which go executable to use
        let go_executable = if context.use_system_path {
            "go".to_string() // Use system go
        } else {
            // Ensure go is installed in vx environment
            let active_version = match self.get_active_version().await {
                Ok(version) => version,
                Err(_) => {
                    // No version installed, try to install latest
                    match self.install_version("latest", false).await {
                        Ok(_) => "latest".to_string(),
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to install go: {}", e));
                        }
                    }
                }
            };

            // Get the path to the vx-managed go executable
            let install_dir = self.get_version_install_dir(&active_version);
            match self.get_executable_path(&install_dir).await {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => {
                    // Executable not found, try to install
                    match self.install_version("latest", false).await {
                        Ok(_) => {
                            let latest_dir = self.get_version_install_dir("latest");
                            match self.get_executable_path(&latest_dir).await {
                                Ok(path) => path.to_string_lossy().to_string(),
                                Err(e) => {
                                    return Err(anyhow::anyhow!(
                                        "Failed to find go executable after installation: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to install go: {}", e));
                        }
                    }
                }
            }
        };

        let mut cmd = std::process::Command::new(&go_executable);
        cmd.args(args);

        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute go: {}", e))?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(1),
            stdout: None,
            stderr: None,
        })
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        Ok(GoUrlBuilder::download_url(version))
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://golang.org/".to_string());
        meta.insert("ecosystem".to_string(), "go".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/golang/go".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_tool_basic() {
        let tool = GoTool::default();

        assert_eq!(tool.name(), "go");
        assert_eq!(tool.description(), "Go programming language");
        assert!(tool.aliases().contains(&"golang"));
    }

    #[test]
    fn test_go_tool_metadata() {
        let tool = GoTool::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"go".to_string()));
    }
}
