//! Global package command arguments

use clap::{Args as ClapArgs, Subcommand};

/// Global package management subcommand
#[derive(Subcommand, Clone, Debug)]
pub enum GlobalCommand {
    /// Install a package globally (isolated)
    Install(InstallGlobalArgs),

    /// List globally installed packages
    #[command(alias = "ls")]
    List(ListGlobalArgs),

    /// Uninstall a global package
    #[command(alias = "rm")]
    Uninstall(UninstallGlobalArgs),

    /// Show information about a global package
    Info(InfoGlobalArgs),

    /// Update shims after manual changes
    #[command(name = "shim-update")]
    ShimUpdate,
}

/// Arguments for `vx install-global` / `vx global install`
#[derive(ClapArgs, Clone, Debug)]
pub struct InstallGlobalArgs {
    /// Package specification (e.g., typescript@5.3, npm:typescript, pip:black@24.1)
    ///
    /// Formats:
    /// - package@version (auto-detect ecosystem)
    /// - ecosystem:package@version (explicit ecosystem)
    /// - package (latest version, auto-detect ecosystem)
    #[arg(required = true)]
    pub package: String,

    /// Force reinstallation even if already installed
    #[arg(short, long)]
    pub force: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Extra arguments to pass to the package manager
    #[arg(last = true)]
    pub extra_args: Vec<String>,
}

/// Arguments for `vx list-global` / `vx global list`
#[derive(ClapArgs, Clone, Debug)]
pub struct ListGlobalArgs {
    /// Filter by ecosystem (npm, pip, cargo, go, gem)
    #[arg(long)]
    pub ecosystem: Option<String>,

    /// Output format (table, json, plain)
    #[arg(long, value_enum, default_value = "table")]
    pub format: GlobalListFormat,

    /// Show detailed information including executables and paths
    #[arg(short, long)]
    pub verbose: bool,
}

/// Output format for global list command
///
/// Note: This is a local format enum for the global list command.
/// The global --json/--format flags (RFC 0031) take precedence when specified.
#[derive(Clone, Debug, Default, clap::ValueEnum)]
pub enum GlobalListFormat {
    #[default]
    Table,
    Json,
    Plain,
}

/// Arguments for `vx uninstall-global` / `vx global uninstall`
#[derive(ClapArgs, Clone, Debug)]
pub struct UninstallGlobalArgs {
    /// Package specification (e.g., typescript, npm:typescript)
    ///
    /// Formats:
    /// - package (auto-detect ecosystem from registry)
    /// - ecosystem:package (explicit ecosystem)
    #[arg(required = true)]
    pub package: String,

    /// Force removal without confirmation
    #[arg(short, long)]
    pub force: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Arguments for `vx info-global` / `vx global info`
#[derive(ClapArgs, Clone, Debug)]
pub struct InfoGlobalArgs {
    /// Package name or executable name
    #[arg(required = true)]
    pub package: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}
