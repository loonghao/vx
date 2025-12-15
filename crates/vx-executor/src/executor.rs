//! Dynamic executor - the core command forwarding engine
//!
//! This module implements the main execution logic:
//! 1. Resolve tool and dependencies
//! 2. Auto-install missing components
//! 3. Forward command to the appropriate executable

use crate::{ExecutorConfig, Result, ToolResolver};
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_plugin::BundleRegistry;

/// Dynamic executor for tool command forwarding
pub struct DynamicExecutor {
    /// Configuration
    config: ExecutorConfig,

    /// Tool resolver
    resolver: ToolResolver,

    /// Optional bundle registry for installation
    registry: Option<BundleRegistry>,
}

impl DynamicExecutor {
    /// Create a new dynamic executor
    pub fn new(config: ExecutorConfig) -> Result<Self> {
        let resolver = ToolResolver::new(config.clone())?;
        Ok(Self {
            config,
            resolver,
            registry: None,
        })
    }

    /// Create an executor with a bundle registry for auto-installation
    pub fn with_registry(config: ExecutorConfig, registry: BundleRegistry) -> Result<Self> {
        let resolver = ToolResolver::new(config.clone())?;
        Ok(Self {
            config,
            resolver,
            registry: Some(registry),
        })
    }

    /// Execute a tool with the given arguments
    ///
    /// This is the main entry point for command forwarding.
    /// Format: vx <tool> <args...>
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use vx_executor::{DynamicExecutor, ExecutorConfig};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let executor = DynamicExecutor::new(ExecutorConfig::default())?;
    ///
    ///     // Execute: npm install express
    ///     let exit_code = executor.execute("npm", &["install".into(), "express".into()]).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute(&self, tool_name: &str, args: &[String]) -> Result<i32> {
        debug!("Executing: {} {}", tool_name, args.join(" "));

        // Resolve the tool
        let resolution = self.resolver.resolve(tool_name)?;

        // Check if we need to install anything
        if !resolution.install_order.is_empty() {
            if self.config.auto_install {
                info!(
                    "Auto-installing missing tools: {:?}",
                    resolution.install_order
                );
                self.install_tools(&resolution.install_order).await?;
            } else {
                // Report missing dependencies
                let missing = if resolution.tool_needs_install {
                    format!(
                        "Tool '{}' is not installed. Missing dependencies: {:?}",
                        tool_name, resolution.missing_dependencies
                    )
                } else {
                    format!(
                        "Missing dependencies for '{}': {:?}",
                        tool_name, resolution.missing_dependencies
                    )
                };

                return Err(anyhow::anyhow!(
                    "{}. Run 'vx install {}' or enable auto-install.",
                    missing,
                    tool_name
                ));
            }
        }

        // Build the command
        let mut cmd = self.build_command(&resolution, args)?;

        // Execute
        let status = self.run_command(&mut cmd).await?;

        Ok(status.code().unwrap_or(1))
    }

    /// Install a list of tools in order
    async fn install_tools(&self, tools: &[String]) -> Result<()> {
        for tool in tools {
            self.install_tool(tool).await?;
        }
        Ok(())
    }

    /// Install a single tool
    async fn install_tool(&self, tool_name: &str) -> Result<()> {
        info!("Installing: {}", tool_name);

        // Try using the bundle registry first
        if let Some(registry) = &self.registry {
            if let Some(tool) = registry.get_tool(tool_name) {
                // Install latest version
                let versions = tool.fetch_versions(false).await?;
                if let Some(latest) = versions.first() {
                    info!("Installing {} version {}", tool_name, latest.version);
                    tool.install_version(&latest.version, false).await?;
                    return Ok(());
                }
            }
        }

        // Fallback: try to install using known methods
        self.install_tool_fallback(tool_name).await
    }

    /// Fallback installation methods for tools
    async fn install_tool_fallback(&self, tool_name: &str) -> Result<()> {
        let spec = self.resolver.get_spec(tool_name);

        match tool_name {
            // Node.js ecosystem
            "node" | "nodejs" => {
                self.install_via_command("node", &["--version"])
                    .await
                    .map_err(|_| {
                        anyhow::anyhow!(
                        "Node.js is not installed. Please install it from https://nodejs.org/ or run 'vx install node'"
                    )
                    })?;
            }

            "npm" | "npx" => {
                // npm/npx come with Node.js
                warn!("{} should be installed with Node.js", tool_name);
                return Err(anyhow::anyhow!(
                    "{} is bundled with Node.js. Please install Node.js first.",
                    tool_name
                ));
            }

            "yarn" => {
                // Install yarn via npm
                self.run_install_command("npm", &["install", "-g", "yarn"])
                    .await?;
            }

            "pnpm" => {
                // Install pnpm via npm
                self.run_install_command("npm", &["install", "-g", "pnpm"])
                    .await?;
            }

            // Python ecosystem
            "uv" => {
                // Install uv via pip or standalone installer
                if self.check_command_exists("pip").await {
                    self.run_install_command("pip", &["install", "uv"]).await?;
                } else if self.check_command_exists("pip3").await {
                    self.run_install_command("pip3", &["install", "uv"]).await?;
                } else {
                    // Use standalone installer
                    #[cfg(windows)]
                    {
                        self.run_install_command(
                            "powershell",
                            &[
                                "-ExecutionPolicy",
                                "ByPass",
                                "-c",
                                "irm https://astral.sh/uv/install.ps1 | iex",
                            ],
                        )
                        .await?;
                    }
                    #[cfg(not(windows))]
                    {
                        self.run_install_command(
                            "sh",
                            &["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"],
                        )
                        .await?;
                    }
                }
            }

            "uvx" => {
                // uvx is part of uv
                return Err(anyhow::anyhow!(
                    "uvx is part of uv. Please install uv first."
                ));
            }

            // Rust ecosystem
            "rustup" => {
                #[cfg(windows)]
                {
                    return Err(anyhow::anyhow!(
                        "Please install Rust from https://rustup.rs/"
                    ));
                }
                #[cfg(not(windows))]
                {
                    self.run_install_command(
                        "sh",
                        &[
                            "-c",
                            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
                        ],
                    )
                    .await?;
                }
            }

            "cargo" | "rustc" => {
                return Err(anyhow::anyhow!(
                    "{} is installed via rustup. Please install Rust first.",
                    tool_name
                ));
            }

            // Go
            "go" | "golang" => {
                return Err(anyhow::anyhow!(
                    "Please install Go from https://go.dev/dl/ or run 'vx install go'"
                ));
            }

            // Bun
            "bun" => {
                #[cfg(windows)]
                {
                    self.run_install_command(
                        "powershell",
                        &[
                            "-ExecutionPolicy",
                            "ByPass",
                            "-c",
                            "irm bun.sh/install.ps1 | iex",
                        ],
                    )
                    .await?;
                }
                #[cfg(not(windows))]
                {
                    self.run_install_command(
                        "sh",
                        &["-c", "curl -fsSL https://bun.sh/install | bash"],
                    )
                    .await?;
                }
            }

            _ => {
                // Unknown tool
                if let Some(spec) = spec {
                    return Err(anyhow::anyhow!(
                        "Cannot auto-install '{}' ({}). Please install it manually.",
                        tool_name,
                        spec.description
                    ));
                } else {
                    return Err(anyhow::anyhow!(
                        "Unknown tool '{}'. Cannot auto-install.",
                        tool_name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if a command exists
    async fn check_command_exists(&self, cmd: &str) -> bool {
        which::which(cmd).is_ok()
    }

    /// Run an installation command
    async fn run_install_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        info!("Running: {} {}", cmd, args.join(" "));

        let status = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Installation command failed with exit code: {:?}",
                status.code()
            ));
        }

        Ok(())
    }

    /// Try to run a command to verify installation
    async fn install_via_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(cmd)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Command failed"))
        }
    }

    /// Build the command to execute
    fn build_command(
        &self,
        resolution: &crate::resolver::ResolutionResult,
        args: &[String],
    ) -> Result<Command> {
        let mut cmd = Command::new(&resolution.executable);

        // Add command prefix if any (e.g., "tool run" for uvx)
        for prefix in &resolution.command_prefix {
            cmd.arg(prefix);
        }

        // Add user arguments
        cmd.args(args);

        // Inherit stdio
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        Ok(cmd)
    }

    /// Run the command and return its status
    async fn run_command(&self, cmd: &mut Command) -> Result<ExitStatus> {
        let status = if let Some(timeout) = self.config.execution_timeout {
            tokio::time::timeout(timeout, cmd.status())
                .await
                .map_err(|_| anyhow::anyhow!("Command execution timed out"))??
        } else {
            cmd.status().await?
        };

        Ok(status)
    }

    /// Get the resolver (for inspection)
    pub fn resolver(&self) -> &ToolResolver {
        &self.resolver
    }

    /// Get the configuration
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }
}

/// Execute a tool directly using system PATH (simple fallback)
pub async fn execute_system_tool(tool_name: &str, args: &[String]) -> Result<i32> {
    debug!("Executing system tool: {} {}", tool_name, args.join(" "));

    let status = Command::new(tool_name)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute '{}': {}", tool_name, e))?;

    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let config = ExecutorConfig::default();
        let executor = DynamicExecutor::new(config);
        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_executor_with_disabled_auto_install() {
        let config = ExecutorConfig::default().without_auto_install();
        let executor = DynamicExecutor::new(config).unwrap();
        assert!(!executor.config().auto_install);
    }
}
