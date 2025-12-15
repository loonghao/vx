//! Tests for runtime specification

use rstest::rstest;
use vx_resolver::{Ecosystem, RuntimeDependency, RuntimeSpec};

#[rstest]
fn test_runtime_spec_creation() {
    let spec = RuntimeSpec::new("npm", "Node.js package manager")
        .with_alias("npm-cli")
        .with_ecosystem(Ecosystem::Node)
        .with_dependency(RuntimeDependency::required(
            "node",
            "npm requires Node.js runtime",
        ));

    assert_eq!(spec.name, "npm");
    assert!(spec.matches("npm"));
    assert!(spec.matches("npm-cli"));
    assert!(!spec.matches("yarn"));
    assert_eq!(spec.ecosystem, Ecosystem::Node);
    assert_eq!(spec.required_dependencies().len(), 1);
}

#[rstest]
fn test_runtime_dependency() {
    let dep = RuntimeDependency::required("node", "Required runtime")
        .with_min_version(">=16.0.0")
        .provided_by("node-provider");

    assert!(dep.required);
    assert_eq!(dep.runtime_name, "node");
    assert_eq!(dep.min_version, Some(">=16.0.0".to_string()));
    assert_eq!(dep.provided_by, Some("node-provider".to_string()));
}

#[rstest]
fn test_optional_dependency() {
    let dep = RuntimeDependency::optional("typescript", "Optional TypeScript support");

    assert!(!dep.required);
    assert_eq!(dep.runtime_name, "typescript");
}

#[rstest]
fn test_ecosystem_display() {
    assert_eq!(format!("{}", Ecosystem::Node), "node");
    assert_eq!(format!("{}", Ecosystem::Python), "python");
    assert_eq!(format!("{}", Ecosystem::Rust), "rust");
    assert_eq!(format!("{}", Ecosystem::Go), "go");
    assert_eq!(format!("{}", Ecosystem::Generic), "generic");
}

#[rstest]
fn test_runtime_spec_executable() {
    let spec = RuntimeSpec::new("uvx", "Python application runner")
        .with_executable("uv")
        .with_command_prefix(vec!["tool", "run"]);

    assert_eq!(spec.get_executable(), "uv");
    assert_eq!(spec.command_prefix, vec!["tool", "run"]);
}

#[rstest]
fn test_runtime_spec_default_executable() {
    let spec = RuntimeSpec::new("node", "Node.js runtime");
    assert_eq!(spec.get_executable(), "node");
}
