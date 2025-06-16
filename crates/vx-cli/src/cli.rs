// CLI module - modular command structure
// Each command is implemented in its own module for better maintainability

use crate::commands::{global::GlobalCommand, venv_cmd::VenvCommand};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

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
    #[command(alias = "ls")]
    List {
        /// Tool name to show details for (optional)
        tool: Option<String>,
        /// Show installation status for tools
        #[arg(long)]
        status: bool,
        /// Show only installed tools
        #[arg(long)]
        installed: bool,
        /// Show only available tools
        #[arg(long)]
        available: bool,
    },

    /// Install a specific tool version
    #[command(alias = "i")]
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
    #[command(alias = "up")]
    Update {
        /// Tool name (optional, updates all if not specified)
        tool: Option<String>,
        /// Apply updates automatically
        #[arg(long)]
        apply: bool,
    },

    /// Uninstall tool versions (preferred over remove)
    #[command(alias = "rm")]
    Uninstall {
        /// Tool name
        tool: String,
        /// Version to uninstall (optional, removes all if not specified)
        version: Option<String>,
        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Show which tool version is being used (preferred over where)
    Which {
        /// Tool name
        tool: String,
        /// Show all installed versions
        #[arg(long)]
        all: bool,
    },

    /// Show available versions for a tool (preferred over fetch)
    Versions {
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

    /// Switch to a different version of a tool
    Switch {
        /// Tool and version (e.g., go@1.21.6, node@18.0.0)
        tool_version: String,
        /// Make this the global default
        #[arg(long)]
        global: bool,
    },

    /// Configuration management
    #[command(alias = "cfg")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },

    /// Search available tools
    Search {
        /// Search query
        query: Option<String>,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Show only installed tools
        #[arg(long)]
        installed_only: bool,
        /// Show only available (not installed) tools
        #[arg(long)]
        available_only: bool,
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        /// Show verbose information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Sync project tools from .vx.toml
    Sync {
        /// Only check, don't install
        #[arg(long)]
        check: bool,
        /// Force reinstall all tools
        #[arg(long)]
        force: bool,
        /// Preview operations without executing
        #[arg(long)]
        dry_run: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Disable parallel installation
        #[arg(long)]
        no_parallel: bool,
        /// Disable auto-install
        #[arg(long)]
        no_auto_install: bool,
    },

    /// Initialize vx configuration for current project
    Init {
        /// Interactive initialization
        #[arg(long)]
        interactive: bool,
        /// Use predefined template
        #[arg(long)]
        template: Option<String>,
        /// Specify tools to include (comma-separated)
        #[arg(long)]
        tools: Option<String>,
        /// Force overwrite existing configuration
        #[arg(long)]
        force: bool,
        /// Preview configuration without creating file
        #[arg(long)]
        dry_run: bool,
        /// List available templates
        #[arg(long)]
        list_templates: bool,
    },

    /// Clean up system (preferred over cleanup)
    #[command(alias = "clean")]
    Clean {
        /// Preview cleanup operations without executing
        #[arg(long)]
        dry_run: bool,
        /// Only clean cache files
        #[arg(long)]
        cache: bool,
        /// Only clean orphaned tool versions
        #[arg(long)]
        orphaned: bool,
        /// Clean all (cache + orphaned)
        #[arg(long)]
        all: bool,
        /// Force cleanup without confirmation
        #[arg(long)]
        force: bool,
        /// Clean files older than specified days
        #[arg(long)]
        older_than: Option<u32>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show package statistics and disk usage
    Stats,

    /// Plugin management commands
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },

    /// Shell integration commands
    Shell {
        #[command(subcommand)]
        command: ShellCommand,
    },

    /// Virtual environment management
    Venv {
        #[command(subcommand)]
        command: VenvCommand,
    },

    /// Global tool management
    Global {
        #[command(subcommand)]
        command: GlobalCommand,
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

#[derive(Subcommand, Clone)]
pub enum ShellCommand {
    /// Generate shell initialization script
    Init {
        /// Shell type (auto-detected if not specified)
        shell: Option<String>,
    },
    /// Generate shell completion script
    Completions {
        /// Shell type
        shell: String,
    },
}
