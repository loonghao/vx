//! Fallback installation methods
//!
//! This module provides fallback installation methods for runtimes that aren't
//! available via provider registry. These are typically system-level installers
//! like apt, brew, or official installation scripts.

use crate::Result;
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

use super::installation::InstallationManager;
use super::pipeline::error::EnsureError;

impl<'a> InstallationManager<'a> {
    /// Fallback installation using known methods (scripts, package managers)
    pub async fn install_runtime_fallback(&self, runtime_name: &str) -> Result<()> {
        match runtime_name {
            // Node.js (via nvm)
            "node" | "nodejs" => {
                if !self.check_command_exists("nvm").await {
                    // Install nvm first
                    #[cfg(not(windows))]
                    {
                        self.run_install_command(
                            "bash",
                            &[
                                "-c",
                                "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh | bash",
                            ],
                        )
                        .await?;
                    }
                    #[cfg(windows)]
                    {
                        return Err(EnsureError::NotInstalled {
                            runtime: "Node.js".to_string(),
                            hint: "Please install it from https://nodejs.org/".to_string(),
                        }
                        .into());
                    }
                }
                self.run_install_command("bash", &["-c", "nvm install --lts"])
                    .await?;
            }

            // Python (via UV - preferred installer)
            "python" | "python3" => {
                // Check if UV is available first
                if self.check_command_exists("uv").await {
                    // UV can manage Python installations
                    info!("Installing Python via UV...");
                    self.run_install_command("uv", &["python", "install"])
                        .await?;
                } else {
                    return Err(EnsureError::NotInstalled {
                        runtime: "Python".to_string(),
                        hint: "Please install UV first ('vx install uv') or install Python from https://www.python.org/".to_string(),
                    }.into());
                }
            }

            // UV (Python package manager)
            "uv" => {
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

            // Rust/Cargo (via rustup)
            "rust" | "cargo" | "rustc" => {
                if !self.check_command_exists("rustup").await {
                    #[cfg(windows)]
                    {
                        return Err(EnsureError::NotInstalled {
                            runtime: "Rust".to_string(),
                            hint: "Please install rustup from https://rustup.rs/".to_string(),
                        }
                        .into());
                    }
                    #[cfg(not(windows))]
                    {
                        self.run_install_command(
                            "sh",
                            &["-c", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"],
                        )
                        .await?;
                    }
                }
                self.run_install_command("rustup", &["default", "stable"])
                    .await?;
            }

            // Go
            "go" | "golang" => {
                return Err(EnsureError::NotInstalled {
                    runtime: "Go".to_string(),
                    hint: "Please install it from https://go.dev/dl/ or run 'vx install go'"
                        .to_string(),
                }
                .into());
            }

            // pnpm
            "pnpm" => {
                // Try corepack first if node is available
                if self.check_command_exists("corepack").await {
                    self.run_install_command("corepack", &["enable", "pnpm"])
                        .await?;
                } else {
                    // Fallback to npm install
                    self.run_install_command("npm", &["install", "-g", "pnpm"])
                        .await?;
                }
            }

            // Yarn
            "yarn" => {
                // Try corepack first if node is available
                if self.check_command_exists("corepack").await {
                    self.run_install_command("corepack", &["enable", "yarn"])
                        .await?;
                } else {
                    // Fallback to npm install
                    self.run_install_command("npm", &["install", "-g", "yarn"])
                        .await?;
                }
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

            // .NET SDK
            "dotnet" => {
                return Err(EnsureError::NotInstalled {
                    runtime: ".NET SDK".to_string(),
                    hint: "Please install it from https://dot.net/ or run 'vx install dotnet'"
                        .to_string(),
                }
                .into());
            }

            // MSBuild (bundled with .NET SDK) - RFC 0028
            // MSBuild is bundled with .NET SDK - need to install dotnet first
            "msbuild" => {
                // Try to trigger dotnet installation through the normal provider mechanism
                // rather than using fallback (which would cause recursion)
                return Err(EnsureError::NotInstalled {
                    runtime: "MSBuild".to_string(),
                    hint: "Requires .NET SDK. Please install it first:\n\n  vx install dotnet\n\n  On Windows, you can also install Visual Studio with C++ build tools.".to_string(),
                }.into());
            }

            _ => {
                // Check if the runtime is in the registry but needs special handling
                if let Some(registry) = self.registry {
                    if let Some(runtime) = registry.get_runtime(runtime_name) {
                        return Err(EnsureError::NotInstalled {
                            runtime: runtime_name.to_string(),
                            hint: format!(
                                "Cannot auto-install '{}' ({}). Please install it manually.",
                                runtime_name,
                                runtime.description()
                            ),
                        }
                        .into());
                    } else {
                        return Err(EnsureError::NotInstalled {
                            runtime: runtime_name.to_string(),
                            hint: "Unknown runtime. Cannot auto-install.".to_string(),
                        }
                        .into());
                    }
                } else {
                    return Err(EnsureError::NotInstalled {
                        runtime: runtime_name.to_string(),
                        hint: "Unknown runtime. Cannot auto-install.".to_string(),
                    }
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Check if a command exists
    pub async fn check_command_exists(&self, cmd: &str) -> bool {
        which::which(cmd).is_ok()
    }

    /// Run an installation command
    pub async fn run_install_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        info!("Running: {} {}", cmd, args.join(" "));

        let status = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            return Err(EnsureError::CommandFailed {
                exit_code: status.code(),
            }
            .into());
        }

        Ok(())
    }

    /// Try to run a command to verify installation
    #[allow(dead_code)]
    pub async fn install_via_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(cmd)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if status.success() {
            Ok(())
        } else {
            Err(EnsureError::CommandFailed {
                exit_code: status.code(),
            }
            .into())
        }
    }
}
