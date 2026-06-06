//! Behavioral tests for `vx ai headroom setup` — dry-run, apply, error handling.

use std::fs;
use std::path::PathBuf;

use serial_test::serial;
use tempfile::TempDir;

use vx_cli::commands::ai::handle_headroom_setup;

struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    fn enter(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().expect("Failed to read current dir");
        std::env::set_current_dir(path).expect("Failed to enter temp dir");
        Self { original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_dry_run_does_not_touch_disk() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    let agents: Vec<String> = vec![];
    handle_headroom_setup(
        &agents, true, /* dry_run */
        false, 8787, 8765, "latest",
    )
    .await
    .expect("dry-run should succeed");

    // Verify no config files were created
    assert!(!temp_dir.path().join(".codex/mcp.json").exists());
    assert!(!temp_dir.path().join(".claude/settings.json").exists());
    assert!(!temp_dir.path().join(".cursor/mcp.json").exists());
    assert!(!temp_dir.path().join(".codex").exists());
    assert!(!temp_dir.path().join(".claude").exists());
    assert!(!temp_dir.path().join(".cursor").exists());
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_apply_creates_new_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    let agents: Vec<String> = vec!["codex".to_string()];
    let result = handle_headroom_setup(&agents, false, true, 8787, 8765, "latest").await;
    assert!(result.is_ok(), "apply should succeed: {:?}", result.err());

    let config_path = temp_dir.path().join(".codex/mcp.json");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).expect("should read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("config should be valid JSON");

    assert_eq!(parsed["mcpServers"]["headroom"]["command"], "vx");
    assert_eq!(parsed["mcpServers"]["headroom"]["args"][0], "ai");
    assert_eq!(parsed["mcpServers"]["headroom"]["args"][1], "headroom");
    assert_eq!(parsed["mcpServers"]["headroom"]["args"][2], "mcp");
    assert_eq!(parsed["mcpServers"]["headroom"]["args"][3], "stdio");
    // HEADROOM_TELEMETRY=off must be present
    let env = &parsed["mcpServers"]["headroom"]["env"];
    assert_eq!(env["HEADROOM_TELEMETRY"], "off");
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_apply_preserves_existing_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    // Pre-create a valid config with another MCP server
    let codex_dir = temp_dir.path().join(".codex");
    fs::create_dir_all(&codex_dir).expect("create .codex dir");
    let pre_existing = serde_json::json!({
        "mcpServers": {
            "other-server": {
                "command": "python",
                "args": ["-m", "other_server"]
            }
        }
    });
    fs::write(
        codex_dir.join("mcp.json"),
        serde_json::to_string_pretty(&pre_existing).unwrap() + "\n",
    )
    .expect("write pre-existing config");

    let agents: Vec<String> = vec!["codex".to_string()];
    let result = handle_headroom_setup(&agents, false, true, 8787, 8765, "latest").await;
    assert!(result.is_ok(), "apply should succeed: {:?}", result.err());

    let content = fs::read_to_string(codex_dir.join("mcp.json")).expect("should read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("config should be valid JSON");

    // Original server preserved
    assert_eq!(parsed["mcpServers"]["other-server"]["command"], "python");
    // Headroom entry added
    assert_eq!(parsed["mcpServers"]["headroom"]["command"], "vx");
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_apply_invalid_json_returns_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    // Pre-create an invalid JSON file
    let codex_dir = temp_dir.path().join(".codex");
    fs::create_dir_all(&codex_dir).expect("create .codex dir");
    fs::write(codex_dir.join("mcp.json"), "not valid json {{{").expect("write invalid config");

    let agents: Vec<String> = vec!["codex".to_string()];
    let result = handle_headroom_setup(&agents, false, true, 8787, 8765, "latest").await;
    assert!(result.is_err(), "should fail on invalid JSON");

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("not valid JSON"),
        "error should mention invalid JSON, got: {}",
        err_msg
    );

    // Original content must NOT be overwritten
    let content = fs::read_to_string(codex_dir.join("mcp.json")).expect("should read config");
    assert_eq!(
        content, "not valid json {{{",
        "original content must be preserved"
    );
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_apply_write_failure_returns_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    // Create .codex as a file (not directory) to cause a write failure
    let codex_path = temp_dir.path().join(".codex");
    fs::write(&codex_path, "not a directory").expect("write blocking file");

    let agents: Vec<String> = vec!["codex".to_string()];
    let result = handle_headroom_setup(&agents, false, true, 8787, 8765, "latest").await;
    assert!(result.is_err(), "should fail when write is impossible");

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Failed to write") || err_msg.contains("Failed to create"),
        "error should mention the failure, got: {}",
        err_msg
    );
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_apply_multiple_agents() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    let agents: Vec<String> = vec!["codex".to_string(), "claude-code".to_string()];
    let result = handle_headroom_setup(&agents, false, true, 8787, 8765, "latest").await;
    assert!(
        result.is_ok(),
        "multi-agent apply should succeed: {:?}",
        result.err()
    );

    // Both config files should exist
    assert!(temp_dir.path().join(".codex/mcp.json").exists());
    assert!(temp_dir.path().join(".claude/settings.json").exists());

    // Untargeted agent should NOT exist
    assert!(!temp_dir.path().join(".cursor/mcp.json").exists());
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_default_targets_all_agents() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    // Empty agents list = all three
    let agents: Vec<String> = vec![];
    let result = handle_headroom_setup(&agents, false, true, 8787, 8765, "latest").await;
    assert!(
        result.is_ok(),
        "default apply should succeed: {:?}",
        result.err()
    );

    assert!(temp_dir.path().join(".codex/mcp.json").exists());
    assert!(temp_dir.path().join(".claude/settings.json").exists());
    assert!(temp_dir.path().join(".cursor/mcp.json").exists());
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_unsupported_agent_is_ignored() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    let agents: Vec<String> = vec!["unknown-agent".to_string()];
    let result = handle_headroom_setup(&agents, false, true, 8787, 8765, "latest").await;
    // No matching targets means nothing to do — should succeed with no files written
    assert!(result.is_ok());

    assert!(!temp_dir.path().join(".codex/mcp.json").exists());
    assert!(!temp_dir.path().join(".claude/settings.json").exists());
    assert!(!temp_dir.path().join(".cursor/mcp.json").exists());
}

#[tokio::test]
#[serial]
async fn test_headroom_setup_custom_ports_in_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());

    let agents: Vec<String> = vec!["codex".to_string()];
    let result = handle_headroom_setup(&agents, false, true, 9999, 7777, "0.5.0").await;
    assert!(result.is_ok(), "apply with custom ports should succeed");

    let content =
        fs::read_to_string(temp_dir.path().join(".codex/mcp.json")).expect("should read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("config should be valid JSON");

    // The MCP command always points to "vx ai headroom mcp stdio"
    assert_eq!(parsed["mcpServers"]["headroom"]["command"], "vx");
}
