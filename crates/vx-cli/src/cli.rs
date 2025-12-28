// CLI module - modular command structure
// Each command is implemented in its own module for better maintainability

use crate::commands::{
    env::EnvCommand, global::GlobalCommand, venv_cmd::VenvCommand, CommandContext, CommandHandler,
    GlobalOptions,
};
use anyhow::Result;
use async_trait::async_trait;
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

    /// Enable debug output (equivalent to RUST_LOG=debug)
    #[arg(long, global = true)]
    pub debug: bool,

    /// Tool and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

/// Convert Cli to GlobalOptions
///
/// This allows easy extraction of global options from CLI args.
/// When adding new global options:
/// 1. Add field to Cli struct
/// 2. Add field to GlobalOptions struct
/// 3. Update this implementation
impl From<&Cli> for GlobalOptions {
    fn from(cli: &Cli) -> Self {
        GlobalOptions {
            use_system_path: cli.use_system_path,
            verbose: cli.verbose,
            debug: cli.debug,
        }
    }
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Show version information
    Version,

    /// Analyze project dependencies, scripts, and tools
    Analyze {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

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
        /// Show all tools including those not supported on current platform
        #[arg(long, short = 'a')]
        all: bool,
    },

    /// Install a specific tool version
    #[command(alias = "i")]
    Install {
        /// Tool name (e.g., uv, node, go, rust)
        tool: String,
        /// Version to install (e.g., 1.0.0, latest, lts)
        version: Option<String>,
        /// Force reinstallation even if already installed
        #[arg(short, long)]
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

    /// Update vx itself to the latest version
    #[command(name = "self-update")]
    SelfUpdate {
        /// Only check for updates, don't install
        #[arg(long)]
        check: bool,
        /// Specific version to install
        version: Option<String>,
        /// GitHub token for authenticated API requests (avoids rate limits)
        #[arg(long)]
        token: Option<String>,
        /// Include pre-release versions
        #[arg(long)]
        prerelease: bool,
        /// Force update even if already up to date
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall tool versions (preferred over remove)
    #[command(alias = "rm", alias = "remove")]
    Uninstall {
        /// Tool name
        tool: String,
        /// Version to uninstall (optional, removes all if not specified)
        version: Option<String>,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Show which tool version is being used (preferred over where)
    #[command(alias = "where")]
    Which {
        /// Tool name
        tool: String,
        /// Show all installed versions
        #[arg(short, long)]
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
    },

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

    /// Clean up system (preferred over cleanup)
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
        #[arg(short, long)]
        all: bool,
        /// Force cleanup without confirmation
        #[arg(short, long)]
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

    /// Environment management
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },

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
        /// Export environment variables for shell activation instead of spawning a shell
        /// Usage: eval "$(vx dev --export)" (bash/zsh)
        ///        Invoke-Expression (vx dev --export --format powershell)
        #[arg(long, short = 'e')]
        export: bool,
        /// Output format for --export: shell, powershell, batch, github
        #[arg(long, short = 'f')]
        format: Option<String>,
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
    },

    /// Add a tool to project configuration
    Add {
        /// Tool name (e.g., node, python, uv)
        tool: String,
        /// Version to use (default: latest)
        #[arg(long)]
        version: Option<String>,
    },

    /// Remove a tool from project configuration
    #[command(name = "rm-tool")]
    RemoveTool {
        /// Tool name to remove
        tool: String,
    },

    /// Run a script defined in .vx.toml
    Run {
        /// Script name
        script: String,
        /// Additional arguments to pass to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Manage development services (Docker/Podman)
    Services {
        #[command(subcommand)]
        command: ServicesCommand,
    },

    /// Execute or manage lifecycle hooks
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },

    /// Container and Dockerfile management
    Container {
        #[command(subcommand)]
        command: ContainerCommand,
    },

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
    /// Migrate .vx.toml to v2 format
    Migrate {
        /// Path to .vx.toml file (default: current directory)
        #[arg(short, long)]
        path: Option<String>,
        /// Preview changes without writing
        #[arg(long)]
        dry_run: bool,
        /// Create backup before migration (use --no-backup to disable)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        backup: bool,
        /// Force migration even if already v2
        #[arg(short, long)]
        force: bool,
    },
    /// Validate .vx.toml configuration
    Validate {
        /// Path to .vx.toml file (default: current directory)
        #[arg(short, long)]
        path: Option<String>,
        /// Show verbose validation output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate JSON Schema for .vx.toml
    Schema {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
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
    /// Install an extension from a remote source (future)
    Install {
        /// Extension source (e.g., github:user/repo)
        source: String,
    },
    /// Uninstall an extension
    Uninstall {
        /// Extension name
        name: String,
    },
}

// =============================================================================
// CommandHandler Implementation for Commands
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
            Commands::Update { .. } => "update",
            Commands::SelfUpdate { .. } => "self-update",
            Commands::Uninstall { .. } => "uninstall",
            Commands::Which { .. } => "which",
            Commands::Versions { .. } => "versions",
            Commands::Switch { .. } => "switch",
            Commands::Config { .. } => "config",
            Commands::Search { .. } => "search",
            Commands::Sync { .. } => "sync",
            Commands::Init { .. } => "init",
            Commands::Clean { .. } => "clean",
            Commands::Stats => "stats",
            Commands::Plugin { .. } => "plugin",
            Commands::Shell { .. } => "shell",
            Commands::Venv { .. } => "venv",
            Commands::Global { .. } => "global",
            Commands::Env { .. } => "env",
            Commands::Dev { .. } => "dev",
            Commands::Setup { .. } => "setup",
            Commands::Add { .. } => "add",
            Commands::RemoveTool { .. } => "rm-tool",
            Commands::Run { .. } => "run",
            Commands::Services { .. } => "services",
            Commands::Hook { .. } => "hook",
            Commands::Container { .. } => "container",
            Commands::Ext { .. } => "ext",
            Commands::X { .. } => "x",
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
            } => {
                commands::list::handle(
                    ctx.registry(),
                    ctx.runtime_context(),
                    tool.as_deref(),
                    *status,
                    *all,
                )
                .await
            }

            Commands::Install {
                tool,
                version,
                force,
            } => {
                commands::install::handle(
                    ctx.registry(),
                    ctx.runtime_context(),
                    tool,
                    version.as_deref(),
                    *force,
                )
                .await
            }

            Commands::Update { tool, apply } => {
                commands::update::handle(
                    ctx.registry(),
                    ctx.runtime_context(),
                    tool.as_deref(),
                    *apply,
                )
                .await
            }

            Commands::SelfUpdate {
                check,
                version: _,
                token,
                prerelease,
                force,
            } => commands::self_update::handle(token.as_deref(), *prerelease, *force, *check).await,

            Commands::Uninstall {
                tool,
                version,
                force,
            } => {
                commands::remove::handle(
                    ctx.registry(),
                    ctx.runtime_context(),
                    tool,
                    version.as_deref(),
                    *force,
                )
                .await
            }

            Commands::Which { tool, all } => {
                commands::where_cmd::handle(ctx.registry(), tool, *all, ctx.use_system_path()).await
            }

            Commands::Versions {
                tool,
                latest,
                prerelease,
                detailed,
                interactive,
            } => {
                commands::fetch::handle(
                    ctx.registry(),
                    ctx.runtime_context(),
                    tool,
                    *latest,
                    *detailed,
                    *interactive,
                    *prerelease,
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
                Some(ConfigCommand::Migrate {
                    path,
                    dry_run,
                    backup,
                    force,
                }) => {
                    commands::config::handle_migrate(path.clone(), *dry_run, *backup, *force).await
                }
                Some(ConfigCommand::Validate { path, verbose }) => {
                    commands::config::handle_validate(path.clone(), *verbose).await
                }
                Some(ConfigCommand::Schema { output }) => {
                    commands::config::handle_schema(output.clone()).await
                }
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

            Commands::Clean {
                dry_run,
                cache,
                orphaned,
                all,
                force,
                older_than,
                verbose,
            } => {
                let cache_only = *cache && !*all;
                let orphaned_only = *orphaned && !*all;
                commands::cleanup::handle(
                    *dry_run,
                    cache_only,
                    orphaned_only,
                    *force,
                    *older_than,
                    *verbose,
                )
                .await
            }

            Commands::Stats => commands::stats::handle(ctx.registry()).await,

            Commands::Plugin { command } => {
                commands::plugin::handle(ctx.registry(), command.clone()).await
            }

            Commands::Venv { command } => commands::venv_cmd::handle(command.clone()).await,

            Commands::Global { command } => commands::global::handle(command.clone()).await,

            Commands::Env { command } => commands::env::handle(command.clone()).await,

            Commands::Search {
                query,
                category,
                installed_only,
                available_only,
                format,
                verbose,
            } => {
                commands::search::handle(
                    ctx.registry(),
                    query.clone(),
                    category.clone(),
                    *installed_only,
                    *available_only,
                    format.clone(),
                    *verbose,
                )
                .await
            }

            Commands::Sync {
                check,
                force,
                dry_run,
                verbose,
                no_parallel,
                no_auto_install,
            } => {
                commands::sync::handle(
                    ctx.registry(),
                    *check,
                    *force,
                    *dry_run,
                    *verbose,
                    *no_parallel,
                    *no_auto_install,
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
            } => {
                commands::dev::handle(
                    shell.clone(),
                    command.clone(),
                    *no_install,
                    *verbose,
                    *export,
                    format.clone(),
                )
                .await
            }

            Commands::Setup {
                force,
                dry_run,
                verbose,
                no_parallel,
                no_hooks,
            } => {
                commands::setup::handle(
                    ctx.registry(),
                    *force,
                    *dry_run,
                    *verbose,
                    *no_parallel,
                    *no_hooks,
                )
                .await
            }

            Commands::Add { tool, version } => {
                commands::setup::add_tool(tool, version.as_deref()).await
            }

            Commands::RemoveTool { tool } => commands::setup::remove_tool(tool).await,

            Commands::Run { script, args } => commands::run::handle(script, args).await,

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
            },

            Commands::X { extension, args } => commands::ext::handle_execute(extension, args).await,
        }
    }
}
