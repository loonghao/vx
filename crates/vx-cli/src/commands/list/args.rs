//! List command arguments

use clap::Args as ClapArgs;

/// List supported tools
#[derive(ClapArgs, Clone, Debug)]
#[command(alias = "ls")]
pub struct Args {
    /// Tool name to show details for (optional)
    pub tool: Option<String>,

    /// Show installation status for tools
    #[arg(long)]
    pub status: bool,

    /// Show only installed tools
    #[arg(long)]
    pub installed: bool,

    /// Show only available tools
    #[arg(long)]
    pub available: bool,

    /// Show all tools including those not supported on current platform
    #[arg(long, short = 'a')]
    pub all: bool,

    /// Show system tools (discovered from PATH and known locations)
    #[arg(long)]
    pub system: bool,
}
