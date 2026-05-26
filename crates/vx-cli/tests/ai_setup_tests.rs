//! Tests for `vx ai setup` skill installation.

use std::fs;
use std::path::PathBuf;

use serial_test::serial;
use tempfile::TempDir;

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
async fn test_ai_setup_installs_token_efficient_builtin_skills() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());
    let agents = vec!["codex".to_string()];

    vx_cli::commands::ai::handle_setup(&agents, false, true)
        .await
        .expect("vx ai setup should succeed");

    let usage = fs::read_to_string(temp_dir.path().join(".agents/skills/vx-usage/SKILL.md"))
        .expect("Failed to read installed vx-usage skill");
    let commands = fs::read_to_string(temp_dir.path().join(".agents/skills/vx-commands/SKILL.md"))
        .expect("Failed to read installed vx-commands skill");

    assert!(
        usage.contains("Compression decision tree for CI/log triage"),
        "embedded vx-usage skill should include the CI compression decision tree"
    );
    assert!(
        usage.contains("vx --compact gh run view <run> --log"),
        "embedded vx-usage skill should teach explicit compact forwarding"
    );
    assert!(
        commands.contains("Forwarded tools such as `vx git`,"),
        "embedded vx-commands skill should explain transparent forwarding"
    );
    assert!(
        commands.contains("--output-format <text|json|toon|compact>"),
        "embedded vx-commands skill should document compact output format"
    );
}
