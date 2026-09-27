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

/// A step vx may take to make Rust available when the provider install failed.
///
/// Kept as data rather than inline shell so the behaviour is unit-testable without
/// rustup, network access, or a mutable global toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustFallbackStep {
    /// Bootstrap rustup itself. This is the only step that sets a global default,
    /// because a fresh rustup install has no default at all — and it only happens
    /// when the user asked for Rust on a machine that has none.
    InstallRustup { toolchain: Option<String> },
    /// Install a toolchain without making it the default.
    InstallToolchain { toolchain: String },
}

impl RustFallbackStep {
    /// Whether this step rewrites the user's global rustup default.
    ///
    /// Everything except a from-scratch rustup bootstrap must return `false`:
    /// silently repointing the default is what made pinned toolchains get ignored.
    pub fn changes_rustup_default(&self) -> bool {
        matches!(self, Self::InstallRustup { .. })
    }
}

/// Decide how to make Rust available, given whether rustup is already installed.
///
/// With rustup present, vx installs the requested toolchain **without** touching the
/// default — the previous `rustup default stable` made every project in the image run
/// on `stable` regardless of what it pinned.
pub fn rust_fallback_steps(
    rustup_available: bool,
    toolchain: Option<&str>,
) -> Vec<RustFallbackStep> {
    if !rustup_available {
        return vec![RustFallbackStep::InstallRustup {
            toolchain: toolchain.map(str::to_string),
        }];
    }

    let toolchain = toolchain
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("stable");

    vec![RustFallbackStep::InstallToolchain {
        toolchain: toolchain.to_string(),
    }]
}

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
                let toolchain = crate::rust_toolchain::detect_toolchain_owner(
                    &std::env::current_dir().unwrap_or_default(),
                )
                .channel()
                .map(str::to_string);

                for step in rust_fallback_steps(
                    self.check_command_exists("rustup").await,
                    toolchain.as_deref(),
                ) {
                    if step.changes_rustup_default() {
                        info!("Bootstrapping rustup — this sets the global default toolchain");
                    }
                    match step {
                        RustFallbackStep::InstallRustup { toolchain } => {
                            #[cfg(windows)]
                            {
                                let next = toolchain
                                    .as_deref()
                                    .map(|t| format!(", then run 'rustup toolchain install {t}'"))
                                    .unwrap_or_default();
                                return Err(EnsureError::NotInstalled {
                                    runtime: "Rust".to_string(),
                                    hint: format!(
                                        "Please install rustup from https://rustup.rs/{next}"
                                    ),
                                }
                                .into());
                            }
                            #[cfg(not(windows))]
                            {
                                let script = match &toolchain {
                                    Some(toolchain) => format!(
                                        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {toolchain}"
                                    ),
                                    None => String::from(
                                        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
                                    ),
                                };
                                self.run_install_command("sh", &["-c", &script]).await?;
                            }
                        }
                        RustFallbackStep::InstallToolchain { toolchain } => {
                            self.run_install_command(
                                "rustup",
                                &["toolchain", "install", &toolchain],
                            )
                            .await?;
                        }
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::{RustFallbackStep, rust_fallback_steps};

    /// Regression guard for PIP-3732: with rustup already installed, vx must never
    /// run `rustup default <toolchain>`. Doing so rewrote the *global* rustup default,
    /// so every project on the machine — including ones that pinned a different
    /// toolchain — silently ran on whatever vx installed.
    #[test]
    fn test_rustup_present_installs_toolchain_without_touching_default() {
        let steps = rust_fallback_steps(true, None);
        assert_eq!(
            steps,
            vec![RustFallbackStep::InstallToolchain {
                toolchain: "stable".to_string()
            }]
        );
        assert!(
            steps.iter().all(|s| !s.changes_rustup_default()),
            "no step may rewrite the global rustup default: {steps:?}"
        );
    }

    #[test]
    fn test_rustup_present_honours_requested_toolchain() {
        let steps = rust_fallback_steps(true, Some("1.83.0"));
        assert_eq!(
            steps,
            vec![RustFallbackStep::InstallToolchain {
                toolchain: "1.83.0".to_string()
            }]
        );
    }

    #[test]
    fn test_rustup_present_ignores_blank_toolchain() {
        let steps = rust_fallback_steps(true, Some("   "));
        assert_eq!(
            steps,
            vec![RustFallbackStep::InstallToolchain {
                toolchain: "stable".to_string()
            }]
        );
    }

    /// Without rustup the only option is to bootstrap it, which necessarily sets a
    /// default. That is an explicit "install Rust" action, not a silent override of an
    /// existing setup, and it is the single step allowed to change the default.
    #[test]
    fn test_rustup_absent_bootstraps_and_that_is_the_only_default_change() {
        let steps = rust_fallback_steps(false, Some("1.83.0"));
        assert_eq!(
            steps,
            vec![RustFallbackStep::InstallRustup {
                toolchain: Some("1.83.0".to_string())
            }]
        );
        assert!(steps[0].changes_rustup_default());
    }

    #[test]
    fn test_rustup_absent_without_toolchain() {
        assert_eq!(
            rust_fallback_steps(false, None),
            vec![RustFallbackStep::InstallRustup { toolchain: None }]
        );
    }
}
