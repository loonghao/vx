//! Dev command arguments

use clap::Args as ClapArgs;

/// Enter development environment
#[derive(ClapArgs, Clone, Debug)]
pub struct Args {
    /// Shell to use (auto-detected if not specified)
    #[arg(long, short)]
    pub shell: Option<String>,

    /// Execute a command instead of entering shell
    #[arg(last = true)]
    pub command: Option<Vec<String>>,

    /// Don't auto-install missing tools
    #[arg(long)]
    pub no_install: bool,

    /// Show verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Export environment variables instead of spawning shell
    #[arg(long)]
    pub export: bool,

    /// Export format (shell, powershell, batch, github)
    #[arg(long)]
    pub format: Option<String>,

    /// Show environment info without entering shell
    #[arg(long, short)]
    pub info: bool,

    /// Inherit system PATH (disable isolation)
    #[arg(long)]
    pub inherit_system: bool,

    /// Additional patterns for passenv (can be specified multiple times)
    #[arg(long = "passenv", value_name = "PATTERN")]
    pub passenv: Vec<String>,
}
