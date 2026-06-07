//! Tests for `vx ai headroom` CLI parsing and subcommand routing.

use clap::Parser;
use vx_cli::cli::*;

// ============================================
// Headroom root command
// ============================================

#[test]
fn test_headroom_root_parses() {
    // Headroom requires a subcommand; parse with one
    let args = vec!["vx", "ai", "headroom", "doctor"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(matches!(cli.command, Some(Commands::Ai { .. })));
}

// ============================================
// headroom install
// ============================================

#[test]
fn test_headroom_install_defaults() {
    let args = vec!["vx", "ai", "headroom", "install"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Install {
                version,
                python,
                mcpcall_version,
                force,
            }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(version, "latest");
    assert_eq!(python, "3.11");
    assert_eq!(mcpcall_version, "0.4.0");
    assert!(!force);
}

#[test]
fn test_headroom_install_with_flags() {
    let args = vec![
        "vx",
        "ai",
        "headroom",
        "install",
        "--version",
        "0.5.0",
        "--python",
        "3.12",
        "--mcpcall-version",
        "0.5.0",
        "--force",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Install {
                version,
                python,
                mcpcall_version,
                force,
            }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(version, "0.5.0");
    assert_eq!(python, "3.12");
    assert_eq!(mcpcall_version, "0.5.0");
    assert!(force);
}

#[test]
fn test_headroom_install_help() {
    // clap Err(DisplayHelp) for --help; verify command routing without --help flag
    let args = vec!["vx", "ai", "headroom", "install", "--force"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(matches!(cli.command, Some(Commands::Ai { .. })));
}

// ============================================
// headroom doctor
// ============================================

#[test]
fn test_headroom_doctor_defaults() {
    let args = vec!["vx", "ai", "headroom", "doctor"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Doctor {
                quick,
                json,
                port,
                mcp_port,
            }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(!quick);
    assert!(!json);
    assert_eq!(port, 8787);
    assert_eq!(mcp_port, 8765);
}

#[test]
fn test_headroom_doctor_quick() {
    let args = vec!["vx", "ai", "headroom", "doctor", "--quick"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command: AiCommand::Headroom(HeadroomCommand::Doctor { quick, .. }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(quick);
}

#[test]
fn test_headroom_doctor_json() {
    let args = vec!["vx", "ai", "headroom", "doctor", "--json"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command: AiCommand::Headroom(HeadroomCommand::Doctor { json, .. }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(json);
}

#[test]
fn test_headroom_doctor_json_quick() {
    let args = vec!["vx", "ai", "headroom", "doctor", "--json", "--quick"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command: AiCommand::Headroom(HeadroomCommand::Doctor { quick, json, .. }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(quick);
    assert!(json);
}

#[test]
fn test_headroom_doctor_custom_ports() {
    let args = vec![
        "vx",
        "ai",
        "headroom",
        "doctor",
        "--port",
        "9999",
        "--mcp-port",
        "8888",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command: AiCommand::Headroom(HeadroomCommand::Doctor { port, mcp_port, .. }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(port, 9999);
    assert_eq!(mcp_port, 8888);
}

// ============================================
// headroom setup
// ============================================

#[test]
fn test_headroom_setup_defaults() {
    let args = vec!["vx", "ai", "headroom", "setup"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Setup {
                agent,
                dry_run,
                apply,
                port,
                mcp_port,
                headroom_version,
            }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(agent.is_empty());
    assert!(!dry_run);
    assert!(!apply);
    assert_eq!(port, 8787);
    assert_eq!(mcp_port, 8765);
    assert_eq!(headroom_version, "latest");
}

#[test]
fn test_headroom_setup_with_agents() {
    let args = vec![
        "vx",
        "ai",
        "headroom",
        "setup",
        "-a",
        "codex",
        "-a",
        "claude-code",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command: AiCommand::Headroom(HeadroomCommand::Setup { agent, .. }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(agent, vec!["codex", "claude-code"]);
}

#[test]
fn test_headroom_setup_apply() {
    let args = vec!["vx", "ai", "headroom", "setup", "--apply"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command: AiCommand::Headroom(HeadroomCommand::Setup { dry_run, apply, .. }),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(!dry_run);
    assert!(apply);
}

// ============================================
// headroom proxy
// ============================================

#[test]
fn test_headroom_proxy_start_defaults() {
    let args = vec!["vx", "ai", "headroom", "proxy", "start"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Proxy(HeadroomProxyCommand::Start {
                host,
                port,
                foreground,
                log_file,
                no_optimize,
            })),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port, 8787);
    assert!(!foreground);
    assert!(log_file.is_none());
    assert!(!no_optimize);
}

#[test]
fn test_headroom_proxy_status_defaults() {
    let args = vec!["vx", "ai", "headroom", "proxy", "status"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Proxy(HeadroomProxyCommand::Status { port, json })),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(port, 8787);
    assert!(!json);
}

#[test]
fn test_headroom_proxy_status_json() {
    let args = vec!["vx", "ai", "headroom", "proxy", "status", "--json"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Proxy(HeadroomProxyCommand::Status { json, .. })),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(json);
}

#[test]
fn test_headroom_proxy_stop() {
    let args = vec!["vx", "ai", "headroom", "proxy", "stop", "--port", "9090"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command: AiCommand::Headroom(HeadroomCommand::Proxy(HeadroomProxyCommand::Stop { port })),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(port, 9090);
}

// ============================================
// headroom mcp
// ============================================

#[test]
fn test_headroom_mcp_stdio() {
    let args = vec!["vx", "ai", "headroom", "mcp", "stdio"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(matches!(
        cli.command,
        Some(Commands::Ai {
            command: AiCommand::Headroom(HeadroomCommand::Mcp(HeadroomMcpCommand::Stdio))
        })
    ));
}

#[test]
fn test_headroom_mcp_test_defaults() {
    let args = vec!["vx", "ai", "headroom", "mcp", "test"];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Mcp(HeadroomMcpCommand::Test {
                url,
                json,
                sample_file,
            })),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert_eq!(url, "http://127.0.0.1:8765/mcp");
    assert!(!json);
    assert!(sample_file.is_none());
}

#[test]
fn test_headroom_mcp_test_json() {
    let args = vec![
        "vx",
        "ai",
        "headroom",
        "mcp",
        "test",
        "--json",
        "--sample-file",
        "test_data.json",
    ];
    let cli = Cli::try_parse_from(args).unwrap();
    let Some(Commands::Ai {
        command:
            AiCommand::Headroom(HeadroomCommand::Mcp(HeadroomMcpCommand::Test {
                json,
                sample_file,
                ..
            })),
    }) = cli.command
    else {
        panic!("unexpected command");
    };
    assert!(json);
    assert_eq!(sample_file.unwrap(), "test_data.json");
}
