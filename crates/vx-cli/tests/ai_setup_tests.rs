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

struct EnvGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let original = std::env::var_os(key);
        // These tests are serialized, so mutating process environment is scoped
        // by EnvGuard and cannot race another ai_setup test.
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.original {
            unsafe {
                std::env::set_var(self.key, value);
            }
        } else {
            unsafe {
                std::env::remove_var(self.key);
            }
        }
    }
}

#[tokio::test]
#[serial]
async fn test_ai_setup_installs_token_efficient_builtin_skills() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());
    fs::write(temp_dir.path().join("vx.toml"), "[tools]\n").expect("Failed to write vx.toml");
    let agents = vec!["codex".to_string()];

    vx_cli::commands::ai::handle_setup(&agents, false, true, true)
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
        usage.contains("Legacy Python 2.7/3.7 Projects"),
        "embedded vx-usage skill should explain legacy Python workflows"
    );
    assert!(
        usage.contains("vx uv venv .venv27 --python 2.7"),
        "embedded vx-usage skill should teach Python 2.7 venv creation"
    );
    assert!(
        commands.contains("Forwarded tools such as `vx git`,"),
        "embedded vx-commands skill should explain transparent forwarding"
    );
    assert!(
        commands.contains("--output-format <text|json|toon|compact>"),
        "embedded vx-commands skill should document compact output format"
    );

    let config = vx_config::parse_config(temp_dir.path().join("vx.toml"))
        .expect("updated vx.toml should parse");
    let recorded_hash = config
        .ai
        .and_then(|ai| ai.skills_hash)
        .expect("project setup should record skills hash");
    assert_eq!(recorded_hash, vx_cli::commands::ai::compute_skills_hash());
}

#[tokio::test]
#[serial]
async fn test_ai_setup_defaults_to_global_scope() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _cwd = CwdGuard::enter(temp_dir.path());
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());
    let agents = vec!["codex".to_string()];

    vx_cli::commands::ai::handle_setup(&agents, false, false, true)
        .await
        .expect("vx ai setup should succeed");

    assert!(
        !temp_dir
            .path()
            .join(".agents/skills/vx-usage/SKILL.md")
            .exists(),
        "default setup should not install project-scoped skills"
    );
    assert!(
        home_dir
            .path()
            .join(".codex/skills/vx-usage/SKILL.md")
            .exists(),
        "default setup should install global skills"
    );
}

#[test]
fn test_ai_config_parses_skills_hash() {
    let config = vx_config::parse_config_str(
        r#"
[ai]
skills_hash = "abc123"
skills_updated_at = "2026-06-02T00:00:00Z"
"#,
    )
    .expect("config should parse");

    let ai = config.ai.expect("ai config should be present");
    assert_eq!(ai.skills_hash.as_deref(), Some("abc123"));
    assert_eq!(
        ai.skills_updated_at.as_deref(),
        Some("2026-06-02T00:00:00Z")
    );
}
