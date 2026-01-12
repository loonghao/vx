//! Check command - Verify if a runtime is available
//!
//! This command checks if a runtime is available on the system or can be installed.
//! It's useful for testing and CI/CD pipelines to verify tool availability.

use crate::commands::{CommandContext, CommandHandler};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

#[derive(Args, Clone)]
pub struct CheckCommand {
    /// Runtime name to check
    pub runtime: String,

    /// Check if runtime is installed in vx store
    #[arg(long)]
    pub installed: bool,

    /// Check if runtime is available on system PATH
    #[arg(long)]
    pub system: bool,

    /// Show detailed detection information
    #[arg(long)]
    pub detailed: bool,

    /// Exit with code 0 if available, 1 if not (silent mode)
    #[arg(short, long)]
    pub quiet: bool,
}

#[async_trait]
impl CommandHandler for CheckCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<()> {
        handle(
            ctx,
            &self.runtime,
            self.installed,
            self.system,
            self.detailed,
            self.quiet,
        )
        .await
    }
}

/// Handle the check command
pub async fn handle(
    ctx: &CommandContext,
    runtime_name: &str,
    check_installed: bool,
    check_system: bool,
    detailed: bool,
    quiet: bool,
) -> Result<()> {
    // Find the provider that supports this runtime
    let provider = ctx
        .registry()
        .get_provider(runtime_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown runtime: {}", runtime_name))?;

    // Get the runtime
    let runtime = provider
        .get_runtime(runtime_name)
        .ok_or_else(|| anyhow::anyhow!("Runtime '{}' not found", runtime_name))?;

    // Check platform support
    let current_platform = vx_runtime::Platform::current();
    
    // Use the runtime's built-in platform check method
    if !runtime.is_platform_supported(&current_platform) {
        if !quiet {
            let supported_platforms = runtime.supported_platforms();
            eprintln!(
                "❌ Runtime '{}' does not support the current platform ({}-{})",
                runtime_name, current_platform.os, current_platform.arch
            );
            if !supported_platforms.is_empty() {
                eprintln!("   Supported platforms:");
                for platform in supported_platforms {
                    eprintln!("   - {}-{}", platform.os, platform.arch);
                }
            }
        }
        std::process::exit(1);
    }

    // Check if auto-installable
    let auto_installable = true; // Most runtimes are auto-installable by default

    // Perform checks based on flags
    let mut available = false;

    // 1. Check if installed in vx store
    if check_installed || !check_system {
        let path_manager = vx_paths::PathManager::new()?;
        let installed_versions = path_manager.list_store_versions(runtime_name)?;
        let has_installed = !installed_versions.is_empty();

        if !quiet {
            if has_installed {
                println!("✓ Runtime '{}' is installed in vx store", runtime_name);
                if detailed {
                    println!("  Versions: {}", installed_versions.join(", "));
                }
            } else {
                println!("✗ Runtime '{}' is not installed in vx store", runtime_name);
            }
        }

        available = available || has_installed;
    }

    // 2. Check if available on system PATH
    if check_system || !check_installed {
        // Try to detect system installation
        // Note: We can't use Runtime::detect_system_installation directly
        // Instead, check if the executable exists in PATH
        let executable_name = runtime.name();
        let exists_in_path = which::which(executable_name).is_ok();

        if exists_in_path {
            if !quiet {
                println!(
                    "✓ Runtime '{}' is available on system PATH",
                    runtime_name
                );
                if detailed {
                    if let Ok(path) = which::which(executable_name) {
                        println!("  Path: {}", path.display());
                    }
                }
            }
            available = true;
        } else {
            if !quiet {
                println!("✗ Runtime '{}' is not available on system PATH", runtime_name);
            }
        }
    }

    // 3. Show auto-installability
    if !quiet && detailed {
        if auto_installable {
            println!("✓ Runtime '{}' can be auto-installed by vx", runtime_name);
        } else {
            println!("✗ Runtime '{}' cannot be auto-installed", runtime_name);
            println!("  Please install it manually");
        }
    }

    // Exit with appropriate code
    if !quiet {
        println!();
        if available {
            println!("✓ Runtime '{}' is available", runtime_name);
        } else if auto_installable {
            println!("⚠ Runtime '{}' is not available but can be auto-installed", runtime_name);
            println!("  Run: vx install {}", runtime_name);
        } else {
            println!("✗ Runtime '{}' is not available", runtime_name);
        }
    }

    if available {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
