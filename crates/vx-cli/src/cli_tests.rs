//! Tests for CLI argument parsing and command routing

use crate::cli::*;
use clap::Parser;

#[test]
fn test_cli_version_command() {
    let args = vec!["vx", "version"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(!cli.verbose);
    assert!(!cli.use_system_path);
    assert!(matches!(cli.command, Some(Commands::Version)));
}

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
fn test_cli_global_flags() {
    let args = vec!["vx", "--verbose", "--use-system-path", "version"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(cli.verbose);
    assert!(cli.use_system_path);
    assert!(matches!(cli.command, Some(Commands::Version)));
}

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
fn test_output_format_enum() {
    // Test that OutputFormat enum works correctly
    assert!(matches!(OutputFormat::Table, OutputFormat::Table));
    assert!(matches!(OutputFormat::Json, OutputFormat::Json));
    assert!(matches!(OutputFormat::Yaml, OutputFormat::Yaml));
}

#[test]
fn test_cli_help() {
    // Test that help can be generated without panicking
    let result = Cli::try_parse_from(vec!["vx", "--help"]);
    assert!(result.is_err()); // Help exits with error code
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
