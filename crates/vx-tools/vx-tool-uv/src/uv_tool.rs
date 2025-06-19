//! UV tool implementations - Python package management tools

use std::collections::HashMap;
use vx_core::{UvUrlBuilder, VxEnvironment, VxTool};
use vx_plugin::types::VersionInfo;
use vx_version::{GitHubVersionFetcher, VersionFetcher};

/// Macro to generate UV tool implementations using VxTool trait
macro_rules! uv_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            _url_builder: UvUrlBuilder,
            _version_fetcher: GitHubVersionFetcher,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    _url_builder: UvUrlBuilder::new(),
                    _version_fetcher: GitHubVersionFetcher::new("astral-sh", "uv"),
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
                // Since vx-version now uses vx-plugin::VersionInfo, no conversion needed
                self._version_fetcher
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
                let env = VxEnvironment::new().expect("Failed to create VX environment");

                // For uvx, check if uv is installed (they are the same binary)
                if self.name() == "uvx" {
                    return Ok(env.is_version_installed("uv", version));
                }

                Ok(env.is_version_installed(self.name(), version))
            }

            async fn get_active_version(&self) -> Result<String, anyhow::Error> {
                let env = VxEnvironment::new().expect("Failed to create VX environment");

                // For uvx, use uv version
                if self.name() == "uvx" {
                    if let Some(active_version) = env.get_active_version("uv")? {
                        return Ok(active_version);
                    }

                    let installed_versions = env
                        .list_installed_versions("uv")
                        .map_err(|e| anyhow::anyhow!("Failed to list versions: {}", e))?;
                    return installed_versions
                        .first()
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Tool uv not installed"));
                }

                // For uv, use default implementation
                if let Some(active_version) = env.get_active_version(self.name())? {
                    return Ok(active_version);
                }

                let installed_versions = env
                    .list_installed_versions(self.name())
                    .map_err(|e| anyhow::anyhow!("Failed to list versions: {}", e))?;
                installed_versions
                    .first()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Tool {} not installed", self.name()))
            }

            async fn get_installed_versions(&self) -> Result<Vec<String>, anyhow::Error> {
                let env = VxEnvironment::new().expect("Failed to create VX environment");

                // For uvx, use uv versions
                if self.name() == "uvx" {
                    return env
                        .list_installed_versions("uv")
                        .map_err(|e| anyhow::anyhow!("Failed to list versions: {}", e));
                }

                env.list_installed_versions(self.name())
                    .map_err(|e| anyhow::anyhow!("Failed to list versions: {}", e))
            }

            async fn execute(
                &self,
                args: &[String],
                context: &vx_plugin::types::ToolContext,
            ) -> Result<vx_plugin::types::ToolExecutionResult, anyhow::Error> {
                // For uvx, find executable in uv installation
                if self.name() == "uvx" && !context.use_system_path {
                    let active_version = self.get_active_version().await?;
                    let env = VxEnvironment::new().expect("Failed to create VX environment");
                    let uv_install_dir = env.get_version_install_dir("uv", &active_version);

                    // uvx is actually the uv binary with tool run behavior
                    let exe_path = env
                        .find_executable_in_dir(&uv_install_dir, "uv")
                        .map_err(|e| anyhow::anyhow!("Failed to find executable: {}", e))?;

                    // Execute with tool run as the first arguments
                    let mut cmd = std::process::Command::new(&exe_path);
                    cmd.arg("tool");
                    cmd.arg("run");
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

                    return Ok(vx_plugin::types::ToolExecutionResult {
                        exit_code: status.code().unwrap_or(1),
                        stdout: None,
                        stderr: None,
                    });
                }

                // For uv or system path execution, use default workflow
                // TODO: Implement proper execute workflow
                Ok(vx_plugin::types::ToolExecutionResult {
                    exit_code: 0,
                    stdout: None,
                    stderr: None,
                })
            }

            async fn get_download_url(
                &self,
                version: &str,
            ) -> Result<Option<String>, anyhow::Error> {
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
