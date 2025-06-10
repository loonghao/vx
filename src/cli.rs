use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vx")]
#[command(about = "Universal version executor for development tools")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Use system PATH to find tools instead of vx-managed versions
    #[arg(long, global = true)]
    pub use_system_path: bool,

    /// Enable verbose output with detailed logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Tool and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show version information
    Version,
    /// List available tools and versions
    List,
    /// Install a specific tool version
    Install {
        /// Tool name (e.g., uv, node, go, rust)
        tool: String,
        /// Version to install (e.g., 1.0.0, latest)
        version: Option<String>,
        /// Force reinstallation even if already installed
        #[arg(long)]
        force: bool,
    },
    /// Set default version for a tool
    Use {
        /// Tool and version (e.g., uv@1.0.0, node@18.0.0)
        tool_version: String,
    },
    /// Show configuration
    Config,
    /// Initialize vx configuration for current project
    Init,
    /// Switch to a different version of a tool
    Switch {
        /// Tool and version (e.g., go@1.21.6, node@18.0.0)
        tool_version: String,
    },
    /// Remove a specific version of a tool
    Remove {
        /// Tool name
        tool: String,
        /// Version to remove (optional, removes all if not specified)
        version: Option<String>,
        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },
    /// Clean up orphaned packages
    Cleanup,
    /// Show package statistics
    Stats,
    /// Check for updates
    Update {
        /// Tool name (optional, checks all if not specified)
        tool: Option<String>,
        /// Actually perform the update
        #[arg(long)]
        apply: bool,
    },
    /// Plugin management commands
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },
}

#[derive(Subcommand)]
pub enum PluginCommand {
    /// List all plugins
    List {
        /// Show only enabled plugins
        #[arg(long)]
        enabled: bool,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
    },
    /// Show plugin information
    Info {
        /// Plugin name
        name: String,
    },
    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },
    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },
    /// Search plugins
    Search {
        /// Search query
        query: String,
    },
    /// Show plugin statistics
    Stats,
}
