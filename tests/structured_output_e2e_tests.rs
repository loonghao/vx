//! E2E coverage for machine-readable output purity.

use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;
use std::process::Output;

fn run_vx(args: &[&str]) -> Output {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("vx").unwrap();
    cmd.args(args).output().expect("failed to run vx binary")
}

fn run_vx_with_home(args: &[&str], vx_home: &Path) -> Output {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("vx").unwrap();
    cmd.env("VX_HOME", vx_home)
        .args(args)
        .output()
        .expect("failed to run vx binary")
}

fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn assert_machine_stdout_has_no_text_ui(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{} should succeed: stdout:\n{}\nstderr:\n{}",
        context,
        stdout_str(output),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = stdout_str(output);
    assert!(
        !stdout.trim().is_empty(),
        "{} should produce machine-readable stdout",
        context
    );
    assert!(
        !stdout.contains("Searching for")
            && !stdout.contains("Available Tools")
            && !stdout.contains("No versions installed")
            && !stdout.contains("Did you mean")
            && !stdout.contains("Use 'vx list'")
            && !stdout.contains('\u{2139}')
            && !stdout.contains('\u{1f4e6}'),
        "{} should not mix text UI with machine-readable stdout:\n{}",
        context,
        stdout
    );
}

#[test]
fn test_search_toon_stdout_has_no_text_ui_header() {
    let output = run_vx(&["--output-format", "toon", "search", "node"]);

    assert_machine_stdout_has_no_text_ui(&output, "vx --output-format toon search node");
}

#[test]
fn test_list_toon_stdout_has_no_text_ui_header() {
    let output = run_vx(&["--output-format", "toon", "list"]);

    assert_machine_stdout_has_no_text_ui(&output, "vx --output-format toon list");
}

#[test]
fn test_version_toon_stdout_has_no_text_ui_header() {
    let output = run_vx(&["--output-format", "toon", "version"]);

    assert_machine_stdout_has_no_text_ui(&output, "vx --output-format toon version");
    assert!(
        stdout_str(&output).contains("version:"),
        "version command should render structured version data"
    );
}

#[test]
fn test_config_show_toon_stdout_has_no_text_ui_header() {
    let output = run_vx(&["--output-format", "toon", "config", "show"]);

    assert_machine_stdout_has_no_text_ui(&output, "vx --output-format toon config show");
    assert!(
        stdout_str(&output).contains("tools_count:"),
        "config show should render structured config data"
    );
}

#[test]
fn test_list_tool_compact_stdout_has_no_text_ui_header() {
    let output = run_vx(&["--compact", "list", "node"]);

    assert_machine_stdout_has_no_text_ui(&output, "vx --compact list node");
}

#[test]
fn test_missing_which_toon_stdout_has_no_text_ui_suggestions() {
    let output = run_vx(&[
        "--output-format",
        "toon",
        "which",
        "definitely-not-a-vx-runtime-xyz",
    ]);

    assert!(
        !output.status.success(),
        "missing runtime lookup should fail"
    );

    let stdout = stdout_str(&output);
    assert!(
        stdout.contains("source: not_found"),
        "missing runtime should render structured not_found stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Did you mean")
            && !stdout.contains("Use 'vx list'")
            && !stdout.contains('\u{2139}'),
        "missing runtime should not mix suggestion UI into stdout:\n{}",
        stdout
    );
}

#[test]
fn test_metrics_tokens_reports_structured_output_savings() {
    let vx_home = tempfile::tempdir().unwrap();

    let list_toon = run_vx_with_home(&["--output-format", "toon", "list"], vx_home.path());
    assert_machine_stdout_has_no_text_ui(&list_toon, "vx --output-format toon list");

    let list_compact = run_vx_with_home(&["--compact", "list", "node"], vx_home.path());
    assert_machine_stdout_has_no_text_ui(&list_compact, "vx --compact list node");

    let summary = run_vx_with_home(
        &["metrics", "tokens", "--last", "20", "--json"],
        vx_home.path(),
    );
    assert!(
        summary.status.success(),
        "metrics tokens should succeed: stdout:\n{}\nstderr:\n{}",
        stdout_str(&summary),
        String::from_utf8_lossy(&summary.stderr)
    );

    let json: Value = serde_json::from_str(&stdout_str(&summary)).unwrap();
    assert!(
        json["records"].as_u64().unwrap_or_default() >= 2,
        "expected at least two token savings records:\n{}",
        stdout_str(&summary)
    );
    let commands = json["commands"].as_array().unwrap();
    assert!(
        commands.iter().any(|entry| entry["command"]
            .as_str()
            .is_some_and(|command| command.contains("--output-format toon"))),
        "summary should include the TOON command:\n{}",
        stdout_str(&summary)
    );
    assert!(
        commands.iter().any(|entry| entry["command"]
            .as_str()
            .is_some_and(|command| command.contains("--compact"))),
        "summary should include the compact command:\n{}",
        stdout_str(&summary)
    );

    let compact = commands
        .iter()
        .find(|entry| {
            entry["command"]
                .as_str()
                .is_some_and(|command| command.contains("--compact"))
        })
        .unwrap();
    assert!(
        compact["net_saved_tokens"].as_i64().unwrap_or_default() > 0,
        "compact output should report positive token savings:\n{}",
        stdout_str(&summary)
    );
}
