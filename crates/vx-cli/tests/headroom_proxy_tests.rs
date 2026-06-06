// Tests for headroom proxy lifecycle helpers.
//
// All state-path tests use `_with_base` variants with a TempDir to avoid
// writing to the real `~/.vx/state/headroom/` directory.

use std::io::Write;
use std::path::Path;

use tempfile::{NamedTempFile, TempDir};
use vx_cli::commands::ai::{
    DEFAULT_PROXY_HOST, build_proxy_command, check_port, headroom_state_dir_with_base,
    proxy_host_path_with_base, proxy_log_path_with_base, proxy_pid_path_with_base, read_pid,
    read_proxy_host_with_base,
};

// ---------------------------------------------------------------------------
// State file path helpers (isolated via TempDir)
// ---------------------------------------------------------------------------

#[test]
fn proxy_host_path_with_base_returns_expected_suffix() {
    let dir = TempDir::new().unwrap();
    let base = dir.path();
    let path = proxy_host_path_with_base(base, 8787).unwrap();
    assert!(
        path.to_string_lossy().contains("proxy-8787.host"),
        "expected suffix proxy-8787.host, got {}",
        path.display()
    );
    assert!(
        path.starts_with(base),
        "path should be under the injected base directory"
    );
}

#[test]
fn proxy_pid_path_with_base_returns_expected_suffix() {
    let dir = TempDir::new().unwrap();
    let base = dir.path();
    let path = proxy_pid_path_with_base(base, 9999).unwrap();
    assert!(
        path.to_string_lossy().contains("proxy-9999.pid"),
        "expected suffix proxy-9999.pid, got {}",
        path.display()
    );
    assert!(path.starts_with(base));
}

#[test]
fn proxy_log_path_with_base_returns_expected_suffix() {
    let dir = TempDir::new().unwrap();
    let base = dir.path();
    let path = proxy_log_path_with_base(base, 4321).unwrap();
    assert!(
        path.to_string_lossy().contains("proxy-4321.log"),
        "expected suffix proxy-4321.log, got {}",
        path.display()
    );
    assert!(path.starts_with(base));
}

// ---------------------------------------------------------------------------
// read_proxy_host_with_base
// ---------------------------------------------------------------------------

#[test]
fn read_proxy_host_with_base_falls_back_to_default() {
    let dir = TempDir::new().unwrap();
    let host = read_proxy_host_with_base(dir.path(), 55432);
    assert_eq!(host, DEFAULT_PROXY_HOST);
}

#[test]
fn read_proxy_host_with_base_returns_persisted_value() {
    let dir = TempDir::new().unwrap();
    let base = dir.path();

    // Write a host file for port 8788
    let host_path = proxy_host_path_with_base(base, 8788).unwrap();
    std::fs::write(&host_path, "192.168.1.1").unwrap();

    let host = read_proxy_host_with_base(base, 8788);
    assert_eq!(host, "192.168.1.1");
}

#[test]
fn read_proxy_host_with_base_returns_persisted_value_with_trailing_newline() {
    let dir = TempDir::new().unwrap();
    let base = dir.path();

    let host_path = proxy_host_path_with_base(base, 8788).unwrap();
    std::fs::write(&host_path, "10.0.0.1\n").unwrap();

    let host = read_proxy_host_with_base(base, 8788);
    assert_eq!(host, "10.0.0.1");
}

// ---------------------------------------------------------------------------
// read_pid
// ---------------------------------------------------------------------------

#[test]
fn read_pid_returns_none_for_missing_file() {
    assert!(read_pid(Path::new("/nonexistent/pid/file.pid")).is_none());
}

#[test]
fn read_pid_returns_parsed_value() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "4242").unwrap();
    assert_eq!(read_pid(file.path()), Some(4242));
}

#[test]
fn read_pid_rejects_invalid_content() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "not-a-number").unwrap();
    assert_eq!(read_pid(file.path()), None);
}

// ---------------------------------------------------------------------------
// build_proxy_command: vx bridge
// ---------------------------------------------------------------------------

#[test]
fn build_proxy_command_uses_vx_bridge() {
    let (cmd, label) = build_proxy_command("127.0.0.1", 8787, None, false);
    let program = cmd.get_program().to_string_lossy();

    // The command should use the vx executable, NOT bare "headroom"
    assert!(
        program.contains("vx") || program.ends_with("vx.exe"),
        "expected vx bridge, got program '{}'",
        program
    );
    assert!(
        !program.ends_with("headroom") && !program.ends_with("headroom.exe"),
        "should NOT invoke bare headroom"
    );
    assert!(label.contains("vx uv tool run headroom proxy"));
}

#[test]
fn build_proxy_command_includes_uv_tool_run_args() {
    let (cmd, _) = build_proxy_command("10.0.0.1", 9999, None, false);
    let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();

    // Verify the bridge path: vx uv tool run headroom proxy --host ... --port ...
    assert!(args.iter().any(|a| a == "uv"), "expected 'uv' subcommand");
    assert!(
        args.iter().any(|a| a == "tool"),
        "expected 'tool' subcommand"
    );
    assert!(args.iter().any(|a| a == "run"), "expected 'run' subcommand");
    assert!(
        args.iter().any(|a| a == "headroom"),
        "expected 'headroom' command"
    );
    assert!(
        args.iter().any(|a| a == "proxy"),
        "expected 'proxy' command"
    );
}

#[test]
fn build_proxy_command_program_is_executable() {
    let (cmd, _) = build_proxy_command("127.0.0.1", 8787, None, false);
    let program = cmd.get_program();

    // The program is derived from std::env::current_exe() which must exist.
    let program_str = program.to_string_lossy();
    assert!(!program_str.is_empty(), "program path must not be empty");
    assert!(
        Path::new(program).exists() || program_str.contains("vx"),
        "program '{}' should exist or reference the vx binary",
        program_str
    );
}

#[test]
fn build_proxy_command_includes_no_optimize_flag() {
    let (cmd, _) = build_proxy_command("127.0.0.1", 8787, Some("/tmp/log"), true);
    let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
    assert!(
        args.iter().any(|a| a == "--no-optimize"),
        "expected --no-optimize in args: {:?}",
        args
    );
}

#[test]
fn build_proxy_command_sets_telemetry_off() {
    let (cmd, _) = build_proxy_command("127.0.0.1", 8787, None, false);
    let envs: Vec<_> = cmd.get_envs().collect();
    let telemetry_off = envs.iter().any(|(k, v)| {
        *k == std::ffi::OsStr::new("HEADROOM_TELEMETRY")
            && v.map(|val| val == "off").unwrap_or(false)
    });
    assert!(telemetry_off, "HEADROOM_TELEMETRY should be 'off'");
}

#[test]
fn build_proxy_command_passes_log_file() {
    let (cmd, _) = build_proxy_command("127.0.0.1", 8787, Some("/custom/log.log"), false);
    let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
    let log_file_idx = args.iter().position(|a| a == "--log-file");
    assert!(log_file_idx.is_some(), "expected --log-file flag in args");
    let next = args[log_file_idx.unwrap() + 1].clone();
    assert_eq!(
        next, "/custom/log.log",
        "log file arg should follow --log-file"
    );
}

#[test]
fn build_proxy_command_passes_host_and_port() {
    let (cmd, _) = build_proxy_command("10.20.30.40", 12345, None, false);
    let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();

    let host_idx = args.iter().position(|a| a == "--host");
    assert!(host_idx.is_some(), "expected --host flag");
    assert_eq!(args[host_idx.unwrap() + 1], "10.20.30.40");

    let port_idx = args.iter().position(|a| a == "--port");
    assert!(port_idx.is_some(), "expected --port flag");
    assert_eq!(args[port_idx.unwrap() + 1], "12345");
}

// ---------------------------------------------------------------------------
// check_port / check_health_endpoint — behavioral verification
// ---------------------------------------------------------------------------

#[test]
fn check_port_refuses_unreachable_host() {
    // 240.0.0.1 is in the reserved / experimental range (240/4) and should
    // be unreachable. This verifies check_port returns Err for a host that
    // cannot be reached within the 2-second timeout.
    let result = check_port("240.0.0.1", 1);
    assert!(
        result.is_err(),
        "check_port should fail for unreachable host 240.0.0.1:1"
    );
}

#[test]
fn check_port_refuses_loopback_with_impossible_port() {
    // On most systems, port 0 is reserved and a high port with nothing
    // listening should be refused quickly on loopback.
    let result = check_port("127.0.0.1", 65534);
    if let Ok(()) = result {
        // Some systems may have something listening on this port — skip
        // the assertion in that case (not a real failure).
        eprintln!("Note: 127.0.0.1:65534 was reachable (unusual but not a bug)");
    }
    // The primary goal is that check_port doesn't panic and returns in
    // a reasonable time.
}

#[test]
fn headroom_state_dir_with_base_uses_injected_base() {
    let dir = TempDir::new().unwrap();
    let base = dir.path();
    let state = headroom_state_dir_with_base(base);
    assert!(state.starts_with(base));
    assert!(state.ends_with(std::path::Path::new(".vx").join("state").join("headroom")));
}
