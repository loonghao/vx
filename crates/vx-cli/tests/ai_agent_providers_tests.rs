//! Tests for the AI agent providers (`vx claude`, `vx codex`, `vx kimi`, …).
//!
//! These providers are RFC 0033 package aliases: the runtime name is what users
//! type, while the package name is what gets installed. For every agent here the
//! two differ (scoped npm packages, or `opencode-ai` shipping `opencode`), so the
//! `executable` key is what makes `vx <agent>` resolve to the right binary.

use std::fs;
use std::path::PathBuf;

use rstest::rstest;
use serial_test::serial;
use tempfile::TempDir;

/// Restore the process working directory on drop.
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

/// Every agent provider, as `(runtime, ecosystem, package, executable)`.
///
/// `executable` is `None` only for `cline`, where the npm package and the binary
/// share a name.
const AGENT_PROVIDERS: &[(&str, &str, &str, Option<&str>)] = &[
    ("claude", "npm", "@anthropic-ai/claude-code", Some("claude")),
    ("codex", "npm", "@openai/codex", Some("codex")),
    ("kimi", "npm", "@moonshot-ai/kimi-code", Some("kimi")),
    ("gemini", "npm", "@google/gemini-cli", Some("gemini")),
    ("opencode", "npm", "opencode-ai", Some("opencode")),
    ("amp", "npm", "@ampcode/cli", Some("amp")),
    ("copilot", "npm", "@github/copilot", Some("copilot")),
    ("cline", "npm", "cline", None),
    ("qwen", "npm", "@qwen-code/qwen-code", Some("qwen")),
    ("aider", "uvx", "aider-chat", Some("aider")),
];

#[rstest]
#[case::claude("claude", "npm", "@anthropic-ai/claude-code", Some("claude"))]
#[case::codex("codex", "npm", "@openai/codex", Some("codex"))]
#[case::kimi("kimi", "npm", "@moonshot-ai/kimi-code", Some("kimi"))]
#[case::gemini("gemini", "npm", "@google/gemini-cli", Some("gemini"))]
#[case::opencode("opencode", "npm", "opencode-ai", Some("opencode"))]
#[case::amp("amp", "npm", "@ampcode/cli", Some("amp"))]
#[case::copilot("copilot", "npm", "@github/copilot", Some("copilot"))]
#[case::cline("cline", "npm", "cline", None)]
#[case::qwen("qwen", "npm", "@qwen-code/qwen-code", Some("qwen"))]
#[case::aider("aider", "uvx", "aider-chat", Some("aider"))]
fn agent_runtime_routes_to_its_package(
    #[case] runtime: &str,
    #[case] ecosystem: &str,
    #[case] package: &str,
    #[case] executable: Option<&str>,
) {
    let alias = vx_cli::registry::find_package_alias(runtime)
        .unwrap_or_else(|| panic!("{runtime} should expose a package_alias"));

    assert_eq!(alias.ecosystem, ecosystem, "ecosystem for {runtime}");
    assert_eq!(alias.package, package, "package for {runtime}");
    assert_eq!(
        alias.executable.as_deref(),
        executable,
        "executable override for {runtime}"
    );
}

/// The names `vx ai setup` uses must reach the same providers, so the two agent
/// tables cannot silently drift apart.
#[rstest]
#[case::claude_code("claude-code", "@anthropic-ai/claude-code")]
#[case::gemini_cli("gemini-cli", "@google/gemini-cli")]
#[case::qwen_code("qwen-code", "@qwen-code/qwen-code")]
#[case::codex("codex", "@openai/codex")]
#[case::copilot("copilot", "@github/copilot")]
#[case::opencode("opencode", "opencode-ai")]
#[case::amp("amp", "@ampcode/cli")]
#[case::cline("cline", "cline")]
#[case::kimi("kimi", "@moonshot-ai/kimi-code")]
fn ai_setup_agent_names_resolve_to_providers(#[case] name: &str, #[case] package: &str) {
    let alias = vx_cli::registry::find_package_alias(name)
        .unwrap_or_else(|| panic!("'{name}' should resolve to a provider"));

    assert_eq!(alias.package, package, "package for '{name}'");
}

/// A scoped package without an `executable` override would make vx look for a
/// binary literally named `@scope/name`, which never exists.
#[test]
fn scoped_packages_declare_an_executable_override() {
    for (runtime, _ecosystem, package, executable) in AGENT_PROVIDERS {
        if package.contains('/') {
            assert!(
                executable.is_some(),
                "scoped package {package} for '{runtime}' must declare an executable override"
            );
        }
        assert_ne!(
            *executable,
            Some(*package),
            "'{runtime}' declares a redundant executable override"
        );
    }
}

/// `vx ai setup -a kimi` must target Kimi Code's own skills directory
/// (`.kimi-code/skills/`), not the generic `.agents/skills/` tier.
#[tokio::test]
#[serial]
async fn ai_setup_targets_kimi_skills_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _cwd = CwdGuard::enter(temp_dir.path());
    fs::write(temp_dir.path().join("vx.toml"), "[tools]\n").expect("Failed to write vx.toml");

    let agents = vec!["kimi".to_string()];
    vx_cli::commands::ai::handle_setup(&agents, false, true, true)
        .await
        .expect("vx ai setup -a kimi should succeed");

    let skill = temp_dir.path().join(".kimi-code/skills/vx-usage/SKILL.md");
    assert!(
        skill.exists(),
        "expected vx-usage skill at {}, found dirs: {:?}",
        skill.display(),
        fs::read_dir(temp_dir.path())
            .map(|entries| entries.flatten().map(|e| e.file_name()).collect::<Vec<_>>())
            .unwrap_or_default()
    );
}
