//! Install command arguments

use clap::Args as ClapArgs;

/// Install tool(s) - supports multiple tools at once
#[derive(ClapArgs, Clone, Debug)]
#[command(alias = "i")]
pub struct Args {
    /// Tools to install (e.g., uv, node@22, go@1.22, rust)
    /// Format: tool or tool@version
    #[arg(required = true, num_args = 1..)]
    pub tools: Vec<String>,

    /// Force reinstallation even if already installed
    #[arg(short, long)]
    pub force: bool,
}
