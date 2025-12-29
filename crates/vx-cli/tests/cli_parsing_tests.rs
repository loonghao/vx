//! Tests for CLI argument parsing and command routing
//!
//! These tests verify that CLI arguments are correctly parsed into commands.

use clap::Parser;
use vx_cli::cli::*;

// ============================================
// Basic Command Tests
// ============================================

#[test]
fn test_cli_version_command() {
    let args = vec!["vx", "version"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(!cli.verbose);
    assert!(!cli.use_system_path);
    assert!(matches!(cli.command, Some(Commands::Version)));
}

#[test]
fn test_cli_stats_command() {
    let args = vec!["vx", "stats"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::Stats)));
}

// ============================================
// List Command Tests
// ============================================

#[test]
fn test_cli_list_command() {
    let args = vec!["vx", "list"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::List {
            tool,
            status,
            installed,
            available,
            all,
        }) => {
            assert!(tool.is_none());
            assert!(!status);
            assert!(!installed);
            assert!(!available);
            assert!(!all);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_cli_list_alias() {
    let args = vec!["vx", "ls"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::List { .. })));
}

#[test]
fn test_cli_list_with_options() {
    let args = vec!["vx", "list", "--status", "--installed", "node"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::List {
            tool,
            status,
            installed,
            available,
            all,
        }) => {
            assert_eq!(tool, Some("node".to_string()));
            assert!(status);
            assert!(installed);
            assert!(!available);
            assert!(!all);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_cli_list_all_flag() {
    let args = vec!["vx", "list", "-a"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::List { all, .. }) => {
            assert!(all);
        }
        _ => panic!("Expected List command"),
    }
}

// ============================================
// Install Command Tests
// ============================================

#[test]
fn test_cli_install_command() {
    let args = vec!["vx", "install", "node", "18.0.0"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Install {
            tool,
            version,
            force,
        }) => {
            assert_eq!(tool, "node");
            assert_eq!(version, Some("18.0.0".to_string()));
            assert!(!force);
        }
        _ => panic!("Expected Install command"),
    }
}

#[test]
fn test_cli_install_alias() {
    let args = vec!["vx", "i", "node"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::Install { .. })));
}

#[test]
fn test_cli_install_with_force() {
    let args = vec!["vx", "install", "node", "--force"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Install {
            tool,
            version,
            force,
        }) => {
            assert_eq!(tool, "node");
            assert!(version.is_none());
            assert!(force);
        }
        _ => panic!("Expected Install command"),
    }
}

#[test]
fn test_cli_install_force_short() {
    let args = vec!["vx", "install", "go", "-f"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Install { force, .. }) => {
            assert!(force);
        }
        _ => panic!("Expected Install command"),
    }
}

// ============================================
// Uninstall Command Tests
// ============================================

#[test]
fn test_cli_uninstall_command() {
    let args = vec!["vx", "uninstall", "node", "18.0.0"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Uninstall {
            tool,
            version,
            force,
        }) => {
            assert_eq!(tool, "node");
            assert_eq!(version, Some("18.0.0".to_string()));
            assert!(!force);
        }
        _ => panic!("Expected Uninstall command"),
    }
}

#[test]
fn test_cli_uninstall_alias_rm() {
    let args = vec!["vx", "rm", "node"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::Uninstall { .. })));
}

#[test]
fn test_cli_uninstall_alias_remove() {
    let args = vec!["vx", "remove", "node"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::Uninstall { .. })));
}

// ============================================
// Update Command Tests
// ============================================

#[test]
fn test_cli_update_command() {
    let args = vec!["vx", "update"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Update { tool, apply }) => {
            assert!(tool.is_none());
            assert!(!apply);
        }
        _ => panic!("Expected Update command"),
    }
}

#[test]
fn test_cli_update_alias() {
    let args = vec!["vx", "up", "node"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Update { tool, .. }) => {
            assert_eq!(tool, Some("node".to_string()));
        }
        _ => panic!("Expected Update command"),
    }
}

#[test]
fn test_cli_update_with_apply() {
    let args = vec!["vx", "update", "--apply"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Update { apply, .. }) => {
            assert!(apply);
        }
        _ => panic!("Expected Update command"),
    }
}

// ============================================
// Self-Update Command Tests
// ============================================

#[test]
fn test_cli_self_update_command() {
    let args = vec!["vx", "self-update"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::SelfUpdate {
            check,
            version,
            token,
            prerelease,
            force,
        }) => {
            assert!(!check);
            assert!(version.is_none());
            assert!(token.is_none());
            assert!(!prerelease);
            assert!(!force);
        }
        _ => panic!("Expected SelfUpdate command"),
    }
}

#[test]
fn test_cli_self_update_check() {
    let args = vec!["vx", "self-update", "--check"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::SelfUpdate { check, .. }) => {
            assert!(check);
        }
        _ => panic!("Expected SelfUpdate command"),
    }
}

#[test]
fn test_cli_self_update_with_version() {
    let args = vec!["vx", "self-update", "0.5.0"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::SelfUpdate { version, .. }) => {
            assert_eq!(version, Some("0.5.0".to_string()));
        }
        _ => panic!("Expected SelfUpdate command"),
    }
}

// ============================================
// Which Command Tests
// ============================================

#[test]
fn test_cli_which_command() {
    let args = vec!["vx", "which", "node", "--all"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Which { tool, all }) => {
            assert_eq!(tool, "node");
            assert!(all);
        }
        _ => panic!("Expected Which command"),
    }
}

#[test]
fn test_cli_which_alias() {
    let args = vec!["vx", "where", "python"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::Which { .. })));
}

// ============================================
// Versions Command Tests
// ============================================

#[test]
fn test_cli_versions_command() {
    let args = vec!["vx", "versions", "node", "--latest", "5", "--prerelease"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Versions {
            tool,
            latest,
            prerelease,
            detailed,
            interactive,
        }) => {
            assert_eq!(tool, "node");
            assert_eq!(latest, Some(5));
            assert!(prerelease);
            assert!(!detailed);
            assert!(!interactive);
        }
        _ => panic!("Expected Versions command"),
    }
}

#[test]
fn test_cli_versions_interactive() {
    let args = vec!["vx", "versions", "go", "-i"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Versions { interactive, .. }) => {
            assert!(interactive);
        }
        _ => panic!("Expected Versions command"),
    }
}

// ============================================
// Switch Command Tests
// ============================================

#[test]
fn test_cli_switch_command() {
    let args = vec!["vx", "switch", "node@18.0.0", "--global"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Switch {
            tool_version,
            global,
        }) => {
            assert_eq!(tool_version, "node@18.0.0");
            assert!(global);
        }
        _ => panic!("Expected Switch command"),
    }
}

#[test]
fn test_cli_switch_local() {
    let args = vec!["vx", "switch", "python@3.11"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Switch { global, .. }) => {
            assert!(!global);
        }
        _ => panic!("Expected Switch command"),
    }
}

// ============================================
// Search Command Tests
// ============================================

#[test]
fn test_cli_search_command() {
    let args = vec!["vx", "search", "python", "--category", "language"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Search {
            query,
            category,
            installed_only,
            available_only,
            format,
            verbose,
        }) => {
            assert_eq!(query, Some("python".to_string()));
            assert_eq!(category, Some("language".to_string()));
            assert!(!installed_only);
            assert!(!available_only);
            assert!(matches!(format, OutputFormat::Table));
            assert!(!verbose);
        }
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_cli_search_json_format() {
    let args = vec!["vx", "search", "--format", "json"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Search { format, .. }) => {
            assert!(matches!(format, OutputFormat::Json));
        }
        _ => panic!("Expected Search command"),
    }
}

// ============================================
// Sync Command Tests
// ============================================

#[test]
fn test_cli_sync_command() {
    let args = vec!["vx", "sync", "--dry-run", "--verbose"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Sync {
            check,
            force,
            dry_run,
            verbose,
            no_parallel,
            no_auto_install,
        }) => {
            assert!(!check);
            assert!(!force);
            assert!(dry_run);
            assert!(verbose);
            assert!(!no_parallel);
            assert!(!no_auto_install);
        }
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_cli_sync_check_only() {
    let args = vec!["vx", "sync", "--check"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Sync { check, .. }) => {
            assert!(check);
        }
        _ => panic!("Expected Sync command"),
    }
}

// ============================================
// Init Command Tests
// ============================================

#[test]
fn test_cli_init_command() {
    let args = vec!["vx", "init", "--interactive", "--template", "node"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Init {
            interactive,
            template,
            tools,
            force,
            dry_run,
            list_templates,
        }) => {
            assert!(interactive);
            assert_eq!(template, Some("node".to_string()));
            assert!(tools.is_none());
            assert!(!force);
            assert!(!dry_run);
            assert!(!list_templates);
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_cli_init_list_templates() {
    let args = vec!["vx", "init", "--list-templates"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Init { list_templates, .. }) => {
            assert!(list_templates);
        }
        _ => panic!("Expected Init command"),
    }
}

// ============================================
// Clean Command Tests
// ============================================

#[test]
fn test_cli_clean_command() {
    let args = vec!["vx", "clean", "--cache", "--dry-run"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Clean {
            dry_run,
            cache,
            orphaned,
            all,
            force,
            older_than,
            verbose,
        }) => {
            assert!(dry_run);
            assert!(cache);
            assert!(!orphaned);
            assert!(!all);
            assert!(!force);
            assert!(older_than.is_none());
            assert!(!verbose);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_cli_clean_all() {
    let args = vec!["vx", "clean", "-a", "-f"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Clean { all, force, .. }) => {
            assert!(all);
            assert!(force);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_cli_clean_older_than() {
    let args = vec!["vx", "clean", "--older-than", "30"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Clean { older_than, .. }) => {
            assert_eq!(older_than, Some(30));
        }
        _ => panic!("Expected Clean command"),
    }
}

// ============================================
// Setup Command Tests
// ============================================

#[test]
fn test_cli_setup_command() {
    let args = vec!["vx", "setup"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Setup {
            force,
            dry_run,
            verbose,
            no_parallel,
            no_hooks,
            ci,
        }) => {
            assert!(!force);
            assert!(!dry_run);
            assert!(!verbose);
            assert!(!no_parallel);
            assert!(!no_hooks);
            assert!(!ci);
        }
        _ => panic!("Expected Setup command"),
    }
}

#[test]
fn test_cli_setup_with_options() {
    let args = vec!["vx", "setup", "--force", "--dry-run", "--no-hooks"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Setup {
            force,
            dry_run,
            no_hooks,
            ..
        }) => {
            assert!(force);
            assert!(dry_run);
            assert!(no_hooks);
        }
        _ => panic!("Expected Setup command"),
    }
}

// ============================================
// Dev Command Tests
// ============================================

#[test]
fn test_cli_dev_command() {
    let args = vec!["vx", "dev"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Dev {
            shell,
            command,
            no_install,
            verbose,
            export,
            format,
        }) => {
            assert!(shell.is_none());
            assert!(command.is_none());
            assert!(!no_install);
            assert!(!verbose);
            assert!(!export);
            assert!(format.is_none());
        }
        _ => panic!("Expected Dev command"),
    }
}

#[test]
fn test_cli_dev_with_shell() {
    let args = vec!["vx", "dev", "--shell", "bash"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Dev { shell, .. }) => {
            assert_eq!(shell, Some("bash".to_string()));
        }
        _ => panic!("Expected Dev command"),
    }
}

#[test]
fn test_cli_dev_export() {
    let args = vec!["vx", "dev", "--export", "--format", "powershell"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Dev { export, format, .. }) => {
            assert!(export);
            assert_eq!(format, Some("powershell".to_string()));
        }
        _ => panic!("Expected Dev command"),
    }
}

#[test]
fn test_cli_dev_with_command() {
    let args = vec!["vx", "dev", "-c", "npm", "run", "build"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Dev { command, .. }) => {
            assert_eq!(
                command,
                Some(vec![
                    "npm".to_string(),
                    "run".to_string(),
                    "build".to_string()
                ])
            );
        }
        _ => panic!("Expected Dev command"),
    }
}

// ============================================
// Add/RemoveTool Command Tests
// ============================================

#[test]
fn test_cli_add_command() {
    let args = vec!["vx", "add", "node", "--version", "18.0.0"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Add { tool, version }) => {
            assert_eq!(tool, "node");
            assert_eq!(version, Some("18.0.0".to_string()));
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_cli_add_without_version() {
    let args = vec!["vx", "add", "python"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Add { tool, version }) => {
            assert_eq!(tool, "python");
            assert!(version.is_none());
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_cli_rm_tool_command() {
    let args = vec!["vx", "rm-tool", "node"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::RemoveTool { tool }) => {
            assert_eq!(tool, "node");
        }
        _ => panic!("Expected RemoveTool command"),
    }
}

// ============================================
// Run Command Tests
// ============================================

#[test]
fn test_cli_run_command() {
    let args = vec!["vx", "run", "build"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Run { script, args }) => {
            assert_eq!(script, "build");
            assert!(args.is_empty());
        }
        _ => panic!("Expected Run command"),
    }
}

#[test]
fn test_cli_run_with_args() {
    // Note: trailing_var_arg requires -- to pass flags as arguments
    let args = vec!["vx", "run", "test", "--", "--coverage"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Run { script, args }) => {
            assert_eq!(script, "test");
            assert_eq!(args, vec!["--coverage"]);
        }
        _ => panic!("Expected Run command"),
    }
}

#[test]
fn test_cli_run_with_positional_args() {
    let args = vec!["vx", "run", "build", "arg1", "arg2"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Run { script, args }) => {
            assert_eq!(script, "build");
            assert_eq!(args, vec!["arg1", "arg2"]);
        }
        _ => panic!("Expected Run command"),
    }
}

// ============================================
// Global Flags Tests
// ============================================

#[test]
fn test_cli_global_flags() {
    let args = vec!["vx", "--verbose", "--use-system-path", "version"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.verbose);
    assert!(cli.use_system_path);
    assert!(matches!(cli.command, Some(Commands::Version)));
}

#[test]
fn test_cli_debug_flag() {
    let args = vec!["vx", "--debug", "version"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.debug);
}

#[test]
fn test_cli_verbose_short() {
    let args = vec!["vx", "-v", "list"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.verbose);
}

// ============================================
// Dynamic Command Execution Tests
// ============================================

#[test]
fn test_cli_dynamic_command_execution() {
    let args = vec!["vx", "node", "--version"];
    let cli = Cli::try_parse_from(args).unwrap();

    // When no subcommand is specified, args should contain the tool and its arguments
    assert!(cli.command.is_none());
    assert_eq!(cli.args, vec!["node", "--version"]);
}

#[test]
fn test_cli_dynamic_command_with_multiple_args() {
    let args = vec!["vx", "uv", "pip", "install", "requests"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.command.is_none());
    assert_eq!(cli.args, vec!["uv", "pip", "install", "requests"]);
}

#[test]
fn test_cli_dynamic_command_with_flags() {
    let args = vec!["vx", "--verbose", "cargo", "build", "--release"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.verbose);
    assert!(cli.command.is_none());
    assert_eq!(cli.args, vec!["cargo", "build", "--release"]);
}

#[test]
fn test_cli_invalid_command() {
    // Test invalid command handling
    let result = Cli::try_parse_from(vec!["vx", "invalid-command-xyz"]);
    // This should be parsed as dynamic command execution
    if let Ok(cli) = result {
        assert!(cli.command.is_none());
        assert_eq!(cli.args, vec!["invalid-command-xyz"]);
    }
}

// ============================================
// OutputFormat Tests
// ============================================

#[test]
fn test_output_format_enum() {
    // Test that OutputFormat enum works correctly
    assert!(matches!(OutputFormat::Table, OutputFormat::Table));
    assert!(matches!(OutputFormat::Json, OutputFormat::Json));
    assert!(matches!(OutputFormat::Yaml, OutputFormat::Yaml));
}

// ============================================
// Help Tests
// ============================================

#[test]
fn test_cli_help() {
    // Test that help can be generated without panicking
    let result = Cli::try_parse_from(vec!["vx", "--help"]);
    assert!(result.is_err()); // Help exits with error code
}

// ============================================
// Config Subcommand Tests
// ============================================

#[test]
fn test_cli_config_show() {
    let args = vec!["vx", "config", "show"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Config {
            command: Some(ConfigCommand::Show),
        }) => {}
        _ => panic!("Expected Config Show command"),
    }
}

#[test]
fn test_cli_config_set() {
    let args = vec!["vx", "config", "set", "defaults.auto_install", "true"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Config {
            command: Some(ConfigCommand::Set { key, value }),
        }) => {
            assert_eq!(key, "defaults.auto_install");
            assert_eq!(value, "true");
        }
        _ => panic!("Expected Config Set command"),
    }
}

#[test]
fn test_cli_config_alias() {
    let args = vec!["vx", "cfg", "show"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::Config { .. })));
}

// ============================================
// Services Subcommand Tests
// ============================================

#[test]
fn test_cli_services_start() {
    let args = vec![
        "vx",
        "services",
        "start",
        "redis",
        "postgres",
        "--foreground",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Services {
            command:
                ServicesCommand::Start {
                    services,
                    foreground,
                    ..
                },
        }) => {
            assert_eq!(services, vec!["redis", "postgres"]);
            assert!(foreground);
        }
        _ => panic!("Expected Services Start command"),
    }
}

#[test]
fn test_cli_services_stop() {
    let args = vec!["vx", "services", "stop"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Services {
            command: ServicesCommand::Stop { services, .. },
        }) => {
            assert!(services.is_empty());
        }
        _ => panic!("Expected Services Stop command"),
    }
}

#[test]
fn test_cli_services_logs() {
    let args = vec!["vx", "services", "logs", "redis", "-f", "--tail", "100"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Services {
            command:
                ServicesCommand::Logs {
                    service,
                    follow,
                    tail,
                },
        }) => {
            assert_eq!(service, "redis");
            assert!(follow);
            assert_eq!(tail, Some(100));
        }
        _ => panic!("Expected Services Logs command"),
    }
}

// ============================================
// Shell Subcommand Tests
// ============================================

#[test]
fn test_cli_shell_init() {
    let args = vec!["vx", "shell", "init", "bash"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Shell {
            command: ShellCommand::Init { shell },
        }) => {
            assert_eq!(shell, Some("bash".to_string()));
        }
        _ => panic!("Expected Shell Init command"),
    }
}

#[test]
fn test_cli_shell_completions() {
    let args = vec!["vx", "shell", "completions", "zsh"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Shell {
            command: ShellCommand::Completions { shell },
        }) => {
            assert_eq!(shell, "zsh");
        }
        _ => panic!("Expected Shell Completions command"),
    }
}

// ============================================
// Hook Subcommand Tests
// ============================================

#[test]
fn test_cli_hook_pre_commit() {
    let args = vec!["vx", "hook", "pre-commit"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Hook {
            command: HookCommand::PreCommit,
        }) => {}
        _ => panic!("Expected Hook PreCommit command"),
    }
}

#[test]
fn test_cli_hook_install() {
    let args = vec!["vx", "hook", "install", "--force"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Hook {
            command: HookCommand::Install { force },
        }) => {
            assert!(force);
        }
        _ => panic!("Expected Hook Install command"),
    }
}

// ============================================
// Extension Subcommand Tests
// ============================================

#[test]
fn test_cli_ext_list() {
    let args = vec!["vx", "ext", "list"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Ext {
            command: ExtCommand::List { .. },
        }) => {}
        _ => panic!("Expected Ext List command"),
    }
}

#[test]
fn test_cli_ext_alias() {
    let args = vec!["vx", "extension", "list"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Some(Commands::Ext { .. })));
}

#[test]
fn test_cli_x_command() {
    let args = vec!["vx", "x", "my-ext", "arg1", "arg2"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::X { extension, args }) => {
            assert_eq!(extension, "my-ext");
            assert_eq!(args, vec!["arg1", "arg2"]);
        }
        _ => panic!("Expected X command"),
    }
}

// ============================================
// Migrate Command Tests
// ============================================

#[test]
fn test_cli_migrate_check() {
    let args = vec!["vx", "migrate", "--check"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Migrate { check, dry_run, .. }) => {
            assert!(check);
            assert!(!dry_run);
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_cli_migrate_dry_run() {
    let args = vec!["vx", "migrate", "--dry-run"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Migrate { dry_run, check, .. }) => {
            assert!(dry_run);
            assert!(!check);
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_cli_migrate_with_path() {
    let args = vec!["vx", "migrate", "--path", "/some/path", "--verbose"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Migrate { path, verbose, .. }) => {
            assert_eq!(path, Some("/some/path".to_string()));
            assert!(verbose);
        }
        _ => panic!("Expected Migrate command"),
    }
}

// ============================================
// Container Subcommand Tests
// ============================================

#[test]
fn test_cli_container_generate() {
    let args = vec![
        "vx",
        "container",
        "generate",
        "--template",
        "node",
        "--with-ignore",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Container {
            command:
                ContainerCommand::Generate {
                    template,
                    with_ignore,
                    ..
                },
        }) => {
            assert_eq!(template, Some("node".to_string()));
            assert!(with_ignore);
        }
        _ => panic!("Expected Container Generate command"),
    }
}

#[test]
fn test_cli_container_build() {
    let args = vec![
        "vx",
        "container",
        "build",
        "--tag",
        "myapp:latest",
        "--no-cache",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Container {
            command: ContainerCommand::Build { tag, no_cache, .. },
        }) => {
            assert_eq!(tag, vec!["myapp:latest"]);
            assert!(no_cache);
        }
        _ => panic!("Expected Container Build command"),
    }
}

// ============================================
// Plugin Subcommand Tests
// ============================================

#[test]
fn test_cli_plugin_list() {
    let args = vec!["vx", "plugin", "list", "--enabled"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Plugin {
            command: PluginCommand::List { enabled, .. },
        }) => {
            assert!(enabled);
        }
        _ => panic!("Expected Plugin List command"),
    }
}

#[test]
fn test_cli_plugin_enable() {
    let args = vec!["vx", "plugin", "enable", "my-plugin"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Plugin {
            command: PluginCommand::Enable { name },
        }) => {
            assert_eq!(name, "my-plugin");
        }
        _ => panic!("Expected Plugin Enable command"),
    }
}
