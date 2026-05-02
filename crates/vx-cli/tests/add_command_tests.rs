//! Tests for `vx add` command — focused on the pure pieces that do not
//! require a live `ProviderRegistry` or network access (spec parsing and
//! TOML-editing helpers exposed via `pub` API).
//!
use toml_edit::DocumentMut;
use vx_cli::commands::add::{AddOptions, AddRuntimeSpec, apply_edits};

#[test]
fn parse_tool_spec_name_only_defaults_latest() {
    let spec = AddRuntimeSpec::parse("node").unwrap();
    assert_eq!(spec.name, "node");
    assert_eq!(spec.version, "latest");
}

#[test]
fn parse_tool_spec_with_exact_version() {
    let spec = AddRuntimeSpec::parse("node@22.14.0").unwrap();
    assert_eq!(spec.name, "node");
    assert_eq!(spec.version, "22.14.0");
}

#[test]
fn parse_tool_spec_with_partial_version() {
    let spec = AddRuntimeSpec::parse("python@3.11").unwrap();
    assert_eq!(spec.name, "python");
    assert_eq!(spec.version, "3.11");
}

#[test]
fn parse_tool_spec_trims_whitespace() {
    let spec = AddRuntimeSpec::parse("  node@22  ").unwrap();
    assert_eq!(spec.name, "node");
    assert_eq!(spec.version, "22");
}

#[test]
fn parse_tool_spec_rejects_empty() {
    assert!(AddRuntimeSpec::parse("").is_err());
    assert!(AddRuntimeSpec::parse("   ").is_err());
}

// ---------------------------------------------------------------------------
// Format-preserving edit tests
// ---------------------------------------------------------------------------

fn parse_doc(input: &str) -> DocumentMut {
    input.parse().expect("valid toml")
}

#[test]
fn apply_edits_adds_new_tool_to_empty_toml() {
    let mut doc = parse_doc("");
    let specs = vec![AddRuntimeSpec::parse("node@22").unwrap()];
    let opts = AddOptions::default();

    let edits = apply_edits(&mut doc, &specs, &opts).unwrap();

    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].name, "node");
    assert_eq!(edits[0].new_version, "22");

    let rendered = doc.to_string();
    assert!(rendered.contains("[tools]"), "should create [tools] table");
    assert!(rendered.contains("node = \"22\""));
}

#[test]
fn apply_edits_preserves_comments_and_existing_entries() {
    let original = r#"# Top comment
[tools]
# python version pinned for ML stack
python = "3.11"
uv = "latest"

[settings]
auto_install = true
"#;
    let mut doc = parse_doc(original);
    let specs = vec![AddRuntimeSpec::parse("node@22").unwrap()];
    let opts = AddOptions::default();

    apply_edits(&mut doc, &specs, &opts).unwrap();

    let rendered = doc.to_string();
    assert!(rendered.contains("# Top comment"));
    assert!(rendered.contains("# python version pinned for ML stack"));
    assert!(rendered.contains("python = \"3.11\""));
    assert!(rendered.contains("node = \"22\""));
    assert!(rendered.contains("[settings]"));
    assert!(rendered.contains("auto_install = true"));
}

#[test]
fn apply_edits_updates_existing_tool_version() {
    let original = r#"[tools]
node = "20"
"#;
    let mut doc = parse_doc(original);
    let specs = vec![AddRuntimeSpec::parse("node@22").unwrap()];
    let opts = AddOptions::default();

    let edits = apply_edits(&mut doc, &specs, &opts).unwrap();

    assert_eq!(edits.len(), 1);
    let rendered = doc.to_string();
    assert!(rendered.contains("node = \"22\""));
    assert!(!rendered.contains("node = \"20\""));
}

#[test]
fn apply_edits_skips_unchanged_entries() {
    let original = r#"[tools]
node = "22"
"#;
    let mut doc = parse_doc(original);
    let specs = vec![AddRuntimeSpec::parse("node@22").unwrap()];
    let opts = AddOptions::default();

    let edits = apply_edits(&mut doc, &specs, &opts).unwrap();
    assert!(edits.is_empty(), "no-op edits should produce no changes");
}

#[test]
fn apply_edits_with_os_writes_detailed_table() {
    let mut doc = parse_doc("");
    let specs = vec![AddRuntimeSpec::parse("pwsh@7.4").unwrap()];
    let opts = AddOptions {
        os: vec!["windows".to_string()],
        ..AddOptions::default()
    };

    apply_edits(&mut doc, &specs, &opts).unwrap();

    let rendered = doc.to_string();
    assert!(
        rendered.contains("[tools.pwsh]"),
        "expected detailed [tools.pwsh] subtable, got:\n{}",
        rendered
    );
    assert!(rendered.contains("version = \"7.4\""));
    assert!(rendered.contains("os = [\"windows\"]"));
}

#[test]
fn apply_edits_preserves_existing_detailed_subtable() {
    let original = r#"[tools.pwsh]
version = "7.4.0"
os = ["windows"]
"#;
    let mut doc = parse_doc(original);
    let specs = vec![AddRuntimeSpec::parse("pwsh@7.4.13").unwrap()];
    let opts = AddOptions::default();

    apply_edits(&mut doc, &specs, &opts).unwrap();

    let rendered = doc.to_string();
    assert!(rendered.contains("[tools.pwsh]"));
    assert!(rendered.contains("version = \"7.4.13\""));
    assert!(
        rendered.contains("os = [\"windows\"]"),
        "os field should be preserved, got:\n{}",
        rendered
    );
}

#[test]
fn apply_edits_multiple_tools_in_one_call() {
    let mut doc = parse_doc("[tools]\n");
    let specs = vec![
        AddRuntimeSpec::parse("node@22").unwrap(),
        AddRuntimeSpec::parse("python@3.12").unwrap(),
        AddRuntimeSpec::parse("uv").unwrap(),
    ];
    let opts = AddOptions::default();

    let edits = apply_edits(&mut doc, &specs, &opts).unwrap();

    assert_eq!(edits.len(), 3);
    let rendered = doc.to_string();
    assert!(rendered.contains("node = \"22\""));
    assert!(rendered.contains("python = \"3.12\""));
    assert!(rendered.contains("uv = \"latest\""));
}

#[test]
fn add_options_default_is_install_and_lock() {
    let opts = AddOptions::default();
    assert!(!opts.no_install, "default should install");
    assert!(!opts.no_lock, "default should update lock");
    assert!(!opts.frozen);
    assert!(!opts.dry_run);
    assert!(!opts.force);
    assert!(opts.os.is_empty());
}
