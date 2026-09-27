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

#[test]
fn test_every_skill_directory_is_embedded() {
    // Guards the distribution gap this repo once had: `skills/` carried
    // directories that `vx ai setup` never shipped, so agents never saw them.
    let skills_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../skills")
        .canonicalize()
        .expect("skills/ directory should exist");

    let embedded: Vec<&str> = vx_cli::commands::ai::VX_SKILLS
        .iter()
        .map(|(name, _)| *name)
        .collect();

    let mut on_disk = Vec::new();
    for entry in fs::read_dir(&skills_root).expect("Failed to read skills/ directory") {
        let entry = entry.expect("Failed to read skills/ entry");
        if !entry.path().join("SKILL.md").exists() {
            continue;
        }
        on_disk.push(entry.file_name().to_string_lossy().to_string());
    }

    assert!(
        !on_disk.is_empty(),
        "skills/ should contain at least one SKILL.md"
    );
    for name in &on_disk {
        assert!(
            embedded.contains(&name.as_str()),
            "skills/{name}/SKILL.md exists on disk but is not embedded by `vx ai setup`; \
             add it to VX_SKILLS in crates/vx-cli/src/commands/ai.rs"
        );
    }
}

#[tokio::test]
#[serial]
async fn test_global_setup_records_skills_hash() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _cwd = CwdGuard::enter(temp_dir.path());
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());
    let agents = vec!["codex".to_string()];

    vx_cli::commands::ai::handle_setup(&agents, false, false, true)
        .await
        .expect("global setup should succeed");

    let state = home_dir.path().join(".vx/ai-skills.toml");
    assert!(state.exists(), "global setup should record a skills hash");

    let content = fs::read_to_string(&state).expect("Failed to read global skills state");
    let doc: toml_edit::DocumentMut = content
        .parse()
        .expect("global skills state should be valid TOML");
    let recorded = doc["ai"]["skills_hash"]
        .as_str()
        .expect("global state should record skills_hash");
    assert_eq!(recorded, vx_cli::commands::ai::compute_skills_hash());
}

#[tokio::test]
#[serial]
async fn test_ai_check_fix_drops_identical_project_duplicates_but_keeps_divergent() {
    let project = TempDir::new().expect("Failed to create temp dir");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _cwd = CwdGuard::enter(project.path());
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());

    // A project that never opted into project scope: no [ai].skills_hash.
    fs::write(project.path().join("vx.toml"), "[tools]\n").expect("Failed to write vx.toml");

    // `.agents/skills` is codex's project-scope directory; its global scope is
    // `~/.codex/skills`, so this cannot collide with the temp home above.
    let skills_dir = project.path().join(".agents/skills");
    let identical_path = skills_dir.join("vx-usage/SKILL.md");
    let divergent_path = skills_dir.join("vx-commands/SKILL.md");

    let (_, usage_content) = vx_cli::commands::ai::VX_SKILLS
        .iter()
        .find(|(name, _)| *name == "vx-usage")
        .expect("vx-usage should be embedded");
    let (_, commands_content) = vx_cli::commands::ai::VX_SKILLS
        .iter()
        .find(|(name, _)| *name == "vx-commands")
        .expect("vx-commands should be embedded");

    fs::create_dir_all(identical_path.parent().unwrap()).expect("Failed to create skill dir");
    fs::write(&identical_path, usage_content).expect("Failed to write identical copy");

    fs::create_dir_all(divergent_path.parent().unwrap()).expect("Failed to create skill dir");
    fs::write(
        &divergent_path,
        format!("{commands_content}\n<!-- locally modified -->\n"),
    )
    .expect("Failed to write divergent copy");

    vx_cli::commands::ai::handle_check(true)
        .await
        .expect("vx ai check --fix should succeed");

    assert!(
        !identical_path.exists(),
        "byte-identical project copies should be removed by --fix"
    );
    assert!(
        divergent_path.exists(),
        "--fix must never delete a locally modified skill copy"
    );
}

#[tokio::test]
#[serial]
async fn test_ai_check_fix_never_touches_the_skills_authoring_directory() {
    // `skills/` at the repo root is where vx *authors* its skills — in this very
    // repository it is the canonical source that `include_str!` reads. Auto-
    // deleting it would destroy upstream content, so --fix must skip it.
    let project = TempDir::new().expect("Failed to create temp dir");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _cwd = CwdGuard::enter(project.path());
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());

    fs::write(project.path().join("vx.toml"), "[tools]\n").expect("Failed to write vx.toml");

    let authored = project.path().join("skills/vx-usage/SKILL.md");
    let (_, usage_content) = vx_cli::commands::ai::VX_SKILLS
        .iter()
        .find(|(name, _)| *name == "vx-usage")
        .expect("vx-usage should be embedded");

    fs::create_dir_all(authored.parent().unwrap()).expect("Failed to create skills dir");
    fs::write(&authored, usage_content).expect("Failed to write authored skill");

    vx_cli::commands::ai::handle_check(true)
        .await
        .expect("vx ai check --fix should succeed");

    assert!(
        authored.exists(),
        "--fix must not delete skills/ — it is the authoring directory"
    );
}
