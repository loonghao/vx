//! Tests for `vx ai check` drift detection and `vx ai check --fix` convergence.
//!
//! Skills are installed **globally** by `vx ai setup`; a repository that carries
//! its own copy of a built-in vx skill has drifted. These tests pin that model:
//!
//! - `vx ai setup` records the skills hash in global mode too (not only `--project`).
//! - `vx ai check` reports repository-local copies of built-in skills.
//! - `vx ai check --fix` removes those copies and refreshes the global install.
//! - `[ai].skills_source = true` marks the upstream repo, where a local `skills/`
//!   directory is the source of truth instead of drift.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use rstest::rstest;
use tempfile::TempDir;

/// Canonical skills directory of the vx repository.
fn skills_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills")
}

/// Lock guarding the process-wide state the `vx ai` commands read: the current
/// directory and `VX_AI_HOME`.
///
/// Test binaries run cases in parallel threads, and `serial_test` attributes are
/// not applied to `rstest`-generated cases, so this file uses an explicit lock
/// instead of relying on test ordering.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Acquire the environment lock.
///
/// A panicking test poisons the mutex; the state it protects is process-wide
/// test scaffolding, so recovering the guard is safe and keeps one failure from
/// cascading into every later test.
fn lock_env() -> MutexGuard<'static, ()> {
    env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    fn enter(path: &Path) -> Self {
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
    fn set(key: &'static str, value: &Path) -> Self {
        let original = std::env::var_os(key);
        // Only used while `lock_env()` is held, so the process environment is
        // never mutated from two tests at once.
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

/// Write a repository-local copy of a skill.
fn write_local_copy(project_root: &Path, dir: &str, skill: &str, content: &str) -> PathBuf {
    let skill_file = project_root.join(dir).join(skill).join("SKILL.md");
    fs::create_dir_all(skill_file.parent().expect("skill file has a parent"))
        .expect("Failed to create local skill dir");
    fs::write(&skill_file, content).expect("Failed to write local skill copy");
    skill_file
}

/// Create a throwaway project with a `vx.toml` and enter it.
///
/// Must be called while `lock_env()` is held.
fn project_with_manifest(manifest: &str) -> (TempDir, CwdGuard) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("vx.toml"), manifest).expect("Failed to write vx.toml");
    let guard = CwdGuard::enter(temp_dir.path());
    (temp_dir, guard)
}

fn recorded_hash(project_root: &Path) -> Option<String> {
    vx_config::parse_config(project_root.join("vx.toml"))
        .expect("vx.toml should parse")
        .ai
        .and_then(|ai| ai.skills_hash)
}

/// Run a `vx ai` command to completion on a fresh runtime.
///
/// The tests are synchronous on purpose: the environment lock is a
/// `std::sync::Mutex`, and a guard must never be held across an `.await`.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("Failed to build runtime")
        .block_on(future)
}

/// Global skills home shared by the parameterized tests.
///
/// The install runs once per process; the directory is intentionally leaked so
/// every case can read it. Lookups only read, so cases do not need the lock.
fn shared_global_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let _lock = lock_env();
        // `keep()` leaks the directory on purpose: every case reads it.
        let home = TempDir::new().expect("Failed to create temp home").keep();
        let _env = EnvGuard::set("VX_AI_HOME", &home);
        block_on(vx_cli::commands::ai::handle_setup(
            &["codex".to_string()],
            true,
            false,
            true,
        ))
        .expect("vx ai setup should succeed");
        home
    })
}

#[test]
fn test_ai_setup_records_hash_in_global_mode() {
    let _lock = lock_env();
    let (project, _cwd) = project_with_manifest("[tools]\n");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());
    let agents = vec!["codex".to_string()];

    // Global scope (no `--project`): the hash must still be recorded so that
    // `vx ai check` can detect drift without a project-local install.
    block_on(vx_cli::commands::ai::handle_setup(
        &agents, true, false, true,
    ))
    .expect("vx ai setup should succeed");

    assert_eq!(
        recorded_hash(project.path()).as_deref(),
        Some(vx_cli::commands::ai::compute_skills_hash().as_str()),
        "global setup should record the embedded skills hash in vx.toml"
    );
    assert!(
        !project
            .path()
            .join(".codex/skills/vx-usage/SKILL.md")
            .exists(),
        "global setup must not write project-local skills"
    );
}

#[test]
fn test_ai_check_fix_removes_local_copies_and_refreshes_global() {
    let _lock = lock_env();
    let (project, _cwd) = project_with_manifest("[tools]\n");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());

    // A verbatim copy under `.claude/skills/` and a forked copy in the top-level
    // `skills/` directory: both duplicate the global install.
    let canonical = fs::read_to_string(skills_dir().join("vx-usage/SKILL.md"))
        .expect("Failed to read canonical vx-usage skill");
    write_local_copy(project.path(), ".claude/skills", "vx-usage", &canonical);
    write_local_copy(
        project.path(),
        ".claude/skills",
        "acme-domain-skill",
        "# project-owned skill\n",
    );
    write_local_copy(project.path(), "skills", "vx-commands", "# forked copy\n");

    block_on(vx_cli::commands::ai::handle_check(true)).expect("vx ai check --fix should succeed");

    assert!(
        !project.path().join(".claude/skills/vx-usage").exists(),
        "--fix should remove the duplicated vx-usage copy"
    );
    assert!(
        !project.path().join("skills/vx-commands").exists(),
        "--fix should remove the forked vx-commands copy"
    );
    assert!(
        project
            .path()
            .join(".claude/skills/acme-domain-skill/SKILL.md")
            .exists(),
        "--fix must keep project-owned skills that are not built-in vx skills"
    );
    assert!(
        home_dir
            .path()
            .join(".claude/skills/vx-usage/SKILL.md")
            .exists(),
        "--fix should refresh the global install"
    );
    assert_eq!(
        recorded_hash(project.path()).as_deref(),
        Some(vx_cli::commands::ai::compute_skills_hash().as_str()),
        "--fix should re-record the embedded skills hash"
    );
}

#[test]
fn test_ai_check_fix_keeps_local_skills_in_source_repository() {
    let _lock = lock_env();
    let (project, _cwd) = project_with_manifest("[tools]\n\n[ai]\nskills_source = true\n");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());

    let source_file = write_local_copy(project.path(), "skills", "vx-usage", "# upstream copy\n");

    block_on(vx_cli::commands::ai::handle_check(true)).expect("vx ai check --fix should succeed");

    assert!(
        source_file.exists(),
        "[ai].skills_source marks the upstream repository; its skills/ must survive --fix"
    );
}

#[test]
fn test_ai_check_reports_local_copies_without_fixing() {
    let _lock = lock_env();
    let (project, _cwd) = project_with_manifest("[tools]\n");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());

    let copy = write_local_copy(
        project.path(),
        ".agents/skills",
        "vx-usage",
        "# duplicate\n",
    );

    block_on(vx_cli::commands::ai::handle_check(false)).expect("vx ai check should succeed");

    assert!(
        copy.exists(),
        "check without --fix is advisory and must not delete anything"
    );
}

/// A repository that only carries project-owned skills passes `vx ai check`
/// unchanged: the audited repos hold domain skills such as `transx` or
/// `mesh-ops`, not copies of the built-in vx skills.
#[test]
fn test_ai_check_passes_for_project_owned_skills_only() {
    let _lock = lock_env();
    let (project, _cwd) = project_with_manifest("[tools]\n");
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let _home = EnvGuard::set("VX_AI_HOME", home_dir.path());

    write_local_copy(project.path(), "skills", "acme-domain-skill", "# ours\n");

    block_on(vx_cli::commands::ai::handle_check(true)).expect("vx ai check --fix should succeed");

    assert!(
        project
            .path()
            .join("skills/acme-domain-skill/SKILL.md")
            .exists(),
        "project-owned skills must never be removed"
    );
}

/// Every skill shipped under `skills/` must be embedded, otherwise
/// `vx ai setup` silently distributes a subset of the canonical set.
#[rstest]
#[case::vx_usage("vx-usage")]
#[case::vx_commands("vx-commands")]
#[case::vx_project("vx-project")]
#[case::vx_troubleshooting("vx-troubleshooting")]
#[case::vx_best_practices("vx-best-practices")]
#[case::vx_agent_workflow("vx-agent-workflow")]
#[case::vx_repo_contract("vx-repo-contract")]
#[case::worktrunk("worktrunk")]
fn test_every_canonical_skill_is_embedded(#[case] skill: &str) {
    let canonical_path = skills_dir().join(skill).join("SKILL.md");
    let canonical = fs::read_to_string(&canonical_path)
        .unwrap_or_else(|_| panic!("canonical skill should exist: {}", canonical_path.display()));

    let installed = shared_global_home()
        .join(".codex/skills")
        .join(skill)
        .join("SKILL.md");
    let installed_content = fs::read_to_string(&installed).unwrap_or_else(|err| {
        panic!(
            "{skill} should be installed at {}: {err}",
            installed.display()
        )
    });

    assert_eq!(
        installed_content, canonical,
        "{skill} is shipped under skills/ but is not embedded in the binary"
    );
}
