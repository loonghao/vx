//! Tests for extension configuration parsing

use rstest::rstest;
use vx_extension::{ExtensionConfig, ExtensionError, ExtensionType, RuntimeRequirement};

// ============ Basic Parsing Tests ============

#[test]
fn test_parse_full_config() {
    let toml = r#"
[extension]
name = "docker-compose"
version = "1.0.0"
description = "Manage Docker Compose services"
type = "command"
authors = ["Test Author"]
license = "MIT"

[runtime]
requires = "python >= 3.10"
dependencies = ["pyyaml", "requests"]

[entrypoint]
main = "main.py"
args = ["--config", "compose.yaml"]

[commands.up]
description = "Start all services"
script = "up.py"
args = ["--detach"]

[commands.down]
description = "Stop all services"
script = "down.py"

[commands.logs]
description = "View service logs"
script = "logs.py"
args = ["-f", "--tail", "100"]
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();

    // Extension metadata
    assert_eq!(config.extension.name, "docker-compose");
    assert_eq!(config.extension.version, "1.0.0");
    assert_eq!(
        config.extension.description,
        "Manage Docker Compose services"
    );
    assert_eq!(config.extension.extension_type, ExtensionType::Command);
    assert_eq!(config.extension.authors, vec!["Test Author"]);
    assert_eq!(config.extension.license, Some("MIT".to_string()));

    // Runtime
    assert_eq!(config.runtime.runtime_name(), Some("python"));
    assert_eq!(config.runtime.version_constraint(), Some(">= 3.10"));
    assert_eq!(config.runtime.dependencies, vec!["pyyaml", "requests"]);

    // Entrypoint
    assert_eq!(config.entrypoint.main, Some("main.py".to_string()));
    assert_eq!(config.entrypoint.args, vec!["--config", "compose.yaml"]);

    // Commands
    assert_eq!(config.commands.len(), 3);
    assert!(config.commands.contains_key("up"));
    assert!(config.commands.contains_key("down"));
    assert!(config.commands.contains_key("logs"));

    let up_cmd = config.commands.get("up").unwrap();
    assert_eq!(up_cmd.description, "Start all services");
    assert_eq!(up_cmd.script, "up.py");
    assert_eq!(up_cmd.args, vec!["--detach"]);
}

#[test]
fn test_parse_minimal_config() {
    let toml = r#"
[extension]
name = "minimal-ext"
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();

    assert_eq!(config.extension.name, "minimal-ext");
    assert_eq!(config.extension.version, "0.1.0"); // default
    assert_eq!(config.extension.extension_type, ExtensionType::Command); // default
    assert!(config.extension.description.is_empty());
    assert!(config.extension.authors.is_empty());
    assert!(config.extension.license.is_none());
    assert!(config.runtime.requires.is_none());
    assert!(config.entrypoint.main.is_none());
    assert!(config.commands.is_empty());
}

// ============ Extension Type Tests ============

#[rstest]
#[case("command", ExtensionType::Command)]
#[case("hook", ExtensionType::Hook)]
#[case("provider", ExtensionType::Provider)]
fn test_extension_types(#[case] type_str: &str, #[case] expected: ExtensionType) {
    let toml = format!(
        r#"
[extension]
name = "test"
type = "{}"
"#,
        type_str
    );

    let config = ExtensionConfig::parse(&toml, None).unwrap();
    assert_eq!(config.extension.extension_type, expected);
}

#[test]
fn test_extension_type_display() {
    assert_eq!(format!("{}", ExtensionType::Command), "command");
    assert_eq!(format!("{}", ExtensionType::Hook), "hook");
    assert_eq!(format!("{}", ExtensionType::Provider), "provider");
}

// ============ Runtime Requirement Tests ============

#[rstest]
#[case("python >= 3.10", Some("python"), Some(">= 3.10"))]
#[case("node >= 18.0.0", Some("node"), Some(">= 18.0.0"))]
#[case("ruby", Some("ruby"), None)]
#[case("go >= 1.21", Some("go"), Some(">= 1.21"))]
fn test_runtime_parsing(
    #[case] requires: &str,
    #[case] expected_name: Option<&str>,
    #[case] expected_constraint: Option<&str>,
) {
    let req = RuntimeRequirement {
        requires: Some(requires.to_string()),
        dependencies: vec![],
    };

    assert_eq!(req.runtime_name(), expected_name);
    assert_eq!(req.version_constraint(), expected_constraint);
}

#[test]
fn test_runtime_no_requirement() {
    let req = RuntimeRequirement {
        requires: None,
        dependencies: vec![],
    };

    assert_eq!(req.runtime_name(), None);
    assert_eq!(req.version_constraint(), None);
}

// ============ Command Config Tests ============

#[test]
fn test_get_command_script() {
    let toml = r#"
[extension]
name = "test"

[commands.build]
description = "Build the project"
script = "build.py"

[commands.test]
description = "Run tests"
script = "test.py"
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();

    let build_cmd = config.get_command_script("build");
    assert!(build_cmd.is_some());
    assert_eq!(build_cmd.unwrap().script, "build.py");

    let test_cmd = config.get_command_script("test");
    assert!(test_cmd.is_some());
    assert_eq!(test_cmd.unwrap().script, "test.py");

    let nonexistent = config.get_command_script("nonexistent");
    assert!(nonexistent.is_none());
}

#[test]
fn test_get_main_script() {
    let toml = r#"
[extension]
name = "test"

[entrypoint]
main = "main.py"
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();
    assert_eq!(config.get_main_script(), Some("main.py"));
}

#[test]
fn test_get_main_script_none() {
    let toml = r#"
[extension]
name = "test"
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();
    assert_eq!(config.get_main_script(), None);
}

// ============ Error Cases Tests ============

#[test]
fn test_parse_invalid_toml() {
    let invalid_toml = r#"
[extension
name = "broken"
"#;

    let result = ExtensionConfig::parse(invalid_toml, None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ExtensionError::ConfigInvalid { .. }));
}

#[test]
fn test_parse_missing_extension_section() {
    let toml = r#"
[runtime]
requires = "python"
"#;

    let result = ExtensionConfig::parse(toml, None);
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_name() {
    let toml = r#"
[extension]
version = "1.0.0"
"#;

    let result = ExtensionConfig::parse(toml, None);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_type() {
    let toml = r#"
[extension]
name = "test"
type = "invalid_type"
"#;

    let result = ExtensionConfig::parse(toml, None);
    assert!(result.is_err());
}

// ============ Hook Config Tests ============

#[test]
fn test_parse_hook_extension() {
    let toml = r#"
[extension]
name = "pre-commit-check"
type = "hook"

[hooks]
pre-install = "check.py"
post-install = "setup.py"
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();

    assert_eq!(config.extension.extension_type, ExtensionType::Hook);
    assert_eq!(config.hooks.len(), 2);
    assert_eq!(
        config.hooks.get("pre-install"),
        Some(&"check.py".to_string())
    );
    assert_eq!(
        config.hooks.get("post-install"),
        Some(&"setup.py".to_string())
    );
}

// ============ Complex Scenarios ============

#[test]
fn test_parse_with_special_characters() {
    let toml = r#"
[extension]
name = "my-extension_v2"
version = "1.0.0-beta.1"
description = "Extension with special chars: <>&\""

[commands."build:prod"]
description = "Production build"
script = "build.py"
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();

    assert_eq!(config.extension.name, "my-extension_v2");
    assert_eq!(config.extension.version, "1.0.0-beta.1");
    assert!(config.commands.contains_key("build:prod"));
}

#[test]
fn test_parse_empty_arrays() {
    let toml = r#"
[extension]
name = "test"
authors = []

[runtime]
dependencies = []

[entrypoint]
args = []
"#;

    let config = ExtensionConfig::parse(toml, None).unwrap();

    assert!(config.extension.authors.is_empty());
    assert!(config.runtime.dependencies.is_empty());
    assert!(config.entrypoint.args.is_empty());
}
