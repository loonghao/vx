// CLI module - modular command structure
// Each command is implemented in its own module for better maintainability
//
// Design Principles (inspired by uv):
// - Clear command grouping: tool management, project management, cache management
// - Unified verbs: add, remove, sync, lock, run
// - Subcommand organization: cache, shell, ext
// - No redundant commands - each command has a single purpose

use crate::commands::{
    CommandContext, CommandHandler, GlobalOptions, env::EnvCommand, global::GlobalCommand,
};

use anyhow::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use vx_runtime::CacheMode;

/// Unified output format for all commands (RFC 0031)
///
/// Replaces the previous fragmented OutputFormat enums.
/// Commands use this to determine how to render their output.
#[derive(ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable colored text output (default)
    #[default]
    Text,
    /// JSON structured output (for scripts/CI/AI parsing)
    Json,
    /// TOON format output (for LLM prompts, saves tokens) — not yet supported
    Toon,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CacheModeArg {
    /// Use cache if valid, otherwise compute/fetch (default)
    Normal,
    /// Force refresh, ignore cache
    Refresh,
    /// Use cache only, fail if not available
    Offline,
    /// Skip cache entirely
    NoCache,
}

impl From<CacheModeArg> for CacheMode {
    fn from(value: CacheModeArg) -> Self {
        match value {
            CacheModeArg::Normal => CacheMode::Normal,
            CacheModeArg::Refresh => CacheMode::Refresh,
            CacheModeArg::Offline => CacheMode::Offline,
            CacheModeArg::NoCache => CacheMode::NoCache,
        }
    }
}

#[derive(Parser)]
#[command(name = "vx")]
#[command(about = "Universal version executor for development tools")]
#[command(version)]
#[command(after_help = "Run 'vx <command> --help' for more information on a command.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Use system PATH to find tools instead of vx-managed versions
    #[arg(long, global = true)]
    pub use_system_path: bool,

    /// Inherit system environment variables when using isolated environments
    #[arg(long, global = true)]
    pub inherit_env: bool,

    /// Cache mode: normal, refresh, offline, no-cache
    #[arg(long, global = true, value_enum, default_value = "normal")]
    pub cache_mode: CacheModeArg,

    /// Enable verbose output with detailed logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable debug output (equivalent to RUST_LOG=debug)
    #[arg(long, global = true)]
    pub debug: bool,

    /// Output format: text, json, toon (RFC 0031)
    #[arg(long = "output-format", global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub output_format: OutputFormat,

    /// JSON output shortcut (equivalent to --output-format json)
    #[arg(long, global = true)]
    pub json: bool,

    /// Additional runtime dependencies to inject into the environment (can be specified multiple times)
    ///
    /// Similar to uvx --with or rez-env, this option injects additional runtimes into the PATH
    /// before executing the tool. Useful when a tool requires multiple runtimes.
    ///
    /// Examples:
    ///   vx --with bun npm:opencode-ai@latest::opencode
    ///   vx --with bun@1.1.0 --with deno node my-script.js
    #[arg(long = "with", short = 'w', action = clap::ArgAction::Append, global = true)]
    pub with_deps: Vec<String>,

    /// Tool and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

impl From<&Cli> for GlobalOptions {
    fn from(cli: &Cli) -> Self {
        // --json flag overrides --format
        let output_format = if cli.json {
            OutputFormat::Json
        } else {
            // Also check VX_OUTPUT environment variable
            match std::env::var("VX_OUTPUT").as_deref() {
                Ok("json") => OutputFormat::Json,
                Ok("toon") => OutputFormat::Toon,
                _ => {
                    // Legacy support: VX_OUTPUT_JSON=1
                    if std::env::var("VX_OUTPUT_JSON").is_ok() {
                        OutputFormat::Json
                    } else {
                        cli.output_format
                    }
                }
            }
        };
        GlobalOptions {
            use_system_path: cli.use_system_path,
            inherit_env: cli.inherit_env,
            cache_mode: cli.cache_mode.into(),
            verbose: cli.verbose,
            debug: cli.debug,
            with_deps: cli.with_deps.clone(),
            output_format,
        }
    }
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    // =========================================================================
    // Tool Management
    // =========================================================================
    /// Install tool(s) - supports multiple tools at once
    #[command(alias = "i")]
    Install {
        /// Tools to install (e.g., uv, node@22, go@1.22, rust)
        #[arg(required = true, num_args = 1..)]
        tools: Vec<String>,
        /// Force reinstallation even if already installed
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall tool versions from global store
    Uninstall {
        /// Tool name (e.g., python, python@3.7)
        tool: String,
        /// Version to uninstall (optional, removes all if not specified)
        version: Option<String>,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List installed tools and available runtimes
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
        /// Show all tools including those not supported on current platform
        #[arg(long, short = 'a')]
        all: bool,
        /// Show system tools (discovered from PATH and known locations)
        #[arg(long)]
        system: bool,
    },

    /// Show available versions for a tool
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

    /// Show which tool version is being used
    #[command(alias = "where")]
    Which {
        /// Tool name
        tool: String,
        /// Show all installed versions
        #[arg(short, long)]
        all: bool,
    },

    /// Switch to a different version of a tool
    Switch {
        /// Tool and version (e.g., go@1.21.6, node@18.0.0)
        tool_version: String,
        /// Make this the global default
        #[arg(long)]
        global: bool,
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
        /// Show verbose information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Global package management (RFC 0025)
    ///
    /// Install, list, and manage globally installed packages with isolation.
    /// Packages are installed to ~/.vx/packages/ and accessed via shims.
    #[command(alias = "g")]
    Global {
        #[command(subcommand)]
        command: GlobalCommand,
    },

    /// Test runtime availability and providers (CI-friendly)
    Test {
        /// Runtime name to test (e.g., "yarn", "node", "go")
        runtime: Option<String>,

        /// Test all registered runtimes
        #[arg(long, conflicts_with_all = &["runtime", "extension", "local"])]
        all: bool,

        /// Test a provider from URL
        #[arg(long, conflicts_with_all = &["runtime", "all", "local"])]
        extension: Option<String>,

        /// Test a local provider directory (for development)
        #[arg(long, conflicts_with_all = &["runtime", "all", "extension"])]
        local: Option<PathBuf>,

        /// Only test platform support (no installation required)
        #[arg(long)]
        platform_only: bool,

        /// Run functional tests (execute --version, etc.)
        #[arg(long)]
        functional: bool,

        /// Test installation process
        #[arg(long)]
        install: bool,

        /// Full CI test: install + functional tests for all runtimes
        #[arg(long)]
        ci: bool,

        /// Specify runtimes to test in CI mode (comma-separated)
        #[arg(long, value_delimiter = ',')]
        ci_runtimes: Option<Vec<String>>,

        /// Skip these runtimes in CI mode (comma-separated)
        #[arg(long, value_delimiter = ',')]
        ci_skip: Option<Vec<String>>,

        /// Timeout for each runtime test in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Continue testing even if some runtimes fail
        #[arg(long)]
        keep_going: bool,

        /// Use a custom VX root directory for testing (isolated environment)
        #[arg(long)]
        vx_root: Option<PathBuf>,

        /// Use a temporary directory as VX root (auto-cleaned after test)
        #[arg(long)]
        temp_root: bool,

        /// Clean up after CI test: uninstall runtimes and verify removal
        #[arg(long)]
        cleanup: bool,

        /// Check if runtime is installed in vx store
        #[arg(long)]
        installed: bool,

        /// Check if runtime is available on system PATH
        #[arg(long)]
        system: bool,

        /// Show detailed test information
        #[arg(long)]
        detailed: bool,

        /// Silent mode: exit code only, no output
        #[arg(short, long)]
        quiet: bool,

        /// JSON output format (for CI integration)
        #[arg(long)]
        json: bool,

        /// Verbose output (show all test steps)
        #[arg(short, long)]
        verbose: bool,
    },

    // =========================================================================
    // Project Management
    // =========================================================================
    /// Initialize vx configuration for current project
    Init {
        /// Interactive initialization
        #[arg(short, long)]
        interactive: bool,
        /// Use predefined template
        #[arg(short, long)]
        template: Option<String>,
        /// Specify tools to include (comma-separated)
        #[arg(long)]
        tools: Option<String>,
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
        /// Preview configuration without creating file
        #[arg(long)]
        dry_run: bool,
        /// List available templates
        #[arg(long)]
        list_templates: bool,
    },

    /// Add a tool to project configuration (vx.toml)
    Add {
        /// Tool name (e.g., node, python, uv)
        tool: String,
        /// Version to use (default: latest)
        #[arg(long)]
        version: Option<String>,
    },

    /// Remove a tool from project configuration (vx.toml)
    #[command(alias = "rm")]
    Remove {
        /// Tool name to remove
        tool: String,
    },

    /// Sync project tools from vx.toml
    Sync {
        /// Only check, don't install
        #[arg(long)]
        check: bool,
        /// Force reinstall all tools
        #[arg(short, long)]
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
        /// Automatically generate/update vx.lock if needed
        #[arg(long)]
        auto_lock: bool,
    },

    /// Generate or update vx.lock for reproducible environments
    Lock {
        /// Update all tools to latest compatible versions
        #[arg(long)]
        update: bool,
        /// Update specific tool only
        #[arg(long)]
        tool: Option<String>,
        /// Preview changes without writing
        #[arg(long)]
        dry_run: bool,
        /// Check lock file consistency with vx.toml (don't update)
        #[arg(long)]
        check: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Check version constraints and tool availability (RFC 0023)
    Check {
        /// Tool name to check (optional, checks all if not specified)
        tool: Option<String>,
        /// Show detailed information about each tool
        #[arg(long, short = 'd')]
        detailed: bool,
        /// Quiet mode: exit code only, no output
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// Create offline development environment bundle
    Bundle {
        #[command(subcommand)]
        command: BundleCommand,
    },

    /// Run a script defined in vx.toml
    Run {
        /// Script name (use --list to see available scripts)
        script: Option<String>,
        /// List available scripts
        #[arg(long, short = 'l')]
        list: bool,
        /// Show help for the run command or script-specific help
        #[arg(long, short = 'H', action = clap::ArgAction::SetTrue)]
        script_help: bool,
        /// Additional arguments to pass to the script
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Analyze project dependencies, scripts, and tools
    Analyze {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    // =========================================================================
    // Environment Management
    // =========================================================================
    /// Enter development environment with all project tools
    Dev {
        /// Shell to use (auto-detected if not specified)
        #[arg(long)]
        shell: Option<String>,
        /// Run a command in the dev environment instead of spawning a shell
        #[arg(short, long, num_args = 1..)]
        command: Option<Vec<String>>,
        /// Don't install missing tools
        #[arg(long)]
        no_install: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Export environment variables for shell activation
        #[arg(long, short = 'e')]
        export: bool,
        /// Output format for --export: shell, powershell, batch, github
        #[arg(long, short = 'f')]
        format: Option<String>,
        /// Show detailed environment information (tools, paths, conflicts)
        #[arg(long, short = 'i')]
        info: bool,
        /// Inherit system PATH (disable isolation mode)
        ///
        /// By default, vx dev uses isolation mode where only vx-managed tools
        /// are available. Use this flag to include system tools in PATH.
        #[arg(long)]
        inherit_system: bool,
        /// Additional environment variables to pass through (can be specified multiple times)
        ///
        /// Supports glob patterns like SSH_*, GITHUB_*
        #[arg(long = "passenv", short = 'p', action = clap::ArgAction::Append)]
        passenv: Vec<String>,
    },

    /// Setup development environment (install all project tools)
    Setup {
        /// Force reinstall all tools
        #[arg(short, long)]
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
        /// Skip lifecycle hooks (pre_setup, post_setup)
        #[arg(long)]
        no_hooks: bool,
        /// CI mode: output tool paths for CI environment
        #[arg(long)]
        ci: bool,
    },

    /// Environment management
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },

    // =========================================================================
    // Cache & Maintenance
    // =========================================================================
    /// Cache management commands
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },

    // =========================================================================
    // Configuration
    // =========================================================================
    /// Configuration management
    #[command(alias = "cfg")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },

    // =========================================================================
    // Shell Integration
    // =========================================================================
    /// Shell integration commands
    Shell {
        #[command(subcommand)]
        command: ShellCommand,
    },

    // =========================================================================
    // Extensions
    // =========================================================================
    /// Extension management
    #[command(alias = "extension")]
    Ext {
        #[command(subcommand)]
        command: ExtCommand,
    },

    /// Execute an extension command
    #[command(name = "x")]
    X {
        /// Extension name
        extension: String,
        /// Arguments to pass to the extension
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Plugin management commands
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },

    // =========================================================================
    // Lifecycle & Hooks
    // =========================================================================
    /// Execute or manage lifecycle hooks
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },

    // =========================================================================
    // Services & Container
    // =========================================================================
    /// Manage development services (Docker/Podman)
    Services {
        #[command(subcommand)]
        command: ServicesCommand,
    },

    /// Container and Dockerfile management
    Container {
        #[command(subcommand)]
        command: ContainerCommand,
    },

    // =========================================================================
    // System
    // =========================================================================
    /// Show version information
    Version,

    /// View execution performance metrics and reports
    Metrics {
        /// Number of recent runs to show (default: 10)
        #[arg(long, short = 'n', default_value = "10")]
        last: usize,
        /// Output as JSON (AI-friendly summary)
        #[arg(long)]
        json: bool,
        /// Export interactive HTML report (optionally specify output path)
        #[arg(long)]
        html: Option<String>,
        /// Remove all metrics data
        #[arg(long)]
        clean: bool,
    },

    /// Update vx itself to the latest version
    #[command(name = "self-update")]
    SelfUpdate {
        /// Only check for updates, don't install
        #[arg(long)]
        check: bool,
        /// Specific version to install
        version: Option<String>,
        /// GitHub token for authenticated API requests
        #[arg(long)]
        token: Option<String>,
        /// Include pre-release versions
        #[arg(long)]
        prerelease: bool,
        /// Force update even if already up to date
        #[arg(short, long)]
        force: bool,
    },

    /// Show system information and capabilities
    Info {
        /// Output as JSON (recommended for AI)
        #[arg(long)]
        json: bool,
        /// Show build warnings and diagnostics
        #[arg(long)]
        warnings: bool,
    },

    /// Migrate configuration and data to latest format
    Migrate {
        /// Path to project directory (default: current directory)
        #[arg(short, long)]
        path: Option<String>,
        /// Preview changes without writing
        #[arg(long)]
        dry_run: bool,
        /// Create backup before migration
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        backup: bool,
        /// Only check which migrations are needed
        #[arg(long)]
        check: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    // =========================================================================
    // Authentication
    // =========================================================================
    /// Authentication for external services (GitHub, etc.)
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },

    // =========================================================================
    // AI Tools
    // =========================================================================
    /// AI agent skills management
    ///
    /// Install vx skills for AI coding agents, manage Vercel Skills CLI,
    /// and configure AI agent integrations.
    Ai {
        #[command(subcommand)]
        command: AiCommand,
    },
}

// =============================================================================
// Subcommands
// =============================================================================

#[derive(Subcommand, Clone)]
pub enum CacheCommand {
    /// Show cache statistics and disk usage
    #[command(alias = "stats")]
    Info,

    /// List cached items
    #[command(alias = "ls")]
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Prune expired and orphaned cache entries (safe cleanup)
    Prune {
        /// Preview cleanup operations without executing
        #[arg(long)]
        dry_run: bool,
        /// Only prune version cache
        #[arg(long)]
        versions: bool,
        /// Only prune download cache
        #[arg(long)]
        downloads: bool,
        /// Only prune resolution cache
        #[arg(long)]
        resolutions: bool,
        /// Only prune orphaned tool versions
        #[arg(long)]
        orphaned: bool,
        /// Prune files older than specified days
        #[arg(long)]
        older_than: Option<u32>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Purge all cache data (destructive)
    Purge {
        /// Only purge version cache
        #[arg(long)]
        versions: bool,
        /// Only purge download cache
        #[arg(long)]
        downloads: bool,
        /// Only purge resolution cache
        #[arg(long)]
        resolutions: bool,
        /// Purge cache for specific tool only
        #[arg(long)]
        tool: Option<String>,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Show cache directory path
    Dir,
}

#[derive(Subcommand, Clone)]
pub enum BundleCommand {
    /// Create offline bundle from vx.lock
    Create {
        /// Only bundle specific tools (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tools: Option<Vec<String>>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Update bundle with changed tools (incremental)
    Update {
        /// Only update specific tools (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tools: Option<Vec<String>>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show bundle status
    Status {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Export bundle to a portable archive
    Export {
        /// Output archive path (default: vx-bundle-{platform}.tar.gz)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Include only specific tools
        #[arg(long, value_delimiter = ',')]
        tools: Option<Vec<String>>,
        /// Export only specific platforms (default: all platforms in bundle)
        #[arg(long, value_delimiter = ',')]
        platforms: Option<Vec<String>>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Import bundle from an archive
    Import {
        /// Archive path to import from
        archive: PathBuf,
        /// Force overwrite existing bundle
        #[arg(short, long)]
        force: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Remove the bundle
    Clean {
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
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
    /// Validate vx.toml configuration
    Validate {
        /// Path to vx.toml file (default: current directory)
        #[arg(short, long)]
        path: Option<String>,
        /// Show verbose validation output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate JSON Schema for vx.toml
    Schema {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show configuration directory path
    Dir,
}

#[derive(Subcommand, Clone)]
pub enum PluginCommand {
    /// List all plugins
    #[command(alias = "ls")]
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

#[derive(Subcommand, Clone)]
pub enum HookCommand {
    /// Run pre-commit hook
    PreCommit,
    /// Run enter hook (directory change)
    Enter,
    /// Install git hooks
    Install {
        /// Force reinstall even if already installed
        #[arg(short, long)]
        force: bool,
    },
    /// Uninstall git hooks
    Uninstall,
    /// Show hook status
    Status,
    /// Run a custom hook by name
    Run {
        /// Hook name
        name: String,
    },
    /// Generate shell integration script for enter hook
    ShellInit {
        /// Shell type (auto-detected if not specified)
        shell: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum ServicesCommand {
    /// Start services
    Start {
        /// Service names (start all if not specified)
        #[arg(num_args = 0..)]
        services: Vec<String>,
        /// Run in foreground (default: detached)
        #[arg(long)]
        foreground: bool,
        /// Force restart if already running
        #[arg(short, long)]
        force: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Stop services
    Stop {
        /// Service names (stop all if not specified)
        #[arg(num_args = 0..)]
        services: Vec<String>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show service status
    Status {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show service logs
    Logs {
        /// Service name
        service: String,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        /// Number of lines to show
        #[arg(long)]
        tail: Option<usize>,
    },
    /// Restart services
    Restart {
        /// Service names (restart all if not specified)
        #[arg(num_args = 0..)]
        services: Vec<String>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum ContainerCommand {
    /// Generate Dockerfile from configuration
    Generate {
        /// Output path (default: Dockerfile)
        #[arg(short, long)]
        output: Option<String>,
        /// Generate .dockerignore as well
        #[arg(long)]
        with_ignore: bool,
        /// Preview without writing
        #[arg(long)]
        dry_run: bool,
        /// Use ecosystem-specific template (node, python, rust, go)
        #[arg(long)]
        template: Option<String>,
    },
    /// Build container image
    Build {
        /// Additional tags
        #[arg(short, long)]
        tag: Vec<String>,
        /// Build target (for multi-stage)
        #[arg(long)]
        target: Option<String>,
        /// Build arguments (KEY=VALUE)
        #[arg(long)]
        build_arg: Vec<String>,
        /// Platform(s) to build for
        #[arg(long)]
        platform: Vec<String>,
        /// Don't use cache
        #[arg(long)]
        no_cache: bool,
        /// Push after build
        #[arg(long)]
        push: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Push container image to registry
    Push {
        /// Image tag to push (default: all configured tags)
        tag: Option<String>,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show container configuration status
    Status,
    /// Login to container registry
    Login {
        /// Registry URL (default: from config)
        registry: Option<String>,
        /// Username
        #[arg(short, long)]
        username: Option<String>,
        /// Password (or use stdin)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// List generated image tags
    Tags {
        /// Show all possible tags
        #[arg(short, long)]
        all: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum ExtCommand {
    /// List installed extensions
    #[command(alias = "ls")]
    List {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show extension information
    Info {
        /// Extension name
        name: String,
    },
    /// Link a local extension for development
    Dev {
        /// Path to the extension directory
        path: String,
        /// Unlink instead of link
        #[arg(long)]
        unlink: bool,
    },
    /// Install an extension from a remote source
    Install {
        /// Extension source (e.g., github:user/repo, https://github.com/user/repo)
        source: String,
    },
    /// Uninstall an extension
    Uninstall {
        /// Extension name
        name: String,
    },
    /// Update an installed extension
    Update {
        /// Extension name (or --all to update all)
        name: Option<String>,
        /// Update all extensions
        #[arg(long)]
        all: bool,
    },
    /// Check for extension updates
    Check {
        /// Extension name (or --all to check all)
        name: Option<String>,
        /// Check all extensions
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum AuthCommand {
    /// Login to a service (GitHub, etc.)
    Login {
        /// Service to authenticate with (github)
        #[arg(default_value = "github")]
        service: String,
        /// Provide token directly instead of using device flow
        #[arg(long)]
        token: Option<String>,
    },
    /// Logout from a service
    Logout {
        /// Service to logout from (github)
        #[arg(default_value = "github")]
        service: String,
    },
    /// Show authentication status
    Status {
        /// Service to check (github, or all)
        #[arg(default_value = "github")]
        service: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum AiCommand {
    /// Install vx skills to AI agent configuration directories
    ///
    /// Copies the built-in vx-usage skill to each agent's skills directory,
    /// so AI coding agents can better understand and use vx.
    Setup {
        /// Target specific agents (default: all supported agents)
        #[arg(short, long, action = clap::ArgAction::Append)]
        agent: Vec<String>,
        /// Install to global (home) directory instead of project
        #[arg(short, long)]
        global: bool,
        /// Force overwrite existing skills
        #[arg(short, long)]
        force: bool,
    },

    /// List supported AI agents and their config paths
    Agents,

    /// Proxy to Vercel Skills CLI (auto-installs if needed)
    ///
    /// Pass any arguments to the skills CLI. Examples:
    ///   vx ai skills add <repo-url>
    ///   vx ai skills list
    ///   vx ai skills find <query>
    ///   vx ai skills init [name]
    Skills {
        /// Arguments to pass to the skills CLI
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

// =============================================================================
// CommandHandler Implementation
// =============================================================================

use crate::commands;

#[async_trait]
impl CommandHandler for Commands {
    fn name(&self) -> &'static str {
        match self {
            Commands::Version => "version",
            Commands::Analyze { .. } => "analyze",
            Commands::List { .. } => "list",
            Commands::Install { .. } => "install",
            Commands::SelfUpdate { .. } => "self-update",
            Commands::Uninstall { .. } => "uninstall",
            Commands::Which { .. } => "which",
            Commands::Versions { .. } => "versions",
            Commands::Switch { .. } => "switch",
            Commands::Config { .. } => "config",
            Commands::Search { .. } => "search",
            Commands::Global { .. } => "global",
            Commands::Test { .. } => "test",
            Commands::Sync { .. } => "sync",
            Commands::Init { .. } => "init",
            Commands::Cache { .. } => "cache",
            Commands::Plugin { .. } => "plugin",
            Commands::Shell { .. } => "shell",
            Commands::Env { .. } => "env",
            Commands::Dev { .. } => "dev",
            Commands::Setup { .. } => "setup",
            Commands::Add { .. } => "add",
            Commands::Remove { .. } => "remove",
            Commands::Run { .. } => "run",
            Commands::Services { .. } => "services",
            Commands::Hook { .. } => "hook",
            Commands::Container { .. } => "container",
            Commands::Ext { .. } => "ext",
            Commands::X { .. } => "x",
            Commands::Migrate { .. } => "migrate",
            Commands::Lock { .. } => "lock",
            Commands::Check { .. } => "check",
            Commands::Bundle { .. } => "bundle",
            Commands::Info { .. } => "info",
            Commands::Metrics { .. } => "metrics",
            Commands::Auth { .. } => "auth",
            Commands::Ai { .. } => "ai",
        }
    }

    async fn execute(&self, ctx: &CommandContext) -> Result<()> {
        match self {
            Commands::Version => commands::version::handle().await,

            Commands::Analyze { json, verbose } => commands::analyze::handle(*json, *verbose).await,

            Commands::List {
                tool,
                status,
                installed: _,
                available: _,
                all,
                system,
            } => {
                let args = commands::list::Args {
                    tool: tool.clone(),
                    status: *status,
                    installed: false,
                    available: false,
                    all: *all,
                    system: *system,
                };
                commands::list::handle(ctx, &args).await
            }

            Commands::Install { tools, force } => {
                let args = commands::install::Args {
                    tools: tools.clone(),
                    force: *force,
                };
                commands::install::handle(ctx, &args).await
            }

            Commands::SelfUpdate {
                check,
                version,
                token,
                prerelease,
                force,
            } => {
                commands::self_update::handle(
                    token.as_deref(),
                    *prerelease,
                    *force,
                    *check,
                    version.as_deref(),
                )
                .await
            }

            Commands::Uninstall {
                tool,
                version,
                force,
            } => {
                // Support tool@version format (e.g., "python@3.7")
                let (tool_name, parsed_version) = if let Some((t, v)) = tool.split_once('@') {
                    (t, Some(v.to_string()))
                } else {
                    (tool.as_str(), None)
                };
                let final_version = version.clone().or(parsed_version);

                // RFC 0033: If the tool has a package_alias, route to global uninstall
                // This makes `vx uninstall rez` equivalent to `vx global uninstall uv:rez`
                if let Some(alias) = ctx.get_package_alias(tool_name) {
                    let pkg_spec = format!("{}:{}", alias.ecosystem, alias.package);
                    tracing::debug!(
                        "RFC 0033: Routing uninstall {} -> global uninstall {} via package_alias",
                        tool_name,
                        pkg_spec
                    );
                    let uninstall_args = commands::global::UninstallGlobalArgs {
                        package: pkg_spec,
                        force: *force,
                        verbose: false,
                    };
                    return commands::global::handle(
                        ctx,
                        &commands::global::GlobalCommand::Uninstall(uninstall_args),
                    )
                    .await;
                }

                commands::remove::handle(
                    ctx.registry(),
                    ctx.runtime_context(),
                    tool_name,
                    final_version.as_deref(),
                    *force,
                )
                .await
            }

            Commands::Which { tool, all } => {
                // Support tool@version format (e.g., "yarn@4")
                let (tool_name, version) = if let Some((t, v)) = tool.split_once('@') {
                    (t.to_string(), Some(v.to_string()))
                } else {
                    (tool.clone(), None)
                };
                commands::where_cmd::handle(
                    ctx.registry(),
                    &tool_name,
                    version.as_deref(),
                    *all,
                    ctx.use_system_path(),
                )
                .await
            }

            Commands::Versions {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            } => {
                // Support tool@version format - extract just the tool name
                let tool_name = if let Some((t, _)) = tool.split_once('@') {
                    t
                } else {
                    tool.as_str()
                };
                commands::fetch::handle(
                    ctx.registry(),
                    ctx.runtime_context(),
                    tool_name,
                    *latest,
                    *prerelease,
                    *detailed,
                    *interactive,
                )
                .await
            }

            Commands::Switch {
                tool_version,
                global,
            } => commands::switch::handle(ctx.registry(), tool_version, *global).await,

            Commands::Config { command } => match command {
                Some(ConfigCommand::Show) | None => commands::config::handle().await,
                Some(ConfigCommand::Set { key, value }) => {
                    commands::config::handle_set(key, value).await
                }
                Some(ConfigCommand::Get { key }) => commands::config::handle_get(key).await,
                Some(ConfigCommand::Reset { key }) => {
                    commands::config::handle_reset(key.clone()).await
                }
                Some(ConfigCommand::Edit) => commands::config::handle_edit().await,
                Some(ConfigCommand::Validate { path, verbose }) => {
                    commands::config::handle_validate(path.clone(), *verbose).await
                }
                Some(ConfigCommand::Schema { output }) => {
                    commands::config::handle_schema(output.clone()).await
                }
                Some(ConfigCommand::Dir) => commands::config::handle_dir().await,
            },

            Commands::Init {
                interactive,
                template,
                tools,
                force,
                dry_run,
                list_templates,
            } => {
                commands::init::handle(
                    *interactive,
                    template.clone(),
                    tools.clone(),
                    *force,
                    *dry_run,
                    *list_templates,
                )
                .await
            }

            Commands::Cache { command } => commands::cache::handle(command.clone()).await,

            Commands::Plugin { command } => {
                commands::plugin::handle(ctx.registry(), command.clone()).await
            }

            Commands::Env { command } => {
                let args = commands::env::Args {
                    command: command.clone(),
                };
                commands::env::handle(&args).await
            }

            Commands::Search {
                query,
                category,
                installed_only,
                available_only,
                verbose,
            } => {
                commands::search::handle(
                    ctx.registry(),
                    query.clone(),
                    category.clone(),
                    *installed_only,
                    *available_only,
                    ctx.output_format(),
                    *verbose,
                )
                .await
            }

            Commands::Global { command } => commands::global::handle(ctx, command).await,

            Commands::Test {
                runtime,
                all,
                extension,
                local,
                platform_only,
                functional,
                install,
                ci,
                ci_runtimes,
                ci_skip,
                timeout,
                keep_going,
                vx_root,
                temp_root,
                cleanup,
                installed,
                system,
                detailed,
                quiet,
                json,
                verbose,
            } => {
                let args = commands::test::Args {
                    runtime: runtime.clone(),
                    all: *all,
                    extension: extension.clone(),
                    local: local.clone(),
                    platform_only: *platform_only,
                    functional: *functional,
                    install: *install,
                    ci: *ci,
                    ci_runtimes: ci_runtimes.clone(),
                    ci_skip: ci_skip.clone(),
                    timeout: *timeout,
                    keep_going: *keep_going,
                    vx_root: vx_root.clone(),
                    temp_root: *temp_root,
                    cleanup: *cleanup,
                    installed: *installed,
                    system: *system,
                    detailed: *detailed,
                    quiet: *quiet,
                    json: *json,
                    verbose: *verbose,
                };
                commands::test::handle(ctx, &args).await
            }

            Commands::Sync {
                check,
                force,
                dry_run,
                verbose,
                no_parallel,
                no_auto_install: _,
                auto_lock,
            } => {
                commands::sync::handle_with_options(
                    ctx.registry(),
                    commands::sync::SyncOptions {
                        check: *check,
                        force: *force,
                        dry_run: *dry_run,
                        verbose: *verbose,
                        no_parallel: *no_parallel,
                        auto_lock: *auto_lock,
                        analyze: true, // Enable project analysis by default
                    },
                )
                .await
            }

            Commands::Shell { command } => match command {
                ShellCommand::Init { shell } => {
                    commands::shell::handle_shell_init(shell.clone()).await
                }
                ShellCommand::Completions { shell } => {
                    commands::shell::handle_completion(shell.clone()).await
                }
            },

            Commands::Dev {
                shell,
                command,
                no_install,
                verbose,
                export,
                format,
                info,
                inherit_system,
                passenv,
            } => {
                let args = commands::dev::Args {
                    shell: shell.clone(),
                    command: command.clone(),
                    no_install: *no_install,
                    verbose: *verbose,
                    export: *export,
                    format: format.clone(),
                    info: *info,
                    inherit_system: *inherit_system,
                    passenv: passenv.clone(),
                };
                commands::dev::handle(&args).await
            }

            Commands::Setup {
                force,
                dry_run,
                verbose,
                no_parallel,
                no_hooks,
                ci,
            } => {
                commands::setup::handle(
                    ctx.registry(),
                    *force,
                    *dry_run,
                    *verbose,
                    *no_parallel,
                    *no_hooks,
                    *ci,
                )
                .await
            }

            Commands::Add { tool, version } => {
                commands::setup::add_tool(tool, version.as_deref()).await
            }

            Commands::Remove { tool } => commands::setup::remove_tool(tool).await,

            Commands::Run {
                script,
                list,
                script_help,
                args,
            } => commands::run::handle(script.as_deref(), *list, *script_help, args).await,

            Commands::Services { command } => match command {
                ServicesCommand::Start {
                    services,
                    foreground,
                    force,
                    verbose,
                } => {
                    let services = if services.is_empty() {
                        None
                    } else {
                        Some(services.clone())
                    };
                    commands::services::handle_start(services, !*foreground, *force, *verbose).await
                }
                ServicesCommand::Stop { services, verbose } => {
                    let services = if services.is_empty() {
                        None
                    } else {
                        Some(services.clone())
                    };
                    commands::services::handle_stop(services, *verbose).await
                }
                ServicesCommand::Status { verbose } => {
                    commands::services::handle_status(*verbose).await
                }
                ServicesCommand::Logs {
                    service,
                    follow,
                    tail,
                } => commands::services::handle_logs(service, *follow, *tail).await,
                ServicesCommand::Restart { services, verbose } => {
                    let services = if services.is_empty() {
                        None
                    } else {
                        Some(services.clone())
                    };
                    commands::services::handle_restart(services, *verbose).await
                }
            },

            Commands::Hook { command } => match command {
                HookCommand::PreCommit => commands::hook::handle_pre_commit().await,
                HookCommand::Enter => commands::hook::handle_enter().await,
                HookCommand::Install { force } => commands::hook::handle_install(*force).await,
                HookCommand::Uninstall => commands::hook::handle_uninstall().await,
                HookCommand::Status => commands::hook::handle_status().await,
                HookCommand::Run { name } => commands::hook::handle_run(name).await,
                HookCommand::ShellInit { shell } => {
                    commands::hook::handle_shell_init(shell.clone()).await
                }
            },

            Commands::Container { command } => match command {
                ContainerCommand::Generate {
                    output,
                    with_ignore,
                    dry_run,
                    template,
                } => {
                    commands::container::handle_generate(
                        output.clone(),
                        *with_ignore,
                        *dry_run,
                        template.clone(),
                    )
                    .await
                }
                ContainerCommand::Build {
                    tag,
                    target,
                    build_arg,
                    platform,
                    no_cache,
                    push,
                    verbose,
                } => {
                    commands::container::handle_build(
                        tag.clone(),
                        target.clone(),
                        build_arg.clone(),
                        platform.clone(),
                        *no_cache,
                        *push,
                        *verbose,
                    )
                    .await
                }
                ContainerCommand::Push { tag, verbose } => {
                    commands::container::handle_push(tag.clone(), *verbose).await
                }
                ContainerCommand::Status => commands::container::handle_status().await,
                ContainerCommand::Login {
                    registry,
                    username,
                    password,
                } => {
                    commands::container::handle_login(
                        registry.clone(),
                        username.clone(),
                        password.clone(),
                    )
                    .await
                }
                ContainerCommand::Tags { all } => commands::container::handle_tags(*all).await,
            },

            Commands::Ext { command } => match command {
                ExtCommand::List { verbose } => commands::ext::handle_list(*verbose).await,
                ExtCommand::Info { name } => commands::ext::handle_info(name).await,
                ExtCommand::Dev { path, unlink } => commands::ext::handle_dev(path, *unlink).await,
                ExtCommand::Install { source } => commands::ext::handle_install(source).await,
                ExtCommand::Uninstall { name } => commands::ext::handle_uninstall(name).await,
                ExtCommand::Update { name, all } => {
                    commands::ext::handle_update(name.as_deref(), *all).await
                }
                ExtCommand::Check { name, all } => {
                    commands::ext::handle_check(name.as_deref(), *all).await
                }
            },

            Commands::X { extension, args } => commands::ext::handle_execute(extension, args).await,

            Commands::Migrate {
                path,
                dry_run,
                backup,
                check,
                verbose,
            } => commands::migrate::handle(path.clone(), *dry_run, *backup, *check, *verbose).await,

            Commands::Lock {
                update,
                tool,
                dry_run,
                check,
                verbose,
            } => {
                if *check {
                    commands::lock::handle_check(*verbose).await
                } else {
                    commands::lock::handle(
                        ctx.registry(),
                        ctx.runtime_context(),
                        *update,
                        tool.as_deref(),
                        *dry_run,
                        *verbose,
                    )
                    .await
                }
            }

            Commands::Check {
                tool,
                detailed,
                quiet,
            } => commands::check::handle(ctx.registry(), tool.clone(), *detailed, *quiet).await,

            Commands::Bundle { command } => match command {
                BundleCommand::Create { tools, verbose } => {
                    commands::bundle::handle_create(
                        ctx.registry(),
                        ctx.runtime_context(),
                        tools.clone(),
                        *verbose,
                    )
                    .await
                }
                BundleCommand::Update { tools, verbose } => {
                    commands::bundle::handle_update(
                        ctx.registry(),
                        ctx.runtime_context(),
                        tools.clone(),
                        *verbose,
                    )
                    .await
                }
                BundleCommand::Status { verbose } => {
                    commands::bundle::handle_status(*verbose).await
                }
                BundleCommand::Export {
                    output,
                    tools,
                    platforms,
                    verbose,
                } => {
                    commands::bundle::handle_export(
                        output.clone(),
                        tools.clone(),
                        platforms.clone(),
                        *verbose,
                    )
                    .await
                }
                BundleCommand::Import {
                    archive,
                    force,
                    verbose,
                } => commands::bundle::handle_import(archive, *force, *verbose).await,
                BundleCommand::Clean { force } => commands::bundle::handle_clean(*force).await,
            },

            Commands::Info { json, warnings } => {
                if *warnings {
                    commands::capabilities::handle_warnings().await
                } else {
                    commands::capabilities::handle(ctx.registry(), *json).await
                }
            }

            Commands::Metrics {
                last,
                json,
                html,
                clean,
            } => commands::metrics::handle(*last, *json, html.clone(), *clean).await,

            Commands::Auth { command } => match command {
                AuthCommand::Login { service, token } => {
                    handle_auth_login(service, token.as_deref()).await
                }
                AuthCommand::Logout { service } => handle_auth_logout(service).await,
                AuthCommand::Status { service } => handle_auth_status(service).await,
            },

            Commands::Ai { command } => match command {
                AiCommand::Setup {
                    agent,
                    global,
                    force,
                } => commands::ai::handle_setup(agent, *global, *force).await,
                AiCommand::Agents => commands::ai::handle_agents().await,
                AiCommand::Skills { args } => commands::ai::handle_skills(ctx, args).await,
            },
        }
    }
}

// =============================================================================
// Auth Command Handlers
// =============================================================================

async fn handle_auth_login(service: &str, token: Option<&str>) -> Result<()> {
    use crate::commands::auth::{
        GitHubDeviceFlow, TokenSource, get_token_status, store_github_token,
    };

    match service {
        "github" | "gh" => {
            // Check if already authenticated
            let status = get_token_status().await?;
            if !matches!(status.source, TokenSource::None) {
                println!("Already authenticated with GitHub via {}", status.source);
                if let Some(rl) = status.rate_limit {
                    println!("Rate limit: {}/{} remaining", rl.remaining, rl.limit);
                }
                println!(
                    "\nTo re-authenticate, run: vx auth logout github && vx auth login github"
                );
                return Ok(());
            }

            // If token provided directly, store it
            if let Some(t) = token {
                store_github_token(t)?;
                println!("GitHub token stored successfully!");
                println!("\nYou can also set GITHUB_TOKEN or GH_TOKEN environment variable.");
                return Ok(());
            }

            // Use Device Flow
            println!("Authenticating with GitHub using Device Flow...\n");

            let flow = GitHubDeviceFlow::new();
            let device_code = flow.start().await?;

            println!("Please visit: {}", device_code.verification_uri);
            println!("And enter code: {}\n", device_code.user_code);

            // Try to open browser
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", &device_code.verification_uri])
                    .spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg(&device_code.verification_uri)
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&device_code.verification_uri)
                    .spawn();
            }

            println!("Waiting for authorization...");

            let token = flow
                .poll_for_token(
                    &device_code.device_code,
                    device_code.interval,
                    device_code.expires_in,
                )
                .await?;

            store_github_token(&token)?;

            println!("\n✓ Successfully authenticated with GitHub!");
            println!("  Token stored in ~/.vx/config/github_token");
            println!("\nYou can also set GITHUB_TOKEN or GH_TOKEN environment variable for CI.");

            Ok(())
        }
        _ => {
            anyhow::bail!("Unknown service: {}. Supported services: github", service);
        }
    }
}

async fn handle_auth_logout(service: &str) -> Result<()> {
    use crate::commands::auth::remove_github_token;

    match service {
        "github" | "gh" => {
            remove_github_token()?;
            println!("✓ Logged out from GitHub");
            println!("\nNote: This only removes the stored token.");
            println!("Environment variables (GITHUB_TOKEN, GH_TOKEN) are not affected.");
            Ok(())
        }
        _ => {
            anyhow::bail!("Unknown service: {}. Supported services: github", service);
        }
    }
}

async fn handle_auth_status(service: &str) -> Result<()> {
    use crate::commands::auth::{TokenSource, get_token_status};

    match service {
        "github" | "gh" | "all" => {
            println!("GitHub Authentication Status\n");

            let status = get_token_status().await?;

            match &status.source {
                TokenSource::None => {
                    println!("  Status: Not authenticated");
                    println!("\n  To authenticate:");
                    println!("    vx auth login github");
                    println!("\n  Or set environment variable:");
                    println!("    export GITHUB_TOKEN=<your-token>");
                }
                source => {
                    println!("  Status: ✓ Authenticated");
                    println!("  Source: {}", source);

                    if !status.scopes.is_empty() {
                        println!("  Scopes: {}", status.scopes.join(", "));
                    }

                    if let Some(rl) = status.rate_limit {
                        println!("\n  Rate Limit:");
                        println!("    Remaining: {}/{}", rl.remaining, rl.limit);

                        // Calculate reset time
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        if rl.reset > now {
                            let mins = (rl.reset - now) / 60;
                            println!("    Resets in: {} minutes", mins);
                        }
                    }
                }
            }

            Ok(())
        }
        _ => {
            anyhow::bail!("Unknown service: {}. Supported services: github", service);
        }
    }
}
