//! Environment command arguments

use clap::{Args as ClapArgs, Subcommand};

/// Environment management
#[derive(ClapArgs, Clone, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: EnvCommand,
}

/// Environment subcommands
#[derive(Subcommand, Clone, Debug)]
pub enum EnvCommand {
    /// Create a new environment
    ///
    /// By default, creates a project-local environment in `.vx/env/` if `vx.toml` exists.
    /// Use `--global` to create a named global environment in `~/.vx/envs/`.
    Create {
        /// Environment name (optional for project environments)
        name: Option<String>,
        /// Create a global environment instead of project-local
        #[arg(long, short)]
        global: bool,
        /// Clone from an existing environment
        #[arg(long)]
        from: Option<String>,
        /// Set as default environment after creation
        #[arg(long)]
        set_default: bool,
    },

    /// Activate an environment
    Use {
        /// Environment name (optional, uses project env if available)
        name: Option<String>,
        /// Set as the global default
        #[arg(long)]
        global: bool,
    },

    /// List all environments
    #[command(alias = "ls")]
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
        /// Show only global environments
        #[arg(long)]
        global: bool,
    },

    /// Delete an environment
    #[command(alias = "rm")]
    Delete {
        /// Environment name (optional for project environment)
        name: Option<String>,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
        /// Delete global environment
        #[arg(long, short)]
        global: bool,
    },

    /// Show current environment details
    Show {
        /// Environment name (defaults to current)
        name: Option<String>,
    },

    /// Add a runtime to an environment
    Add {
        /// Runtime and version (e.g., node@20.0.0)
        runtime_version: String,
        /// Target global environment name
        #[arg(long)]
        env: Option<String>,
        /// Add to global environment instead of project
        #[arg(long, short)]
        global: bool,
    },

    /// Remove a runtime from an environment
    Remove {
        /// Runtime name
        runtime: String,
        /// Target global environment name
        #[arg(long)]
        env: Option<String>,
        /// Remove from global environment
        #[arg(long, short)]
        global: bool,
    },

    /// Sync project environment from vx.toml
    ///
    /// Creates symlinks in `.vx/env/` for all tools defined in `vx.toml`
    Sync,

    /// Enter an environment shell
    ///
    /// Spawns an interactive shell with the environment's tools available in PATH.
    /// Similar to `vx dev` but uses the environment directory instead of vx.toml.
    Shell {
        /// Environment name (defaults to project env or global default)
        name: Option<String>,
        /// Use global environment
        #[arg(long, short)]
        global: bool,
        /// Shell to use (defaults to auto-detect)
        #[arg(long)]
        shell: Option<String>,
        /// Command to execute instead of interactive shell
        #[arg(last = true)]
        command: Option<Vec<String>>,
        /// Export environment variables instead of spawning shell
        #[arg(long)]
        export: bool,
        /// Export format (shell, powershell, batch, github)
        #[arg(long)]
        format: Option<String>,
    },
}
