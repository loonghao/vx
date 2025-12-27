//! Remote development configuration tests
//!
//! Tests for remote development environment configuration parsing.

use rstest::rstest;
use vx_config::parse_config_str;

// ============================================
// Remote Config Parsing Tests
// ============================================

#[test]
fn test_parse_remote_config_basic() {
    let content = r#"
[remote]
enabled = true
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    assert_eq!(remote.enabled, Some(true));
}

#[test]
fn test_parse_remote_config_disabled() {
    let content = r#"
[remote]
enabled = false
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    assert_eq!(remote.enabled, Some(false));
}

#[test]
fn test_parse_codespaces_config() {
    let content = r#"
[remote.codespaces]
enabled = true
machine = "standardLinux32gb"
extensions = ["ms-python.python", "rust-lang.rust-analyzer"]
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let codespaces = remote.codespaces.unwrap();

    assert_eq!(codespaces.enabled, Some(true));
    assert_eq!(codespaces.machine, Some("standardLinux32gb".to_string()));
    assert_eq!(
        codespaces.extensions,
        vec![
            "ms-python.python".to_string(),
            "rust-lang.rust-analyzer".to_string()
        ]
    );
}

#[test]
fn test_parse_codespaces_prebuild() {
    let content = r#"
[remote.codespaces]
enabled = true

[remote.codespaces.prebuild]
enabled = true
branches = ["main", "develop"]
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let codespaces = remote.codespaces.unwrap();

    assert!(codespaces.prebuild.is_some());
    let prebuild = codespaces.prebuild.unwrap();
    assert_eq!(prebuild.enabled, Some(true));
}

#[test]
fn test_parse_codespaces_ports() {
    let content = r#"
[remote.codespaces]
enabled = true

[[remote.codespaces.ports]]
port = 3000
label = "Web Server"
visibility = "public"

[[remote.codespaces.ports]]
port = 5000
label = "API Server"
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let codespaces = remote.codespaces.unwrap();

    assert_eq!(codespaces.ports.len(), 2);
    assert_eq!(codespaces.ports[0].port, 3000);
    assert_eq!(codespaces.ports[1].port, 5000);
}

#[test]
fn test_parse_gitpod_config() {
    let content = r#"
[remote.gitpod]
enabled = true
image = "gitpod/workspace-full"
extensions = ["ms-python.python"]
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let gitpod = remote.gitpod.unwrap();

    assert_eq!(gitpod.enabled, Some(true));
    assert_eq!(gitpod.image, Some("gitpod/workspace-full".to_string()));
    assert_eq!(gitpod.extensions, vec!["ms-python.python".to_string()]);
}

#[test]
fn test_parse_gitpod_tasks() {
    let content = r#"
[remote.gitpod]
enabled = true

[[remote.gitpod.tasks]]
name = "Install"
init = "npm install"

[[remote.gitpod.tasks]]
name = "Start"
command = "npm run dev"
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let gitpod = remote.gitpod.unwrap();

    assert_eq!(gitpod.tasks.len(), 2);
    assert_eq!(gitpod.tasks[0].name, Some("Install".to_string()));
    assert_eq!(gitpod.tasks[0].init, Some("npm install".to_string()));
    assert_eq!(gitpod.tasks[1].name, Some("Start".to_string()));
    assert_eq!(gitpod.tasks[1].command, Some("npm run dev".to_string()));
}

#[test]
fn test_parse_gitpod_prebuilds() {
    let content = r#"
[remote.gitpod.prebuilds]
master = true
branches = true
pull_requests = true
add_check = true
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let gitpod = remote.gitpod.unwrap();
    let prebuilds = gitpod.prebuilds.unwrap();

    assert_eq!(prebuilds.master, Some(true));
    assert_eq!(prebuilds.branches, Some(true));
    assert_eq!(prebuilds.pull_requests, Some(true));
    assert_eq!(prebuilds.add_check, Some(true));
}

#[test]
fn test_parse_devcontainer_config() {
    let content = r#"
[remote.devcontainer]
image = "mcr.microsoft.com/devcontainers/rust:1"
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let devcontainer = remote.devcontainer.unwrap();

    assert_eq!(
        devcontainer.image,
        Some("mcr.microsoft.com/devcontainers/rust:1".to_string())
    );
}

#[test]
fn test_parse_devcontainer_with_features() {
    let content = r#"
[remote.devcontainer]
image = "mcr.microsoft.com/devcontainers/rust:1"

[remote.devcontainer.features]
"ghcr.io/devcontainers/features/node:1" = {}
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();
    let devcontainer = remote.devcontainer.unwrap();

    assert!(devcontainer
        .features
        .contains_key("ghcr.io/devcontainers/features/node:1"));
}

#[test]
fn test_parse_full_remote_config() {
    let content = r#"
[remote]
enabled = true

[remote.codespaces]
enabled = true
machine = "standardLinux32gb"
extensions = ["ms-python.python"]

[remote.gitpod]
enabled = true
image = "gitpod/workspace-full"

[remote.devcontainer]
image = "mcr.microsoft.com/devcontainers/base:ubuntu"
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();

    assert_eq!(remote.enabled, Some(true));
    assert!(remote.codespaces.is_some());
    assert!(remote.gitpod.is_some());
    assert!(remote.devcontainer.is_some());
}

#[test]
fn test_remote_config_empty() {
    let content = r#"
[remote]
"#;
    let config = parse_config_str(content).unwrap();
    let remote = config.remote.unwrap();

    assert!(remote.enabled.is_none());
    assert!(remote.codespaces.is_none());
    assert!(remote.gitpod.is_none());
    assert!(remote.devcontainer.is_none());
}

#[rstest]
#[case("standardLinux32gb")]
#[case("premiumLinux")]
#[case("largePremiumLinux")]
fn test_parse_codespaces_machine_types(#[case] machine: &str) {
    let content = format!(
        r#"
[remote.codespaces]
machine = "{}"
"#,
        machine
    );
    let config = parse_config_str(&content).unwrap();
    let remote = config.remote.unwrap();
    let codespaces = remote.codespaces.unwrap();

    assert_eq!(codespaces.machine, Some(machine.to_string()));
}
