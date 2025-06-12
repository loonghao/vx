// CLI module - modular command structure
// Each command is implemented in its own module for better maintainability

use crate::commands::venv_cmd::VenvCommand;
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

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Show version information
    Version,

    /// List supported tools
    List,

    /// Install a specific tool version
    Install {
        /// Tool name (e.g., uv, node, go, rust)
        tool: String,
        /// Version to install (e.g., 1.0.0, latest, lts)
        version: Option<String>,
        /// Force reinstallation even if already installed
        #[arg(long)]
        force: bool,
    },

    /// Update tools to latest versions
    Update {
        /// Tool name (optional, updates all if not specified)
        tool: Option<String>,
        /// Apply updates automatically
        #[arg(long)]
        apply: bool,
    },

    /// Remove installed tool versions
    Remove {
        /// Tool name
        tool: String,
        /// Version to remove (optional, removes all if not specified)
        version: Option<String>,
        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Show where a tool is installed
    Where {
        /// Tool name
        tool: String,
        /// Show all installed versions
        #[arg(long)]
        all: bool,
    },

    /// Fetch and display available versions for a tool
    Fetch {
        /// Tool name
        tool: String,
        /// Show only latest versions (limit results)
        #[arg(long)]
        latest: Option<usize>,
        /// Include pre-release versions
        #[arg(long)]
        prerelease: bool,
        /// Show detailed version information
        #[arg(long)]
        detailed: bool,
        /// Interactive mode for version selection
        #[arg(short, long)]
        interactive: bool,
    },

    /// Set default version for a tool (deprecated, use switch)
    Use {
        /// Tool and version (e.g., uv@1.0.0, node@18.0.0)
        tool_version: String,
    },

    /// Switch to a different version of a tool
    Switch {
        /// Tool and version (e.g., go@1.21.6, node@18.0.0)
        tool_version: String,
        /// Make this the global default
        #[arg(long)]
        global: bool,
    },

    /// Show configuration
    Config,

    /// Initialize vx configuration for current project
    Init,

    /// Clean up orphaned packages and cache
    Cleanup,

    /// Show package statistics and disk usage
    Stats,

    /// Plugin management commands
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },

    /// Virtual environment management
    Venv {
        #[command(subcommand)]
        command: VenvCommand,
    },
}

#[derive(Subcommand, Clone)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Set configuration value
    Set {
        /// Configuration key (e.g., defaults.auto_install)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Reset configuration to defaults
    Reset {
        /// Reset specific key only
        key: Option<String>,
    },
    /// Edit configuration file
    Edit,
}

#[derive(Subcommand, Clone)]
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
