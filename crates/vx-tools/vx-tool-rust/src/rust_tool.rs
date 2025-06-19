//! Rust toolchain implementations with environment isolation

use crate::config::RustUrlBuilder;
use anyhow::Result;
use std::collections::HashMap;
use vx_plugin::{ToolContext, ToolExecutionResult, VersionInfo, VxTool};
use vx_tool_standard::StandardUrlBuilder;
use vx_version::{GitHubVersionFetcher, VersionFetcher};

/// Macro to generate Rust tool implementations using VxTool trait
macro_rules! rust_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            _url_builder: RustUrlBuilder,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    _url_builder: RustUrlBuilder::new(),
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

            async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
                // For Rust tools, use GitHubVersionFetcher
                let fetcher = GitHubVersionFetcher::new("rust-lang", "rust");
                fetcher
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

            async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
                Ok(RustUrlBuilder::download_url(version))
            }

            fn metadata(&self) -> HashMap<String, String> {
                let mut meta = HashMap::new();
                meta.insert(
                    "homepage".to_string(),
                    "https://www.rust-lang.org/".to_string(),
                );
                meta.insert("ecosystem".to_string(), "rust".to_string());
                meta.insert(
                    "repository".to_string(),
                    "https://github.com/rust-lang/rust".to_string(),
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

// Define Rust tools using the VxTool macro
rust_vx_tool!(CargoTool, "cargo", "Rust package manager and build tool");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_tool_creation() {
        let tool = CargoTool::new();
        assert_eq!(tool.name(), "cargo");
        assert!(!tool.description().is_empty());
        assert!(tool.aliases().is_empty());
    }

    #[test]
    fn test_rust_tool_metadata() {
        let tool = CargoTool::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"rust".to_string()));
    }
}
