//! Bunx tool implementation for vx
//! 
//! Bunx is a package runner that comes with Bun. It allows you to run packages
//! without installing them globally. This tool depends on Bun being installed.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_plugin::{ToolContext, ToolExecutionResult, VxTool};
use vx_version::{TurboCdnVersionFetcher, VersionFetcher, VersionInfo};

/// Bunx tool implementation
#[derive(Debug, Clone)]
pub struct BunxTool {
    bun_tool: vx_tool_bun::BunTool,
}

impl BunxTool {
    pub fn new() -> Self {
        Self {
            bun_tool: vx_tool_bun::BunTool::new(),
        }
    }

    /// Initialize the tool with bun dependency
    pub async fn init() -> Result<Self> {
        let bun_tool = vx_tool_bun::BunTool::init().await?;
        Ok(Self { bun_tool })
    }

    /// Check if bun is installed and available
    async fn ensure_bun_available(&self, context: &ToolContext) -> Result<()> {
        // Check if bun is installed in vx
        let bun_path = context.get_tool_path("bun");
        if !bun_path.exists() {
            return Err(anyhow::anyhow!(
                "Bun is required for bunx but is not installed. Please run 'vx install bun' first."
            ));
        }
        Ok(())
    }
}

impl Default for BunxTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VxTool for BunxTool {
    fn name(&self) -> &str {
        "bunx"
    }

    fn description(&self) -> &str {
        "Package runner for Bun - run packages without installing them globally"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn is_installed(&self, context: &ToolContext) -> Result<bool> {
        // bunx is available if bun is installed
        let bun_path = context.get_tool_path("bun");
        Ok(bun_path.exists())
    }

    async fn get_installed_version(&self, context: &ToolContext) -> Result<Option<String>> {
        // bunx version is the same as bun version
        self.bun_tool.get_installed_version(context).await
    }

    async fn install(&self, version: &str, install_dir: &Path, context: &ToolContext) -> Result<()> {
        // bunx doesn't need separate installation - it comes with bun
        // Just ensure bun is installed
        if !self.is_installed(context).await? {
            // Install bun first
            self.bun_tool.install(version, install_dir, context).await?;
        }
        Ok(())
    }

    async fn uninstall(&self, context: &ToolContext) -> Result<()> {
        // bunx is uninstalled when bun is uninstalled
        self.bun_tool.uninstall(context).await
    }

    async fn execute(
        &self,
        args: Vec<String>,
        context: &ToolContext,
    ) -> Result<ToolExecutionResult> {
        // Ensure bun is available
        self.ensure_bun_available(context).await?;

        // Get bun executable path
        let bun_path = context.get_tool_path("bun");
        let bun_exe = if cfg!(target_os = "windows") {
            bun_path.join("bun.exe")
        } else {
            bun_path.join("bun")
        };

        // Execute bunx command using bun
        let mut cmd_args = vec!["x".to_string()]; // 'bun x' is equivalent to 'bunx'
        cmd_args.extend(args);

        let mut cmd = std::process::Command::new(&bun_exe);
        cmd.args(&cmd_args);

        // Set environment variables
        if let Some(env_vars) = &context.env_vars {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // Set working directory
        if let Some(cwd) = &context.working_dir {
            cmd.current_dir(cwd);
        }

        let output = cmd.output()?;

        Ok(ToolExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // bunx versions are the same as bun versions
        self.bun_tool.fetch_versions(include_prerelease).await
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        // bunx doesn't have separate downloads - it comes with bun
        self.bun_tool.get_download_url(version).await
    }

    async fn get_dependencies(&self) -> Result<Vec<String>> {
        // bunx depends on bun
        Ok(vec!["bun".to_string()])
    }

    async fn get_env_vars(&self, context: &ToolContext) -> Result<HashMap<String, String>> {
        // Use the same environment variables as bun
        self.bun_tool.get_env_vars(context).await
    }

    async fn get_aliases(&self) -> Result<Vec<String>> {
        // Common aliases for bunx
        Ok(vec!["bx".to_string()])
    }

    async fn validate_installation(&self, context: &ToolContext) -> Result<bool> {
        // Validate that bun is properly installed and bunx works
        if !self.is_installed(context).await? {
            return Ok(false);
        }

        // Try to run 'bun x --version' to validate bunx functionality
        let bun_path = context.get_tool_path("bun");
        let bun_exe = if cfg!(target_os = "windows") {
            bun_path.join("bun.exe")
        } else {
            bun_path.join("bun")
        };

        let output = std::process::Command::new(&bun_exe)
            .args(&["x", "--version"])
            .output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }
}

/// Create a new bunx tool plugin
pub fn create_plugin() -> Box<dyn VxTool> {
    Box::new(BunxTool::new())
}
